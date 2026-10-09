use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Seen;

const FILE: &str = "index.json";

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
    pub cursor: Option<String>,
    #[serde(default)]
    pub confirmed: bool,
    pub tree: BTreeMap<String, Mirrored>,
    pub shelf: BTreeMap<String, Seen>,
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
