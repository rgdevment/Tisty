use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};
use tisty_carrier::{Remote, Sealing, knock};
use tisty_core::Paths;
use tisty_core::event::DeviceId;
use tisty_core::witness::{self, Fact, channel};
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::handing::{Ask, OneUse, SigningKey, handed};

use crate::{Session, held};

const KEY: &str = "key";
const JOINED_FOR: i64 = 7 * 24 * 60 * 60;
pub(crate) const NEW_FOR: i64 = 60 * 60;
// Asked again within the hour, so an ask lost or taken away up there never leaves this machine waiting a week.
const ASKED_AGAIN: i64 = 60 * 60;
const ANSWERED_EVERY: Duration = Duration::from_secs(5 * 60);
const ASKED_EVERY: Duration = Duration::from_secs(2 * 60);

/// The one-use key lives only here, so a restart asks again rather than leaving a secret on disk.
#[derive(Default)]
pub struct Knocking {
    busy: AtomicBool,
    pub(crate) asked: Mutex<Option<(Ask, OneUse)>>,
    looked: Mutex<Option<Instant>>,
    opened: Mutex<Option<String>>,
    pub(crate) trusted: Mutex<Option<String>>,
    pub(crate) coded: Mutex<Option<(String, String, String)>>,
    reached: Mutex<Option<(std::path::PathBuf, Arc<dyn Remote>)>>,
}

pub(crate) struct Here {
    pub(crate) paths: Paths,
    pub(crate) device: DeviceId,
    pub(crate) cloud: String,
    pub(crate) plain: Arc<dyn Remote>,
    pub(crate) home: std::path::PathBuf,
}

pub(crate) fn here_now(
    session: &Mutex<Session>,
    knocking: &Knocking,
) -> Result<(Here, Option<CloudKey>), String> {
    let (at, home, device, paths, keyed) = {
        let session = held(session);
        let at = tisty_carrier::cloud_of(session.config.sync.as_ref())
            .cloned()
            .ok_or("this computer syncs with no cloud")?;
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
        return Err("the cloud is not sealed".into());
    };
    let plain = reached(knocking, &home, &at)?;
    let here = Here {
        paths,
        device,
        cloud,
        plain,
        home,
    };
    let key = keyed
        .filter(|keyed| keyed.locked.cloud == here.cloud)
        .map(|keyed| keyed.key)
        .filter(|key| opened(knocking, &here, key));
    Ok((here, key))
}

struct Free<'a>(&'a AtomicBool);

impl Drop for Free<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

pub fn tend(app: AppHandle) {
    if app.state::<Knocking>().busy.swap(true, Ordering::AcqRel) {
        return;
    }
    std::thread::spawn(move || {
        let knocking = app.state::<Knocking>();
        let _free = Free(&knocking.busy);
        tended(&app, &knocking);
    });
}

// Looked at before the provider is reached, so a round with nothing due costs no keychain read and no client.
fn due(knocking: &Knocking, every: Duration) -> bool {
    let mut looked = knocking
        .looked
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if looked.is_some_and(|at| at.elapsed() < every) {
        return false;
    }
    *looked = Some(Instant::now());
    true
}

fn tended(app: &AppHandle, knocking: &Knocking) {
    let session = app.state::<Mutex<Session>>();
    let (home, keyed) = {
        let session = held(&session);
        let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()) else {
            return;
        };
        (session.reach.home_of(at), session.reach.keyed())
    };
    let Sealing::Sealed(cloud) = tisty_carrier::sealing_of(&tisty_carrier::mirrored_at(&home))
    else {
        return;
    };
    let every = match keyed.is_some_and(|keyed| keyed.locked.cloud == cloud) {
        true => ANSWERED_EVERY,
        false => ASKED_EVERY,
    };
    if !due(knocking, every) {
        return;
    }
    let Ok((here, key)) = here_now(&session, knocking) else {
        return;
    };
    // A key only guessed from a phrase may not be this cloud's, and handing it out would hand out nothing.
    let done = match key {
        Some(key) => answering(&session, &here, &key),
        None => asking(&session, knocking, &here).map(|took| {
            if took {
                let _ = app.emit("stirred", ());
            }
        }),
    };
    if let Err(why) = done {
        noted(why);
    }
}

fn noted(why: String) {
    witness::note(
        channel::CLOUD,
        "the cloud's key could not be asked for or handed this round",
        &[("why", Fact::Why(why))],
    );
}

