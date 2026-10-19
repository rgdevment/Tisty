use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const FILE: &str = "uploading.json";

#[derive(Serialize, Deserialize)]
struct Session {
    uri: String,
    opened: u64,
}

type Sessions = BTreeMap<String, Session>;

/// The same file, to the same place, unchanged since: anything else is begun again.
pub fn key_of(url: &str, name: &str, from: &Path, bytes: u64) -> Option<String> {
    let changed = std::fs::metadata(from)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some(format!("{url}\u{0}{name}\u{0}{bytes}\u{0}{changed}"))
}

/// Where long uploads had got to, so one cut short carries on after a restart instead of starting over.
pub struct Uploads {
    at: PathBuf,
    kept_for: u64,
}

impl Uploads {
    pub fn under(home: &Path, kept_for: u64) -> Self {
        Self {
            at: home.join(FILE),
            kept_for,
        }
    }

    pub fn session_for(&self, key: &str) -> Option<String> {
        read(&self.at)
            .get(key)
            .filter(|one| self.fresh(one.opened))
            .map(|one| one.uri.clone())
    }

    pub fn opened(&self, key: &str, uri: &str) {
        self.changed(|all| {
            all.insert(
                key.to_string(),
                Session {
                    uri: uri.to_string(),
                    opened: tisty_core::lately::now(),
                },
            );
        });
    }

    pub fn closed(&self, key: &str) {
        self.changed(|all| {
            all.remove(key);
        });
    }

    fn fresh(&self, opened: u64) -> bool {
        tisty_core::lately::now().saturating_sub(opened) < self.kept_for
    }

    fn changed(&self, change: impl FnOnce(&mut Sessions)) {
        let mut all = read(&self.at);
        all.retain(|_, one| self.fresh(one.opened));
        change(&mut all);
        let written = match all.is_empty() {
            true => std::fs::remove_file(&self.at).or_else(|e| match e.kind() {
                std::io::ErrorKind::NotFound => Ok(()),
                _ => Err(e),
            }),
            false => serde_json::to_vec(&all)
                .map_err(std::io::Error::other)
                .and_then(|bytes| {
                    if let Some(dir) = self.at.parent() {
                        std::fs::create_dir_all(dir)?;
                    }
                    tisty_core::store::write_atomic(&self.at, &bytes).map_err(std::io::Error::other)
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

#[cfg(test)]
#[path = "uploads_test.rs"]
mod tests;
