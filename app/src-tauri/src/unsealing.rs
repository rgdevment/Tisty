use std::path::Path;
use std::sync::Mutex;

use tauri::{AppHandle, Emitter};
use tisty_carrier::{Remote, Sealing, knock};
use tisty_core::event::DeviceId;
use tisty_core::witness::{self, Fact, channel};
use tisty_share_proto::notice::Unsealed;

use crate::handed::Knocking;
use crate::{Session, held};

// Left in the plain root, since the machines still reading the sealed side can never pull the log's own word of it.
pub fn say(session: &Mutex<Session>, plain: &dyn Remote, home: &Path, cloud: &str, named: &str) {
    let (paths, device) = {
        let session = held(session);
        (session.paths.clone(), session.config.device_id.clone())
    };
    let Some(signer) = tisty_core::signing::mine(&paths, &device) else {
        return;
    };
    let notice = Unsealed::signed(
        cloud,
        named,
        &device.0,
        &signer,
        jiff::Timestamp::now().as_second(),
    );
    if let Err(why) = knock::said_unsealed(plain, home, &notice) {
        witness::warn(
            channel::CLOUD,
            "the cloud went back to plain and the notice for the other machines could not be left",
            &[("why", Fact::Why(why.code().to_string()))],
        );
    }
}

// Believed only from a machine the log seats, retiring the very key the log names, and later than the log's own word that it was sealed.
fn believed(session: &Session, notice: &Unsealed) -> bool {
    let state = &session.state;
    let by = DeviceId(notice.by.clone());
    notice.holds()
        && state.devices.contains(&by)
        && !state.assistants.contains(&by)
        && notice
            .key()
            .is_some_and(|key| state.keys.get(&by) == Some(&tisty_core::signing::shown_of(&key)))
        && state.sealed.get(&notice.cloud) == Some(&notice.named)
        && state
            .sealed_at
            .get(&notice.cloud)
            .is_some_and(|at| notice.at > *at)
}

pub fn believed_now(session: &Session) -> Option<String> {
    let (cloud, at) = session.reach.unsealed()?;
    let later = session
        .state
        .sealed_at
        .get(&cloud)
        .is_some_and(|sealed| at > *sealed);
    later.then_some(cloud)
}

fn hears(session: &Session) -> bool {
    let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()) else {
        return false;
    };
    if session.state.sealed_cloud().is_none() || believed_now(session).is_some() {
        return false;
    }
    let home = session.reach.home_of(at);
    if tisty_carrier::Turning::left(&home).is_some() {
        return false;
    }
    let said_plain = matches!(
        tisty_carrier::sealing_of(&tisty_carrier::mirrored_at(&home)),
        Sealing::Plain | Sealing::Unsaid
    );
    said_plain || session.reach.tried().is_some()
}

/// True when this round was spent looking for the notice, so the rest of the knocking waits for the next.
pub fn looked_for(app: &AppHandle, session: &Mutex<Session>, knocking: &Knocking) -> bool {
    let (at, home) = {
        let session = held(session);
        if !hears(&session) {
            return false;
        }
        let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()).cloned() else {
            return false;
        };
        let home = session.reach.home_of(&at);
        (at, home)
    };
    if !crate::handed::due(knocking, crate::handed::ASKED_EVERY) {
        return true;
    }
    let Ok(plain) = crate::handed::reached(knocking, &home, &at) else {
        return true;
    };
    let notice = match knock::unsealed_of(&*plain, &home) {
        Ok(Some(notice)) => notice,
        Ok(None) => return true,
        Err(why) => {
            witness::note(
                channel::CLOUD,
                "the notice of a cloud turned back to plain could not be read",
                &[("why", Fact::Why(why.code().to_string()))],
            );
            return true;
        }
    };
    let took = {
        let session = held(session);
        let took = believed(&session, &notice);
        if took {
            session
                .reach
                .unsealed_is(Some((notice.cloud.clone(), notice.at)));
            session.reach.tried_for(None);
        }
        took
    };
    if took {
        tauri::async_runtime::block_on(crate::identity::seal_the_cloud(session));
        let _ = app.emit("stirred", ());
    }
    true
}

#[cfg(test)]
#[path = "unsealing_test.rs"]
mod tests;
