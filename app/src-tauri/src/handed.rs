use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};
use tisty_carrier::{Remote, Sealing, knock};
use tisty_core::Paths;
use tisty_core::event::DeviceId;
use tisty_core::witness::{self, Fact, channel};
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::handing::{Ask, Handed, OneUse, SigningKey, handed};

use crate::{Session, held};

const KEY: &str = "key";
const JOINED_FOR: i64 = 7 * 24 * 60 * 60;
pub(crate) const NEW_FOR: i64 = 60 * 60;
// Asked again within the hour, so an ask lost or taken away up there never leaves this machine waiting a week.
const ASKED_AGAIN: i64 = 60 * 60;
const ANSWERED_EVERY: Duration = Duration::from_secs(5 * 60);
pub(crate) const ASKED_EVERY: Duration = Duration::from_secs(2 * 60);

/// The one-use key lives only here, so a restart asks again rather than leaving a secret on disk.
#[derive(Default)]
pub struct Knocking {
    busy: AtomicBool,
    pub(crate) asked: Mutex<Option<(Ask, OneUse)>>,
    looked: Mutex<Option<Instant>>,
    opened: Mutex<Option<String>>,
    pub(crate) trusted: Mutex<Option<String>>,
    pub(crate) coded: Mutex<Option<(String, String, String)>>,
    pub(crate) revealed: Mutex<Option<String>>,
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
    let (here, keyed) = place_now(session, knocking)?;
    let key = match keyed.filter(|keyed| keyed.locked.cloud == here.cloud) {
        Some(keyed) => opened(knocking, &here, &keyed.key)?.then_some(keyed.key),
        None => None,
    };
    Ok((here, key))
}

