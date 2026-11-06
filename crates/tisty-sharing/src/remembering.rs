use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tisty_core::Paths;

use crate::Unshared;

const SHELF: &str = "sharing.json";

// Its keys open what was sent, so it stays in this machine's private folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Remembered {
    pub id: String,
    pub version: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files_print: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files_hash: Option<String>,
    // Without it, files sent under another key could be kept by mistake.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admin_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub words_print: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_as: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_name: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cited: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held_by: Option<String>,
}

fn shelf_at(paths: &Paths) -> PathBuf {
    paths.private().join(SHELF)
}

pub fn remembered(paths: &Paths) -> Result<BTreeMap<String, Remembered>, Unshared> {
    let said = match std::fs::read(shelf_at(paths)) {
        Ok(said) => said,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(e) => return Err(Unshared::Unread(e.to_string())),
    };
    serde_json::from_slice(&said).map_err(|e| Unshared::Unread(e.to_string()))
}

// Called once the envelopes have reached where they go, and never before: only then can the next version count on them.
pub fn sent(paths: &Paths, went: &[(String, Remembered)]) -> Result<(), Unshared> {
    let mut shelf = remembered(paths)?;
    shelf.extend(went.iter().cloned());
    written(paths, &shelf)
}

pub fn forgotten(paths: &Paths, files: &[String]) -> Result<(), Unshared> {
    let mut shelf = remembered(paths)?;
    for file in files {
        shelf.remove(file);
    }
    written(paths, &shelf)
}

fn written(paths: &Paths, shelf: &BTreeMap<String, Remembered>) -> Result<(), Unshared> {
    let unwritten = |e: &dyn std::fmt::Display| Unshared::Unwritten(e.to_string());
    std::fs::create_dir_all(paths.private()).map_err(|e| unwritten(&e))?;
    let bytes = serde_json::to_vec(shelf).map_err(|e| unwritten(&e))?;
    let at = shelf_at(paths);
    tisty_core::store::write_atomic(&at, &bytes).map_err(|e| unwritten(&e))?;
    tisty_core::paths::ours_alone(&at).map_err(|e| unwritten(&e))
}
