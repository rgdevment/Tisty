use std::collections::BTreeMap;
use std::path::Path;

use crate::witness::{self, Fact, channel};

pub const USED: &str = "used.json";
/// When the file was begun: everything counts as used then, so a lost or broken file frees nothing.
pub const SINCE: &str = "*";

const KEPT_AT_MOST: usize = 4096;
const NO_SOONER_THAN: u64 = 60 * 60 * 24;

pub type Used = BTreeMap<String, u64>;

pub fn every(at: &Path) -> Used {
    let mut seen = read(at).unwrap_or_default();
    seen.remove(SINCE);
    seen
}

fn read(at: &Path) -> std::io::Result<Used> {
    match std::fs::read_to_string(at) {
        // Days written under a shelf name are the same attachment's under its one name now.
        Ok(said) => serde_json::from_str::<Used>(&said)
            .map(|seen| {
                seen.into_iter()
                    .map(|(named, when)| (plain(&named), when))
                    .collect()
            })
            .map_err(std::io::Error::other),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Used::default()),
        Err(e) => Err(e),
    }
}

/// What is written down, with the day it was begun; a file that cannot be read is set aside and begun again.
pub fn begun(at: &Path) -> std::io::Result<Used> {
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let guard = std::fs::File::create(at.with_extension("lock"))?;
    guard.lock()?;
    let seen = held(at)?;
    Ok(seen)
}

fn held(at: &Path) -> std::io::Result<Used> {
    let mut seen = read(at).unwrap_or_else(|e| {
        told_unwritten(at, &e.to_string());
        let _ = std::fs::rename(at, at.with_extension("unreadable"));
        Used::default()
    });
    if !seen.contains_key(SINCE) {
        seen.insert(SINCE.to_string(), now());
        written(at, &seen)?;
    }
    Ok(seen)
}

pub fn last(at: &Path, reference: &str) -> Option<u64> {
    every(at).get(&plain(reference)).copied()
}

/// The day it counts as reached: its own, or the day the file began or forgot older ones, whichever is later.
pub fn reached(at: &Path, reference: &str) -> Option<u64> {
    let seen = read(at).unwrap_or_default();
    [seen.get(&plain(reference)), seen.get(SINCE)]
        .into_iter()
        .flatten()
        .copied()
        .max()
}

pub fn used(at: &Path, reference: &str) {
    used_on(at, reference, now());
}

pub fn used_on(at: &Path, reference: &str, when: u64) {
    if let Some(parent) = at.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let guard = std::fs::File::create(at.with_extension("lock"));
    let _locked = guard.as_ref().ok().filter(|one| one.lock().is_ok());
    let mut seen = match held(at) {
        Ok(seen) => seen,
        Err(e) => return told_unwritten(at, &e.to_string()),
    };
    let reference = plain(reference);
    if seen
        .get(&reference)
        .is_some_and(|said| *said <= when && when - said < NO_SOONER_THAN)
    {
        return;
    }
    seen.insert(reference, when);
    while seen.len() > KEPT_AT_MOST + 1 {
        let Some(oldest) = seen
            .iter()
            .filter(|one| one.0 != SINCE)
            .min_by(|one, two| one.1.cmp(two.1).then_with(|| one.0.cmp(two.0)))
            .map(|(named, when)| (named.clone(), *when))
        else {
            break;
        };
        seen.remove(&oldest.0);
        // What is forgotten counts as reached as late as the last one forgotten, so freeing never takes it.
        let since = seen.entry(SINCE.to_string()).or_insert(oldest.1);
        *since = (*since).max(oldest.1);
    }
    if let Err(e) = written(at, &seen) {
        told_unwritten(at, &e.to_string());
    }
}

fn written(at: &Path, seen: &Used) -> std::io::Result<()> {
    let body = serde_json::to_vec(seen).map_err(std::io::Error::other)?;
    crate::store::write_atomic(at, &body).map_err(std::io::Error::other)
}

fn plain(reference: &str) -> String {
    crate::attach::canonical(reference).unwrap_or_else(|| {
        reference
            .split(['#', '?'])
            .next()
            .unwrap_or(reference)
            .to_string()
    })
}

fn told_unwritten(at: &Path, why: &str) {
    witness::warn(
        channel::ATTACH,
        "when an attachment was last reached for could not be written down",
        &[
            ("at", Fact::Path(at.to_path_buf())),
            ("why", Fact::Why(why.to_string())),
        ],
    );
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|one| one.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "lately_test.rs"]
mod tests;
