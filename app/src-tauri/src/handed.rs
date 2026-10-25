use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use tauri::{AppHandle, Emitter, Manager};
use tisty_carrier::{Remote, Sealing, knock};
use tisty_core::Paths;
use tisty_core::event::DeviceId;
use tisty_core::witness::{self, Fact, channel};
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::handing::{Ask, OneUse, SigningKey, handed};
use zeroize::Zeroizing;

use crate::{Session, held};

const KEY: &str = "key";
const JOINED_FOR: i64 = 7 * 24 * 60 * 60;
const NEW_FOR: i64 = 60 * 60;
const AHEAD_AT_MOST: i64 = 5 * 60;
const LOOKED_EVERY: Duration = Duration::from_secs(5 * 60);

/// The one-use key lives only here, so a restart asks again rather than leaving a secret on disk.
#[derive(Default)]
pub struct Knocking {
    busy: AtomicBool,
    asked: Mutex<Option<(Ask, OneUse)>>,
    looked: Mutex<Option<Instant>>,
}

struct Here {
    paths: Paths,
    device: DeviceId,
    cloud: String,
    plain: Arc<dyn Remote>,
    home: std::path::PathBuf,
}

/// Run beside each round and never in its way: a machine without the key asks, one with it answers.
pub fn tend(app: AppHandle) {
    std::thread::spawn(move || {
        let knocking = app.state::<Knocking>();
        if knocking.busy.swap(true, Ordering::AcqRel) {
            return;
        }
        tended(&app, &knocking);
        knocking.busy.store(false, Ordering::Release);
    });
}

fn tended(app: &AppHandle, knocking: &Knocking) {
    let session = app.state::<Mutex<Session>>();
    let (at, home, device, paths, keyed) = {
        let session = held(&session);
        let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()).cloned() else {
            return;
        };
        let home = session.reach.home_of(&at);
        let device = session.config.device_id.clone();
        (
            at,
            home,
            device,
            session.paths.clone(),
            session.reach.keyed(),
        )
    };
    let Sealing::Sealed(cloud) = tisty_carrier::sealing_of(&tisty_carrier::mirrored_at(&home))
    else {
        return;
    };
    let Ok(plain) = crate::reaching::plain_at(&home, &at) else {
        return;
    };
    let here = Here {
        paths,
        device,
        cloud,
        plain,
        home,
    };
    let done = match keyed.filter(|keyed| keyed.locked.cloud == here.cloud) {
        Some(keyed) => answering(&session, knocking, &here, &keyed.key),
        None => asking(&session, knocking, &here).map(|took| {
            if took {
                let _ = app.emit("stirred", ());
            }
        }),
    };
    if let Err(why) = done {
        witness::note(
            channel::CLOUD,
            "the cloud's key could not be asked for or handed this round",
            &[("why", Fact::Why(why))],
        );
    }
}

fn now() -> i64 {
    jiff::Timestamp::now().as_second()
}

fn signer(here: &Here) -> Result<SigningKey, String> {
    tisty_core::signing::mine(&here.paths, &here.device)
        .ok_or_else(|| "this machine has no signing key".to_string())
}

fn hitched(hitch: tisty_carrier::Hitch) -> String {
    hitch.code().to_string()
}

/// Only a machine already on the list asks on its own; a new one asks once the person says so.
fn asking(session: &Mutex<Session>, knocking: &Knocking, here: &Here) -> Result<bool, String> {
    let (seated, name) = {
        let session = held(session);
        let name = session
            .state
            .named
            .get(&here.device)
            .map(|one| one.name.clone())
            .unwrap_or_else(|| tisty_core::config::nicknamed(&here.device.0));
        (session.state.devices.contains(&here.device), name)
    };
    let mut asked = knocking.asked.lock().map_err(|e| e.to_string())?;
    let stale = asked
        .as_ref()
        .is_none_or(|(ask, _)| ask.cloud != here.cloud || now() - ask.at > JOINED_FOR);
    if stale {
        if !seated {
            return Ok(false);
        }
        let one_use = OneUse::minted().ok_or("no randomness for a one-use key")?;
        let ask = Ask::signed(
            &here.device.0,
            &name,
            &signer(here)?,
            &one_use,
            &here.cloud,
            now(),
        );
        knock::knocked(&*here.plain, &here.home, &ask).map_err(hitched)?;
        *asked = Some((ask, one_use));
        return Ok(false);
    }
    let Some((ask, one_use)) = asked.as_ref() else {
        return Ok(false);
    };
    let Some(answer) =
        knock::answered(&*here.plain, &here.home, &here.device.0).map_err(hitched)?
    else {
        return Ok(false);
    };
    let Some(key) = answer.opened(ask, one_use).filter(|key| opens(here, key)) else {
        return Ok(false);
    };
    took(session, here, &key)?;
    *asked = None;
    Ok(true)
}

