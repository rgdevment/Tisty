use std::collections::BTreeSet;
use std::path::Path;

use crate::{STORE, written};
use tisty_core::paths::{is_the_one, told_of};

pub(crate) const CARRIED_TO: &str = "carried-to";
const ADOPTING: &str = "adopting";

// A memo another identity left, as a reinstall that kept the cache does, is not being here.
pub(crate) fn carried_here(aside: Option<&Path>, dest: &Path, device: Option<&str>) -> bool {
    let Some(aside) = aside else {
        return false;
    };
    let Ok(last) = std::fs::read_to_string(aside.join(CARRIED_TO)) else {
        return false;
    };
    let (at, by) = match last.rsplit_once("\nby ") {
        Some((at, by)) => (at, Some(by.trim())),
        None => (last.as_str(), None),
    };
    is_the_one(at, dest)
        && match (by, device) {
            (Some(by), Some(device)) => by.eq_ignore_ascii_case(device),
            _ => true,
        }
}

pub fn forget_carried_to(aside: &Path, dest: &Path) {
    if carried_here(Some(aside), dest, None) {
        let _ = std::fs::remove_file(aside.join(CARRIED_TO));
    }
}

pub(crate) fn note_carried(aside: Option<&Path>, dest: &Path, device: &str) {
    let Some(aside) = aside else {
        return;
    };
    if std::fs::create_dir_all(aside).is_ok() {
        let _ = written(
            &aside.join(CARRIED_TO),
            format!("{}\nby {device}", told_of(dest)).as_bytes(),
        );
    }
}

pub(crate) fn names_in(dest: &Path) -> BTreeSet<String> {
    let Ok(entries) = std::fs::read_dir(dest.join(STORE)) else {
        return BTreeSet::new();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .filter(|named| tisty_core::store::is_device_name(named))
        .collect()
}

/// A machine still in the cloud on joining would otherwise wait for the confirmation joining gave.
pub(crate) fn still_adopting(aside: Option<&Path>, dest: &Path) -> BTreeSet<String> {
    let Some(kept) = aside.and_then(|at| std::fs::read_to_string(at.join(ADOPTING)).ok()) else {
        return BTreeSet::new();
    };
    match kept.split_once('\n') {
        Some((names, at)) if is_the_one(at, dest) => {
            names.split_whitespace().map(str::to_string).collect()
        }
        _ => BTreeSet::new(),
    }
}

pub(crate) fn keep_adopting(aside: Option<&Path>, dest: &Path, names: &BTreeSet<String>) {
    let Some(aside) = aside else {
        return;
    };
    let at = aside.join(ADOPTING);
    if names.is_empty() {
        let _ = std::fs::remove_file(at);
        return;
    }
    if std::fs::create_dir_all(aside).is_ok() {
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let _ = written(
            &at,
            format!("{}\n{}", names.join(" "), told_of(dest)).as_bytes(),
        );
    }
}
