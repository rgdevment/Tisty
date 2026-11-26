use std::sync::Mutex;

use tisty_core::agents;

use super::earlier::Shelf;
use crate::{Answer, Session, held};

pub(crate) struct FileKept {
    pub(crate) at: String,
    pub(crate) named: String,
    pub(crate) kept: Vec<agents::Kept>,
}

/// A copy of an agent's file, kept on this computer alone: only its size and when travel here, never its words.
#[derive(serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Copy {
    at: String,
    named: String,
    n: u64,
    bytes: u64,
    pinned: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    when: Option<String>,
}

/// The document's versions and the copies of its files, read together for one history.
#[derive(serde::Serialize)]
pub struct History {
    #[serde(flatten)]
    shelf: Shelf,
    copies: Vec<Copy>,
}

pub(crate) fn copies(files: Vec<FileKept>) -> Vec<Copy> {
    files
        .into_iter()
        .flat_map(|file| {
            let FileKept { at, named, kept } = file;
            kept.into_iter().map(move |one| Copy {
                at: at.clone(),
                named: named.clone(),
                n: one.n,
                bytes: one.bytes,
                pinned: one.pinned,
                when: one.at.map(|when| when.to_string()),
            })
        })
        .collect()
}

#[tauri::command(async)]
pub fn agent_history(session: tauri::State<'_, Mutex<Session>>, file: String) -> Answer<History> {
    let files = {
        let mut one = held(&session);
        one.reload()?;
        super::agent_docs::kept_for(&one, &file)?
    };
    Ok(History {
        shelf: super::earlier::shelf(&session, &file)?,
        copies: copies(files),
    })
}

#[cfg(test)]
#[path = "agent_history_test.rs"]
mod tests;