// Kept while the place stays the same, so a window asking every few seconds never reads the keychain each time.
fn reached(
    knocking: &Knocking,
    home: &std::path::Path,
    at: &tisty_core::config::CloudAt,
) -> Result<Arc<dyn Remote>, String> {
    let mut kept = knocking
        .reached
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some((was, plain)) = kept.as_ref()
        && was == home
    {
        return Ok(plain.clone());
    }
    let plain = crate::reaching::plain_at(home, at)?;
    *kept = Some((home.to_path_buf(), plain.clone()));
    Ok(plain)
}

pub(crate) fn now() -> i64 {
    jiff::Timestamp::now().as_second()
}

pub(crate) fn signer(here: &Here) -> Result<SigningKey, String> {
    tisty_core::signing::mine(&here.paths, &here.device)
        .ok_or_else(|| "this machine has no signing key".to_string())
}

pub(crate) fn hitched(hitch: tisty_carrier::Hitch) -> String {
    hitch.code().to_string()
}

pub(crate) fn name_here(session: &Mutex<Session>, device: &DeviceId) -> String {
    let session = held(session);
    session
        .state
        .named
        .get(device)
        .map(|one| one.name.clone())
        .unwrap_or_else(|| tisty_core::config::nicknamed(&device.0))
}

pub(crate) fn ask_now(
    session: &Mutex<Session>,
    asked: &mut Option<(Ask, OneUse)>,
    here: &Here,
) -> Result<(), String> {
    let one_use = OneUse::minted().ok_or("no randomness for a one-use key")?;
    let ask = Ask::signed(
        &here.device.0,
        &name_here(session, &here.device),
        &signer(here)?,
        &one_use,
        &here.cloud,
        now(),
    );
    knock::knocked(&*here.plain, &here.home, &ask).map_err(hitched)?;
    *asked = Some((ask, one_use));
    Ok(())
}

