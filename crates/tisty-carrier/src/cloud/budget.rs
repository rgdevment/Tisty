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
const CHARGE_WAITED_OUT: std::time::Duration = std::time::Duration::from_secs(5);

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
            .and_then(|bytes| serde_json::from_slice::<Spent>(&bytes).ok())
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
}

impl<'a> Metered<'a> {
    pub fn over(remote: &'a dyn Remote, home: &Path, class: Class) -> Self {
        Self {
            remote,
            limits: policy::applied(remote.limits(), policy::kept(home).as_ref()),
            home: home.to_path_buf(),
            class: Mutex::new(class),
        }
    }

    pub fn as_class(&self, class: Class) {
        if let Ok(mut now) = self.class.lock() {
            *now = class;
        }
    }

    fn allowed(&self) -> Result<(), Hitch> {
        if policy::paused_here(&self.home)
            || policy::halted(policy::kept(&self.home).as_ref(), policy::now())
        {
            return Err(Hitch::Paused);
        }
        let class = self.class.lock().map_or(Class::History, |one| *one);
        let (units, bytes) = match class {
            Class::Asked => return Ok(()),
            Class::History => (self.limits.units_a_day, self.limits.bytes_a_day),
            Class::Attachments => (
                self.limits.units_a_day - self.limits.units_a_day / 100 * RESERVED_PERCENT,
                self.limits.bytes_a_day - self.limits.bytes_a_day / 100 * RESERVED_PERCENT,
            ),
        };
        let spent = Spent::load(&self.home);
        match spent.units >= units || spent.bytes >= bytes {
            true => Err(Hitch::Spent),
            false => Ok(()),
        }
    }

    fn charge(&self, requests: u64, cost: u64, bytes: u64) {
        // Threads of one process queue here without a deadline, so only another process can ever wait the lock out.
        static CHARGING: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _ours = CHARGING
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Unlocked, two charges at once can still lose one; never charging loses every one.
        let _turn = match Held::wait_for(&self.home, BUDGETING, CHARGE_WAITED_OUT) {
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
        let mut spent = Spent::load(&self.home);
        spent.units = spent.units.saturating_add(requests.saturating_mul(cost));
        spent.bytes = spent.bytes.saturating_add(bytes);
        spent.save(&self.home);
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
        self.charge(
            self.limits.requests_to_list(entries),
            self.limits.costs.list,
            0,
        );
        seen
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        self.allowed()?;
        let mut tallied = Tallied { into, bytes: 0 };
        let got = self.remote.fetch(name, from, &mut tallied);
        self.charge(1, self.limits.costs.fetch, tallied.bytes);
        got
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        self.allowed()?;
        let weighs = std::fs::metadata(from).map_or(0, |one| one.len());
        let put = self.remote.put(name, from, expect);
        self.charge(
            self.limits.requests_to_put(weighs),
            self.limits.costs.put,
            0,
        );
        put
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        self.allowed()?;
        let gone = self.remote.delete(name, expect);
        self.charge(1, self.limits.costs.delete, 0);
        gone
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        self.remote.hash_of(local)
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        self.allowed()?;
        let seen = self.remote.about(name);
        self.charge(1, self.limits.costs.about, 0);
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
        self.charge(
            self.limits.requests_to_list(entries),
            self.limits.costs.changes,
            0,
        );
        changes
    }

    fn hears(&self, since: &str) -> Option<Box<dyn Watch>> {
        self.remote.hears(since)
    }

    fn lends(&self, name: &str) -> Result<Option<String>, Hitch> {
        self.allowed()?;
        let lent = self.remote.lends(name);
        self.charge(1, self.limits.costs.about, 0);
        lent
    }
}

#[cfg(test)]
#[path = "budget_test.rs"]
mod tests;
