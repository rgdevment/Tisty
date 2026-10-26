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
    let said_plain = matches!(
        tisty_carrier::sealing_of(&tisty_carrier::mirrored_at(&home)),
        Sealing::Plain | Sealing::Unsaid
    );
    said_plain || session.reach.tried().is_some()
}

fn due(app: &AppHandle) -> bool {
    let hearing = app.state::<Hearing>();
    let mut last = hearing
        .0
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if last.is_some_and(|at| at.elapsed() < HEARD_EVERY) {
        return false;
    }
    *last = Some(Instant::now());
    true
}

/// True once the log no longer names the cloud sealed, so the round can be worked out again.
pub async fn heard(app: &AppHandle, session: &tauri::State<'_, Mutex<Session>>) -> bool {
    let (carrier, here, alive, holds, paths) = {
        let session = held(session);
        if !hears(&session) || !due(app) {
            return false;
        }
        let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()).cloned() else {
            return false;
        };
        let Some(carrier) = session.reach.apart(&at) else {
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
    if let Ok(Err(trouble)) = &heard {
        witness::note(
            channel::SYNC,
            "the plain side of a cloud the log still names sealed could not be heard",
            &[("why", Fact::Why(format!("{trouble:?}")))],
        );
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
