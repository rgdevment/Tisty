use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use super::lock::{BUDGETING, Held};
use super::policy;
use crate::{Changes, Expect, Hitch, Limits, Remote, Seen, Watch};

const FILE: &str = "budget.json";
const RESERVED_PERCENT: u64 = 10;
const DAY_SECS: u64 = 24 * 60 * 60;
const CHARGE_WAITED_OUT: std::time::Duration = std::time::Duration::from_secs(30);
const STALLED_UNITS: u64 = 5_000;
const STALLED_ASKS: u64 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    History,
    Attachments,
    Asked,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spent {
    pub day: u64,
    pub units: u64,
    pub bytes: u64,
}

impl Spent {
    pub fn load(home: &Path) -> Self {
        let today = today();
        std::fs::read(home.join(FILE))
            .ok()
            .and_then(|bytes| match serde_json::from_slice::<Spent>(&bytes) {
                Ok(spent) => Some(spent),
                Err(why) => {
                    unreadable_once(&why);
                    None
                }
            })
            .filter(|spent| spent.day == today)
            .unwrap_or(Spent {
                day: today,
                ..Spent::default()
            })
    }

    fn save(&self, home: &Path) {
        if let Ok(bytes) = serde_json::to_vec(self) {
            let _ = std::fs::create_dir_all(home);
            let _ = tisty_core::store::write_atomic(&home.join(FILE), &bytes);
        }
    }
}

// Read as nothing spent, since the budget only guards against a loop; said once, not on every request.
fn unreadable_once(why: &serde_json::Error) {
    static SAID: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !SAID.swap(true, std::sync::atomic::Ordering::Relaxed) {
        tisty_core::witness::warn(
            tisty_core::witness::channel::CLOUD,
            "what the cloud spent today could not be read, so it starts again from nothing",
            &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
        );
    }
}

// A request that never left this machine cost the provider nothing.
fn went_out<T>(answer: &Result<T, Hitch>) -> bool {
    !matches!(answer, Err(Hitch::Unreachable(..)))
}

// An answer that says how things stand is no sign of a loop by its cost, only by how many there are.
fn answered<T>(answer: &Result<T, Hitch>) -> bool {
    matches!(answer, Ok(_) | Err(Hitch::Missing(_) | Hitch::Changed(_)))
}

fn today() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs() / DAY_SECS)
}

pub struct Metered<'a> {
    remote: &'a dyn Remote,
    limits: Limits,
    home: PathBuf,
    class: Mutex<Class>,
    idle: Mutex<Idle>,
}

#[derive(Default)]
struct Idle {
    units: u64,
    asks: u64,
    told: bool,
}

impl<'a> Metered<'a> {
    pub fn over(remote: &'a dyn Remote, home: &Path, class: Class) -> Self {
        Self {
            remote,
            limits: policy::applied(remote.limits(), policy::kept(home).as_ref()),
            home: home.to_path_buf(),
            class: Mutex::new(class),
            idle: Mutex::new(Idle::default()),
        }
    }

    pub fn as_class(&self, class: Class) {
        if let Ok(mut now) = self.class.lock() {
            *now = class;
        }
    }

    fn allowed(&self) -> Result<(), Hitch> {
        let class = self.class.lock().map_or(Class::History, |one| *one);
        allowed_at(&self.home, &self.limits, class)?;
        self.unstalled()
    }

    // A round that keeps asking file by file with nothing landing stops, instead of spending the day without a word.
    fn unstalled(&self) -> Result<(), Hitch> {
        let Ok(mut idle) = self.idle.lock() else {
            return Ok(());
        };
        if idle.units < STALLED_UNITS && idle.asks < STALLED_ASKS {
            return Ok(());
        }
        if !idle.told {
            idle.told = true;
            tisty_core::witness::warn(
                tisty_core::witness::channel::CLOUD,
                "a round kept asking the cloud with nothing landing, so it stopped",
                &[
                    (
                        "units",
                        tisty_core::witness::Fact::Count(
                            usize::try_from(idle.units).unwrap_or(usize::MAX),
                        ),
                    ),
                    (
                        "asks",
                        tisty_core::witness::Fact::Count(
                            usize::try_from(idle.asks).unwrap_or(usize::MAX),
                        ),
                    ),
                ],
            );
        }
        Err(Hitch::Troubled(
            "the round kept asking the cloud with nothing landing".into(),
        ))
    }

