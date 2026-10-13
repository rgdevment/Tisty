use std::path::Path;

use super::budget::Spent;
use super::index::Index;
use super::{FORGOTTEN, policy};
use crate::{BYTES_A_DAY, UNITS_A_DAY};

/// Another machine's bytes a provider still has to put back, written by the provider beside the mirror.
pub const OWED_BACK: &str = "owed-back.json";

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
    }
}

#[cfg(test)]
#[path = "standing_test.rs"]
mod tests;
