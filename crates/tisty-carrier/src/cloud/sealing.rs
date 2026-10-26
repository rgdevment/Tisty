use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tisty_share_proto::cloud::{CloudKey, Mark};
use tisty_sync::{NAMED, Sealing, Trouble, Turn};

use super::budget::{Class, Metered};
use super::lock::{Held, SEALING};
use super::policy::now;
use crate::{Expect, Hitch, Remote, Seen};

const JOURNAL: &str = "sealing.json";
const TWINS: &str = "sealing.twins";
const BATCH: usize = 64;
const BATCH_BYTES: u64 = if cfg!(test) {
    64 * 1024
} else {
    32 * 1024 * 1024
};
const SAID: u32 = 1;
// A round that read the word before the close may still be sending, so the place stays closed this long first.
const SETTLE: u64 = if cfg!(test) { 0 } else { 10 * 60 };
const WIPED_BETWEEN_LOOKS: usize = 16;
const STIRRED_EVERY: u64 = 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Phase {
    GivingUp,
    Claiming,
    Claimed,
    Copied,
    Closed,
    Settled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Journal {
    v: u32,
    cloud: String,
    named: String,
    device: String,
    nonce: String,
    on: bool,
    phase: Phase,
    moved: u64,
    #[serde(default)]
    closed: u64,
    #[serde(default)]
    refused: BTreeMap<String, u64>,
    #[serde(default)]
    orphan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Twin {
    name: String,
    from: String,
    bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stepped {
    Claimed,
    Copying { done: usize, left: usize },
    Closed,
    Settling { left: usize },
    Settled,
    Done,
    GaveUp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Left {
    pub on: bool,
    pub cloud: String,
    pub moved: u64,
    pub closed: bool,
    pub giving_up: bool,
}

pub struct Turning {
    plain: Arc<dyn Remote>,
    home: PathBuf,
    key: CloudKey,
    journal: Journal,
    pending: Option<Owed>,
}

fn broke(e: impl ToString) -> Hitch {
    Hitch::Broke(e.to_string())
}

fn turned_away(cloud: &str) -> Hitch {
    Hitch::Barred(Trouble::Sealing(cloud.to_string()))
}

fn locked(home: &Path) -> Result<Held, Hitch> {
    Held::take(home, SEALING)
        .map_err(broke)?
        .ok_or_else(|| Hitch::Busy {
            wait: std::time::Duration::from_secs(5),
        })
}

fn word(remote: &dyn Remote) -> Result<(String, String), Hitch> {
    let mut said = Vec::new();
    let seen = remote.fetch(NAMED, 0, &mut said)?;
    let text = String::from_utf8(said).map_err(broke)?;
    Ok((text, seen.revision))
}

fn rewrite(remote: &dyn Remote, text: &str, revision: &str) -> Result<(), Hitch> {
    let held = crate::temp::Temp::new();
    let mut file = held.create().map_err(broke)?;
    file.write_all(text.as_bytes()).map_err(broke)?;
    drop(file);
    remote.put(NAMED, &held.0, Expect::Revision(revision.to_string()))?;
    Ok(())
}

fn sealed_over<'a>(
    plain: &'a dyn Remote,
    home: &'a Path,
    key: CloudKey,
    cloud: &str,
) -> crate::Sealed<Metered<'a>> {
    crate::Sealed::new(Metered::over(plain, home, Class::Sealing), key, cloud)
}

fn ours_to_turn(name: &str, folds: bool) -> bool {
    (super::mirror::mirrored(name) || super::index::on_shelf(name))
        && !crate::sealed::stays_plain(name, folds)
        && !name.starts_with(&format!("{}/", crate::sealed::INSIDE))
}

fn deleted(remote: &dyn Remote, name: &str) -> Result<(), Hitch> {
    match remote.delete(name, None) {
        Ok(()) | Err(Hitch::Missing(_)) => Ok(()),
        Err(other) => Err(other),
    }
}

impl Journal {
    fn claim(&self) -> String {
        format!("{}:{}", self.device, self.nonce)
    }

    fn load(home: &Path) -> Option<Self> {
        let said = std::fs::read(home.join(JOURNAL)).ok()?;
        serde_json::from_slice::<Self>(&said)
            .ok()
            .filter(|one| one.v == SAID)
    }

    fn save(&mut self, home: &Path) -> Result<(), Hitch> {
        self.moved = now();
        let said = serde_json::to_vec(self).map_err(broke)?;
        std::fs::create_dir_all(home).map_err(broke)?;
        tisty_core::store::write_atomic(&home.join(JOURNAL), &said).map_err(broke)
    }
}

// Appended one line a file, so a cut leaves at worst one torn line, which is read as never written.
fn twins(home: &Path) -> BTreeMap<String, Twin> {
    let Ok(said) = std::fs::read_to_string(home.join(TWINS)) else {
        return BTreeMap::new();
    };
    said.lines()
        .filter_map(|line| serde_json::from_str::<Twin>(line).ok())
        .map(|twin| (twin.name.clone(), twin))
        .collect()
}

fn twinned(home: &Path, twin: &Twin) -> Result<(), Hitch> {
    let mut line = serde_json::to_vec(twin).map_err(broke)?;
    line.push(b'\n');
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(home.join(TWINS))
        .map_err(broke)?;
    file.write_all(&line).map_err(broke)
}

fn forget(home: &Path) {
    let _ = std::fs::remove_file(home.join(JOURNAL));
    let _ = std::fs::remove_file(home.join(TWINS));
}

struct Owed {
    copy: Vec<Seen>,
    gone: Vec<String>,
    refused: BTreeMap<String, u64>,
    done: usize,
}

impl Owed {
    fn left(&self) -> usize {
        self.copy.len() + self.gone.len()
    }
}

fn mark_holds(said: Option<Mark>, cloud: &str) -> Result<(), Hitch> {
    match said {
        Some(Mark::Holds) => Ok(()),
        Some(Mark::OtherPhrase) => Err(Hitch::Barred(Trouble::OtherPhrase(cloud.to_string()))),
        _ => Err(Hitch::Barred(Trouble::MarkDamaged(cloud.to_string()))),
    }
}

impl Turning {
    pub fn begin(
        plain: Arc<dyn Remote>,
        home: &Path,
        key: CloudKey,
        cloud: &str,
        device: &str,
        turn: Turn,
    ) -> Result<Self, Hitch> {
        {
            let _held = locked(home)?;
            if Journal::load(home).is_some() {
                return Err(turned_away(cloud));
            }
            let metered = Metered::over(&*plain, home, Class::Sealing);
            if turn == Turn::On && metered.about(tisty_core::moved::NAMED)?.is_some() {
                return Err(Hitch::Barred(Trouble::Refused(
                    "a place that was left is never sealed".into(),
                )));
            }
            let (text, _) = word(&metered)?;
            match (turn, tisty_sync::sealing_in(&text)) {
                (Turn::On, Sealing::Plain | Sealing::Unsaid) => {}
                (Turn::Off, Sealing::Sealed(said)) if said == cloud => {
                    let sealed = sealed_over(&*plain, home, key.clone(), cloud);
                    mark_holds(sealed.marked()?, cloud)?;
                }
                (_, Sealing::Copying(_) | Sealing::Closed) => return Err(turned_away(cloud)),
                (Turn::On, _) => return Err(Hitch::Barred(Trouble::Sealed(cloud.to_string()))),
                (Turn::Off, _) => return Err(Hitch::Barred(Trouble::Unsealed(cloud.to_string()))),
            }
        }
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |at| at.as_nanos())
            .to_string();
        let mut journal = Journal {
            v: SAID,
            cloud: cloud.to_string(),
            named: key.named_for(cloud),
            device: device.to_string(),
            nonce,
            on: turn == Turn::On,
            phase: Phase::Claiming,
            moved: now(),
            closed: 0,
            refused: BTreeMap::new(),
            orphan: false,
        };
        journal.save(home)?;
        let mut turning = Self {
            plain,
            home: home.to_path_buf(),
            key,
            journal,
            pending: None,
        };
        turning.step()?;
        Ok(turning)
    }

    pub fn resume(plain: Arc<dyn Remote>, home: &Path, key: CloudKey) -> Option<Self> {
        let journal = Journal::load(home)?;
        (key.named_for(&journal.cloud) == journal.named).then(|| Self {
            plain,
            home: home.to_path_buf(),
            key,
            journal,
            pending: None,
        })
    }

    pub fn left(home: &Path) -> Option<Left> {
        Journal::load(home).map(|journal| Left {
            on: journal.on,
            closed: matches!(journal.phase, Phase::Closed | Phase::Settled),
            giving_up: journal.phase == Phase::GivingUp,
            cloud: journal.cloud,
            moved: journal.moved,
        })
    }

    // `before` is told the way the word says the turn went, under the lock, before anything is undone.
    pub fn orphaned(
        plain: Arc<dyn Remote>,
        home: &Path,
        key: CloudKey,
        cloud: &str,
        idle_after: u64,
        before: impl FnOnce(Turn) -> Result<(), Hitch>,
    ) -> Result<Option<Turn>, Hitch> {
        let _held = locked(home)?;
        if Journal::load(home).is_some() {
            return Err(turned_away(cloud));
        }
        let metered = Metered::over(&*plain, home, Class::Sealing);
        let (text, _) = word(&metered)?;
        drop(metered);
        let Some(claim) = tisty_sync::claim_in(&text) else {
            return Ok(None);
        };
        let (Some((device, nonce)), Some(turn)) = (claim.split_once(':'), turn_of(&text)) else {
            return Ok(None);
        };
        if idle_for(&text).is_none_or(|idle| idle < idle_after) {
            return Ok(None);
        }
        before(turn)?;
        let mut turning = Self {
            plain,
            home: home.to_path_buf(),
            journal: Journal {
                v: SAID,
                cloud: cloud.to_string(),
                named: key.named_for(cloud),
                device: device.to_string(),
                nonce: nonce.to_string(),
                on: turn == Turn::On,
                phase: Phase::GivingUp,
                moved: now(),
                closed: 0,
                refused: BTreeMap::new(),
                orphan: true,
            },
            key,
            pending: None,
        };
        turning.journal.save(home)?;
        turning.given_up().map(|_| Some(turn))
    }

    pub fn said(&self) -> (&str, &str) {
        (&self.journal.cloud, &self.journal.named)
    }

    pub fn turn(&self) -> Turn {
        match self.journal.on {
            true => Turn::On,
            false => Turn::Off,
        }
    }

    pub fn refused(&self) -> &BTreeMap<String, u64> {
        &self.journal.refused
    }

    pub fn moved(&self) -> u64 {
        self.journal.moved
    }

    fn sealed(&self) -> crate::Sealed<Metered<'_>> {
        sealed_over(
            &*self.plain,
            &self.home,
            self.key.clone(),
            &self.journal.cloud,
        )
    }

    pub fn step(&mut self) -> Result<Stepped, Hitch> {
        let (plain, home) = (self.plain.clone(), self.home.clone());
        let _held = locked(&home)?;
        if self.journal.phase == Phase::GivingUp {
            return self.given_up();
        }
        let metered = Metered::over(&*plain, &home, Class::Sealing);
        let (text, revision) = word(&metered)?;
        let claim = self.journal.claim();
        let (on, cloud) = (self.journal.on, self.journal.cloud.clone());
        let way = self.turn();
        let ours = tisty_sync::claimed_by(&text, &claim);
        match (self.journal.phase, tisty_sync::sealing_in(&text)) {
            (_, Sealing::Sealed(said)) if on && said == cloud => {
                forget(&self.home);
                return Ok(Stepped::Done);
            }
            (Phase::Settled, Sealing::Plain | Sealing::Unsaid) if !on => {
                self.sealed().unmark()?;
                forget(&self.home);
                return Ok(Stepped::Done);
            }
            (Phase::Claiming, Sealing::Plain | Sealing::Unsaid) if on => {
                return self.claimed(&metered, &text, &revision);
            }
            (Phase::Claiming, Sealing::Sealed(said)) if !on && said == cloud => {
                return self.claimed(&metered, &text, &revision);
            }
            (Phase::Claiming, Sealing::Copying(went)) if went == way && ours => {
                return self.held_claim(&metered);
            }
            (Phase::Claimed | Phase::Copied, Sealing::Copying(went)) if went == way && ours => {}
            (Phase::Closed | Phase::Settled, Sealing::Closed) if ours => {}
            _ => return Err(turned_away(&cloud)),
        }
        if matches!(self.journal.phase, Phase::Claimed | Phase::Closed)
            && idle_for(&text).is_some_and(|idle| idle >= STIRRED_EVERY)
        {
            let stirred =
                tisty_sync::stirred(&text, &claim, now()).ok_or_else(|| turned_away(&cloud))?;
            rewrite(&metered, &stirred, &revision)?;
        }
        let sealed = sealed_over(&*plain, &home, self.key.clone(), &cloud);
        let (from, to): (&dyn Remote, &dyn Remote) = match on {
            true => (&metered, &sealed),
            false => (&sealed, &metered),
        };
        match self.journal.phase {
            Phase::Claimed => match self.batch(from, to)? {
                0 => {
                    self.journal.phase = Phase::Copied;
                    self.journal.save(&self.home)?;
                    Ok(Stepped::Copying {
                        done: self.done(),
                        left: 0,
                    })
                }
                left => Ok(Stepped::Copying {
                    done: self.done(),
                    left,
                }),
            },
            Phase::Copied => {
                let closed =
                    tisty_sync::closed(&text, &claim).ok_or_else(|| turned_away(&cloud))?;
                rewrite(&metered, &closed, &revision)?;
                self.pending = None;
                self.journal.phase = Phase::Closed;
                self.journal.closed = now();
                self.journal.save(&self.home)?;
                Ok(Stepped::Closed)
            }
            Phase::Closed => match self.batch(from, to)? {
                0 if now() < self.journal.closed + SETTLE => {
                    self.pending = None;
                    Ok(Stepped::Settling { left: 0 })
                }
                0 => {
                    if on {
                        self.opens_one(&sealed)?;
                    }
                    self.journal.phase = Phase::Settled;
                    self.journal.save(&self.home)?;
                    Ok(Stepped::Settled)
                }
                left => Ok(Stepped::Settling { left }),
            },
            Phase::Settled => {
                let to_be = on.then_some(cloud.as_str());
                let switched = tisty_sync::switched(&text, &claim, to_be)
                    .ok_or_else(|| turned_away(&cloud))?;
                rewrite(&metered, &switched, &revision)?;
                if !on {
                    sealed.unmark()?;
                }
                forget(&self.home);
                Ok(Stepped::Done)
            }
            Phase::Claiming | Phase::GivingUp => Err(turned_away(&cloud)),
        }
    }

    fn done(&self) -> usize {
        self.pending.as_ref().map_or(0, |owed| owed.done)
    }

    // Listed again only once a batch runs dry, so a large cloud is not read whole for every batch.
    fn batch(&mut self, from: &dyn Remote, to: &dyn Remote) -> Result<usize, Hitch> {
        let mut owed = match self.pending.take() {
            Some(owed) if owed.left() > 0 => owed,
            _ => self.owed(from, to)?,
        };
        self.journal.refused = owed.refused.clone();
        if owed.left() == 0 {
            self.pending = Some(owed);
            return Ok(0);
        }
        self.copied(from, to, &mut owed)?;
        self.journal.save(&self.home)?;
        let left = owed.left();
        self.pending = Some(owed);
        Ok(left.max(1))
    }

    fn claimed(
        &mut self,
        metered: &dyn Remote,
        text: &str,
        revision: &str,
    ) -> Result<Stepped, Hitch> {
        let phase = tisty_sync::phase(self.turn(), &self.journal.device, &self.journal.nonce);
        let claimed =
            tisty_sync::claimed(text, &phase).ok_or_else(|| turned_away(&self.journal.cloud))?;
        match rewrite(metered, &claimed, revision) {
            Ok(()) => self.held_claim(metered),
            Err(Hitch::Changed(_)) => Err(turned_away(&self.journal.cloud)),
            Err(other) => Err(other),
        }
    }

    // Read back before the mark: on Drive a claim can be checked, then overwritten by another's.
    fn held_claim(&mut self, metered: &dyn Remote) -> Result<Stepped, Hitch> {
        let (back, _) = word(metered)?;
        if !tisty_sync::claimed_by(&back, &self.journal.claim()) {
            return Err(turned_away(&self.journal.cloud));
        }
        self.sealed().mark_it()?;
        self.journal.phase = Phase::Claimed;
        self.journal.save(&self.home)?;
        Ok(Stepped::Claimed)
    }

    // Waits out the batch a driver may be in the middle of, and false says the turn had already switched.
    pub fn abort(mut self, before: impl FnOnce() -> Result<(), Hitch>) -> Result<bool, Hitch> {
        let _held = Held::wait_for(&self.home, SEALING, std::time::Duration::from_secs(120))
            .map_err(broke)?
            .ok_or(Hitch::Busy {
                wait: std::time::Duration::from_secs(5),
            })?;
        if self.journal.phase != Phase::GivingUp {
            let metered = Metered::over(&*self.plain, &self.home, Class::Sealing);
            let (text, _) = word(&metered)?;
            let switched = match tisty_sync::sealing_in(&text) {
                Sealing::Sealed(cloud) => self.journal.on && cloud == self.journal.cloud,
                Sealing::Plain | Sealing::Unsaid => {
                    !self.journal.on && self.journal.phase == Phase::Settled
                }
                _ => false,
            };
            if switched || Journal::load(&self.home).is_none() {
                return Ok(false);
            }
            before()?;
            self.journal.phase = Phase::GivingUp;
            self.journal.save(&self.home)?;
        }
        self.given_up().map(|_| true)
    }

    // Kept in the journal before anything is undone, so a cut half way is undone the rest of the way next time.
    fn given_up(&self) -> Result<Stepped, Hitch> {
        let metered = Metered::over(&*self.plain, &self.home, Class::Sealing);
        let (text, revision) = word(&metered)?;
        if let Some(back) = tisty_sync::dropped(&text, &self.journal.claim()) {
            rewrite(&metered, &back, &revision)?;
        }
        let (now, _) = word(&metered)?;
        // What another machine claimed or switched to since is theirs, and nothing of it is undone here.
        let left_as_it_was = match (self.journal.on, tisty_sync::sealing_in(&now)) {
            (true, Sealing::Plain | Sealing::Unsaid) => true,
            (false, Sealing::Sealed(cloud)) => cloud == self.journal.cloud,
            _ => false,
        };
        let (_, read) = word(&metered)?;
        let still = |done: usize| -> Result<bool, Hitch> {
            Ok(!done.is_multiple_of(WIPED_BETWEEN_LOOKS) || word(&metered)?.1 == read)
        };
        if left_as_it_was && self.journal.on {
            let sealed = self.sealed();
            let names: Vec<String> = metered
                .list(crate::sealed::INSIDE)?
                .into_iter()
                .map(|one| one.name)
                .filter(|name| !sealed.is_mark(name))
                .collect();
            for (done, name) in names.iter().enumerate() {
                if !still(done)? {
                    forget(&self.home);
                    return Ok(Stepped::GaveUp);
                }
                deleted(&metered, name)?;
            }
            if word(&metered)?.1 == read {
                sealed.unmark()?;
            }
        }
        if left_as_it_was && !self.journal.on {
            // Another machine's copies are not listed here, so every plain one its sealed twin still stands for goes.
            let copies: Vec<String> = match self.journal.orphan {
                true => standing_twins(&metered, &self.sealed())?,
                false => twins(&self.home).into_keys().collect(),
            };
            for (done, name) in copies.iter().enumerate() {
                if !still(done)? {
                    break;
                }
                deleted(&metered, name)?;
            }
        }
        forget(&self.home);
        Ok(Stepped::GaveUp)
    }

    fn owed(&self, from: &dyn Remote, to: &dyn Remote) -> Result<Owed, Hitch> {
        let folds = from.limits().folds_case;
        let (room, held) = (to.limits().most_per_file, from.limits().most_per_file);
        let listed = |side: &dyn Remote| -> Result<BTreeMap<String, Seen>, Hitch> {
            Ok(side
                .list("")?
                .into_iter()
                .filter(|one| ours_to_turn(&one.name, folds))
                .map(|one| (one.name.clone(), one))
                .collect())
        };
        let (sources, there) = (listed(from)?, listed(to)?);
        let twins = twins(&self.home);
        let mut owed = Owed {
            copy: Vec::new(),
            // One too big for the side read from was never there to copy, so it is never taken for gone.
            gone: there
                .iter()
                .filter(|(name, one)| !sources.contains_key(*name) && one.bytes <= held)
                .map(|(name, _)| name.clone())
                .collect(),
            refused: BTreeMap::new(),
            done: 0,
        };
        for (name, source) in sources {
            if source.bytes > room {
                if there.contains_key(&name) {
                    owed.gone.push(name.clone());
                }
                owed.refused.insert(name, source.bytes);
                continue;
            }
            let twin = twins.get(&name);
            let same = there
                .get(&name)
                .is_some_and(|one| one.bytes == source.bytes)
                && twin
                    .is_some_and(|twin| twin.from == source.revision && twin.bytes == source.bytes);
            match same {
                true => owed.done += 1,
                false => owed.copy.push(source),
            }
        }
        Ok(owed)
    }

    fn copied(&self, from: &dyn Remote, to: &dyn Remote, owed: &mut Owed) -> Result<(), Hitch> {
        let gone = owed.gone.len().min(BATCH);
        for name in owed.gone.drain(..gone).collect::<Vec<_>>() {
            deleted(to, &name)?;
        }
        let mut carried = 0;
        let copy = owed
            .copy
            .iter()
            .take(BATCH - gone)
            .take_while(|one| {
                let room = carried < BATCH_BYTES;
                carried += one.bytes;
                room
            })
            .count();
        for source in owed.copy.drain(..copy).collect::<Vec<_>>() {
            let held = crate::temp::Temp::new();
            let mut file = held.create().map_err(broke)?;
            let got = match from.fetch(&source.name, 0, &mut file) {
                Ok(got) => got,
                Err(Hitch::Missing(_)) => continue,
                Err(other) => return Err(other),
            };
            drop(file);
            to.put(&source.name, &held.0, Expect::Any)?;
            twinned(
                &self.home,
                &Twin {
                    name: source.name.clone(),
                    from: got.revision,
                    bytes: got.bytes,
                },
            )?;
            owed.done += 1;
        }
        Ok(())
    }

    fn opens_one(&self, sealed: &dyn Remote) -> Result<(), Hitch> {
        let folds = sealed.limits().folds_case;
        let smallest = sealed
            .list(crate::STORE)?
            .into_iter()
            .filter(|one| ours_to_turn(&one.name, folds))
            .min_by_key(|one| one.bytes);
        match smallest {
            Some(one) => sealed.fetch(&one.name, 0, &mut std::io::sink()).map(|_| ()),
            None => Ok(()),
        }
    }
}

