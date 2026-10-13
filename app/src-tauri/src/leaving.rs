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
    owed_all(cache).iter().any(|owed| {
        tisty_carrier::cloud_of(Some(&owed.way)).is_some_and(|at| at.account == account)
    })
}

fn owed_all(cache: &Path) -> Vec<Owed> {
    let Ok(bytes) = std::fs::read(cache.join(OWED)) else {
        return Vec::new();
    };
    serde_json::from_slice::<Vec<Owed>>(&bytes)
        .or_else(|_| serde_json::from_slice::<Owed>(&bytes).map(|one| vec![one]))
        .unwrap_or_default()
}

fn owed_kept(cache: &Path, all: &[Owed]) {
    let at = cache.join(OWED);
    if all.is_empty() {
        let _ = std::fs::remove_file(at);
        return;
    }
    let written = serde_json::to_vec(all)
        .map_err(std::io::Error::other)
        .and_then(|bytes| {
            std::fs::create_dir_all(cache)?;
            tisty_core::store::write_atomic(&at, &bytes).map_err(std::io::Error::other)
        });
    if let Err(why) = written {
        witness::warn(
            channel::SYNC,
            "the word to the machines left behind could not even be kept for later",
            &[("why", Fact::Why(why.to_string()))],
        );
    }
}

fn owe(cache: &Path, left: &Sync, said: &str) {
    let mut all = owed_all(cache);
    all.retain(|owed| !same_place(&owed.way, left));
    all.push(Owed {
        way: left.clone(),
        said: said.to_string(),
    });
    owed_kept(cache, &all);
}

pub fn owed_no_more(cache: &Path, chosen: &Sync) {
    let mut all = owed_all(cache);
    let before = all.len();
    all.retain(|owed| !same_place(&owed.way, chosen));
    if all.len() != before {
        owed_kept(cache, &all);
    }
}

pub struct Owing {
    cache: PathBuf,
    way: Sync,
    old: tisty_carrier::Shared,
    said: String,
    release: Option<String>,
}

pub fn owing(paths: &tisty_core::Paths, reach: &Reaching, now: Option<&Sync>) -> Option<Owing> {
    let cache = paths.cache().to_path_buf();
    let mut all = owed_all(&cache);
    let before = all.len();
    all.retain(|owed| {
        owed.way.leaving() != tisty_core::config::Leaving::Free
            && !now.is_some_and(|now| same_place(&owed.way, now))
    });
    if all.len() != before {
        owed_kept(&cache, &all);
    }
    let heading_back = crate::answers::relocating::pending_cloud(paths);
    all.into_iter().find_map(|owed| {
        let old = match tisty_carrier::cloud_of(Some(&owed.way)) {
            Some(at) => reach.apart(at),
            None => tisty_carrier::chosen_with(Some(&owed.way), Some(reach)).carrier,
        }?;
        let release = tisty_carrier::cloud_of(Some(&owed.way))
            .map(|at| at.account.clone())
            .filter(|account| heading_back.as_ref() != Some(account));
        Some(Owing {
            cache: cache.clone(),
            way: owed.way,
            old,
            said: owed.said,
            release,
        })
    })
}

impl Owing {
    pub fn delivered(self) -> bool {
        let reached = self.old.leave_word(&self.said).is_ok();
        if reached {
            let mut all = owed_all(&self.cache);
            all.retain(|owed| !same_place(&owed.way, &self.way));
            owed_kept(&self.cache, &all);
            if let Some(account) = &self.release {
                crate::answers::clouding::let_go_of(account);
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
