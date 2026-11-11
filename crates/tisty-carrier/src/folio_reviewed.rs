use std::collections::BTreeMap;
use std::path::Path;

const KEPT: &str = "folio-reviewed.json";
const ASKED: &str = "folio-languages-asked.json";
const DAY: u64 = 24 * 60 * 60;

fn all(home: &Path, name: &str) -> BTreeMap<String, u64> {
    std::fs::read(home.join(name))
        .ok()
        .and_then(|said| serde_json::from_slice(&said).ok())
        .unwrap_or_default()
}

fn kept(home: &Path, name: &str, all: &BTreeMap<String, u64>) {
    if let Ok(said) = serde_json::to_vec(all) {
        let _ = std::fs::create_dir_all(home);
        let _ = tisty_core::store::write_atomic(&home.join(name), &said);
    }
}

/// A word below the highest one seen here is an older one served again, so its reviews never count (166).
pub fn seen(home: &Path, anchor: &str) -> u64 {
    all(home, KEPT).get(anchor).copied().unwrap_or(0)
}

pub fn saw(home: &Path, anchor: &str, seq: u64) {
    let mut seen = all(home, KEPT);
    if seen.get(anchor).is_some_and(|was| *was >= seq) {
        return;
    }
    seen.insert(anchor.to_owned(), seq);
    kept(home, KEPT, &seen);
}

// A language others added rarely leaves Folio, so the owner asks after them once a day, not on every press.
pub fn languages_due(home: &Path, anchor: &str, now: u64) -> bool {
    all(home, ASKED)
        .get(anchor)
        .is_none_or(|at| now.saturating_sub(*at) >= DAY)
}

pub fn languages_asked(home: &Path, anchor: &str, now: u64) {
    let mut asked = all(home, ASKED);
    asked.insert(anchor.to_owned(), now);
    kept(home, ASKED, &asked);
}

#[cfg(test)]
#[path = "folio_reviewed_test.rs"]
mod tests;