// Only a name whose twin stands with the same size goes, and the word is read again every batch in case a turn began.
pub fn wipe_behind(
    plain: Arc<dyn Remote>,
    home: &Path,
    key: CloudKey,
    cloud: &str,
) -> Result<usize, Hitch> {
    let _held = locked(home)?;
    let metered = Metered::over(&*plain, home, Class::Sealing);
    let (text, revision) = word(&metered)?;
    let sealed = sealed_over(&*plain, home, key, cloud);
    let (behind, kept): (&dyn Remote, &dyn Remote) = match tisty_sync::sealing_in(&text) {
        Sealing::Sealed(said) if said == cloud => (&metered, &sealed),
        Sealing::Plain | Sealing::Unsaid => (&sealed, &metered),
        _ => return Err(turned_away(cloud)),
    };
    let mut gone: usize = 0;
    for name in standing_twins(behind, kept)? {
        if gone.is_multiple_of(WIPED_BETWEEN_LOOKS) && word(&metered)?.1 != revision {
            return Err(turned_away(cloud));
        }
        deleted(behind, &name)?;
        gone += 1;
    }
    Ok(gone)
}

fn standing_twins(behind: &dyn Remote, kept: &dyn Remote) -> Result<Vec<String>, Hitch> {
    let folds = behind.limits().folds_case;
    let standing: BTreeMap<String, u64> = kept
        .list("")?
        .into_iter()
        .filter(|one| ours_to_turn(&one.name, folds))
        .map(|one| (one.name, one.bytes))
        .collect();
    Ok(behind
        .list("")?
        .into_iter()
        .filter(|one| ours_to_turn(&one.name, folds) && standing.get(&one.name) == Some(&one.bytes))
        .map(|one| one.name)
        .collect())
}

pub fn turn_size(plain: &dyn Remote) -> Result<u64, Hitch> {
    let folds = plain.limits().folds_case;
    Ok(plain
        .list("")?
        .into_iter()
        .filter(|one| ours_to_turn(&one.name, folds))
        .map(|one| one.bytes)
        .sum())
}

// The nonce says when a claim was made and the turn stirs the word as it works, so the word tells how long it stood still.
pub fn idle_for(text: &str) -> Option<u64> {
    let claim = tisty_sync::claim_in(text)?;
    let (_, nonce) = claim.split_once(':')?;
    let made = u64::try_from(nonce.parse::<u128>().ok()? / 1_000_000_000).unwrap_or(u64::MAX);
    let last = tisty_sync::stirred_in(text).map_or(made, |stirred| stirred.max(made));
    Some(now().saturating_sub(last))
}

pub fn turn_of(text: &str) -> Option<Turn> {
    match tisty_sync::sealing_in(text) {
        Sealing::Copying(turn) => Some(turn),
        Sealing::Closed => Some(match tisty_sync::sealed_in(text) {
            Some(_) => Turn::Off,
            None => Turn::On,
        }),
        _ => None,
    }
}
