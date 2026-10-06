use std::sync::Mutex;

use crate::{Answer, Session, held, report};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThisMachine {
    pub id: String,
    pub name: Option<String>,
    pub os: Option<String>,
    pub code: Option<String>,
}

/// Only the machines a round left waiting, without the whole store read the audit needs.
#[tauri::command(async)]
pub fn waiting_machines(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<report::Machine>> {
    let session = held(&session);
    let dest = match &session.config.sync {
        Some(tisty_core::config::Sync::Folder(at)) => Some(at.clone()),
        _ => None,
    };
    let mine = session.config.device_id.0.clone();
    Ok(report::machines(
        &[],
        &mine,
        &report::Known {
            gone: &session.state.dropped,
            assistants: &session.state.assistants,
            keys: &session.state.keys,
            named: &session.state.named,
        },
        &session.paths,
        dest.as_deref(),
    )
    .into_iter()
    .filter(|one| !one.mine && one.turned_away.as_deref() == Some("unconfirmed"))
    .collect())
}

#[tauri::command(async)]
pub fn this_machine(session: tauri::State<'_, Mutex<Session>>) -> Answer<ThisMachine> {
    let session = held(&session);
    let who = session.config.device_id.clone();
    let named = tisty_core::called::here();
    Ok(ThisMachine {
        code: tisty_core::signing::shown_kept(&session.paths, &who)
            .as_deref()
            .and_then(tisty_core::signing::spoken),
        id: who.0,
        name: named.as_ref().map(|one| one.name.clone()),
        os: named.and_then(|one| one.os),
    })
}
