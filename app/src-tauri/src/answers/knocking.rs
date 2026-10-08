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
    // Read out under the lock and let go of it: the folder may be a slow cloud mount.
    let (paths, dest, mine, gone, assistants, keys, named, hosts) = {
        let session = held(&session);
        (
            session.paths.clone(),
            session.place(),
            session.config.device_id.0.clone(),
            session.state.dropped.clone(),
            session.state.assistants.clone(),
            session.state.keys.clone(),
            session.state.named.clone(),
            session.state.hosts.clone(),
        )
    };
    Ok(report::machines(
        &[],
        &mine,
        &report::Known {
            gone: &gone,
            assistants: &assistants,
            keys: &keys,
            named: &named,
            hosts: &hosts,
        },
        &paths,
        dest.as_deref(),
    )
    .into_iter()
    .filter(|one| !one.mine && one.turned_away.as_deref() == Some("unconfirmed"))
    .collect())
}

#[tauri::command(async)]
pub fn this_machine(session: tauri::State<'_, Mutex<Session>>) -> Answer<ThisMachine> {
    let (paths, who, given) = {
        let session = held(&session);
        (
            session.paths.clone(),
            session.config.device_id.clone(),
            session.config.called.clone(),
        )
    };
    let named = tisty_core::called::chosen(given.as_deref());
    Ok(ThisMachine {
        code: super::agents::spoken_by(&paths, Some(&who)),
        id: who.0,
        name: named.as_ref().map(|one| one.name.clone()),
        os: named.and_then(|one| one.os),
    })
}

#[tauri::command(async)]
pub fn rename_machine(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
) -> Answer<ThisMachine> {
    held(&session).rename(&name)?;
    this_machine(session)
}
