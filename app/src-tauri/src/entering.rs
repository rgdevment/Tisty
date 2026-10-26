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
    pub device: String,
    pub name: String,
    pub os: Option<String>,
    pub at: i64,
}

fn heard(session: &Session) -> Vec<Heard> {
    let state = &session.state;
    // A clock once ahead would pin a machine in the future for good, so nothing reads later than now.
    let now = jiff::Timestamp::now().as_second();
    let mut heard: Vec<Heard> = tisty_core::person::machines(state)
        .into_iter()
        .filter(|device| *device != session.config.device_id)
        .filter_map(|device| {
            let named = state.named.get(&device);
            Some(Heard {
                name: named.map_or_else(
                    || tisty_core::config::nicknamed(&device.0),
                    |one| one.name.clone(),
                ),
                os: named.and_then(|one| one.os.clone()),
                at: (*state.heard.get(&device)?).min(now),
                device: device.0,
            })
        })
        .collect();
    heard.sort_by_key(|one| std::cmp::Reverse(one.at));
    heard
}

#[tauri::command(async)]
pub fn sealing_heard(session: tauri::State<'_, Mutex<Session>>) -> Vec<Heard> {
    heard(&held(&session))
}

fn checked(here: &Here, key: &CloudKey) -> Answer<()> {
    match opens(here, key) {
        Ok(true) => Ok(()),
        Ok(false) => Err(Refusal::of("phraseNotThisCloud")),
        Err(why) => {
            tisty_core::witness::note(
                tisty_core::witness::channel::CLOUD,
                "the cloud's mark could not be read to check a phrase",
                &[("why", tisty_core::witness::Fact::Why(why))],
            );
            Err(Refusal::of("cloudNotNow"))
        }
    }
}

// Only the cloud's key is kept, never the seed: the root signs for this computer once and is let go.
fn entered(session: &Mutex<Session>, knocking: &Knocking, here: &Here, seed: &Seed) -> Answer<()> {
    let key = CloudKey::of(seed);
    checked(here, &key)?;
    {
        // Checked and signed under one hold, so a round in between never meets a second person.
        let mut session = held(session);
        if !crate::identity::fits(seed, &session.state) {
            return Err(Refusal::of("phraseOtherPerson"));
        }
        crate::identity::begun(&mut session, seed)?;
    }
    kept_for(session, &here.cloud, &key).map_err(|why| {
        tisty_core::witness::warn(
            tisty_core::witness::channel::STORE,
            "the cloud's key could not be kept on this computer",
            &[("why", tisty_core::witness::Fact::Why(why))],
        );
        Refusal::of("identityNotKept")
    })?;
    if let Ok(mut asked) = knocking.asked.lock() {
        *asked = None;
    }
    // An ask left up would be answered with a copy of the key nobody comes for.
    if let Err(why) = tisty_carrier::knock::withdrawn(&*here.plain, &here.home, &here.device.0) {
        tisty_core::witness::note(
            tisty_core::witness::channel::CLOUD,
            "the cloud was opened with the phrase, and this computer's ask stays up there for now",
            &[(
                "why",
                tisty_core::witness::Fact::Why(why.code().to_string()),
            )],
        );
    }
    Ok(())
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
        entered(&session, &knocking, &here, &seed)
    })
    .await??;
    crate::identity::seal_the_cloud(&app.state::<Mutex<Session>>()).await;
    Ok(())
}

#[cfg(test)]
#[path = "entering_test.rs"]
mod tests;