// Only the place: taking an ask away needs no key, so it never waits on the mark.
pub(crate) fn place_now(
    session: &Mutex<Session>,
    knocking: &Knocking,
) -> Result<(Here, Option<crate::reaching::Keyed>), String> {
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
    Ok((here, keyed))
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
pub(crate) fn due(knocking: &Knocking, every: Duration) -> bool {
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
    if crate::unsealing::looked_for(app, &session, knocking) {
        return;
    }
    let (home, keyed, named) = {
        let session = held(&session);
        let Some(at) = tisty_carrier::cloud_of(session.config.sync.as_ref()) else {
            return;
        };
        (
            session.reach.home_of(at),
            session.reach.keyed(),
            session.state.sealed.clone(),
        )
    };
    let Sealing::Sealed(cloud) = tisty_carrier::sealing_of(&tisty_carrier::mirrored_at(&home))
    else {
        return;
    };
    let every = match keyed
        .as_ref()
        .is_some_and(|keyed| keyed.locked.cloud == cloud)
    {
        true => ANSWERED_EVERY,
        false => ASKED_EVERY,
    };
    if !due(knocking, every) {
        return;
    }
    let Ok((here, key)) = here_now(&session, knocking) else {
        return;
    };
    // A key the signed log names for this cloud is this cloud's even when the mark up there will not open: that mark was spoiled.
    let logged = keyed
        .is_some_and(|keyed| named.get(&here.cloud) == Some(&keyed.key.named_for(&here.cloud)));
    // A key only guessed from a phrase may not be this cloud's, and handing it out would hand out nothing.
    let done = match (key, logged) {
        (Some(key), _) => answering(&session, &here, &key),
        (None, true) => {
            Err("the cloud's mark does not open with the key the log names for it".into())
        }
        (None, false) => asking(&session, knocking, &here).map(|took| {
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
pub(crate) fn reached(
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
    let seated = joined_here(&held(session), &here.home);
    let mut asked = knocking
        .asked
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let stale = asked
        .as_ref()
        .is_none_or(|(ask, _)| ask.cloud != here.cloud || now() - ask.at > ASKED_AGAIN);
    // A machine off the list asks again only from its window, which also knows whether the person trusted a giver.
    if stale && seated {
        ask_now(session, &mut asked, here)?;
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
    let vouched = from_ours(session, &answer)
        || trusted.as_deref() == Some(answer.key.as_str())
        || ours_vouched(session, &answer, ask);
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

pub(crate) fn opens(here: &Here, key: &CloudKey) -> Result<bool, String> {
    knock::opens(&*here.plain, &here.home, key, &here.cloud).map_err(hitched)
}

// Fetched once a key, and a mark that could not be fetched is a hitch, never a wrong key.
fn opened(knocking: &Knocking, here: &Here, key: &CloudKey) -> Result<bool, String> {
    let named = key.named_for(&here.cloud);
    let mut known = knocking
        .opened
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if known.as_deref() == Some(named.as_str()) {
        return Ok(true);
    }
    let opens = opens(here, key)?;
    if opens {
        *known = Some(named);
    }
    Ok(opens)
}

// The mark sits with the provider, who could seal a mark of its own, so only a machine the log knows may hand a key.
fn from_ours(session: &Mutex<Session>, answer: &Handed) -> bool {
    seated(session, &answer.by)
        && answer.giver().is_some_and(|key| {
            held(session).state.keys.get(&DeviceId(answer.by.clone()))
                == Some(&tisty_core::signing::shown_of(&key))
        })
}

pub(crate) fn kept_for(
    session: &Mutex<Session>,
    cloud: &str,
    key: &CloudKey,
) -> Result<(), String> {
    keep(cloud, key)?;
    let was = {
        let mut session = held(session);
        let was = session.config.key_handed.clone();
        let named = cloud.to_string();
        session
            .keep(|config| config.key_handed = Some(named))
            .map_err(|refusal| refusal.code.to_string())?;
        session.reach.tried_for(None);
        session.reach.seal_with(None);
        was
    };
    if let Some(was) = was.filter(|was| was != cloud) {
        forget(&was);
    }
    Ok(())
}

fn took(session: &Mutex<Session>, here: &Here, key: &CloudKey) -> Result<(), String> {
    kept_for(session, &here.cloud, key)?;
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
    let voucher = std::cell::OnceCell::new();
    for one in knocks {
        if let Err(why) = answered(session, here, key, &signer, &voucher, &one) {
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
    voucher: &std::cell::OnceCell<Option<Voucher>>,
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
    let voucher = voucher.get_or_init(|| self::voucher(session, here));
    let answer = sealed_for(here, ask, key, signer, voucher.as_ref())?;
    knock::answer(&*here.plain, &here.home, &one.device, &answer).map_err(hitched)
}

pub(crate) type Voucher = (
    tisty_share_proto::identity::ShareKey,
    tisty_share_proto::identity::Certificate,
);

// Read once a round and never minted here: a key no root certified vouches for nothing.
pub(crate) fn voucher(session: &Mutex<Session>, here: &Here) -> Option<Voucher> {
    let at = tisty_core::person::kept_at(&here.paths, &here.device)?;
    let bytes = zeroize::Zeroizing::new(std::fs::read(at).ok()?);
    let share = tisty_share_proto::identity::ShareKey::from_bytes(
        &<[u8; 32]>::try_from(bytes.as_slice()).ok()?,
    );
    let session = held(session);
    let persons = &session.state.persons;
    let person = persons.person(&here.device)?;
    let vouched = persons.vouched_by(&here.device, &person)?;
    if vouched.share != share.public() {
        return None;
    }
    let certificate = persons
        .certificates
        .iter()
        .find(|one| one.unchecked() == Some(vouched))?
        .clone();
    Some((share, certificate))
}

// Signed again with the share key the person's root certified, so an asker whose log is behind still knows it is theirs.
pub(crate) fn sealed_for(
    here: &Here,
    ask: &Ask,
    key: &CloudKey,
    signer: &SigningKey,
    voucher: Option<&Voucher>,
) -> Result<Handed, String> {
    let mut answer = handed(ask, key, &here.device.0, signer)
        .ok_or("the cloud's key could not be sealed for the machine that asked")?;
    if let Some((share, certificate)) = voucher {
        answer.vouch(ask, share, certificate);
    }
    Ok(answer)
}

fn ours_vouched(session: &Mutex<Session>, answer: &Handed, ask: &Ask) -> bool {
    let session = held(session);
    let persons = &session.state.persons;
    matches!(
        answer.vouched_for(ask, &persons.revocations),
        tisty_share_proto::identity::Verdict::Holds { person, .. } if persons.names(&person)
    )
}

pub(crate) fn fits(ask: &Ask, here: &Here) -> bool {
    ask.holds() && ask.cloud == here.cloud && ask.device != here.device.0
}

pub(crate) fn joined_here(session: &Session, home: &std::path::Path) -> bool {
    let here = &session.config.device_id;
    session.state.devices.contains(here)
        && !session.state.assistants.contains(here)
        && tisty_carrier::cloud_joined(home, &session.here())
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

// Any other machine hands the key again, so a cloud left or turned back to plain keeps none of it here.
pub(crate) fn let_go(session: &Mutex<Session>, knocking: &Knocking) {
    knocking.reset();
    let cloud = {
        let mut session = held(session);
        session.reach.tried_for(None);
        session.reach.seal_with(None);
        let cloud = session.config.key_handed.clone();
        if cloud.is_some()
            && let Err(why) = session.keep(|config| config.key_handed = None)
        {
            witness::warn(
                channel::CONFIG,
                "the cloud this computer was handed the key of could not be let go of",
                &[("why", Fact::Why(why.code.to_string()))],
            );
        }
        cloud
    };
    if let Some(cloud) = cloud {
        forget(&cloud);
    }
}

impl Knocking {
    fn reset(&self) {
        *self.asked.lock().unwrap_or_else(PoisonError::into_inner) = None;
        *self.trusted.lock().unwrap_or_else(PoisonError::into_inner) = None;
        *self.coded.lock().unwrap_or_else(PoisonError::into_inner) = None;
        *self.revealed.lock().unwrap_or_else(PoisonError::into_inner) = None;
        *self.reached.lock().unwrap_or_else(PoisonError::into_inner) = None;
        *self.opened.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}

// Only the account this machine syncs with holds the cloud that key opens.
pub(crate) fn let_go_of(
    session: &Mutex<Session>,
    knocking: &Knocking,
    at: &tisty_core::config::CloudAt,
) {
    let ours = tisty_carrier::cloud_of(held(session).config.sync.as_ref())
        .is_some_and(|now| now.same_account(at));
    if ours {
        let_go(session, knocking);
    }
}

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