    // Only a landing starts the count again; an answer that says how things stand counts as an ask but weighs nothing.
    fn weighed<T>(&self, answer: &Result<T, Hitch>, units: u64, lands: bool) {
        match answer {
            Ok(_) if lands => self.moved_on(),
            _ if answered(answer) => self.idled(0),
            _ => self.idled(units),
        }
    }

    fn idled(&self, units: u64) {
        if let Ok(mut idle) = self.idle.lock() {
            idle.units = idle.units.saturating_add(units);
            idle.asks = idle.asks.saturating_add(1);
        }
    }

    fn charge(&self, requests: u64, cost: u64, bytes: u64) -> u64 {
        let units = requests
            .saturating_mul(cost)
            .saturating_add(self.remote.spent_beyond());
        charged(&self.home, 1, units, bytes);
        units
    }

    fn charge_checked(&self, requests: u64, cost: u64, bytes: u64) -> u64 {
        let units = requests
            .saturating_mul(cost)
            .saturating_add(self.limits.costs.checks)
            .saturating_add(self.remote.spent_beyond());
        charged(&self.home, 1, units, bytes);
        units
    }
}

fn allowed_at(home: &Path, limits: &Limits, class: Class) -> Result<(), Hitch> {
    if policy::paused_here(home) || policy::halted(policy::kept(home).as_ref(), policy::now()) {
        return Err(Hitch::Paused);
    }
    let (units, bytes) = match class {
        Class::Asked => return Ok(()),
        Class::History => (limits.units_a_day, limits.bytes_a_day),
        Class::Attachments => (
            limits.units_a_day - limits.units_a_day / 100 * RESERVED_PERCENT,
            limits.bytes_a_day - limits.bytes_a_day / 100 * RESERVED_PERCENT,
        ),
    };
    let spent = Spent::load(home);
    match spent.units >= units || spent.bytes >= bytes {
        true => Err(Hitch::Spent),
        false => Ok(()),
    }
}

fn charged(home: &Path, requests: u64, cost: u64, bytes: u64) {
    // Threads of one process queue here without a deadline, so only another process can ever wait the lock out.
    static CHARGING: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _ours = CHARGING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // Unlocked, two charges at once can still lose one; never charging loses every one.
    let _turn = match Held::wait_for(home, BUDGETING, CHARGE_WAITED_OUT) {
        Ok(Some(turn)) => Some(turn),
        Ok(None) => {
            tisty_core::witness::warn(
                tisty_core::witness::channel::CLOUD,
                "the budget is charged without its lock, which another process held too long",
                &[],
            );
            None
        }
        Err(why) => {
            tisty_core::witness::warn(
                tisty_core::witness::channel::CLOUD,
                "the budget is charged without its lock, which could not be taken",
                &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
            );
            None
        }
    };
    let mut spent = Spent::load(home);
    spent.units = spent.units.saturating_add(requests.saturating_mul(cost));
    spent.bytes = spent.bytes.saturating_add(bytes);
    spent.save(home);
}

// Every ask a long listen makes is weighed like any other, so a pause or a spent day stops it between two.
struct Listening {
    watch: Box<dyn Watch>,
    home: PathBuf,
    limits: Limits,
}

impl Watch for Listening {
    fn told(&mut self, within: std::time::Duration) -> Result<bool, Hitch> {
        allowed_at(&self.home, &self.limits, Class::History)?;
        let heard = self.watch.told(within);
        charged(&self.home, 1, self.limits.costs.changes, 0);
        heard
    }
}

struct Tallied<'w> {
    into: &'w mut dyn Write,
    bytes: u64,
}

