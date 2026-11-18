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
