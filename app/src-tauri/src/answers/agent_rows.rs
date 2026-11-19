use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::agents::{self, Shown};

use super::agents_sync::{docs_in, root_of, tree};
use crate::{Answer, Session, held};

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    file: String,
    #[serde(flatten)]
    shown: Shown,
}

/// The files under AGENTS, taken while the session is held; their words are read after it is let go.
pub(crate) fn listed(session: &Session) -> (PathBuf, Vec<String>) {
    let files = root_of(&session.state)
        .map(|root| {
            docs_in(&session.state, &tree(&session.state, root))
                .into_iter()
                .map(|one| one.file.clone())
                .collect()
        })
        .unwrap_or_default();
    (session.paths.docs(), files)
}

pub(crate) fn rows(docs: &Path, files: &[String]) -> Vec<Row> {
    files
        .iter()
        .filter_map(|file| {
            let body = tisty_core::docs::read(docs, file).ok()?;
            Some(Row {
                file: file.clone(),
                shown: agents::shown(&body)?,
            })
        })
        .collect()
}

pub(crate) fn asked_for(files: Vec<String>, only: Option<Vec<String>>) -> Vec<String> {
    match only {
        Some(only) => files.into_iter().filter(|one| only.contains(one)).collect(),
        None => files,
    }
}

#[tauri::command(async)]
pub fn agents_rows(
    session: tauri::State<'_, Mutex<Session>>,
    only: Option<Vec<String>>,
) -> Answer<Vec<Row>> {
    let (docs, files) = listed(&held(&session));
    Ok(rows(&docs, &asked_for(files, only)))
}

#[cfg(test)]
#[path = "agent_rows_test.rs"]
mod tests;
