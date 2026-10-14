use std::path::Path;

use super::budget::Spent;
use super::index::Index;
use super::{FORGOTTEN, policy};
use crate::{BYTES_A_DAY, UNITS_A_DAY};

/// Another machine's bytes a provider still has to put back, written by the provider beside the mirror.
pub const OWED_BACK: &str = "owed-back.json";
pub(super) const LAST_ROUND: &str = "last-round.json";
pub(super) const CARRIED: &str = "carried-here";

/// How the last round ended, in the words the window already reads: `ok`, a refusal's code, or `deferred-<reason>`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastRound {
    pub at: u64,
    pub code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub left: usize,
    #[serde(default)]
    pub names: Vec<String>,
}

/// A cloud chosen anew starts with no word of a round it never ran.
pub fn round_forgotten(home: &Path) {
    let _ = std::fs::remove_file(home.join(LAST_ROUND));
}

/// Only this machine's own trace counts: a first round mirrors what it pulls before it ever sends.
pub fn cloud_joined(home: &Path, here: &crate::Here) -> bool {
    home.join(CARRIED).exists()
        || tisty_sync::been_here(&here.aside, &home.join(super::TREE), &here.device)
}

/// How this machine's history stands to the place as last mirrored here, asking the provider nothing.
pub fn kin_mirrored(home: &Path, here: &crate::Here) -> crate::Kin {
    tisty_sync::kinship(&here.data.join(tisty_sync::STORE), &home.join(super::TREE))
}

pub fn round_noted(home: &Path, round: &LastRound) {
    // Kept beside the mirror rather than in the cache, which the system may empty.
    if round.code == "ok" && !home.join(CARRIED).exists() {
        let _ = std::fs::create_dir_all(home)
            .and_then(|_| std::fs::write(home.join(CARRIED), round.at.to_string()));
    }
    let written = serde_json::to_vec(round)
        .map_err(std::io::Error::other)
        .and_then(|bytes| {
            std::fs::create_dir_all(home)?;
            tisty_core::store::write_atomic(&home.join(LAST_ROUND), &bytes)
                .map_err(std::io::Error::other)
        });
    if let Err(why) = written {
        tisty_core::witness::warn(
            tisty_core::witness::channel::CLOUD,
            "how the last round ended could not be written down",
            &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
        );
    }
}

/// What `tisty doctor` says about a cloud, read off this disk alone: no request, no keychain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Standing {
    pub listed: bool,
    pub confirmed: bool,
    pub mirrored: usize,
    pub on_the_shelf: usize,
    pub owed: usize,
    pub forgotten: usize,
    pub owed_back: usize,
    pub units_spent: u64,
    pub bytes_spent: u64,
    pub units_a_day: u64,
    pub bytes_a_day: u64,
    pub paused_here: bool,
    pub halted_until: Option<u64>,
    pub serial: Option<u64>,
    pub last: Option<LastRound>,
}

pub fn standing(home: &Path) -> Standing {
    let index = Index::load(home);
    let spent = Spent::load(home);
    let kept = policy::kept(home);
    let now = policy::now();
    let forgotten = std::fs::read_to_string(home.join(FORGOTTEN))
        .map(|said| said.lines().filter(|one| !one.trim().is_empty()).count())
        .unwrap_or(0);
    let owed_back = std::fs::read(home.join(OWED_BACK))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Vec<serde_json::Value>>(&bytes).ok())
        .map_or(0, |all| all.len());
    Standing {
        listed: index.pulled(),
        confirmed: index.confirmed,
        mirrored: index.tree.len(),
        on_the_shelf: index.shelf.len(),
        owed: index.owed.len(),
        forgotten,
        owed_back,
        units_spent: spent.units,
        bytes_spent: spent.bytes,
        units_a_day: policy::within(UNITS_A_DAY, kept.as_ref().and_then(|one| one.units_a_day)),
        bytes_a_day: policy::within(BYTES_A_DAY, kept.as_ref().and_then(|one| one.bytes_a_day)),
        paused_here: policy::paused_here(home),
        halted_until: kept
            .as_ref()
            .filter(|one| policy::halted(Some(one), now))
            .and_then(|one| one.halted_until),
        serial: kept.map(|one| one.serial),
        last: std::fs::read(home.join(LAST_ROUND))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok()),
    }
}

#[cfg(test)]
#[path = "standing_test.rs"]
mod tests;
