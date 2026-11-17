use std::sync::Mutex;

use tisty_core::forgotten::{self, Forgotten};

use crate::{Answer, Session, held};

#[tauri::command(async)]
pub fn docs_forgotten(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<Forgotten>> {
    let mut session = held(&session);
    session.reload()?;
    let opened = forgotten::opened_here(&session.paths.opened_file());
    Ok(forgotten::forgotten(
        &session.paths.docs(),
        &session.state,
        &opened,
        tisty_core::lately::now(),
    ))
}

#[tauri::command(async)]
pub fn doc_at_random(session: tauri::State<'_, Mutex<Session>>) -> Answer<Option<String>> {
    let mut session = held(&session);
    session.reload()?;
    let opened = forgotten::opened_here(&session.paths.opened_file());
    let picked = forgotten::at_random(
        &session.paths.docs(),
        &session.state,
        &opened,
        tisty_core::lately::now(),
        tisty_core::brainstorm::roll(),
    );
    Ok(picked.map(|doc| doc.to_string()))
}
