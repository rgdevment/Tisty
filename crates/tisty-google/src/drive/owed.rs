use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tisty_carrier::Hitch;

use super::Drive;
use super::tree::File;

/// Bytes a write covered or a deletion set aside and could not yet put back, kept until they are.
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
        name: String,
        id: String,
        parent: String,
        bin: String,
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

    pub(super) fn puts_back(&self, file: &str) -> bool {
        matches!(self, Self::Unbinned { id, .. } if id == file)
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

    // Read one by one, so a debt of a kind this build no longer owes never hides the others; None is a list that could not be read.
    pub(super) fn owed(&self) -> Option<Vec<Owed>> {
        let Some(at) = &self.owing else {
            return Some(Vec::new());
        };
        let bytes = match std::fs::read(at) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Some(Vec::new()),
            Err(_) => return None,
        };
        let all = serde_json::from_slice::<Vec<serde_json::Value>>(&bytes).ok()?;
        Some(
            all.into_iter()
                .filter_map(|one| serde_json::from_value(one).ok())
                .collect(),
        )
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

    // A list that cannot be read is never written over, since the debts it holds would go with it.
    pub(super) fn owe(&self, one: Owed) -> Result<(), Hitch> {
        let _writing = self.held(WRITING, true);
        let mut all = self.owed().ok_or_else(|| {
            Hitch::Broke(
                "what Drive still owes could not be read, so no more is written down".into(),
            )
        })?;
        if !all.contains(&one) {
            all.push(one);
        }
        self.keep_owing(&all);
        Ok(())
    }

    // Paid before any change is read, so the bytes it puts back come in as another machine's change.
    pub(super) fn repaid(&self) {
        let Some(_repaying) = self.held(REPAYING, false) else {
            return;
        };
        let Some(all) = self.owed() else {
            return;
        };
        if all.is_empty() {
            if self.owing.as_ref().is_some_and(|at| at.exists()) {
                let _writing = self.held(WRITING, true);
                if self.owed().is_some_and(|all| all.is_empty()) {
                    self.keep_owing(&[]);
                }
            }
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
        if let Some(now) = self.owed() {
            self.keep_owing(&unpaid(now, &paid));
        }
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
            Owed::Unbinned {
                name,
                id,
                parent,
                bin,
            } => self.put_back(name, id, (bin, parent)),
        }
    }

    fn head_of(&self, id: &str) -> Result<Option<String>, Hitch> {
        Ok(self.looked_at(id)?.head_revision_id)
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
