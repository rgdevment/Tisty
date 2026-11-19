use std::sync::Mutex;

use tisty_core::brain::{self, Card, Network};

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

#[tauri::command]
pub async fn brain_card(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Card>> {
    let (paths, hidden) = {
        let session = held(&session);
        (session.paths.clone(), brain::kept_out(&session.state))
    };
    let today = jiff::Zoned::now().date();
    elsewhere(move || brain::card_of(&paths, &id, today, |one| !hidden.contains(one))).await
}
