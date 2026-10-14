use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::Manager;
use tisty_core::Paths;
use tisty_core::config::{CloudAt, Sync};
use tisty_core::witness::{self, Fact, channel};

use crate::reaching::Reaching;
use crate::{Answer, Refusal, Session};

use tisty_core::moved::OWED;

const TOLD_AGAIN: std::time::Duration = std::time::Duration::from_secs(15 * 60);

#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Owed {
    way: Sync,
    said: String,
}

/// The same account under a new address is still the same place, and nobody there is to be told otherwise.
pub fn same_place(was: &Sync, now: &Sync) -> bool {
    match (
        tisty_carrier::cloud_of(Some(was)),
        tisty_carrier::cloud_of(Some(now)),
    ) {
        (Some(was), Some(now)) => was.provider == now.provider && was.account == now.account,
        _ => was == now,
    }
}

pub fn backed_up(paths: &Paths) -> Answer<PathBuf> {
    tisty_core::backup::before_a_change(paths, jiff::Timestamp::now()).map_err(|e| {
        witness::error(
            channel::BACKUP,
            "the way of syncing was left as it was, because tasks and documents could not be kept first",
            &[("why", Fact::Why(e.to_string()))],
        );
        Refusal::of("changeUnbacked")
    })
}

// What the machines left behind are told, in words a person reads on the other side.
fn where_to(chosen: &Sync) -> String {
    if let Some(at) = tisty_carrier::cloud_of(Some(chosen)) {
        return format!("cloud:{}", at.email);
    }
    match chosen.place() {
        Some(at) => format!(
            "folder:{}",
            at.file_name().map_or_else(
                || at.display().to_string(),
                |one| one.to_string_lossy().into_owned()
            )
        ),
        None => "alone".to_string(),
    }
}

#[cfg(test)]
pub fn leave_word(
    paths: &Paths,
    config: &tisty_core::Config,
    left: &Sync,
    old: Option<tisty_carrier::Shared>,
    chosen: &Sync,
) {
    if let Some(said) = word_for(paths, config, left, chosen) {
        left_said(paths, left, &said, old);
    }
}

/// Written down as owed before it is tried, so a window closed halfway still owes it; let go only once it landed.
pub fn left_said(
    paths: &Paths,
    left: &Sync,
    said: &str,
    old: Option<tisty_carrier::Shared>,
) -> bool {
    owe(paths, left, said);
    let reached = old.is_some_and(|old| old.leave_word(said).is_ok());
    if reached {
        landed(paths, left, said);
    }
    reached
}

pub fn word_for(
    paths: &Paths,
    config: &tisty_core::Config,
    left: &Sync,
    chosen: &Sync,
) -> Option<String> {
    let key = tisty_core::signing::mine(paths, &config.device_id)?;
    Some(tisty_core::moved::said(
        &key,
        tisty_core::moved::Leaving {
            by: &config.device_id,
            to: &where_to(chosen),
            at: jiff::Timestamp::now().as_second().unsigned_abs(),
            store: &tisty_core::moved::store_of(paths.data()),
            place: &tisty_core::moved::place_of(left),
        },
    ))
}

pub fn owed_to(paths: &Paths, account: &str) -> bool {
    changing(paths, |all| {
        all.iter().any(|owed| {
            tisty_carrier::cloud_of(Some(&owed.way)).is_some_and(|at| at.account == account)
        })
    })
}

fn owed_at(paths: &Paths) -> PathBuf {
    paths.private().join(OWED)
}

static OWING: Mutex<()> = Mutex::new(());

// The timer, a round and a change of way all rewrite this list, so one at a time or a word goes missing.
fn changing<T>(paths: &Paths, change: impl FnOnce(&mut Vec<Owed>) -> T) -> T {
    let _one = OWING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (mut all, older) = owed_read(paths);
    let before = serde_json::to_vec(&all).ok();
    let done = change(&mut all);
    if older || serde_json::to_vec(&all).ok() != before {
        owed_kept(paths, &all);
    }
    done
}

// Kept in the cache by the builds before, where a restore or a reset wipes it: read there once and carried over.
fn owed_read(paths: &Paths) -> (Vec<Owed>, bool) {
    let read = |at: &Path| {
        let bytes = std::fs::read(at).ok()?;
        Some(
            serde_json::from_slice::<Vec<Owed>>(&bytes)
                .or_else(|_| serde_json::from_slice::<Owed>(&bytes).map(|one| vec![one]))
                .unwrap_or_default(),
        )
    };
    if let Some(all) = read(&owed_at(paths)) {
        return (all, false);
    }
    match read(&paths.cache().join(OWED)) {
        Some(all) => (all, true),
        None => (Vec::new(), false),
    }
}

#[cfg(test)]
fn owed_all(paths: &Paths) -> Vec<Owed> {
    changing(paths, |all| all.clone())
}

