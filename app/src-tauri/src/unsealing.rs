use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};
use tisty_carrier::Sealing;
use tisty_core::witness::{self, Fact, channel};

use crate::{Session, held};

const HEARD_EVERY: Duration = Duration::from_secs(10 * 60);

#[derive(Default)]
pub struct Hearing(Mutex<Option<Instant>>);

// Only the signed log can turn a cloud back to plain, so a place said plain, or one no key here opens, is first only heard.
fn hears(session: &Session) -> bool {
    let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()) else {
        return false;
    };
    if session.state.sealed_cloud().is_none() {
        return false;
    }
    let home = session.reach.home_of(at);
    // A turn of this machine reads its own side, and its journal says which.
    if tisty_carrier::Turning::left(&home).is_some() {
        return false;
    }
    let said_plain = matches!(
        tisty_carrier::sealing_of(&tisty_carrier::mirrored_at(&home)),
        Sealing::Plain | Sealing::Unsaid
    );
    said_plain || session.reach.tried().is_some()
}

fn due(app: &AppHandle) -> bool {
    app.state::<Hearing>()
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_none_or(|at| at.elapsed() >= HEARD_EVERY)
}

// Stamped once a hearing ran, so one that found the round busy or the provider resting is tried again next round.
fn heard_now(app: &AppHandle) {
    *app.state::<Hearing>()
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Instant::now());
}

pub async fn heard(app: &AppHandle, session: &tauri::State<'_, Mutex<Session>>) -> bool {
    let (carrier, here, alive, holds, paths) = {
        let session = held(session);
        if !hears(&session) || !due(app) {
            return false;
        }
        let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()).cloned() else {
            return false;
        };
        let Some(carrier) = session.reach.hearing(&at) else {
            return false;
        };
        (
            carrier,
            session.here(),
            session.alive(),
            session.config.holds(),
            session.paths.clone(),
        )
    };
    let store = paths.store();
    let before = tisty_core::cache::fingerprint(&store);
    let heard = tauri::async_runtime::spawn_blocking(move || {
        let mut saying = |_| {};
        carrier.hear(
            &here,
            tisty_carrier::Round {
                way: tisty_carrier::Way::Pull,
                alive: &alive,
                holds,
                saying: &mut saying,
            },
        )
    })
    .await;
    let waited = matches!(
        &heard,
        Ok(Ok(moved)) if moved.deferred.as_ref().is_some_and(|one| matches!(
            one.reason,
            tisty_carrier::Reason::Busy | tisty_carrier::Reason::Limit | tisty_carrier::Reason::Offline
        ))
    );
    if !waited {
        heard_now(app);
    }
    match &heard {
        // Still said sealed is what a machine without the key expects to hear, not a trouble.
        Ok(Err(tisty_carrier::Trouble::Sealed(_))) | Ok(Ok(_)) => {}
        Ok(Err(trouble)) => witness::note(
            channel::SYNC,
            "the plain side of a cloud the log still names sealed could not be heard",
            &[("why", Fact::Why(format!("{trouble:?}")))],
        ),
        Err(_) => witness::warn(
            channel::SYNC,
            "hearing the plain side of a sealed cloud fell over",
            &[],
        ),
    }
    if tisty_core::cache::fingerprint(&store) != before
        && crate::answers::storing::catching_up(
            session,
            &paths,
            "the store would not project after hearing the plain side",
            channel::SYNC,
        )
        .await
        .is_err()
    {
        return false;
    }
    held(session).state.sealed_cloud().is_none()
}

#[cfg(test)]
#[path = "unsealing_test.rs"]
mod tests;
