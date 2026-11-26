use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::agents::{self, Shown};

use super::agents_sync::{docs_in, root_of, tree};
use crate::{Answer, Session, held};

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Row {
    file: String,
    /// Outside every computer's own folder, so it is for all of them.
    everyone: bool,
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

/// The files under AGENTS that sit in no computer's own folder.
pub(crate) fn shared(session: &Session) -> BTreeSet<String> {
    let state = &session.state;
    root_of(state)
        .map(|root| {
            docs_in(state, &tree(state, root))
                .into_iter()
                .filter(|one| agents::computer_of(state, one.folder).is_none())
                .map(|one| one.file.clone())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn rows(docs: &Path, files: &[String], shared: &BTreeSet<String>) -> Vec<Row> {
    files
        .iter()
        .filter_map(|file| {
            let body = tisty_core::docs::read(docs, file).ok()?;
            Some(Row {
                file: file.clone(),
                everyone: shared.contains(file),
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
    let ((docs, files), shared) = {
        let session = held(&session);
        (listed(&session), shared(&session))
    };
    Ok(rows(&docs, &asked_for(files, only), &shared))
}

#[cfg(test)]
#[path = "agent_rows_test.rs"]
mod tests;
