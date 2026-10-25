use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::seed::Seed;
use zeroize::Zeroizing;

use crate::handed::{Here, Knocking, here_now, kept_for, opens};
use crate::{Answer, Refusal, Session, elsewhere, held};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Heard {
    pub name: String,
    pub os: Option<String>,
    pub at: i64,
}

fn heard(session: &Session) -> Vec<Heard> {
    let state = &session.state;
    let mut heard: Vec<Heard> = state
        .devices
        .iter()
        .filter(|device| **device != session.config.device_id && !state.assistants.contains(device))
        .filter_map(|device| {
            let named = state.named.get(device);
            Some(Heard {
                name: named.map_or_else(
                    || tisty_core::config::nicknamed(&device.0),
                    |one| one.name.clone(),
                ),
                os: named.and_then(|one| one.os.clone()),
                at: *state.heard.get(device)?,
            })
        })
        .collect();
    heard.sort_by_key(|one| std::cmp::Reverse(one.at));
    heard
}

/// Which of the person's other computers this one has heard from, and when, to tell whether one is at hand.
#[tauri::command(async)]
pub fn sealing_heard(session: tauri::State<'_, Mutex<Session>>) -> Vec<Heard> {
    heard(&held(&session))
}

// Only the cloud's key is kept, never the seed: the root signs for this computer once and is let go (decision 12).
fn entered(session: &Mutex<Session>, here: &Here, seed: &Seed) -> Answer<()> {
    {
        let session = held(session);
        let persons = &session.state.persons;
        if persons.names_a_person() && !persons.names(&seed.root().person()) {
            return Err(Refusal::of("phraseOtherPerson"));
        }
    }
    let key = CloudKey::of(seed);
    match opens(here, &key) {
        Ok(true) => {}
        Ok(false) => return Err(Refusal::of("phraseNotThisCloud")),
        Err(why) => {
            tisty_core::witness::note(
                tisty_core::witness::channel::CLOUD,
                "the cloud's mark could not be read to check a phrase",
                &[("why", tisty_core::witness::Fact::Why(why))],
            );
            return Err(Refusal::of("cloudNotNow"));
        }
    }
    kept_for(session, &here.cloud, &key).map_err(|why| {
        tisty_core::witness::warn(
            tisty_core::witness::channel::STORE,
            "the cloud's key could not be kept on this computer",
            &[("why", tisty_core::witness::Fact::Why(why))],
        );
        Refusal::of("identityNotKept")
    })?;
    crate::identity::begun(&mut held(session), seed)
}

#[tauri::command(async)]
pub async fn sealing_phrase(app: AppHandle, said: String) -> Answer<()> {
    let said = Zeroizing::new(said);
    let seed = crate::identity::seed_said(&said)?;
    let entering = app.clone();
    elsewhere(move || {
        let (session, knocking) = (
            entering.state::<Mutex<Session>>(),
            entering.state::<Knocking>(),
        );
        let (here, _) = here_now(&session, &knocking).map_err(|why| {
            tisty_core::witness::note(
                tisty_core::witness::channel::CLOUD,
                "the sealed cloud could not be reached to enter it with a phrase",
                &[("why", tisty_core::witness::Fact::Why(why))],
            );
            Refusal::of("cloudNotNow")
        })?;
        entered(&session, &here, &seed)
    })
    .await??;
    crate::identity::seal_the_cloud(&app.state::<Mutex<Session>>()).await;
    Ok(())
}

#[cfg(test)]
#[path = "entering_test.rs"]
mod tests;
