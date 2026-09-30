use std::collections::BTreeMap;
use std::path::Path;

use crate::witness::{self, Fact, channel};

pub const USED: &str = "used.json";

const KEPT_AT_MOST: usize = 4096;
const NO_SOONER_THAN: u64 = 60 * 60 * 24;

pub type Used = BTreeMap<String, u64>;

pub fn every(at: &Path) -> Used {
    std::fs::read_to_string(at)
        .ok()
        .and_then(|said| serde_json::from_str(&said).ok())
        .unwrap_or_default()
}

pub fn last(at: &Path, reference: &str) -> Option<u64> {
    every(at).get(reference).copied()
}

pub fn used(at: &Path, reference: &str) {
    used_on(at, reference, now());
}

pub fn used_on(at: &Path, reference: &str, when: u64) {
    let mut seen = every(at);
    if seen
        .get(reference)
        .is_some_and(|said| when.saturating_sub(*said) < NO_SOONER_THAN)
    {
        return;
    }
    seen.insert(reference.to_string(), when);
    while seen.len() > KEPT_AT_MOST {
        let Some(oldest) = seen
            .iter()
            .min_by(|one, two| one.1.cmp(two.1).then_with(|| one.0.cmp(two.0)))
            .map(|(named, _)| named.clone())
        else {
            break;
        };
        seen.remove(&oldest);
    }
    let Ok(body) = serde_json::to_vec(&seen) else {
        return;
    };
    if let Some(parent) = at.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = crate::store::write_atomic(at, &body) {
        witness::warn(
            channel::ATTACH,
            "when an attachment was last reached for could not be written down",
            &[
                ("at", Fact::Path(at.to_path_buf())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
    }
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