impl Write for Tallied<'_> {
    fn write(&mut self, said: &[u8]) -> std::io::Result<usize> {
        let wrote = self.into.write(said)?;
        self.bytes += wrote as u64;
        Ok(wrote)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.into.flush()
    }
}

impl Remote for Metered<'_> {
    fn limits(&self) -> Limits {
        self.limits
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        self.allowed()?;
        let seen = self.remote.list(under);
        let entries = seen.as_ref().map_or(0, Vec::len);
        if went_out(&seen) {
            self.charge(
                self.limits.requests_to_list(entries),
                self.limits.costs.list,
                0,
            );
        }
        seen
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        self.allowed()?;
        let mut tallied = Tallied { into, bytes: 0 };
        let got = self.remote.fetch(name, from, &mut tallied);
        if went_out(&got) || tallied.bytes > 0 {
            let units = self.charge_checked(1, self.limits.costs.fetch, tallied.bytes);
            self.weighed(&got, units, true);
        }
        got
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        self.allowed()?;
        let weighs = std::fs::metadata(from).map_or(0, |one| one.len());
        let put = self.remote.put(name, from, expect);
        // Bytes may have gone up before the answer broke, so an upload is charged however it ended, unless refused on the look before it.
        let requests = match &put {
            Err(Hitch::Changed(_) | Hitch::Missing(_)) => 0,
            _ => self.limits.requests_to_put(weighs),
        };
        let units = self.charge_checked(requests, self.limits.costs.put, 0);
        self.weighed(&put, units, true);
        put
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        self.allowed()?;
        let gone = self.remote.delete(name, expect);
        if went_out(&gone) {
            let units = self.charge_checked(1, self.limits.costs.delete, 0);
            self.weighed(&gone, units, true);
        }
        gone
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        self.remote.hash_of(local)
    }

    fn renamed(&self, from: &str, to: &str) -> Result<(), Hitch> {
        self.allowed()?;
        let moved = self.remote.renamed(from, to);
        if went_out(&moved) {
            let units = self.charge_checked(1, self.limits.costs.put, 0);
            self.weighed(&moved, units, true);
        }
        moved
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        self.allowed()?;
        let seen = self.remote.about(name);
        if went_out(&seen) {
            let units = self.charge(1, self.limits.costs.about, 0);
            self.weighed(&seen, units, false);
        }
        seen
    }

    fn changes(&self, since: Option<&str>) -> Result<Changes, Hitch> {
        self.allowed()?;
        let changes = self.remote.changes(since);
        let entries = match &changes {
            Ok(Changes::Whole { seen, .. }) => seen.len(),
            Ok(Changes::Since { changed, gone, .. }) => changed.len() + gone.len(),
            Err(_) => 0,
        };
        if went_out(&changes) {
            self.charge(
                self.limits.requests_to_list(entries),
                self.limits.costs.changes,
                0,
            );
        }
        changes
    }

    // Listening asks the provider too, so a paused cloud or a spent day hears nothing.
    fn hears(&self, since: &str) -> Option<Box<dyn Watch>> {
        self.allowed().ok()?;
        let watch = self.remote.hears(since)?;
        Some(Box::new(Listening {
            watch,
            home: self.home.clone(),
            limits: self.limits,
        }))
    }

    fn lends(&self, name: &str) -> Result<Option<String>, Hitch> {
        self.allowed()?;
        let lent = self.remote.lends(name);
        let units = self.charge(1, self.limits.costs.about, 0);
        self.weighed(&lent, units, false);
        lent
    }

    fn moved_on(&self) {
        if let Ok(mut idle) = self.idle.lock() {
            *idle = Idle::default();
        }
    }

    fn root(&self) -> &str {
        self.remote.root()
    }

    fn mark_up(&self) -> Result<(), Hitch> {
        self.allowed()?;
        let marked = self.remote.mark_up();
        self.charge(2, self.limits.costs.about, 0);
        marked
    }
}

#[cfg(test)]
#[path = "budget_test.rs"]
mod tests;