pub(crate) fn asking(
    session: &Mutex<Session>,
    knocking: &Knocking,
    here: &Here,
) -> Result<bool, String> {
    let seated = held(session).state.devices.contains(&here.device);
    let mut asked = knocking.asked.lock().map_err(|e| e.to_string())?;
    let stale = asked
        .as_ref()
        .is_none_or(|(ask, _)| ask.cloud != here.cloud || now() - ask.at > ASKED_AGAIN);
    if stale {
        if seated {
            ask_now(session, &mut asked, here)?;
        }
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
    let trusted = knocking
        .trusted
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let vouched = from_ours(session, &answer) || trusted.as_deref() == Some(answer.key.as_str());
    // A machine no log names waits for the person to say the digits match before it judges any answer.
    if !vouched && !seated && trusted.is_none() {
        return Ok(false);
    }
    let opened = answer.opened(ask, one_use).filter(|_| vouched);
    let opened = match opened {
        Some(key) => opens(here, &key)?.then_some(key),
        None => None,
    };
    let Some(key) = opened else {
        // An answer that does not open, stale or forged, would keep every machine from answering again.
        knock::unanswered(&*here.plain, &here.home, &here.device.0).map_err(hitched)?;
        return Ok(false);
    };
    took(session, here, &key)?;
    *asked = None;
    *knocking
        .trusted
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = None;
    Ok(true)
}

fn opens(here: &Here, key: &CloudKey) -> Result<bool, String> {
    knock::opens(&*here.plain, &here.home, key, &here.cloud).map_err(hitched)
}

// The mark is fetched once a key, not every time the box is looked at.
fn opened(knocking: &Knocking, here: &Here, key: &CloudKey) -> bool {
    let named = key.named_for(&here.cloud);
    let mut known = knocking
        .opened
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if known.as_deref() == Some(named.as_str()) {
        return true;
    }
    let opens = opens(here, key).unwrap_or(false);
    if opens {
        *known = Some(named);
    }
    opens
}

// The mark sits with the provider, who could seal a mark of its own, so only a machine the log knows may hand a key.
fn from_ours(session: &Mutex<Session>, answer: &tisty_share_proto::handing::Handed) -> bool {
    seated(session, &answer.by)
        && answer.giver().is_some_and(|key| {
            held(session).state.keys.get(&DeviceId(answer.by.clone()))
                == Some(&tisty_core::signing::shown_of(&key))
        })
}

fn took(session: &Mutex<Session>, here: &Here, key: &CloudKey) -> Result<(), String> {
    keep(&here.cloud, key)?;
    let was = {
        let mut session = held(session);
        let was = session.config.key_handed.clone();
        let cloud = here.cloud.clone();
        session
            .keep(|config| config.key_handed = Some(cloud))
            .map_err(|refusal| refusal.code.to_string())?;
        session.reach.tried_for(None);
        session.reach.seal_with(None);
        was
    };
    if let Some(was) = was.filter(|was| *was != here.cloud) {
        forget(&was);
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

// One ask that cannot be answered or taken away never holds up the others.
fn answering(session: &Mutex<Session>, here: &Here, key: &CloudKey) -> Result<(), String> {
    let knocks = knock::knocks(&*here.plain, &here.home).map_err(hitched)?;
    let signer = signer(here)?;
    for one in knocks {
        if let Err(why) = answered(session, here, key, &signer, &one) {
            noted(why);
        }
    }
    Ok(())
}

fn answered(
    session: &Mutex<Session>,
    here: &Here,
    key: &CloudKey,
    signer: &SigningKey,
    one: &knock::Knock,
) -> Result<(), String> {
    let seated = seated(session, &one.device);
    let Some(ask) = &one.ask else {
        if seated {
            return Ok(());
        }
        // An answer waits for its machine; a new one has its hour, after which nobody comes for it.
        let fresh = knock::ask_of(&*here.plain, &here.home, &one.device)
            .map_err(hitched)?
            .is_some_and(|ask| (-NEW_FOR..=NEW_FOR).contains(&(now() - ask.at)));
        return match fresh {
            true => Ok(()),
            false => knock::withdrawn(&*here.plain, &here.home, &one.device).map_err(hitched),
        };
    };
    let age = now() - ask.at;
    let lasts = if seated { JOINED_FOR } else { NEW_FOR };
    // The time only tidies the box; an old ask replayed opens for nobody without its one-use key.
    if age > lasts || age < -JOINED_FOR {
        return knock::withdrawn(&*here.plain, &here.home, &one.device).map_err(hitched);
    }
    if !seated || !signs_as_known(session, ask) || !fits(ask, here) {
        return Ok(());
    }
    let answer = handed(ask, key, &here.device.0, signer)
        .ok_or("the cloud's key could not be sealed for the machine that asked")?;
    knock::answer(&*here.plain, &here.home, &one.device, &answer).map_err(hitched)
}

pub(crate) fn fits(ask: &Ask, here: &Here) -> bool {
    ask.holds() && ask.cloud == here.cloud && ask.device != here.device.0
}

pub(crate) fn seated(session: &Mutex<Session>, device: &str) -> bool {
    let session = held(session);
    let device = DeviceId(device.to_string());
    session.state.devices.contains(&device) && !session.state.assistants.contains(&device)
}

fn signs_as_known(session: &Mutex<Session>, ask: &Ask) -> bool {
    let session = held(session);
    ask.key().is_some_and(|key| {
        session.state.keys.get(&DeviceId(ask.device.clone()))
            == Some(&tisty_core::signing::shown_of(&key))
    })
}

// Under the cloud's id, which joining stores never changes, and beside the seed: a handed key is no identity.
fn scope(cloud: &str) -> String {
    format!("cloud-{cloud}")
}

pub fn kept(cloud: &str) -> Result<Option<CloudKey>, String> {
    let secrets = crate::reaching::seed_secrets(&scope(cloud)).map_err(|e| format!("{e:?}"))?;
    let Some(said) = secrets.read(KEY).map_err(|e| format!("{e:?}"))? else {
        return Ok(None);
    };
    let said = zeroize::Zeroizing::new(said);
    let key = CloudKey::from_text(&said);
    if key.is_none() {
        witness::warn(
            channel::STORE,
            "what was kept as the cloud's key is not one, so it counts as none",
            &[],
        );
    }
    Ok(key)
}

pub fn keep(cloud: &str, key: &CloudKey) -> Result<(), String> {
    let secrets = crate::reaching::seed_secrets(&scope(cloud)).map_err(|e| format!("{e:?}"))?;
    secrets
        .write(KEY, &key.to_text())
        .map_err(|e| format!("{e:?}"))
}

// A handed key is handed again by any other machine, so one for a cloud left behind is not kept.
fn forget(cloud: &str) {
    let gone = crate::reaching::seed_secrets(&scope(cloud)).and_then(|secrets| secrets.forget(KEY));
    if let Err(why) = gone {
        witness::note(
            channel::STORE,
            "the key of a cloud this computer no longer reads could not be let go of",
            &[("why", Fact::Why(format!("{why:?}")))],
        );
    }
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
pub(crate) mod tests;
