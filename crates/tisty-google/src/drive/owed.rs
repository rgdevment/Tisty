use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tisty_carrier::Hitch;

use super::Drive;
use super::tree::File;

const OWED: &str = "drive-owed.json";

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
        self.owing = Some(home.into().join(OWED));
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

    pub(super) fn owe(&self, one: Owed) {
        let mut all = self.owed();
        if !all.contains(&one) {
            all.push(one);
        }
        self.keep_owing(&all);
    }

    // Paid before any change is read, so the bytes it puts back come in as another machine's change.
    pub(super) fn repaid(&self) {
        let all = self.owed();
        if all.is_empty() {
            return;
        }
        let left: Vec<Owed> = all
            .into_iter()
            .filter(|one| match self.repay(one) {
                Ok(()) | Err(Hitch::Missing(_)) => false,
                Err(_) => true,
            })
            .collect();
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
                self.restored(name, &landed, &checked).map(|_| ())
            }
            Owed::Unbinned { id } => self.trashed(id, false).map(|_| ()),
        }
    }
}
