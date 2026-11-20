use std::sync::Mutex;

use tisty_core::brain::{self, Card, Network};

use crate::{Answer, Session, elsewhere, finding, held};

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct FileFacts {
    bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    changed: Option<String>,
}

#[tauri::command]
pub async fn brain_network(
    session: tauri::State<'_, Mutex<Session>>,
    near: Option<String>,
) -> Answer<Option<Network>> {
    let (paths, hidden, touched, dates) = {
        let session = held(&session);
        let hidden = brain::kept_out(&session.paths.docs(), &session.state);
        let touched = brain::touched_last(&session.state, &hidden);
        (
            session.paths.clone(),
            hidden,
            touched,
            brain::dates(&session.state),
        )
    };
    let near: Vec<String> = near.into_iter().chain(touched).collect();
    elsewhere(move || {
        brain::network(&paths, |id| !hidden.contains(id), &near).map(|mut network| {
            network.dated(&dates);
            network
        })
    })
    .await
}

#[tauri::command]
pub async fn brain_card(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Card>> {
    let (paths, hidden) = {
        let session = held(&session);
        (
            session.paths.clone(),
            brain::kept_out(&session.paths.docs(), &session.state),
        )
    };
    let today = jiff::Zoned::now().date();
    elsewhere(move || brain::card_of(&paths, &id, today, |one| !hidden.contains(one))).await
}

// Told only of a file the network shows, so one used by nothing in sight stays as if it were not there.
#[tauri::command]
pub async fn brain_file(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<FileFacts>> {
    let (paths, hidden, bytes, looking) = {
        let session = held(&session);
        let Some((_, bytes)) = session.state.kept.get(&id) else {
            return Ok(None);
        };
        (
            session.paths.clone(),
            brain::kept_out(&session.paths.docs(), &session.state),
            *bytes,
            (session.paths.data().to_path_buf(), session.shared_now()),
        )
    };
    elsewhere(move || {
        brain::file_seen(&paths, &id, |one| !hidden.contains(one)).then(|| FileFacts {
            bytes,
            changed: changed_at(&id, &looking.0, looking.1.as_deref()),
        })
    })
    .await
}

fn changed_at(
    reference: &str,
    data: &std::path::Path,
    shared: Option<&std::path::Path>,
) -> Option<String> {
    // What a cloud left in the place of a file it freed says when it was freed, not when the file changed.
    let at = finding::where_it_lies(reference, data, shared)
        .filter(|at| !tisty_core::holes::a_hole(at))?;
    let changed = std::fs::metadata(at).ok()?.modified().ok()?;
    Some(jiff::Timestamp::try_from(changed).ok()?.to_string())
}

#[cfg(test)]
#[path = "brainsty_test.rs"]
mod tests;
