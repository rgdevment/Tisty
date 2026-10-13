use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::Drive;

const FILE: &str = "uploading.json";
// Google keeps a resumable session for a week; one a day short of that is begun again instead.
const KEPT_FOR: u64 = 6 * 24 * 60 * 60;

#[derive(Serialize, Deserialize)]
struct Session {
    uri: String,
    opened: u64,
}

type Sessions = BTreeMap<String, Session>;

/// The same file, to the same place, unchanged since: anything else is begun again.
pub(super) fn key_of(url: &str, name: &str, from: &Path, bytes: u64) -> Option<String> {
    let changed = std::fs::metadata(from)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some(format!("{url}\u{0}{name}\u{0}{bytes}\u{0}{changed}"))
}

impl Drive {
    fn sessions_at(&self) -> Option<PathBuf> {
        Some(self.owing.as_ref()?.parent()?.join(FILE))
    }

    pub(super) fn session_for(&self, key: &str) -> Option<String> {
        let all = read(&self.sessions_at()?);
        all.get(key)
            .filter(|one| fresh(one.opened))
            .map(|one| one.uri.clone())
    }

    pub(super) fn session_opened(&self, key: &str, uri: &str) {
        self.sessions_changed(|all| {
            all.insert(
                key.to_string(),
                Session {
                    uri: uri.to_string(),
                    opened: tisty_core::lately::now(),
                },
            );
        });
    }

    pub(super) fn session_closed(&self, key: &str) {
        self.sessions_changed(|all| {
            all.remove(key);
        });
    }

    fn sessions_changed(&self, change: impl FnOnce(&mut Sessions)) {
        let Some(at) = self.sessions_at() else {
            return;
        };
        let mut all = read(&at);
        all.retain(|_, one| fresh(one.opened));
        change(&mut all);
        let written = match all.is_empty() {
            true => std::fs::remove_file(&at).or_else(|e| match e.kind() {
                std::io::ErrorKind::NotFound => Ok(()),
                _ => Err(e),
            }),
            false => serde_json::to_vec(&all)
                .map_err(std::io::Error::other)
                .and_then(|bytes| {
                    if let Some(dir) = at.parent() {
                        std::fs::create_dir_all(dir)?;
                    }
                    tisty_core::store::write_atomic(&at, &bytes).map_err(std::io::Error::other)
                }),
        };
        if let Err(why) = written {
            tisty_core::witness::warn(
                tisty_core::witness::channel::CLOUD,
                "where a long upload had got to could not be written down, so it may start over",
                &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
            );
        }
    }
}

fn read(at: &Path) -> Sessions {
    std::fs::read(at)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

fn fresh(opened: u64) -> bool {
    tisty_core::lately::now().saturating_sub(opened) < KEPT_FOR
}