fn opens(here: &Here, key: &CloudKey) -> bool {
    knock::opens(&*here.plain, &here.home, key, &here.cloud)
}

fn took(session: &Mutex<Session>, here: &Here, key: &CloudKey) -> Result<(), String> {
    keep(&here.paths, key)?;
    let named = key.named_for(&here.cloud);
    {
        let mut session = held(session);
        session
            .keep(|config| config.key_handed = Some(named))
            .map_err(|refusal| refusal.code.to_string())?;
        session.reach.tried_for(None);
    }
    if let Err(why) = knock::withdrawn(&*here.plain, &here.home, &here.device.0) {
        witness::note(
            channel::CLOUD,
            "the cloud's key came, and the ask for it stays up there until the next round",
            &[("why", Fact::Why(hitched(why)))],
        );
    }
    tauri::async_runtime::block_on(crate::identity::seal_the_cloud(session));
    Ok(())
}

fn answering(
    session: &Mutex<Session>,
    knocking: &Knocking,
    here: &Here,
    key: &CloudKey,
) -> Result<(), String> {
    {
        let mut looked = knocking.looked.lock().map_err(|e| e.to_string())?;
        if looked.is_some_and(|at| at.elapsed() < LOOKED_EVERY) {
            return Ok(());
        }
        *looked = Some(Instant::now());
    }
    let knocks = knock::knocks(&*here.plain, &here.home).map_err(hitched)?;
    for one in knocks {
        let (seated, known) = seat_of(session, &one.ask);
        let age = now() - one.ask.at;
        if age > if seated { JOINED_FOR } else { NEW_FOR } {
            knock::withdrawn(&*here.plain, &here.home, &one.ask.device).map_err(hitched)?;
            continue;
        }
        if one.answered || !known || !fits(&one.ask, here, age) {
            continue;
        }
        let answer = handed(&one.ask, key, &here.device.0, &signer(here)?)
            .ok_or("the cloud's key could not be sealed for the machine that asked")?;
        knock::answer(&*here.plain, &here.home, &one.ask.device, &answer).map_err(hitched)?;
    }
    Ok(())
}

fn fits(ask: &Ask, here: &Here, age: i64) -> bool {
    ask.holds() && ask.cloud == here.cloud && ask.device != here.device.0 && age >= -AHEAD_AT_MOST
}

/// Seated, and asking with the very key the log knows it signs with.
fn seat_of(session: &Mutex<Session>, ask: &Ask) -> (bool, bool) {
    let session = held(session);
    let device = DeviceId(ask.device.clone());
    let seated =
        session.state.devices.contains(&device) && !session.state.assistants.contains(&device);
    let known = seated
        && ask.key().is_some_and(|key| {
            session.state.keys.get(&device) == Some(&tisty_core::signing::shown_of(&key))
        });
    (seated, known)
}

/// Beside the seed and never in its place: a machine handed the cloud's key holds no identity.
fn scope(paths: &Paths) -> Result<String, String> {
    tisty_core::store::peek_identity(paths.store())
        .map(|store| format!("cloud-{store}"))
        .ok_or_else(|| "this store has no name yet".to_string())
}

pub fn kept(paths: &Paths) -> Result<Option<CloudKey>, String> {
    let secrets = crate::reaching::seed_secrets(&scope(paths)?).map_err(|e| format!("{e:?}"))?;
    let Some(said) = secrets.read(KEY).map_err(|e| format!("{e:?}"))? else {
        return Ok(None);
    };
    let said = Zeroizing::new(said);
    let bytes = Zeroizing::new(URL_SAFE_NO_PAD.decode(said.as_bytes()).unwrap_or_default());
    match <[u8; 32]>::try_from(bytes.as_slice()) {
        Ok(key) => Ok(Some(CloudKey::from_bytes(&Zeroizing::new(key)))),
        Err(_) => {
            witness::warn(
                channel::STORE,
                "what was kept as the cloud's key is not one, so it counts as none",
                &[],
            );
            Ok(None)
        }
    }
}

pub fn keep(paths: &Paths, key: &CloudKey) -> Result<(), String> {
    let secrets = crate::reaching::seed_secrets(&scope(paths)?).map_err(|e| format!("{e:?}"))?;
    let said = Zeroizing::new(URL_SAFE_NO_PAD.encode(key.to_bytes().as_ref()));
    secrets.write(KEY, &said).map_err(|e| format!("{e:?}"))
}

pub fn unread(why: String) {
    witness::warn(
        channel::STORE,
        "the cloud's key handed to this computer could not be read",
        &[("why", Fact::Why(why))],
    );
}

#[cfg(test)]
#[path = "handed_test.rs"]
mod tests;
