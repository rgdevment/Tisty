use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

const KEPT: &str = "folio-reviewed.json";
const ASKED: &str = "folio-languages-asked.json";
const PRINTED: &str = "folio-languages-printed.json";
const LOOKED: &str = "folio-languages-looked.json";
const COPIES: &str = "folio-kept-copies";
const DAY: u64 = 24 * 60 * 60;
const QUARTER: u64 = 15 * 60;

fn all<T: DeserializeOwned>(home: &Path, name: &str) -> BTreeMap<String, T> {
    std::fs::read(home.join(name))
        .ok()
        .and_then(|said| serde_json::from_slice(&said).ok())
        .unwrap_or_default()
}

fn kept<T: Serialize>(home: &Path, name: &str, all: &BTreeMap<String, T>) {
    if let Ok(said) = serde_json::to_vec(all) {
        let _ = std::fs::create_dir_all(home);
        let _ = tisty_core::store::write_atomic(&home.join(name), &said);
    }
}

/// A word below the highest one seen here is an older one served again, so its reviews never count (166).
pub fn seen(home: &Path, anchor: &str) -> u64 {
    all::<u64>(home, KEPT).get(anchor).copied().unwrap_or(0)
}

pub fn saw(home: &Path, anchor: &str, seq: u64) {
    let mut seen = all::<u64>(home, KEPT);
    if seen.get(anchor).is_some_and(|was| *was >= seq) {
        return;
    }
    seen.insert(anchor.to_owned(), seq);
    kept(home, KEPT, &seen);
}

fn due(home: &Path, name: &str, anchor: &str, now: u64, every: u64) -> bool {
    all::<u64>(home, name)
        .get(anchor)
        .is_none_or(|at| now.saturating_sub(*at) >= every)
}

fn asked(home: &Path, name: &str, anchor: &str, now: u64) {
    let mut asked = all::<u64>(home, name);
    asked.insert(anchor.to_owned(), now);
    kept(home, name, &asked);
}

// A language others added rarely leaves Folio, so the owner asks after them once a day, not on every press.
pub fn languages_due(home: &Path, anchor: &str, now: u64) -> bool {
    due(home, ASKED, anchor, now, DAY)
}

pub fn languages_asked(home: &Path, anchor: &str, now: u64) {
    asked(home, ASKED, anchor, now);
}

// The row is drawn each time a document is opened, so the versions it keeps are asked after at most every quarter of an hour.
pub fn prints_due(home: &Path, anchor: &str, now: u64) -> bool {
    due(home, LOOKED, anchor, now, QUARTER)
}

pub fn prints_asked(home: &Path, anchor: &str, now: u64) {
    asked(home, LOOKED, anchor, now);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Printed {
    pub version: u64,
    pub print: String,
}

pub fn prints(home: &Path) -> BTreeMap<String, Printed> {
    all(home, PRINTED)
}

pub fn printed(home: &Path, document: &str) -> Option<Printed> {
    prints(home).remove(document)
}

pub fn print_known(home: &Path, document: &str, now: Printed) {
    let mut known = all::<Printed>(home, PRINTED);
    if known
        .get(document)
        .is_some_and(|was| was.version > now.version || *was == now)
    {
        return;
    }
    known.insert(document.to_owned(), now);
    kept(home, PRINTED, &known);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Copy {
    pub print: String,
    pub title: Option<String>,
    pub body: String,
}

// A document's id names its file, so one that could reach outside the folder never does.
fn copy_at(home: &Path, document: &str) -> Option<PathBuf> {
    let plain = !document.is_empty() && document.chars().all(|one| one.is_ascii_alphanumeric());
    plain.then(|| home.join(COPIES).join(format!("{document}.json")))
}

pub fn copy_kept(home: &Path, document: &str, copy: &Copy) {
    let (Some(at), Ok(said)) = (copy_at(home, document), serde_json::to_vec(copy)) else {
        return;
    };
    let _ = std::fs::create_dir_all(home.join(COPIES));
    let _ = tisty_core::store::write_atomic(&at, &said);
}

pub fn kept_copy(home: &Path, document: &str) -> Option<Copy> {
    let said = std::fs::read(copy_at(home, document)?).ok()?;
    serde_json::from_slice(&said).ok()
}

pub fn copy_dropped(home: &Path, document: &str) {
    if let Some(at) = copy_at(home, document) {
        let _ = std::fs::remove_file(at);
    }
}

// Someone else's words stay here only while their language is still shared with this machine.
pub fn forgotten(home: &Path, document: &str) {
    copy_dropped(home, document);
    let mut known = prints(home);
    if known.remove(document).is_some() {
        kept(home, PRINTED, &known);
    }
}

#[cfg(test)]
#[path = "folio_reviewed_test.rs"]
mod tests;
