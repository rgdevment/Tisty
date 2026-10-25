use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tisty_share_proto::cloud::CloudKey;
use tisty_sync::{NAMED, Sealing, Trouble, Turn};

use super::budget::{Class, Metered};
use super::lock::{Held, SEALING};
use crate::{Expect, Hitch, Remote, Seen};

const JOURNAL: &str = "sealing.json";
const TWINS: &str = "sealing.twins";
const BATCH: usize = 64;
const SAID: u32 = 1;

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
    started: u64,
    moved: u64,
    #[serde(default)]
    refused: BTreeMap<String, u64>,
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
    Settling { changed: usize },
    Done,
}

/// A cloud turning sealed, one bounded step at a time, every step kept in a journal beside the mirror.
pub struct Turning {
    plain: Arc<dyn Remote>,
    home: PathBuf,
    key: CloudKey,
    journal: Journal,
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |at| at.as_secs())
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

impl Turning {
    /// Claims the cloud for a turn: the word says so first, and turning on puts the mark up under `sealed/`.
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
            let metered = Metered::over(&*plain, home, Class::Sealing);
            if turn == Turn::On && metered.about(tisty_core::moved::NAMED)?.is_some() {
                return Err(Hitch::Barred(Trouble::Refused(
                    "a place that was left is never sealed".into(),
                )));
            }
            let (text, _) = word(&metered)?;
            match (turn, tisty_sync::sealing_in(&text)) {
                (Turn::On, Sealing::Plain | Sealing::Unsaid) => {}
                (Turn::Off, Sealing::Sealed(said)) if said == cloud => {}
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
            started: now(),
            moved: now(),
            refused: BTreeMap::new(),
        };
        forget(home);
        journal.save(home)?;
        let mut turning = Self {
            plain,
            home: home.to_path_buf(),
            key,
            journal,
        };
        turning.step()?;
        Ok(turning)
    }

    /// A turning a cut left half way, taken up again from its journal.
    pub fn resume(plain: Arc<dyn Remote>, home: &Path, key: CloudKey) -> Option<Self> {
        let journal = Journal::load(home)?;
        (key.named_for(&journal.cloud) == journal.named).then(|| Self {
            plain,
            home: home.to_path_buf(),
            key,
            journal,
        })
    }

    /// The cloud and the key id the signed `cloud.sealed` must name, written by the app once the claim stands.
    pub fn said(&self) -> (&str, &str) {
        (&self.journal.cloud, &self.journal.named)
    }

    pub fn turn(&self) -> Turn {
        match self.journal.on {
            true => Turn::On,
            false => Turn::Off,
        }
    }

    /// When the last step moved, so a claim nobody carries on can be told apart.
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
        let ours = |text: &str| text.contains(&claim);
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
            (Phase::Claiming, Sealing::Copying(went)) if went == way && ours(&text) => {
                return self.held_claim(&metered);
            }
            (Phase::Claimed | Phase::Copied, Sealing::Copying(went))
                if went == way && ours(&text) => {}
            (Phase::Closed | Phase::Settled, Sealing::Closed) if ours(&text) => {}
            _ => return Err(turned_away(&cloud)),
        }
        let sealed = sealed_over(&*plain, &home, self.key.clone(), &cloud);
        let (from, to): (&dyn Remote, &dyn Remote) = match on {
            true => (&metered, &sealed),
            false => (&sealed, &metered),
        };
        match self.journal.phase {
            Phase::Claiming | Phase::GivingUp => Err(turned_away(&cloud)),
            Phase::Claimed => {
                let owed = self.owed(from, to)?;
                self.journal.refused = owed.refused.clone();
                if owed.copy.is_empty() && owed.gone.is_empty() {
                    self.journal.phase = Phase::Copied;
                    self.journal.save(&self.home)?;
                    return Ok(Stepped::Copying {
                        done: owed.done,
                        left: 0,
                    });
                }
                let left = owed.copy.len() + owed.gone.len();
                let moved = self.copied(from, to, owed)?;
                self.journal.save(&self.home)?;
                Ok(Stepped::Copying {
                    done: moved,
                    left: left.saturating_sub(moved),
                })
            }
            Phase::Copied => {
                let closed = tisty_sync::closed(&text, &claim, tisty_sync::TURNS)
                    .ok_or_else(|| turned_away(&cloud))?;
                rewrite(&metered, &closed, &revision)?;
                self.journal.phase = Phase::Closed;
                self.journal.save(&self.home)?;
                Ok(Stepped::Closed)
            }
            Phase::Closed => {
                let owed = self.owed(from, to)?;
                self.journal.refused = owed.refused.clone();
                if owed.copy.is_empty() && owed.gone.is_empty() {
                    if on {
                        self.opens_one(&sealed)?;
                    }
                    self.journal.phase = Phase::Settled;
                    self.journal.save(&self.home)?;
                    return Ok(Stepped::Settling { changed: 0 });
                }
                let changed = self.copied(from, to, owed)?;
                self.journal.save(&self.home)?;
                Ok(Stepped::Settling { changed })
            }
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
        }
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
        if !back.contains(&self.journal.claim()) {
            return Err(turned_away(&self.journal.cloud));
        }
        if self.journal.on {
            self.sealed().mark_it()?;
        }
        self.journal.phase = Phase::Claimed;
        self.journal.save(&self.home)?;
        Ok(Stepped::Claimed)
    }

    /// Before the switch the side read from was never touched, so giving up takes back the claim and what was copied.
    pub fn abort(mut self) -> Result<(), Hitch> {
        {
            let _held = locked(&self.home)?;
            let metered = Metered::over(&*self.plain, &self.home, Class::Sealing);
            let (text, _) = word(&metered)?;
            let switched = match tisty_sync::sealing_in(&text) {
                Sealing::Sealed(cloud) => self.journal.on && cloud == self.journal.cloud,
                Sealing::Plain | Sealing::Unsaid => {
                    !self.journal.on && self.journal.phase == Phase::Settled
                }
                _ => false,
            };
            if switched {
                return Err(turned_away(&self.journal.cloud));
            }
            self.journal.phase = Phase::GivingUp;
            self.journal.save(&self.home)?;
        }
        self.step().map(|_| ())
    }

    // Kept in the journal before anything is undone, so a cut half way is undone the rest of the way next time.
    fn given_up(&self) -> Result<Stepped, Hitch> {
        let metered = Metered::over(&*self.plain, &self.home, Class::Sealing);
        let (text, revision) = word(&metered)?;
        if let Some(back) = tisty_sync::dropped(&text, &self.journal.claim()) {
            rewrite(&metered, &back, &revision)?;
        }
        if self.journal.on {
            let sealed = self.sealed();
            for one in metered.list(crate::sealed::INSIDE)? {
                if sealed.is_mark(&one.name) {
                    continue;
                }
                match metered.delete(&one.name, None) {
                    Ok(()) | Err(Hitch::Missing(_)) => {}
                    Err(other) => return Err(other),
                }
            }
            sealed.unmark()?;
        }
        forget(&self.home);
        Ok(Stepped::Done)
    }

    fn owed(&self, from: &dyn Remote, to: &dyn Remote) -> Result<Owed, Hitch> {
        let folds = from.limits().folds_case;
        let room = to.limits().most_per_file;
        let sources: BTreeMap<String, Seen> = from
            .list("")?
            .into_iter()
            .filter(|one| ours_to_turn(&one.name, folds))
            .map(|one| (one.name.clone(), one))
            .collect();
        let there: BTreeMap<String, Seen> = to
            .list("")?
            .into_iter()
            .filter(|one| ours_to_turn(&one.name, folds))
            .map(|one| (one.name.clone(), one))
            .collect();
        let twins = twins(&self.home);
        let mut owed = Owed {
            copy: Vec::new(),
            gone: there
                .keys()
                .filter(|name| !sources.contains_key(*name))
                .cloned()
                .collect(),
            refused: BTreeMap::new(),
            done: 0,
        };
        for (name, source) in sources {
            if source.bytes > room {
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

    fn copied(&self, from: &dyn Remote, to: &dyn Remote, owed: Owed) -> Result<usize, Hitch> {
        let mut moved = 0;
        for name in owed.gone.iter().take(BATCH) {
            match to.delete(name, None) {
                Ok(()) | Err(Hitch::Missing(_)) => moved += 1,
                Err(other) => return Err(other),
            }
        }
        for source in owed.copy.iter().take(BATCH.saturating_sub(moved)) {
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
            moved += 1;
        }
        Ok(moved)
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

/// What the turn left behind, once the place says it is over: the plain copy a sealed cloud no longer reads,
/// or the sealed copy a plain one no longer reads. Only a name whose twin stands on the other side goes.
pub fn wipe_behind(
    plain: Arc<dyn Remote>,
    home: &Path,
    key: CloudKey,
    cloud: &str,
) -> Result<usize, Hitch> {
    let _held = locked(home)?;
    let metered = Metered::over(&*plain, home, Class::Sealing);
    let (text, _) = word(&metered)?;
    let sealed = sealed_over(&*plain, home, key, cloud);
    let (behind, kept): (&dyn Remote, &dyn Remote) = match tisty_sync::sealing_in(&text) {
        Sealing::Sealed(said) if said == cloud => (&metered, &sealed),
        Sealing::Plain | Sealing::Unsaid => (&sealed, &metered),
        _ => return Err(turned_away(cloud)),
    };
    let folds = metered.limits().folds_case;
    let standing: BTreeMap<String, u64> = kept
        .list("")?
        .into_iter()
        .filter(|one| ours_to_turn(&one.name, folds))
        .map(|one| (one.name, one.bytes))
        .collect();
    let mut gone = 0;
    for one in behind.list("")? {
        if !ours_to_turn(&one.name, folds) || standing.get(&one.name) != Some(&one.bytes) {
            continue;
        }
        match behind.delete(&one.name, None) {
            Ok(()) | Err(Hitch::Missing(_)) => gone += 1,
            Err(other) => return Err(other),
        }
    }
    Ok(gone)
}
