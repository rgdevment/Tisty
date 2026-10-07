use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub(crate) const LANDING: std::time::Duration = std::time::Duration::from_secs(60 * 60);
const REMEMBERED: std::time::Duration = std::time::Duration::from_secs(24 * 60 * 60);

/// When this machine first saw each document's body with no history answering for it yet.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Awaited {
    since: BTreeMap<String, u64>,
    checked: BTreeSet<String>,
}

fn ledger(data: &Path) -> PathBuf {
    data.join("awaited")
}

pub(crate) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

impl Awaited {
    pub(crate) fn read(data: &Path) -> Self {
        let said = std::fs::read_to_string(ledger(data)).unwrap_or_default();
        Self {
            since: said
                .lines()
                .filter_map(|line| line.split_once(' '))
                .filter_map(|(id, at)| Some((id.to_string(), at.parse().ok()?)))
                .collect(),
            checked: BTreeSet::new(),
        }
    }

    pub(crate) fn save(&self, data: &Path) {
        if self.since.is_empty() {
            let _ = std::fs::remove_file(ledger(data));
            return;
        }
        let said: String = self
            .since
            .iter()
            .map(|(id, at)| format!("{id} {at}\n"))
            .collect();
        let _ = tisty_core::store::write_atomic(&ledger(data), said.as_bytes());
    }

    pub(crate) fn knows(&self, id: &str) -> bool {
        self.since.contains_key(id)
    }

    // Counted from this machine's first sight, so whoever writes the folder cannot wind it back.
    pub(crate) fn still_landing(&mut self, id: &str, now: u64) -> bool {
        self.checked.insert(id.to_string());
        let first = self.since.entry(id.to_string()).or_insert(now);
        *first = (*first).min(now);
        now - *first < LANDING.as_secs()
    }

    pub(crate) fn arrived(&mut self, id: &str) {
        self.since.remove(id);
    }

    // A day keeps a body planted again and again from earning a fresh hour each time.
    pub(crate) fn forget_settled(&mut self, alive: &[String], now: u64) {
        let checked = &self.checked;
        self.since.retain(|id, first| {
            alive.contains(id)
                && (checked.contains(id) || now.saturating_sub(*first) < REMEMBERED.as_secs())
        });
    }
}

#[cfg(test)]
#[path = "awaited_test.rs"]
mod tests;
