use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tisty_core::signing::SigningKey;
use tisty_sync::{NAMED_AT_MOST, carry_through, forget_carried_to, forget_shape};

use crate::{
    Brought, Carrier, Changes, Deferred, Elsewhere, Here, Holding, Keep, Kin, LetGo, Moved, Reason,
    Remote, Round, STORE, Seen, Signed, Stitched, Told, Trouble, Watch,
};

mod asking;
mod budget;
mod index;
mod judge;
pub(crate) mod lock;
mod mirror;
mod pace;
pub mod policy;
mod resting;
pub mod sealing;
mod shelf;
pub mod standing;
mod wiping;

use budget::{Class, Metered};
use index::Index;
use judge::{Judged, judge};
use pace::Pace;
use shelf::{Letting, Shelf};

const TREE: &str = "tree";
const FORGOTTEN: &str = "forgotten";
const FORGETTING: &str = "forgotten.working";
const FORGOTTEN_ASIDE: &str = "forgotten.aside.";
const ASIDE_LIST: &str = ".list";
const PATIENCE: Duration = Duration::from_secs(5);
const DELETE_TRIES: u32 = 4;
const CARRIED_LATELY: u64 = 60 * 60;
const WAITED_AT_MOST: Duration = Duration::from_secs(60);
const ASKED_WAITED_OUT: Duration = Duration::from_secs(30);
const QUIET_STRETCH: u32 = 4;
const STIRRING: &str = "stirring.json";
// Written when the store up there was found deleted whole, so only then does sending it all again start the mirror over.
const EMPTIED: &str = "emptied-there";

#[derive(Clone, Copy)]
struct Turn {
    waits: bool,
    sends: bool,
    homes: bool,
}

const ASKED: Turn = Turn {
    waits: true,
    sends: true,
    homes: false,
};
const PASSING: Turn = Turn {
    waits: false,
    sends: true,
    homes: false,
};
const HOMING: Turn = Turn {
    waits: false,
    sends: false,
    homes: true,
};
const BINNING: Turn = Turn {
    waits: true,
    sends: false,
    homes: false,
};

struct Done<T> {
    value: Option<T>,
    deferred: Option<Deferred>,
    stopped: Option<Trouble>,
    uploaded: usize,
}

pub struct Cloud {
    remote: Arc<dyn Remote>,
    home: PathBuf,
    tried: AtomicBool,
    marked: AtomicBool,
    root: String,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Stir {
    asked_ms: u64,
    every_ms: u64,
    mark: u64,
}

impl Cloud {
    fn carried_lately(&self) -> bool {
        standing::last_round(&self.home).is_some_and(|last| {
            last.code == "ok" && policy::now().saturating_sub(last.at) < CARRIED_LATELY
        })
    }

    pub fn over(remote: Arc<dyn Remote>, home: impl Into<PathBuf>) -> Self {
        Self {
            root: remote.root().to_string(),
            remote,
            home: home.into(),
            tried: AtomicBool::new(false),
            marked: AtomicBool::new(false),
        }
    }

    fn index(&self) -> Index {
        let index = Index::load(&self.home);
        match index.root == self.root {
            true => index,
            false => Index {
                root: self.root.clone(),
                ..Index::default()
            },
        }
    }

    fn tree(&self) -> PathBuf {
        standing::mirrored_at(&self.home)
    }

    // Unconfirmed again, the next round forgets the shape and the marks it left here and runs as a first one.
    // A new root asks for the mirror again, but this machine has still carried to this cloud and is no newcomer.
    fn started_over(&self, why: &'static str, carried_stays: bool) -> Result<(), Trouble> {
        self.marked.store(false, Ordering::Relaxed);
        tisty_core::witness::note(tisty_core::witness::channel::CLOUD, why, &[]);
        let broke = |e: std::io::Error| Trouble::Broke(e.to_string());
        index::forget(&self.home).map_err(broke)?;
        match std::fs::remove_dir_all(self.tree()) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(broke(e)),
            _ => {}
        }
        for left in [
            standing::OWED_BACK,
            standing::CARRIED,
            standing::LAST_ROUND,
            EMPTIED,
        ]
        .into_iter()
        .filter(|left| !(carried_stays && *left == standing::CARRIED))
        {
            let _ = std::fs::remove_file(self.home.join(left));
        }
        Ok(())
    }