fn owed_kept(paths: &Paths, all: &[Owed]) {
    let at = owed_at(paths);
    let written = match all.is_empty() {
        true => match std::fs::remove_file(&at) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        },
        false => serde_json::to_vec(all)
            .map_err(std::io::Error::other)
            .and_then(|bytes| {
                std::fs::create_dir_all(paths.private())?;
                tisty_core::store::write_atomic(&at, &bytes).map_err(std::io::Error::other)
            }),
    };
    match written {
        Ok(()) => {
            let _ = std::fs::remove_file(paths.cache().join(OWED));
        }
        Err(why) => witness::warn(
            channel::SYNC,
            "the word to the machines left behind could not even be kept for later",
            &[("why", Fact::Why(why.to_string()))],
        ),
    }
}

pub fn owe(paths: &Paths, left: &Sync, said: &str) {
    changing(paths, |all| {
        all.retain(|owed| !same_place(&owed.way, left));
        all.push(Owed {
            way: left.clone(),
            said: said.to_string(),
        });
    });
}

// Only the word that landed goes: one written meanwhile for the same place is still owed.
fn landed(paths: &Paths, left: &Sync, said: &str) {
    changing(paths, |all| {
        all.retain(|owed| !(same_place(&owed.way, left) && owed.said == said));
    });
}

pub fn owed_no_more(paths: &Paths, chosen: &Sync) {
    changing(paths, |all| {
        all.retain(|owed| !same_place(&owed.way, chosen))
    });
}

/// A cloud nobody can reach again is owed nothing, or its word would hold its access and copies forever.
pub fn owed_no_more_to(paths: &Paths, account: &str) {
    changing(paths, |all| {
        all.retain(|owed| {
            tisty_carrier::cloud_of(Some(&owed.way)).is_none_or(|at| at.account != account)
        });
    });
}

pub struct Owing {
    paths: Paths,
    way: Sync,
    old: tisty_carrier::Shared,
    said: String,
    release: Option<CloudAt>,
}

pub fn owing(paths: &Paths, reach: &Reaching, now: Option<&Sync>) -> Vec<Owing> {
    let lost = |way: &Sync| {
        tisty_carrier::cloud_of(Some(way))
            .is_some_and(|at| crate::answers::clouding::lost_for_good_at(&reach.home_of(at), at))
    };
    let all = changing(paths, |all| {
        all.retain(|owed| {
            owed.way.leaving() != tisty_core::config::Leaving::Free
                && !now.is_some_and(|now| same_place(&owed.way, now))
                && !lost(&owed.way)
        });
        all.clone()
    });
    let heading_back = crate::answers::relocating::pending_cloud(paths);
    all.into_iter()
        .filter_map(|owed| {
            let old = match tisty_carrier::cloud_of(Some(&owed.way)) {
                Some(at) => reach.apart(at),
                None => tisty_carrier::chosen_with(Some(&owed.way), Some(reach)).carrier,
            }?;
            let release = tisty_carrier::cloud_of(Some(&owed.way))
                .cloned()
                .filter(|at| {
                    heading_back.as_ref().is_none_or(|back| {
                        back.provider != at.provider || back.account != at.account
                    })
                });
            Some(Owing {
                paths: paths.clone(),
                way: owed.way,
                old,
                said: owed.said,
                release,
            })
        })
        .collect()
}

impl Owing {
    pub fn delivered(self) -> bool {
        let reached = self.old.leave_word(&self.said).is_ok();
        if reached {
            landed(&self.paths, &self.way, &self.said);
            if let Some(at) = &self.release {
                crate::answers::clouding::let_go_of(at);
            }
        }
        reached
    }
}

/// Words owed go out at opening and every so often whatever this copy syncs with now, alone included.
pub fn keep_telling(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        loop {
            told_what_is_owed(&app);
            std::thread::sleep(TOLD_AGAIN);
        }
    });
}

fn told_what_is_owed(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<Mutex<Session>>() else {
        return;
    };
    let owed = {
        let session = crate::held(&state);
        owing(&session.paths, &session.reach, session.config.sync.as_ref())
    };
    for one in owed {
        one.delivered();
    }
}

/// A machine the person confirmed here moved away from this place, so this one stops instead of syncing alone.
pub fn heeded(
    said: Option<&str>,
    data: &Path,
    me: &tisty_core::DeviceId,
    place: &str,
) -> Option<Refusal> {
    let word = tisty_core::moved::heeded(said, data, me, place)?;
    Some(match word.to.split_once(':') {
        Some(("cloud", email)) => Refusal::about("movedToCloud", email.to_string()),
        Some(("folder", named)) => Refusal::about("movedAway", named.to_string()),
        _ => Refusal::of("movedAlone"),
    })
}

#[cfg(test)]
#[path = "leaving_test.rs"]
mod tests;
