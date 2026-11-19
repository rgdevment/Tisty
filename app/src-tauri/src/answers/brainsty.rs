use std::sync::Mutex;

use tisty_core::brain::{self, Network};

use crate::{Answer, Session, elsewhere, held};

#[tauri::command]
pub async fn brain_network(
    session: tauri::State<'_, Mutex<Session>>,
    near: Option<String>,
) -> Answer<Option<Network>> {
    let (paths, hidden, touched, dates) = {
        let session = held(&session);
        (
            session.paths.clone(),
            brain::kept_out(&session.state),
            brain::touched_last(&session.state),
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

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct Group {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,
}

pub(crate) fn groups_of(state: &tisty_core::State) -> Vec<Group> {
    let folders = state.folders.values().map(|one| Group {
        id: one.id.to_string(),
        name: one.name.clone(),
        color: one.color.clone(),
    });
    let lists = state.lists.values().map(|one| Group {
        id: one.id.to_string(),
        name: one.name.clone(),
        color: one.color.clone(),
    });
    folders.chain(lists).collect()
}

#[tauri::command]
pub fn brainsty_groups(session: tauri::State<'_, Mutex<Session>>) -> Vec<Group> {
    groups_of(&held(&session).state)
}

#[cfg(test)]
#[path = "brainsty_test.rs"]
mod tests;
