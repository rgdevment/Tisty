use std::sync::Mutex;

use tisty_core::brain::Citing;

use crate::{Answer, Session, elsewhere, held};

#[tauri::command]
pub async fn doc_cited_by(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Vec<Citing>>> {
    let paths = held(&session).paths.clone();
    elsewhere(move || tisty_core::brain::cited_by(&paths, &id)).await
}
