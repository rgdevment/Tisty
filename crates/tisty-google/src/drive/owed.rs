use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tisty_carrier::Hitch;

use super::tree::{FILE_FIELDS, File};
use super::wire::json;
use super::{Drive, READ};
use crate::ASKED_WITHIN;

/// Another machine's bytes a write covered and could not yet put back, kept until they are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "owed", rename_all = "lowercase")]
pub(super) enum Owed {
    Back {
        name: String,
        id: String,
        checked_head: Option<String>,
        checked_md5: Option<String>,
        landed_head: Option<String>,
        landed_md5: Option<String>,
    },
    Unbinned {
        id: String,
    },
}

impl Owed {
    pub(super) fn back(name: &str, landed: &File, checked: &File) -> Self {
        Self::Back {
            name: name.to_string(),
            id: landed.id.clone(),
            checked_head: checked.head_revision_id.clone(),
            checked_md5: checked.md5_checksum.clone(),
            landed_head: landed.head_revision_id.clone(),
            landed_md5: landed.md5_checksum.clone(),
        }
    }
}

const WRITING: &str = "lock";
const REPAYING: &str = "repaying.lock";

// What was owed meanwhile stays owed: only what this pass paid comes off the list as it stands now.
pub(super) fn unpaid(now: Vec<Owed>, paid: &[Owed]) -> Vec<Owed> {
    now.into_iter().filter(|one| !paid.contains(one)).collect()
}

fn file(id: &str, head: &Option<String>, md5: &Option<String>) -> File {
    File {
        id: id.to_string(),
        head_revision_id: head.clone(),
        md5_checksum: md5.clone(),
        ..File::default()
    }
}

impl Drive {
    /// Where a Drive that must outlive a crash writes down what it still owes.
    pub fn owing_at(mut self, home: impl Into<PathBuf>) -> Self {
        self.owing = Some(home.into().join(tisty_carrier::OWED_BACK));
        self
    }

    fn owed(&self) -> Vec<Owed> {
        self.owing
            .as_ref()
            .and_then(|at| std::fs::read(at).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    fn keep_owing(&self, all: &[Owed]) {
        let Some(at) = &self.owing else {
            return;
        };
        let written = match all.is_empty() {
            true => std::fs::remove_file(at).or_else(|e| match e.kind() {
                std::io::ErrorKind::NotFound => Ok(()),
                _ => Err(e),
            }),
            false => serde_json::to_vec(all)
                .map_err(std::io::Error::other)
                .and_then(|bytes| {
                    if let Some(dir) = at.parent() {
                        std::fs::create_dir_all(dir)?;
                    }
                    tisty_core::store::write_atomic(at, &bytes).map_err(std::io::Error::other)
                }),
        };
        if let Err(why) = written {
            tisty_core::witness::warn(
                tisty_core::witness::channel::CLOUD,
                "what Drive still owes another machine could not be written down",
                &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
            );
        }
    }

    // The list is read and written under its lock, since a poll repays while a round may owe.
    fn held(&self, lock: &str, waits: bool) -> Option<std::fs::File> {
        let at = self.owing.as_ref()?.with_extension(lock);
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(at)
            .ok()?;
        let taken = match waits {
            true => file.lock().is_ok(),
            false => file.try_lock().is_ok(),
        };
        taken.then_some(file)
    }

    pub(super) fn owe(&self, one: Owed) {
        let _writing = self.held(WRITING, true);
        let mut all = self.owed();
        if !all.contains(&one) {
            all.push(one);
        }
        self.keep_owing(&all);
    }

    // Paid before any change is read, so the bytes it puts back come in as another machine's change.
    pub(super) fn repaid(&self) {
        let Some(_repaying) = self.held(REPAYING, false) else {
            return;
        };
        let all = self.owed();
        if all.is_empty() {
            return;
        }
        let paid: Vec<Owed> = all
            .into_iter()
            .filter(|one| match self.repay(one) {
                Ok(()) | Err(Hitch::Missing(_)) => true,
                Err(_) => false,
            })
            .collect();
        let _writing = self.held(WRITING, true);
        let left = unpaid(self.owed(), &paid);
        self.keep_owing(&left);
    }

    fn repay(&self, one: &Owed) -> Result<(), Hitch> {
        match one {
            Owed::Back {
                name,
                id,
                checked_head,
                checked_md5,
                landed_head,
                landed_md5,
            } => {
                let landed = file(id, landed_head, landed_md5);
                let checked = file(id, checked_head, checked_md5);
                if self.head_of(id)? != *landed_head {
                    return self.kept_aside(name, &landed, &checked);
                }
                self.restored(name, &landed, &checked).map(|_| ())
            }
            Owed::Unbinned { id } => self.trashed(id, false).map(|_| ()),
        }
    }

    fn head_of(&self, id: &str) -> Result<Option<String>, Hitch> {
        let url = format!("{}/files/{id}", self.at.api);
        self.spent(READ);
        let now: File = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[("fields", FILE_FIELDS)])
        })?)?;
        Ok(now.head_revision_id)
    }

    // A newer write came after the one that covered them, so they stay in Drive's history rather than going over it.
    fn kept_aside(&self, name: &str, landed: &File, checked: &File) -> Result<(), Hitch> {
        let Some(theirs) = self.written_between(landed, checked)? else {
            return Ok(());
        };
        self.pinned(&landed.id, &theirs)?;
        tisty_core::witness::warn(
            tisty_core::witness::channel::CLOUD,
            "another machine's bytes stay in Drive's version history, since a newer write came after the one that covered them",
            &[
                ("name", tisty_core::witness::Fact::Id(name.to_string())),
                ("revision", tisty_core::witness::Fact::Id(theirs)),
            ],
        );
        Ok(())
    }
}
