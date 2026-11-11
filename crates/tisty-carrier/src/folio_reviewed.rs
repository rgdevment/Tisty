use std::collections::BTreeMap;
use std::path::Path;

const KEPT: &str = "folio-reviewed.json";

fn all(home: &Path) -> BTreeMap<String, u64> {
    std::fs::read(home.join(KEPT))
        .ok()
        .and_then(|said| serde_json::from_slice(&said).ok())
        .unwrap_or_default()
}

/// A word below the highest one seen here is an older one served again, so its reviews never count (166).
pub fn seen(home: &Path, anchor: &str) -> u64 {
    all(home).get(anchor).copied().unwrap_or(0)
}

pub fn saw(home: &Path, anchor: &str, seq: u64) {
    let mut all = all(home);
    if all.get(anchor).is_some_and(|was| *was >= seq) {
        return;
    }
    all.insert(anchor.to_owned(), seq);
    if let Ok(said) = serde_json::to_vec(&all) {
        let _ = std::fs::create_dir_all(home);
        let _ = tisty_core::store::write_atomic(&home.join(KEPT), &said);
    }
}

#[cfg(test)]
#[path = "folio_reviewed_test.rs"]
mod tests;
