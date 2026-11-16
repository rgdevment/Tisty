use std::collections::BTreeMap;
use std::path::Path;

use crate::witness::{self, Fact, channel};

pub const USED: &str = "used.json";
/// When the file was begun: everything counts as used then, so a lost or broken file frees nothing.
pub const SINCE: &str = "*";

const KEPT_AT_MOST: usize = 4096;
const NO_SOONER_THAN: u64 = 60 * 60 * 24;

pub type Used = BTreeMap<String, u64>;

type Keyed = fn(&str) -> String;

pub fn every(at: &Path) -> Used {
    every_keyed(at, plain)
}

/// The same days for what is named as it is, such as a document's file, never read as an attachment.
pub fn every_as_named(at: &Path) -> Used {
    every_keyed(at, as_named)
}

fn every_keyed(at: &Path, keyed: Keyed) -> Used {
    let mut seen = read(at, keyed).unwrap_or_default();
    seen.remove(SINCE);
    seen
}

fn read(at: &Path, keyed: Keyed) -> std::io::Result<Used> {
    match std::fs::read_to_string(at) {
        // Days written under a shelf name are the same attachment's under its one name now.
        Ok(said) => serde_json::from_str::<Used>(&said)
            .map(|seen| {
                seen.into_iter()
                    .map(|(named, when)| (keyed(&named), when))
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
    let seen = held(at, plain)?;
    Ok(seen)
}

fn held(at: &Path, keyed: Keyed) -> std::io::Result<Used> {
    let mut seen = read(at, keyed).unwrap_or_else(|e| {
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
    let seen = read(at, plain).unwrap_or_default();
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
    noted_on(at, reference, when, plain);
}

pub fn used_as_named_on(at: &Path, name: &str, when: u64) {
    noted_on(at, name, when, as_named);
}

fn noted_on(at: &Path, reference: &str, when: u64, keyed: Keyed) {
    if let Some(parent) = at.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let guard = std::fs::File::create(at.with_extension("lock"));
    let _locked = guard.as_ref().ok().filter(|one| one.lock().is_ok());
    let mut seen = match held(at, keyed) {
        Ok(seen) => seen,
        Err(e) => return told_unwritten(at, &e.to_string()),
    };
    let reference = keyed(reference);
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

fn as_named(name: &str) -> String {
    name.to_string()
}

fn told_unwritten(at: &Path, why: &str) {
    witness::warn(
        channel::ATTACH,
        "when something was last reached for could not be written down",
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
