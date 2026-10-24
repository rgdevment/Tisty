use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Seen;

const FILE: &str = "index.json";
/// How the store is laid out up there; an index of an older layout names files no longer read, so the mirror starts over.
pub const LAYOUT: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mirrored {
    pub seen: Seen,
    pub len: u64,
    pub stamp: u128,
    #[serde(default)]
    pub digest: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    #[serde(default, serialize_with = "laid_out_now")]
    pub layout: u32,
    pub cursor: Option<String>,
    #[serde(default)]
    pub confirmed: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub root: String,
    pub tree: BTreeMap<String, Mirrored>,
    pub shelf: BTreeMap<String, Seen>,
    #[serde(default)]
    pub spoiled: BTreeMap<String, String>,
    /// What a pull could not take yet; the cursor moves past it and the next pull asks again.
    #[serde(default)]
    pub owed: BTreeMap<String, Seen>,
    /// Attachments a pull saw leave the cloud, so opening one says it is gone instead of not there yet.
    #[serde(default)]
    pub vanished: BTreeSet<String>,
}

fn laid_out_now<S: serde::Serializer>(_: &u32, to: S) -> Result<S::Ok, S::Error> {
    to.serialize_u32(LAYOUT)
}

/// Only an index that was written and read is judged: none at all is a first round anyway.
pub fn outdated(home: &Path) -> bool {
    std::fs::read(home.join(FILE))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Index>(&bytes).ok())
        .is_some_and(|index| index.layout < LAYOUT)
}

/// Revisions, cursor and what is owed belong to the root they were read from, so another root reads none of them.
pub fn rooted_elsewhere(home: &Path, root: &str) -> bool {
    std::fs::read(home.join(FILE))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Index>(&bytes).ok())
        .is_some_and(|index| index.root != root)
}

pub fn forget(home: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(home.join(FILE)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

impl Index {
    pub fn load(home: &Path) -> Self {
        std::fs::read(home.join(FILE))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, home: &Path) -> std::io::Result<()> {
        let bytes = serde_json::to_vec(self).map_err(std::io::Error::other)?;
        std::fs::create_dir_all(home)?;
        tisty_core::store::write_atomic(&home.join(FILE), &bytes).map_err(std::io::Error::other)
    }

    pub fn pulled(&self) -> bool {
        self.cursor.is_some()
    }

    pub fn shelved(&mut self, name: &str, seen: Seen) {
        self.vanished.remove(name);
        self.shelf.insert(name.to_string(), seen);
    }

    pub fn spoiled(&self, reference: &str) -> bool {
        self.shelf
            .get(reference)
            .is_some_and(|seen| self.spoiled.get(reference) == Some(&seen.revision))
    }
}

pub fn stamp_of(meta: &std::fs::Metadata) -> u128 {
    meta.modified()
        .ok()
        .and_then(|when| when.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |since| since.as_nanos())
}

pub fn digest_of(at: &Path) -> Option<String> {
    tisty_core::attach::hashed(at)
        .ok()
        .map(|(digest, _)| digest)
}

pub fn on_shelf(name: &str) -> bool {
    name.starts_with(&format!("{}/", tisty_sync::HELD))
}

pub fn path_of(tree: &Path, name: &str) -> Option<PathBuf> {
    crate::named_well(name).ok()?;
    let mut at = tree.to_path_buf();
    for part in name.split('/') {
        let odd = part.contains([':', '<', '>', '"', '|', '?', '*'])
            || part.ends_with(['.', ' '])
            || tisty_core::attach::reserved(part);
        if odd {
            return None;
        }
        at.push(part);
    }
    Some(at)
}
