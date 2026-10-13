use std::path::{Path, PathBuf};

use tisty_core::config::Sync;
use tisty_core::witness::{self, Fact, channel};

use crate::reaching::Reaching;
use crate::{Answer, Refusal};

use tisty_core::moved::OWED;

#[derive(serde::Serialize, serde::Deserialize)]
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

pub fn backed_up(paths: &tisty_core::Paths) -> Answer<PathBuf> {
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
    paths: &tisty_core::Paths,
    config: &tisty_core::Config,
    left: &Sync,
    old: Option<tisty_carrier::Shared>,
    chosen: &Sync,
) {
    if let Some(said) = word_for(paths, config, chosen) {
        left_said(paths.cache(), left, &said, old);
    }
}

pub fn left_said(
    cache: &Path,
    left: &Sync,
    said: &str,
    old: Option<tisty_carrier::Shared>,
) -> bool {
    let reached = old.is_some_and(|old| old.leave_word(said).is_ok());
    if !reached {
        owe(cache, left, said);
    }
    reached
}

pub fn word_for(
    paths: &tisty_core::Paths,
    config: &tisty_core::Config,
    chosen: &Sync,
) -> Option<String> {
    let key = tisty_core::signing::mine(paths, &config.device_id)?;
    Some(tisty_core::moved::said(
        &key,
        &config.device_id,
        &where_to(chosen),
        jiff::Timestamp::now().as_second().unsigned_abs(),
    ))
}

pub fn owed_to(cache: &Path, account: &str) -> bool {
    std::fs::read(cache.join(OWED))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Owed>(&bytes).ok())
        .is_some_and(|owed| {
            tisty_carrier::cloud_of(Some(&owed.way)).is_some_and(|at| at.account == account)
        })
}

fn owe(cache: &Path, left: &Sync, said: &str) {
    let owed = Owed {
        way: left.clone(),
        said: said.to_string(),
    };
    let written = serde_json::to_vec(&owed)
        .map_err(std::io::Error::other)
        .and_then(|bytes| {
            std::fs::create_dir_all(cache)?;
            tisty_core::store::write_atomic(&cache.join(OWED), &bytes)
                .map_err(std::io::Error::other)
        });
    if let Err(why) = written {
        witness::warn(
            channel::SYNC,
            "the word to the machines left behind could not even be kept for later",
            &[("why", Fact::Why(why.to_string()))],
        );
    }
}

/// Coming back to the place a word was owed to means there is nothing to tell it any more.
pub fn owed_no_more(cache: &Path, chosen: &Sync) {
    let at = cache.join(OWED);
    let back = std::fs::read(&at)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Owed>(&bytes).ok())
        .is_some_and(|owed| same_place(&owed.way, chosen));
    if back {
        let _ = std::fs::remove_file(at);
    }
}

/// A word still owed to the place left behind, with what reaches it; built under the session, delivered off it.
pub struct Owing {
    at: PathBuf,
    old: tisty_carrier::Shared,
    said: String,
    way: Sync,
}

pub fn owing(cache: &Path, reach: &Reaching, now: Option<&Sync>) -> Option<Owing> {
    let at = cache.join(OWED);
    let owed: Owed = serde_json::from_slice(&std::fs::read(&at).ok()?).ok()?;
    let nobody_there = owed.way.leaving() == tisty_core::config::Leaving::Free;
    if nobody_there || now.is_some_and(|now| same_place(&owed.way, now)) {
        let _ = std::fs::remove_file(&at);
        return None;
    }
    let old = tisty_carrier::chosen_with(Some(&owed.way), Some(reach)).carrier?;
    Some(Owing {
        at,
        old,
        said: owed.said,
        way: owed.way,
    })
}

impl Owing {
    pub fn delivered(self) -> bool {
        let reached = self.old.leave_word(&self.said).is_ok();
        if reached {
            let _ = std::fs::remove_file(self.at);
            if let Some(at) = tisty_carrier::cloud_of(Some(&self.way)) {
                crate::answers::clouding::let_go_of(&at.account);
            }
        }
        reached
    }
}

/// A machine the person confirmed here moved away from this place, so this one stops instead of syncing alone.
pub fn heeded(said: Option<&str>, data: &Path, me: &tisty_core::DeviceId) -> Option<Refusal> {
    let word = tisty_core::moved::heeded(said, data, me)?;
    Some(match word.to.split_once(':') {
        Some(("cloud", email)) => Refusal::about("movedToCloud", email.to_string()),
        Some(("folder", named)) => Refusal::about("movedAway", named.to_string()),
        _ => Refusal::of("movedAlone"),
    })
}

#[cfg(test)]
#[path = "leaving_test.rs"]
mod tests;
