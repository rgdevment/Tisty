use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use tisty_core::witness::{self, Fact, channel};

pub(crate) const LANDING: u64 = 60 * 60;
const REMEMBERED: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Seen {
    first: u64,
    last: u64,
}

/// When this machine first and last saw each document's body with no history answering for it.
#[derive(Debug, Default, Clone)]
pub(crate) struct Awaited {
    since: BTreeMap<String, Seen>,
    read: BTreeMap<String, Seen>,
    checked: BTreeSet<String>,
    unread: bool,
}

fn ledger(data: &Path) -> PathBuf {
    data.join("awaited")
}

fn seen(line: &str) -> Option<(String, Seen)> {
    let mut parts = line.split_whitespace();
    let id = parts.next()?.to_string();
    let first = parts.next()?.parse().ok()?;
    let last = parts.next().and_then(|at| at.parse().ok()).unwrap_or(first);
    Some((id, Seen { first, last }))
}

impl Awaited {
    pub(crate) fn read(data: &Path) -> Self {
        match std::fs::read_to_string(ledger(data)) {
            Ok(said) => {
                let since: BTreeMap<_, _> = said.lines().filter_map(seen).collect();
                Self {
                    read: since.clone(),
                    since,
                    ..Self::default()
                }
            }
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(_) => Self {
                unread: true,
                ..Self::default()
            },
        }
    }

    // An unreadable ledger is left as it is, so one bad read never starts every wait over.
    pub(crate) fn save(&self, data: &Path) {
        if self.unread || self.since == self.read {
            return;
        }
        let kept = match self.since.is_empty() {
            true => std::fs::remove_file(ledger(data)),
            false => {
                let said: String = self
                    .since
                    .iter()
                    .map(|(id, at)| format!("{id} {} {}\n", at.first, at.last))
                    .collect();
                tisty_core::store::write_atomic(&ledger(data), said.as_bytes())
                    .map_err(|e| std::io::Error::other(e.to_string()))
            }
        };
        if let Err(why) = kept {
            witness::warn(
                channel::SYNC,
                "what this machine awaits could not be kept, so a body may wait again from the start",
                &[("why", Fact::Why(why.to_string()))],
            );
        }
    }

    pub(crate) fn knows(&self, id: &str) -> bool {
        self.since.contains_key(id)
    }

    // Counted from this machine's first sight, so whoever writes the folder cannot wind it back.
    pub(crate) fn still_landing(&mut self, id: &str, now: u64) -> bool {
        self.checked.insert(id.to_string());
        let at = self.since.entry(id.to_string()).or_insert(Seen {
            first: now,
            last: now,
        });
        at.first = at.first.min(now);
        at.last = now;
        now - at.first < LANDING
    }

    pub(crate) fn arrived(&mut self, ids: &[String]) {
        for id in ids {
            self.since.remove(id);
        }
    }

    // A day keeps a body planted again and again from earning a fresh hour each time.
    pub(crate) fn forget_settled(&mut self, alive: &[String], now: u64) {
        let checked = &self.checked;
        self.since.retain(|id, at| {
            alive.contains(id) && (checked.contains(id) || now.saturating_sub(at.last) < REMEMBERED)
        });
    }
}

#[cfg(test)]
#[path = "awaited_test.rs"]
mod tests;
