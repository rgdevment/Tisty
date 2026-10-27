use std::collections::BTreeSet;
use std::path::Path;

use crate::{STORE, written};
use tisty_core::paths::{same_folder, told_of};

pub(crate) const CARRIED_TO: &str = "carried-to";
const ADOPTING: &str = "adopting";

// A memo another identity left, as a reinstall that kept the cache does, is not being here.
pub(crate) fn carried_here(
    aside: Option<&Path>,
    dest: &Path,
    device: Option<&str>,
    holds_our_store: bool,
) -> bool {
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
    same_folder(at, dest, holds_our_store)
        && match (by, device) {
            (Some(by), Some(device)) => by.eq_ignore_ascii_case(device),
            _ => true,
        }
}

pub fn forget_carried_to(aside: &Path, dest: &Path) {
    if carried_here(Some(aside), dest, None, false) {
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

pub fn let_in(dest: &Path, handed: Option<&str>) -> bool {
    matches!(crate::sealing_of(dest), crate::Sealing::Sealed(cloud) if handed == Some(cloud.as_str()))
}

/// The other machines writing in a place: the ones a word left there is meant for.
pub fn others_in(dest: &Path, device: &str) -> Vec<String> {
    names_in(dest)
        .into_iter()
        .filter(|named| !named.eq_ignore_ascii_case(device))
        .collect()
}

/// A machine still in the cloud on joining would otherwise wait for the confirmation joining gave.
pub(crate) fn still_adopting(
    aside: Option<&Path>,
    dest: &Path,
    holds_our_store: bool,
) -> BTreeSet<String> {
    match adopting_kept(aside) {
        Some((names, at, since))
            if same_folder(&at, dest, holds_our_store)
                && since.elapsed().is_ok_and(|one| one < ADOPTING_FOR) =>
        {
            names
        }
        _ => BTreeSet::new(),
    }
}

/// Past a week, a key nobody said is a person's to confirm.
const ADOPTING_FOR: std::time::Duration = std::time::Duration::from_secs(7 * 24 * 60 * 60);

type Adopting = (BTreeSet<String>, String, std::time::SystemTime);

const SINCE: &str = "since ";

/// Written before the week was kept, a list reads as begun now.
fn adopting_kept(aside: Option<&Path>) -> Option<Adopting> {
    let kept = std::fs::read_to_string(aside?.join(ADOPTING)).ok()?;
    let (names, rest) = kept.split_once('\n')?;
    let names = names.split_whitespace().map(str::to_string).collect();
    let (since, at) = match rest
        .strip_prefix(SINCE)
        .and_then(|rest| rest.split_once('\n'))
        .and_then(|(secs, at)| Some((secs.trim().parse::<u64>().ok()?, at)))
    {
        Some((secs, at)) => (
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs),
            at,
        ),
        None => (std::time::SystemTime::now(), rest),
    };
    Some((names, at.to_string(), since))
}

pub(crate) fn keep_adopting(
    aside: Option<&Path>,
    dest: &Path,
    holds_our_store: bool,
    names: &BTreeSet<String>,
) {
    let Some(aside) = aside else {
        return;
    };
    let at = aside.join(ADOPTING);
    if names.is_empty() {
        let _ = std::fs::remove_file(at);
        return;
    }
    let here = told_of(dest);
    let since = adopting_kept(Some(aside))
        .filter(|(_, was, _)| same_folder(was, dest, holds_our_store))
        .map_or_else(std::time::SystemTime::now, |(_, _, since)| since);
    let secs = since
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |one| one.as_secs());
    if std::fs::create_dir_all(aside).is_ok() {
        let names: Vec<&str> = names.iter().map(String::as_str).collect();
        let _ = written(
            &at,
            format!("{}\n{SINCE}{secs}\n{here}", names.join(" ")).as_bytes(),
        );
    }
}