    // Asked again whenever the mirror starts over, so a machine with another phrase never sends into this cloud.
    fn marked_up(&self, remote: &dyn Remote) -> Result<(), crate::Hitch> {
        if crate::sealed::reading(&self.root).is_none() || self.marked.load(Ordering::Relaxed) {
            return Ok(());
        }
        remote.mark_up()?;
        self.marked.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn announced(&self, remote: &dyn Remote, tree: &Path) -> Result<(), crate::Hitch> {
        let Some(cloud) = crate::sealed::reading(&self.root) else {
            return Ok(());
        };
        self.marked_up(remote)?;
        tisty_sync::seal_word(tree, cloud).map_err(|e| crate::Hitch::Broke(e.to_string()))?;
        match tisty_sync::sealing_of(tree) {
            tisty_sync::Sealing::Sealed(said) if said == cloud => Ok(()),
            tisty_sync::Sealing::Copying(tisty_sync::Turn::Off) => Ok(()),
            tisty_sync::Sealing::Copying(tisty_sync::Turn::On) | tisty_sync::Sealing::Closed => {
                Err(crate::Hitch::Barred(Trouble::Sealing(
                    tree.display().to_string(),
                )))
            }
            tisty_sync::Sealing::Sealed(_) => Err(crate::Hitch::Barred(Trouble::Sealed(
                tree.display().to_string(),
            ))),
            tisty_sync::Sealing::Plain | tisty_sync::Sealing::Unsaid => Err(crate::Hitch::Barred(
                Trouble::Unsealed(tree.display().to_string()),
            )),
        }
    }

    fn with_mirror<T>(
        &self,
        here: Option<&Here>,
        turn: Turn,
        work: impl FnOnce(&Path, &mut Index, &dyn Remote) -> Result<(T, Option<Deferred>), Trouble>,
    ) -> Result<Done<T>, Trouble> {
        self.with_mirror_along(here, turn, false, &mut |_| {}, |tree, index, remote, _| {
            work(tree, index, remote)
        })
    }

    fn with_mirror_along<T>(
        &self,
        here: Option<&Here>,
        turn: Turn,
        again: bool,
        along: &mut dyn FnMut(crate::Reached),
        work: impl FnOnce(
            &Path,
            &mut Index,
            &dyn Remote,
            &mut dyn FnMut(crate::Reached),
        ) -> Result<(T, Option<Deferred>), Trouble>,
    ) -> Result<Done<T>, Trouble> {
        // What the person asked for is tried anyway; only the window's own pace waits.
        if let Some(left) = resting::left(&self.home).filter(|_| !turn.waits) {
            return Ok(Done::deferred(Reason::Limit, Some(left), None));
        }
        let taken = match turn.waits {
            true => lock::Round::wait_for(&self.home, PATIENCE),
            false => lock::Round::take(&self.home),
        };
        let _round = match taken {
            Ok(Some(round)) => round,
            Ok(None) => return Ok(Done::deferred(Reason::Busy, None, None)),
            Err(why) => return Err(Trouble::Broke(why.to_string())),
        };
        let emptied = self.home.join(EMPTIED);
        if let Some(stale) = index::stale(&self.home, &self.root) {
            match stale {
                index::Stale::Layout => self.started_over(
                    "the cloud's index was of an older layout, so the mirror starts over",
                    false,
                )?,
                index::Stale::Root => self.started_over(
                    "the cloud is read from another root, so the mirror starts over",
                    true,
                )?,
            }
        } else if again && emptied.exists() {
            self.started_over(
                "everything is sent again to a store deleted up there",
                false,
            )?;
        }
        let tree = self.tree();
        std::fs::create_dir_all(&tree).map_err(|e| Trouble::Broke(e.to_string()))?;
        self.forget_asked(&tree);
        mirror::sweep(&tree);
        let metered = Metered::over(&*self.remote, &self.home, Class::History);
        let remote: &dyn Remote = &metered;
        let mut index = self.index();
        let data = here.map(|here| here.data.as_path());
        mirror::heal(&tree, &mut index);
        let pulled = mirror::pull(remote, &tree, &mut index, data, turn.homes, along, &|| {
            self.marked_up(remote)
        });
        // A store deleted whole leaves nothing to bring home, so a move away goes on, while a round here still says it is gone.
        if self.still_emptied(&pulled, &index) && (turn.homes || here.is_some()) {
            return match turn.homes {
                true => {
                    self.left_emptied(&mut index)?;
                    Ok(Done {
                        value: None,
                        deferred: None,
                        stopped: None,
                        uploaded: 0,
                    })
                }
                false => {
                    let _ = index.save(&self.home);
                    Err(Trouble::CloudEmptied {
                        provider: self.remote.limits().provider.to_string(),
                    })
                }
            };
        }
        let held_back = match pulled {
            Ok(held_back) => held_back,
            Err(hitch) => {
                let _ = index.save(&self.home);
                return match judge(hitch, self.remote.limits().provider) {
                    Judged::Pause(reason, wait, cause) => {
                        let deferred = Done::deferred(reason, wait, cause);
                        self.heeded(deferred.deferred.as_ref());
                        Ok(deferred)
                    }
                    Judged::Stop(trouble) => Err(trouble),
                };
            }
        };
        if let Some(here) = here
            && !index.confirmed
        {
            forget_shape(&here.data, &tree);
            forget_carried_to(&here.aside, &tree);
        }
        metered.as_class(Class::Attachments);
        let mut pace = Pace::new(Some(&self.home), along);
        let worked = work(&tree, &mut index, remote, pace.saying());
        metered.as_class(Class::History);
        let (value, held) = match worked {
            Ok(worked) => worked,
            Err(trouble) => {
                let _ = index.save(&self.home);
                return Err(trouble);
            }
        };
        let quiet = !turn.sends
            || held
                .as_ref()
                .is_some_and(|one| matches!(one.reason, Reason::Limit | Reason::Busy));
        let (pushed, hitch) = match quiet {
            true => (mirror::Pushed::default(), None),
            false => match self.announced(remote, &tree) {
                Ok(()) => mirror::push(remote, &tree, &mut index, &mut pace),
                Err(hitch) => (mirror::Pushed::default(), Some(hitch)),
            },
        };
        let left_there = (held_back > 0).then_some(Deferred {
            reason: Reason::Changed,
            left: held_back,
            retry_after: None,
            names: Vec::new(),
            cause: None,
        });
        let mut deferred = merged(held, left_there);
        let mut stopped = None;
        match hitch.map(|hitch| judge(hitch, remote.limits().provider)) {
            Some(Judged::Pause(reason, wait, cause)) => {
                deferred = merged(
                    deferred,
                    Some(Deferred {
                        reason,
                        left: pushed.left,
                        retry_after: wait,
                        names: Vec::new(),
                        cause,
                    }),
                );
            }
            Some(Judged::Stop(trouble)) => {
                stopped = Some(match (trouble, pushed.stuck.clone()) {
                    (Trouble::Broke(why), Some(name)) => Trouble::StuckOn { name, why },
                    (trouble, _) => trouble,
                })
            }
            None if pushed.left > 0 => {
                deferred = merged(
                    deferred,
                    Some(Deferred {
                        reason: Reason::Changed,
                        left: pushed.left,
                        retry_after: None,
                        names: Vec::new(),
                        cause: None,
                    }),
                );
            }
            None => {}
        }
        if let Err(why) = index.save(&self.home) {
            stopped = stopped.or(Some(Trouble::Broke(why.to_string())));
        }
        self.heeded(deferred.as_ref());
        Ok(Done {
            value: Some(value),
            deferred,
            stopped,
            uploaded: pushed.put,
        })
    }

    fn still_emptied(&self, pulled: &Result<usize, crate::Hitch>, index: &Index) -> bool {
        let emptied = self.home.join(EMPTIED);
        let stands = index.tree.contains_key(tisty_sync::NAMED);
        match pulled {
            Err(crate::Hitch::Emptied) => {
                let _ = std::fs::write(&emptied, b"");
            }
            Ok(_) if stands => {
                let _ = std::fs::remove_file(&emptied);
            }
            _ => {}
        }
        !stands && emptied.exists()
    }

    // Leaving takes the deletion as read, so coming back later is a first round; the cursor and what vanished stay, so nothing is owed from up there.
    fn left_emptied(&self, index: &mut Index) -> Result<(), Trouble> {
        self.started_over(
            "a store deleted up there is left, so coming back starts the mirror over",
            false,
        )?;
        let read = Index {
            cursor: index.cursor.take(),
            root: std::mem::take(&mut index.root),
            vanished: std::mem::take(&mut index.vanished),
            ..Index::default()
        };
        read.save(&self.home)
            .map_err(|e| Trouble::Broke(e.to_string()))
    }

    // The wait a provider asked for outlives this round, so the window's own pace does not ask again before it.
    fn heeded(&self, deferred: Option<&Deferred>) {
        let reason = deferred.map(|one| one.reason);
        match deferred.and_then(|one| one.retry_after) {
            Some(wait) if reason != Some(Reason::Busy) => resting::rest(&self.home, wait),
            // Only a round the provider let through ends the wait; one cut short for another reason says nothing of it.
            _ if matches!(reason, None | Some(Reason::Changed | Reason::Unsent)) => {
                resting::rested(&self.home)
            }
            _ => {}
        }
    }

    fn only<T>(
        &self,
        here: Option<&Here>,
        work: impl FnOnce(&Path, &mut Index, &dyn Remote) -> Result<T, Trouble>,
    ) -> Result<T, Trouble> {
        let done = self.with_mirror(here, ASKED, |tree, index, remote| {
            Ok((work(tree, index, remote)?, None))
        })?;
        match (done.value, done.deferred) {
            (Some(value), _) => Ok(value),
            (None, Some(spent)) if spent.reason == Reason::Budget => Err(Trouble::CloudSpent {
                provider: self.remote.limits().provider.to_string(),
            }),
            (None, deferred) => Err(Trouble::NotNow(unavailable(deferred))),
        }
    }

    fn looked_again(&self) -> PathBuf {
        let _ = self.with_mirror(None, PASSING, |_, _, _| Ok(((), None)));
        self.tree()
    }

    fn rounded(&self, here: &Here, round: Round, fetching: bool) -> Result<Moved, Trouble> {
        let turn = match fetching {
            true => HOMING,
            false => PASSING,
        };
        let Round {
            way,
            alive,
            holds,
            saying,
        } = round;
        let again = matches!(way, crate::Way::Again);
        let done = self.with_mirror_along(
            Some(here),
            turn,
            again,
            saying,
            |tree, index, remote, saying| {
                shelf::sweep(&here.data);
                shelf::flattened(remote, index);
                let history = Metered::over(&*self.remote, &self.home, Class::History);
                let shelf = match fetching {
                    true => Shelf::fetching_all(remote, index),
                    false => Shelf::over(remote, index),
                };
                let mut shelf = shelf.history_first(tree, &history).kept_in(&self.home);
                let moved = carry_through(
                    &here.data,
                    Some(&here.aside),
                    &here.device,
                    tree,
                    way,
                    alive,
                    holds,
                    saying,
                    &mut shelf,
                )?;
                Ok((
                    (moved, shelf.sent(), shelf.early(), shelf.full()),
                    shelf.paused(),
                ))
            },
        )?;
        let (mut moved, attached, early, full) = done.value.unwrap_or_default();
        let mirrored = moved.sent.saturating_sub(attached);
        moved.sent = attached + mirrored.min(done.uploaded + early);
        let unsent = moved.deferred.take();
        if let Some(trouble) = done.stopped.or(full) {
            if moved == Moved::default() {
                return Err(trouble);
            }
            moved.refused = Some(trouble);
        }
        moved.deferred = merged(unsent, done.deferred);
        Ok(moved)
    }

    fn stirred_last(&self) -> Stir {
        std::fs::read(self.home.join(STIRRING))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    fn asked_again(&self) -> u64 {
        let poll = self.remote.limits().poll_every;
        let at = self.home.join(STIRRING);
        let mut stir = self.stirred_last();
        let now = now_ms();
        if stir.asked_ms > 0 && now >= stir.asked_ms && now - stir.asked_ms < stir.every_ms {
            return stir.mark;
        }
        if resting::left(&self.home).is_some() {
            return stir.mark;
        }
        stir.asked_ms = now;
        let every = Duration::from_millis(stir.every_ms);
        let index = self.index();
        let metered = Metered::over(&*self.remote, &self.home, Class::History);
        let asked = if mirror::barred(&metered, &self.tree(), &index).is_some() {
            mirror::word_moved(&metered, &index).map(|moved| {
                let moved = moved && !index.owed.contains_key(tisty_sync::NAMED);
                if moved {
                    stir.mark = stir.mark.wrapping_add(1);
                }
                moved
            })
        } else {
            metered.changes(index.cursor.as_deref()).map(|changes| {
                let news = news(&index, &changes);
                if news {
                    stir.mark = marked(&changes);
                }
                news
            })
        };
        let next = match asked {
            Ok(true) => poll,
            Ok(false) => (every * 2).clamp(poll, poll * QUIET_STRETCH),
            Err(crate::Hitch::Limited { wait }) => {
                resting::rest(&self.home, wait);
                wait.max(poll)
            }
            Err(_) => every.max(poll),
        };
        stir.every_ms = u64::try_from(next.as_millis()).unwrap_or(u64::MAX);
        if let Ok(bytes) = serde_json::to_vec(&stir) {
            let _ = std::fs::create_dir_all(&self.home);
            let _ = tisty_core::store::write_atomic(&at, &bytes);
        }
        stir.mark
    }

    fn fresh(&self) -> PathBuf {
        if !self.index().pulled() && !self.tried.swap(true, Ordering::Relaxed) {
            let _ = self.with_mirror(None, PASSING, |_, _, _| Ok(((), None)));
        }
        self.tree()
    }

    fn forget_asked(&self, tree: &Path) {
        let (list, working) = (self.home.join(FORGOTTEN), self.home.join(FORGETTING));
        for turn in 0..2 {
            if turn == 1 {
                // Renamed only while no writer holds the list open, or its line lands in a file nobody reads.
                let Ok(Some(_writing)) =
                    lock::Held::wait_for(&self.home, lock::FORGETTING, PATIENCE)
                else {
                    continue;
                };
                let _ = std::fs::rename(&list, &working);
            }
            let Ok(text) = std::fs::read_to_string(&working) else {
                continue;
            };
            for id in text.lines().filter(|line| !line.is_empty()) {
                tisty_sync::forget_paper(tree, id);
            }
            let _ = std::fs::remove_file(&working);
        }
        for aside in asked_aside(&self.home) {
            let Ok(text) = std::fs::read_to_string(&aside) else {
                continue;
            };
            for id in text.lines().filter(|line| !line.is_empty()) {
                tisty_sync::forget_paper(tree, id);
            }
            let _ = std::fs::remove_file(&aside);
        }
    }
}

impl<T> Done<T> {
    fn deferred(reason: Reason, wait: Option<Duration>, cause: Option<crate::Cause>) -> Self {
        Self {
            value: None,
            deferred: Some(Deferred {
                reason,
                left: 0,
                retry_after: wait,
                names: Vec::new(),
                cause,
            }),
            stopped: None,
            uploaded: 0,
        }
    }
}

fn seconds(at: SystemTime) -> u64 {
    at.duration_since(UNIX_EPOCH).map_or(0, |one| one.as_secs())
}

fn idle(here: &Here, unused_since: SystemTime) -> Result<BTreeSet<String>, Trouble> {
    let since = seconds(unused_since);
    let opened = tisty_core::lately::begun(&here.data.join(tisty_core::lately::USED))
        .map_err(|e| Trouble::Broke(e.to_string()))?;
    let begun = opened
        .get(tisty_core::lately::SINCE)
        .copied()
        .unwrap_or(u64::MAX);
    let events = tisty_core::store::read_all(here.data.join(STORE))
        .map_err(|e| Trouble::Broke(e.to_string()))?;
    let mut arrived: BTreeMap<&str, u64> = BTreeMap::new();
    for event in &events {
        if let tisty_core::Op::AttachKept { d } = &event.op {
            let when = u64::try_from(event.timestamp.as_second()).unwrap_or(0);
            let last = arrived.entry(d.at.as_str()).or_default();
            *last = (*last).max(when);
        }
    }
    Ok(tisty_core::State::replay(&events)
        .kept
        .into_keys()
        .filter(|reference| {
            let used = opened.get(reference).copied().unwrap_or(0);
            let came = arrived.get(reference.as_str()).copied().unwrap_or(0);
            used.max(came).max(begun) < since
        })
        .collect())
}

// What the person can do least about speaks: a spent day outlasts a file that changed mid-round.
fn weight(reason: Reason) -> u8 {
    match reason {
        Reason::Paused => 7,
        Reason::Budget => 6,
        Reason::Offline => 5,
        Reason::Troubled => 4,
        Reason::Limit => 3,
        Reason::Busy => 2,
        Reason::Changed => 1,
        // Lightest, so a round left unfinished for any other reason is never read as one that only left files here.
        Reason::Unsent => 0,
    }
}

fn merged(first: Option<Deferred>, second: Option<Deferred>) -> Option<Deferred> {
    match (first, second) {
        (Some(one), Some(other)) => Some(Deferred {
            reason: match weight(other.reason) > weight(one.reason) {
                true => other.reason,
                false => one.reason,
            },
            cause: match weight(other.reason) > weight(one.reason) {
                true => other.cause,
                false => one.cause,
            },
            left: one.left + other.left,
            retry_after: one.retry_after.max(other.retry_after),
            names: one
                .names
                .into_iter()
                .chain(other.names)
                .take(NAMED_AT_MOST)
                .collect(),
        }),
        (one, other) => one.or(other),
    }
}

fn unavailable(deferred: Option<Deferred>) -> String {
    match deferred {
        Some(one) => format!("the cloud cannot be used now: {:?}", one.reason),
        None => "the cloud cannot be used now".to_string(),
    }
}

impl Carrier for Cloud {
    fn place(&self) -> Option<&Path> {
        None
    }

    fn frees_space(&self) -> bool {
        true
    }

    fn lends_from(&self) -> Option<PathBuf> {
        None
    }

    fn brought(&self, here: &Here, reference: &str) -> Result<Brought, Trouble> {
        // A body names an attachment by its leaf; the shelf, the log and the cloud name it whole.
        let Some(whole) = tisty_core::attach::canonical(reference) else {
            return Ok(Brought::Absent);
        };
        let reference = whole.as_str();
        let state = crate::shift::logged(here);
        if tisty_sync::buried_now(&state, &here.data).contains(reference) {
            return Ok(Brought::Absent);
        }
        let logged = state.kept.get(reference).map(|(sha256, _)| sha256.as_str());
        let mut index = self.index();
        let home = tisty_core::attach::resolve(reference, &here.data).is_ok_and(|at| at.is_file());
        // Read again before saying it is gone, as a machine may have put it back since.
        if !home && index.vanished.contains(reference) {
            self.looked_again();
            index = self.index();
        }
        if !home && index.spoiled(reference) {
            return Err(Trouble::Broke(format!(
                "{reference} up there does not hold the bytes its name and the log vouch for"
            )));
        }
        if !home && index.pulled() && !index.shelf.contains_key(reference) {
            let gone = index.vanished.contains(reference) && !index.owed.contains_key(reference);
            return Ok(if gone { Brought::Gone } else { Brought::Later });
        }
        let metered = Metered::over(&*self.remote, &self.home, Class::Asked);
        match shelf::brought(&metered, &here.data, reference, logged) {
            Ok(Some((at, None))) => Ok(Brought::At(at)),
            Ok(Some((at, Some((sha256, bytes))))) => Ok(Brought::Unvouched { at, sha256, bytes }),
            Ok(None) => Ok(Brought::Absent),
            Err(hitch) => match judge(hitch, self.remote.limits().provider) {
                Judged::Pause(..) => Ok(Brought::Later),
                Judged::Stop(trouble) => Err(trouble),
            },
        }
    }

    fn shelf(&self) -> Option<std::collections::BTreeMap<String, u64>> {
        let index = self.index();
        index.pulled().then(|| {
            index
                .shelf
                .iter()
                .filter(|(reference, _)| !index.spoiled(reference))
                .map(|(reference, seen)| (reference.clone(), seen.bytes))
                .collect()
        })
    }

    fn reachable(&self) -> bool {
        true
    }

    fn theirs(&self) -> Option<String> {
        tisty_sync::theirs(&self.fresh())
    }

    fn later(&self) -> Option<Trouble> {
        mirror::barred(&*self.remote, &self.tree(), &self.index())
    }

    fn been_here(&self, here: &Here) -> bool {
        tisty_sync::been_here(&here.aside, &self.tree(), &here.device)
    }

    fn kin(&self, here: &Here) -> Kin {
        tisty_sync::kinship(&here.data.join(STORE), &self.looked_again())
    }

    fn others(&self, here: &Here) -> Vec<String> {
        tisty_sync::others_in(&self.fresh(), &here.device)
    }

    fn room(&self) -> Option<u64> {
        match self.remote.room() {
            Ok(room) => room,
            Err(hitch) => {
                tisty_core::witness::note(
                    tisty_core::witness::channel::CLOUD,
                    "how much the cloud can still take could not be asked",
                    &[
                        ("kind", tisty_core::witness::Fact::Code(hitch.code())),
                        ("why", tisty_core::witness::Fact::Why(format!("{hitch:?}"))),
                    ],
                );
                None
            }
        }
    }

    fn signed(&self) -> Signed {
        tisty_sync::signed_here(&self.looked_again())
    }

    fn stirring(&self) -> u64 {
        self.asked_again()
    }

    fn hears(&self) -> Option<Box<dyn Watch>> {
        let index = self.index();
        let pushed = index.cursor.as_deref().and_then(|cursor| {
            Metered::over(&*self.remote, &self.home, Class::History).hears(cursor)
        });
        Some(pushed.unwrap_or_else(|| asking::Asking::of(self)))
    }

    fn unclaimed(&self) -> Holding {
        tisty_sync::unclaimed(&self.fresh())
    }

    fn carry(&self, here: &Here, round: Round) -> Result<Moved, Trouble> {
        self.rounded(here, round, false)
    }

    fn bring_home(&self, here: &Here, round: Round) -> Result<Moved, Trouble> {
        let round = Round {
            way: crate::Way::Pull,
            holds: tisty_core::config::Holds::Everywhere,
            ..round
        };
        self.rounded(here, round, true)
    }

    fn could_free(&self, here: &Here, unused_since: SystemTime) -> LetGo {
        let Ok(idle) = idle(here, unused_since) else {
            return LetGo::default();
        };
        let letting = Letting {
            chosen: &|reference, _| idle.contains(reference),
            elsewhere: &|_| false,
            told: &mut |_| true,
            only_counting: true,
        };
        shelf::let_go(&*self.remote, &self.index(), &here.data, letting)
    }

    fn set_free(
        &self,
        here: &Here,
        unused_since: SystemTime,
        told: Told,
    ) -> Result<LetGo, Trouble> {
        self.only(Some(here), |_, index, remote| {
            let idle = idle(here, unused_since)?;
            let used = here.data.join(tisty_core::lately::USED);
            let since = seconds(unused_since);
            // Asked again right before each one goes, so an attachment opened meanwhile stays.
            let unused_now = |reference: &str| {
                tisty_core::lately::reached(&used, reference).is_none_or(|when| when < since)
            };
            let letting = Letting {
                chosen: &|reference, _| idle.contains(reference) && unused_now(reference),
                elsewhere: &|_| false,
                told,
                only_counting: false,
            };
            Ok(shelf::let_go(remote, index, &here.data, letting))
        })
    }

    fn stitch(&self, here: &Here, key: Option<SigningKey>) -> Result<Stitched, Trouble> {
        self.only(Some(here), |tree, _, _| {
            tisty_sync::stitch(&here.data, &here.device, tree, key)
        })
    }

    fn let_go(
        &self,
        here: &Here,
        above: u64,
        elsewhere: Elsewhere,
        told: Told,
    ) -> Result<LetGo, Trouble> {
        self.only(Some(here), |_, index, remote| {
            let letting = Letting {
                chosen: &|_, len| len > above,
                elsewhere,
                told,
                only_counting: false,
            };
            Ok(shelf::let_go(remote, index, &here.data, letting))
        })
    }

    fn paper_waiting(&self, id: &str) -> bool {
        tisty_sync::paper_waiting(&self.fresh(), id)
    }

    fn paper_print(&self, id: &str) -> Option<String> {
        tisty_sync::held_there(&self.fresh(), id)
    }

    fn both_papers(&self, here: &Here, id: &str) -> Result<(String, String), Trouble> {
        self.only(Some(here), |tree, _, _| {
            tisty_sync::both_papers(&here.data, tree, id)
        })
    }

    fn settle(
        &self,
        here: &Here,
        id: &str,
        keep: Keep,
        seen: Option<&str>,
    ) -> Result<Option<String>, Trouble> {
        self.only(Some(here), |tree, _, _| {
            tisty_sync::settle(&here.data, tree, id, keep, seen)
        })
    }

    // A push that failed only says so in `stopped` or `deferred`, so the word counts once the provider hands it back.
    fn leave_word(&self, said: &str) -> Result<(), Trouble> {
        let done = self.with_mirror(None, ASKED, |tree, _, _| {
            tisty_sync::raise_place(tree);
            tisty_core::store::write_atomic(&tree.join(tisty_core::moved::NAMED), said.as_bytes())
                .map_err(|e| Trouble::Broke(e.to_string()))?;
            Ok(((), None))
        })?;
        if done.value.is_none() {
            return Err(Trouble::NotNow(unavailable(done.deferred)));
        }
        let mut there = Vec::new();
        let metered = Metered::over(&*self.remote, &self.home, Class::History);
        match metered.fetch(tisty_core::moved::NAMED, 0, &mut there) {
            Ok(_) if there == said.as_bytes() => Ok(()),
            Ok(_) | Err(crate::Hitch::Missing(_)) => Err(done
                .stopped
                .unwrap_or_else(|| Trouble::NotNow(unavailable(done.deferred)))),
            Err(hitch) => Err(match judge(hitch, self.remote.limits().provider) {
                Judged::Pause(..) => Trouble::NotNow(unavailable(done.deferred)),
                Judged::Stop(trouble) => trouble,
            }),
        }
    }

    fn only_word_left(&self) -> bool {
        self.remote.list("").is_ok_and(|all| {
            !all.is_empty() && all.iter().all(|one| one.name == tisty_core::moved::NAMED)
        })
    }

    // Read after a pull, so a machine stops before the round that would write there alone.
    fn word_left(&self) -> Option<String> {
        std::fs::read_to_string(self.looked_again().join(tisty_core::moved::NAMED)).ok()
    }

    // Only a document missing here is taken for deleted up there, so the word is taken down by name.
    fn forget_word(&self) -> Result<(), Trouble> {
        self.only(None, |tree, index, remote| {
            let _ = std::fs::remove_file(tree.join(tisty_core::moved::NAMED));
            match remote.delete(tisty_core::moved::NAMED, None) {
                Ok(()) | Err(crate::Hitch::Missing(_)) => {
                    index.tree.remove(tisty_core::moved::NAMED);
                    Ok(())
                }
                Err(hitch) => Err(Trouble::Broke(format!("{hitch:?}"))),
            }
        })
    }

    fn holds_a_store(&self) -> bool {
        tisty_core::store::inhabited(self.tree().join(STORE))
    }

    // Unmetered: a cloud not chosen yet keeps no spending of its own on this disk.
    fn peeks_a_store(&self, ours: bool) -> Option<bool> {
        if ours && self.carried_lately() {
            return Some(self.holds_a_store());
        }
        self.remote
            .about(&format!("{STORE}/{}", tisty_sync::MARKER))
            .ok()
            .map(|seen| seen.is_some())
    }

    fn wipe_along(&self, along: &mut dyn FnMut(usize, usize)) -> Result<usize, Trouble> {
        let provider = self.remote.limits().provider;
        let stopped = |hitch| match judge(hitch, provider) {
            Judged::Pause(reason, ..) => Trouble::NotNow(format!("{reason:?}")),
            Judged::Stop(trouble) => trouble,
        };
        let done = self.with_mirror(None, BINNING, |_, index, remote| {
            let all = remote.list("").map_err(stopped)?;
            if !all.iter().any(|one| one.name == tisty_core::moved::NAMED) {
                return Err(Trouble::NotThere(tisty_core::moved::NAMED.to_string()));
            }
            let going: Vec<&Seen> = all
                .iter()
                .filter(|one| one.name != tisty_core::moved::NAMED)
                .collect();
            let mut gone = 0;
            along(gone, going.len());
            let ended = wiping::deleted_all(
                remote,
                &going,
                &mut |name| {
                    index.tree.remove(name);
                    index.shelf.remove(name);
                    gone += 1;
                    along(gone, going.len());
                },
                |remote, one| match patiently(|| remote.delete(&one.name, Some(&one.revision))) {
                    Ok(()) | Err(crate::Hitch::Missing(_)) => Ok(()),
                    Err(hitch) => Err(hitch),
                },
            );
            ended.map_err(stopped)?;
            Ok((gone, None))
        })?;
        let gone = done
            .value
            .ok_or_else(|| Trouble::NotNow(unavailable(done.deferred)))?;
        let _ = std::fs::remove_dir_all(&self.home);
        Ok(gone)
    }

    fn forget_paper(&self, id: &str) {
        if !forgotten_later(&self.home, id) {
            return;
        }
        if let Ok(Some(_round)) = lock::Round::take(&self.home) {
            self.forget_asked(&self.tree());
        }
    }
}

/// What the last listing of a cloud held and this machine does not, read off this disk alone, for a cloud that can no longer be asked.
pub fn only_up_there(home: &Path, here: &Here) -> crate::Owed {
    let index = Index::load(home);
    let shelf: BTreeMap<String, u64> = index
        .shelf
        .iter()
        .map(|(reference, seen)| (reference.clone(), seen.bytes))
        .collect();
    crate::shift::owed_against(index.pulled().then_some(&shelf), here, u64::MAX)
}

/// Queued without the provider, so a cloud that cannot be reached now still forgets it at its next round.
pub fn forgotten_later(home: &Path, id: &str) -> bool {
    if tisty_core::docs::resolve(Path::new(""), id).is_err()
        || std::fs::create_dir_all(home).is_err()
    {
        return false;
    }
    let Ok(Some(_writing)) = lock::Held::wait_for(home, lock::FORGETTING, ASKED_WAITED_OUT) else {
        // Never appended unheld, where a round taking the list could read it before the line lands.
        let aside = format!(
            "{FORGOTTEN_ASIDE}{}-{}{ASIDE_LIST}",
            std::process::id(),
            now_ms()
        );
        return tisty_core::store::write_atomic(&home.join(aside), format!("{id}\n").as_bytes())
            .is_ok();
    };
    let asked = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(home.join(FORGOTTEN));
    match asked {
        Ok(mut list) => writeln!(list, "{id}").is_ok(),
        Err(_) => false,
    }
}

fn asked_aside(home: &Path) -> Vec<PathBuf> {
    let Ok(all) = std::fs::read_dir(home) else {
        return Vec::new();
    };
    all.filter_map(|one| one.ok())
        .filter(|one| {
            one.file_name().to_str().is_some_and(|named| {
                named.starts_with(FORGOTTEN_ASIDE) && named.ends_with(ASIDE_LIST)
            })
        })
        .map(|one| one.path())
        .collect()
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
}

fn news(index: &Index, changes: &Changes) -> bool {
    let ours = |name: &str| mirror::mirrored(name) || index::on_shelf(name);
    let held = |name: &str| index.tree.contains_key(name) || index.shelf.contains_key(name);
    let known = |one: &Seen| {
        index.tree.get(&one.name).map(|was| &was.seen.revision) == Some(&one.revision)
            || index.shelf.get(&one.name).map(|was| &was.revision) == Some(&one.revision)
            || index.owed.get(&one.name).map(|was| &was.revision) == Some(&one.revision)
    };
    match changes {
        Changes::Since { changed, gone, .. } => {
            changed.iter().any(|one| ours(&one.name) && !known(one))
                || gone.iter().any(|name| held(name))
        }
        Changes::Whole { seen, .. } => {
            let listed: std::collections::BTreeSet<&str> =
                seen.iter().map(|one| one.name.as_str()).collect();
            seen.iter().any(|one| ours(&one.name) && !known(one))
                || index
                    .tree
                    .keys()
                    .chain(index.shelf.keys())
                    .any(|name| !listed.contains(name.as_str()))
        }
    }
}

fn marked(changes: &Changes) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut said = std::collections::hash_map::DefaultHasher::new();
    match changes {
        Changes::Whole { seen, .. } => {
            let mut all: Vec<(&str, &str)> = seen
                .iter()
                .map(|one| (one.name.as_str(), one.revision.as_str()))
                .collect();
            all.sort_unstable();
            all.hash(&mut said);
        }
        Changes::Since { cursor, .. } => cursor.hash(&mut said),
    }
    said.finish()
}

// One slow answer from the provider would otherwise stop a wipe of hundreds of files halfway.
fn patiently(mut once: impl FnMut() -> Result<(), crate::Hitch>) -> Result<(), crate::Hitch> {
    let mut failed = 0;
    loop {
        let wait = match once() {
            Err(crate::Hitch::Unreachable(..) | crate::Hitch::Troubled(_))
                if failed + 1 < DELETE_TRIES =>
            {
                Duration::from_secs(2u64.pow(failed))
            }
            Err(crate::Hitch::Limited { wait } | crate::Hitch::Busy { wait })
                if failed + 1 < DELETE_TRIES && wait <= WAITED_AT_MOST =>
            {
                wait
            }
            other => return other,
        };
        failed += 1;
        std::thread::sleep(wait);
    }
}

#[cfg(test)]
#[path = "cloud_test.rs"]
mod tests;
