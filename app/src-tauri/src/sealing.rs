use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tisty_carrier::{Hitch, Left, Remote, Sealing, Stepped, Turn, Turning};
use tisty_core::Op;
use tisty_core::config::CloudAt;
use tisty_core::witness::{self, Fact, channel};
use tisty_share_proto::cloud::CloudKey;

use crate::{Answer, Refusal, Session, elsewhere, held};

const ORPHANED_AFTER: u64 = 7 * 24 * 60 * 60;
const PLAIN_WIPED: &str = "plain-wiped";
const SETTLING_FOR: Duration = Duration::from_secs(30);
const LOOKING_EVERY: Duration = Duration::from_secs(5);
const BROKE_TRIES: u32 = 3;
const BROKE_AGAIN: Duration = Duration::from_secs(20);

/// Every drive or cancel moves `driven` on, so a thread left from before sees it no longer carries the turn.
/// `keying` keeps the key named at a claim from landing after a cancel has taken it back.
#[derive(Default)]
pub struct Turns {
    driven: AtomicU64,
    last: Mutex<Option<Told>>,
    keying: Mutex<()>,
    wiping: AtomicBool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Told {
    pub on: bool,
    pub step: &'static str,
    pub done: usize,
    pub left: usize,
    pub refused: usize,
    pub stopped: bool,
    pub why: Option<String>,
    pub waiting: Option<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    pub cloud: Option<String>,
    pub state: &'static str,
    pub joined: bool,
    pub held: bool,
    pub confirmed: bool,
    pub phrase_elsewhere: bool,
    pub behind: Vec<String>,
    pub turning: Option<Told>,
    pub orphan: bool,
    pub wiping: bool,
    pub plain_left: bool,
    pub reading: Option<tisty_carrier::Reading>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Room {
    pub free: Option<u64>,
    pub needed: u64,
}

fn turned_at(session: &Session) -> Option<(CloudAt, PathBuf)> {
    let at = tisty_carrier::cloud_of(session.config.sync.as_ref())?.clone();
    let home = session.reach.home_of(&at);
    Some((at, home))
}

pub(crate) fn word_here(home: &Path) -> String {
    std::fs::read_to_string(tisty_carrier::mirrored_at(home).join("tisty.toml")).unwrap_or_default()
}

// This machine runs a build that knows how a cloud turns, so only the others are waited for.
fn behind(session: &Session) -> Vec<String> {
    let here = &session.config.device_id;
    session
        .state
        .not_yet_on(tisty_carrier::TURNS)
        .into_iter()
        .filter(|device| device != here)
        .map(|device| {
            session
                .state
                .named
                .get(&device)
                .map(|one| one.name.clone())
                .unwrap_or_else(|| tisty_core::config::nicknamed(&device.0))
        })
        .collect()
}

fn told_last(turns: &Turns, on: bool) -> Option<Told> {
    turns
        .last
        .lock()
        .ok()
        .and_then(|last| last.clone())
        .filter(|told| told.on == on)
}

#[tauri::command(async)]
pub fn sealing_shown(
    session: tauri::State<'_, Mutex<Session>>,
    turns: tauri::State<'_, Turns>,
) -> Answer<Shown> {
    let (kept, confirmed) = crate::identity::standing(&session)?;
    let session = held(&session);
    let phrase_elsewhere = !kept && session.state.persons.names_a_person();
    let Some((at, home)) = turned_at(&session) else {
        return Ok(Shown {
            cloud: None,
            state: "none",
            joined: false,
            held: kept,
            confirmed,
            phrase_elsewhere,
            behind: Vec::new(),
            turning: None,
            orphan: false,
            wiping: false,
            plain_left: false,
            reading: None,
        });
    };
    let word = word_here(&home);
    let ours = Turning::left(&home);
    let state = match (tisty_carrier::sealing_in(&word), &ours) {
        (_, Some(_)) => "turning",
        (Sealing::Sealed(cloud), None)
            if session
                .reach
                .keyed()
                .is_none_or(|keyed| keyed.locked.cloud != *cloud) =>
        {
            "locked"
        }
        (Sealing::Sealed(_), None) => "sealed",
        (Sealing::Copying(_) | Sealing::Closed, None) => "elsewhere",
        (Sealing::Plain | Sealing::Unsaid, None) => "plain",
    };
    let turning = ours.map(|left| told_last(&turns, left.on).unwrap_or_else(|| standing_of(&left)));
    let orphan = state == "elsewhere"
        && tisty_carrier::idle_for(&word).is_some_and(|idle| idle >= ORPHANED_AFTER);
    Ok(Shown {
        cloud: Some(at.provider),
        state,
        joined: crate::handed::joined_here(&session, &home),
        held: kept,
        confirmed,
        phrase_elsewhere,
        behind: behind(&session),
        turning,
        orphan,
        wiping: turns.wiping.load(Ordering::Acquire),
        plain_left: match tisty_carrier::sealing_in(&word) {
            Sealing::Sealed(cloud) => plain_left(&home, &cloud),
            _ => false,
        },
        reading: tisty_carrier::reading(&home),
    })
}

// Built off the session's lock, since reaching the provider reads its secrets and may wait on the keychain.
fn reached(home: &Path, at: &CloudAt) -> Answer<Arc<dyn Remote>> {
    crate::reaching::plain_at(home, at).map_err(|why| {
        witness::warn(
            channel::CLOUD,
            "the cloud could not be reached to turn it",
            &[("why", Fact::Why(why))],
        );
        Refusal::of("cloudNotNow")
    })
}

fn plain_of(session: &Mutex<Session>) -> Answer<(Arc<dyn Remote>, PathBuf, CloudAt)> {
    let (at, home) = turned_at(&held(session)).ok_or_else(|| Refusal::of("sealingNoCloud"))?;
    Ok((reached(&home, &at)?, home, at))
}

fn refused_by(hitch: Hitch, provider: &str) -> Refusal {
    if matches!(hitch, Hitch::Busy { .. }) {
        return Refusal::of("sealingBusy");
    }
    match tisty_carrier::pause_for(hitch, provider) {
        Err(trouble) => crate::said(trouble),
        Ok(_) => Refusal::of("cloudNotNow"),
    }
}

fn said_sealed(session: &Mutex<Session>, cloud: String, named: Option<String>) -> Answer<()> {
    held(session)
        .commit(Op::CloudSealed { s: cloud, k: named })
        .map_err(|e| {
            crate::blamed(
                channel::STORE,
                "the turn of the cloud could not be written",
                e,
            )
        })
}

fn keyed(session: &Mutex<Session>, cloud: &str, named: &str) -> Answer<()> {
    let known = held(session).state.sealed.get(cloud).cloned();
    match known.as_deref() == Some(named) {
        true => Ok(()),
        false => said_sealed(session, cloud.to_string(), Some(named.to_string())),
    }
}

fn key_of(session: &Mutex<Session>, cloud: &str) -> Answer<CloudKey> {
    crate::identity::cloud_key(session, cloud)?.ok_or_else(|| Refusal::of("sealingPhrase"))
}

#[tauri::command(async)]
pub async fn sealing_room(session: tauri::State<'_, Mutex<Session>>) -> Answer<Room> {
    let (plain, _, at) = plain_of(&session)?;
    elsewhere(move || {
        let free = plain.room().ok().flatten();
        let needed =
            tisty_carrier::turn_size(&*plain).map_err(|hitch| refused_by(hitch, &at.provider))?;
        Ok(Room { free, needed })
    })
    .await?
}

fn chosen_cloud(session: &Session, on: bool) -> Answer<String> {
    if !behind(session).is_empty() {
        return Err(Refusal::of("sealingBehind"));
    }
    let (_, home) = turned_at(session).ok_or_else(|| Refusal::of("sealingNoCloud"))?;
    match (on, tisty_carrier::sealing_in(&word_here(&home))) {
        (true, _) => tisty_core::store::peek_identity(session.paths.store()),
        (false, Sealing::Sealed(cloud)) => Some(cloud),
        (false, _) => None,
    }
    .ok_or_else(|| Refusal::of("sealingNoCloud"))
}

// Turning on names the key once the claim stands, while every machine still reads plain; a cancel since keeps it unsaid.
fn claim_keyed(app: &AppHandle, home: &Path, cloud: &str, named: &str) {
    let turns = app.state::<Turns>();
    let _keying = turns.keying.lock().unwrap_or_else(PoisonError::into_inner);
    if !Turning::left(home).is_some_and(|left| left.on && !left.giving_up) {
        return;
    }
    if let Err(why) = keyed(&app.state::<Mutex<Session>>(), cloud, named) {
        witness::warn(
            channel::CLOUD,
            "the key of a cloud being sealed could not be named yet",
            &[("why", Fact::Why(why.code.to_string()))],
        );
    }
}

#[tauri::command(async)]
pub async fn sealing_begin(
    app: AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    turns: tauri::State<'_, Turns>,
    on: bool,
) -> Answer<()> {
    let (cloud, device) = {
        let session = held(&session);
        (
            chosen_cloud(&session, on)?,
            session.config.device_id.0.clone(),
        )
    };
    let key =
        crate::identity::confirmed_key(&session)?.ok_or_else(|| Refusal::of("sealingPhrase"))?;
    let (plain, home, at) = plain_of(&session)?;
    let named = key.named_for(&cloud);
    let turn = if on { Turn::On } else { Turn::Off };
    let (said, there) = (cloud.clone(), home.clone());
    let begun = elsewhere(move || Turning::begin(plain, &there, key, &said, &device, turn)).await?;
    if let Err(hitch) = begun {
        if Turning::left(&home).is_some() {
            drive(app);
        }
        return Err(refused_by(hitch, &at.provider));
    }
    let unmarked = home.join(PLAIN_WIPED);
    let _ = elsewhere(move || std::fs::remove_file(unmarked)).await;
    if let (Some(left), Ok(mut last)) = (Turning::left(&home), turns.last.lock()) {
        *last = Some(going_from(&left));
    }
    if on {
        let keying = app.clone();
        elsewhere(move || claim_keyed(&keying, &home, &cloud, &named)).await?;
    }
    drive(app);
    Ok(())
}

#[tauri::command(async)]
pub fn sealing_resume(app: AppHandle) {
    drive(app);
}

// The key is taken back under the turn's lock and before anything is undone, so a plain place is never left with it named.
fn given_up(
    app: &AppHandle,
    plain: Arc<dyn Remote>,
    home: &Path,
    key: CloudKey,
    store: &str,
    provider: &str,
) -> Answer<bool> {
    let session = app.state::<Mutex<Session>>();
    let turns = app.state::<Turns>();
    let _keying = turns.keying.lock().unwrap_or_else(PoisonError::into_inner);
    let mut refused = None;
    let mut unkeyed = |on: bool, cloud: &str| -> Result<(), Hitch> {
        if !on {
            return Ok(());
        }
        said_sealed(&session, cloud.to_string(), None).map_err(|refusal| {
            refused = Some(refusal);
            Hitch::Broke("the key could not be taken back".into())
        })
    };
    let done = match Turning::left(home) {
        Some(left) => match Turning::resume(plain, home, key) {
            Some(turning) => turning.abort(|| unkeyed(left.on, &left.cloud)),
            None => return Err(Refusal::of("sealingPhrase")),
        },
        None => Turning::orphaned(plain, home, key, store, ORPHANED_AFTER, |turn| {
            unkeyed(turn == Turn::On, store)
        })
        .map(|turn| turn.is_some()),
    };
    done.map_err(|hitch| {
        refused
            .take()
            .unwrap_or_else(|| refused_by(hitch, provider))
    })
}

#[tauri::command(async)]
pub async fn sealing_cancel(
    app: AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    turns: tauri::State<'_, Turns>,
) -> Answer<()> {
    turns.driven.fetch_add(1, Ordering::AcqRel);
    let cancelled = cancelled(&app, &session).await;
    // Whatever stopped short of giving the turn up leaves it to be carried on, so it is driven again.
    if !matches!(cancelled, Ok(true)) {
        drive(app.clone());
    }
    if cancelled? {
        told(&app, &turns, None);
    }
    Ok(())
}

async fn cancelled(app: &AppHandle, session: &Mutex<Session>) -> Answer<bool> {
    let (plain, home, at) = plain_of(session)?;
    let store = tisty_core::store::peek_identity(held(session).paths.store())
        .ok_or_else(|| Refusal::of("sealingNoCloud"))?;
    let cloud = Turning::left(&home).map_or_else(|| store.clone(), |left| left.cloud);
    let key = key_of(session, &cloud)?;
    let app = app.clone();
    elsewhere(move || given_up(&app, plain, &home, key, &store, &at.provider)).await?
}

#[tauri::command(async)]
pub async fn sealing_wipe(
    session: tauri::State<'_, Mutex<Session>>,
    turns: tauri::State<'_, Turns>,
) -> Answer<usize> {
    if turns.wiping.swap(true, Ordering::AcqRel) {
        return Err(Refusal::of("sealingWiping"));
    }
    witness::note(
        channel::CLOUD,
        "deleting the old unencrypted copy began",
        &[],
    );
    let wiped = wiped_behind(&session).await;
    turns.wiping.store(false, Ordering::Release);
    match &wiped {
        Ok(gone) => witness::note(
            channel::CLOUD,
            "the old unencrypted copy was deleted",
            &[("gone", Fact::Count(*gone))],
        ),
        Err(refusal) => witness::warn(
            channel::CLOUD,
            "the old unencrypted copy could not be deleted",
            &[("why", Fact::Code(refusal.code))],
        ),
    }
    wiped
}

async fn wiped_behind(session: &Mutex<Session>) -> Answer<usize> {
    let (plain, home, at) = plain_of(session)?;
    let cloud = match tisty_carrier::sealing_in(&word_here(&home)) {
        Sealing::Sealed(cloud) => cloud,
        _ => return Err(Refusal::of("sealingNoCloud")),
    };
    let key = key_of(session, &cloud)?;
    let marked = (home.join(PLAIN_WIPED), cloud.clone());
    let wiped = elsewhere(move || tisty_carrier::wipe_behind(plain, &home, key, &cloud))
        .await?
        .map_err(|hitch| refused_by(hitch, &at.provider))?;
    if wiped.left == 0 {
        let _ = elsewhere(move || std::fs::write(&marked.0, &marked.1)).await;
    }
    Ok(wiped.gone)
}

// Asking the provider whether old files are left would list the whole cloud, so this computer remembers it wiped them.
fn plain_left(home: &Path, cloud: &str) -> bool {
    std::fs::read_to_string(home.join(PLAIN_WIPED)).map_or(true, |said| said.trim() != cloud)
}

fn told(app: &AppHandle, turns: &Turns, said: Option<Told>) {
    if let Ok(mut last) = turns.last.lock() {
        last.clone_from(&said);
    }
    let _ = app.emit("sealing", said);
}

fn standing_of(left: &Left) -> Told {
    Told {
        on: left.on,
        step: if left.closed { "closed" } else { "copying" },
        done: 0,
        left: 0,
        refused: 0,
        stopped: true,
        why: None,
        waiting: None,
    }
}

fn going_from(left: &Left) -> Told {
    Told {
        stopped: false,
        ..standing_of(left)
    }
}

fn retold(
    app: &AppHandle,
    turns: &Turns,
    on: bool,
    stopped: bool,
    why: Option<String>,
    waiting: Option<&'static str>,
) {
    let mut said = told_last(turns, on).unwrap_or(Told {
        on,
        step: "copying",
        done: 0,
        left: 0,
        refused: 0,
        stopped,
        why: None,
        waiting: None,
    });
    said.stopped = stopped;
    said.why = why;
    said.waiting = waiting;
    told(app, turns, Some(said));
}

pub fn stopped_here(app: &AppHandle, home: &Path) -> Option<bool> {
    let left = Turning::left(home)?;
    Some(told_last(&app.state::<Turns>(), left.on).is_none_or(|said| said.stopped))
}

fn waiting_on(hitch: &Hitch) -> &'static str {
    match hitch {
        Hitch::Unreachable(Some(cause), _) => cause.code(),
        Hitch::Unreachable(None, _) => "offline",
        Hitch::Busy { .. } => "busy",
        Hitch::Spent => "budget",
        Hitch::Paused => "paused",
        _ => "cloud",
    }
}

fn told_of(turning: &Turning, stepped: &Stepped) -> Told {
    let (step, done, left) = match stepped {
        Stepped::Claimed => ("claimed", 0, 0),
        Stepped::Copying { done, left } => ("copying", *done, *left),
        Stepped::Closed | Stepped::Settled => ("closed", 0, 0),
        Stepped::Settling { left } => ("settling", 0, *left),
        Stepped::Done | Stepped::GaveUp => ("done", 0, 0),
    };
    Told {
        on: turning.turn() == Turn::On,
        step,
        done,
        left,
        refused: turning.refused().len(),
        stopped: false,
        why: None,
        waiting: None,
    }
}

pub fn drive(app: AppHandle) {
    let ours = app.state::<Turns>().driven.fetch_add(1, Ordering::AcqRel) + 1;
    std::thread::spawn(move || driven(&app, ours));
}

fn still(turns: &Turns, ours: u64) -> bool {
    turns.driven.load(Ordering::Acquire) == ours
}

fn rested(turns: &Turns, ours: u64, wait: Duration) -> bool {
    let mut left = wait;
    while !left.is_zero() {
        if !still(turns, ours) {
            return false;
        }
        let nap = left.min(LOOKING_EVERY);
        std::thread::sleep(nap);
        left -= nap;
    }
    still(turns, ours)
}

fn driven(app: &AppHandle, ours: u64) {
    let session = app.state::<Mutex<Session>>();
    let turns = app.state::<Turns>();
    let Some((at, home)) = turned_at(&held(&session)) else {
        return;
    };
    let Some(left) = Turning::left(&home) else {
        return;
    };
    match told_last(&turns, left.on) {
        None => told(app, &turns, Some(going_from(&left))),
        Some(_) => retold(app, &turns, left.on, false, None, None),
    }
    let halted = |why: &str| retold(app, &turns, left.on, true, Some(why.to_string()), None);
    let key = match key_of(&session, &left.cloud) {
        Ok(key) => key,
        Err(refusal) => return halted(refusal.code),
    };
    let plain = match reached(&home, &at) {
        Ok(plain) => plain,
        Err(refusal) => return halted(refusal.code),
    };
    let Some(mut turning) = Turning::resume(plain.clone(), &home, key.clone()) else {
        return halted("cloudOtherPhrase");
    };
    let mut broken = 0;
    while still(&turns, ours) {
        let stepped = turning.step();
        if stepped.is_ok() {
            broken = 0;
        }
        match stepped {
            Ok(Stepped::Done) => {
                told(app, &turns, Some(told_of(&turning, &Stepped::Done)));
                ended(
                    &session,
                    &app.state::<crate::handed::Knocking>(),
                    &turning,
                    plain,
                    &home,
                    key,
                    at,
                );
                let _ = app.emit("told", ());
                return;
            }
            Ok(Stepped::GaveUp) => {
                told(app, &turns, None);
                return;
            }
            Ok(stepped @ Stepped::Settling { left: 0 }) => {
                told(app, &turns, Some(told_of(&turning, &stepped)));
                if !rested(&turns, ours, SETTLING_FOR) {
                    return;
                }
            }
            Ok(stepped) => told(app, &turns, Some(told_of(&turning, &stepped))),
            Err(Hitch::Broke(why)) if broken < BROKE_TRIES => {
                broken += 1;
                witness::note(
                    channel::CLOUD,
                    "a step of the turn failed and is tried again",
                    &[("why", Fact::Why(why))],
                );
                retold(app, &turns, left.on, false, None, Some("local"));
                if !rested(&turns, ours, BROKE_AGAIN * broken) {
                    return;
                }
            }
            Err(hitch) => {
                let on_what = waiting_on(&hitch);
                match tisty_carrier::pause_for(hitch, &at.provider) {
                    Ok(wait) => {
                        retold(app, &turns, left.on, false, None, Some(on_what));
                        if !rested(&turns, ours, wait) {
                            return;
                        }
                    }
                    Err(trouble) => {
                        return halted(crate::said(trouble).code);
                    }
                }
            }
        }
    }
}

// The carrier is built from the turn itself, since the mirror only learns the new word at its next pull.
fn ended(
    session: &Mutex<Session>,
    knocking: &crate::handed::Knocking,
    turning: &Turning,
    plain: Arc<dyn Remote>,
    home: &Path,
    key: CloudKey,
    at: CloudAt,
) {
    let (cloud, named) = {
        let (cloud, named) = turning.said();
        (cloud.to_string(), named.to_string())
    };
    if turning.turn() == Turn::On {
        // A claim whose key could not be named then is named now, since the place is sealed whatever happens next.
        if let Err(why) = keyed(session, &cloud, &named) {
            witness::warn(
                channel::CLOUD,
                "the cloud was sealed and the log could not name its key",
                &[("why", Fact::Why(why.code.to_string()))],
            );
        }
        let session = held(session);
        session.reach.tried_for(None);
        session.reach.seal_with(Some(crate::reaching::Keyed {
            locked: crate::reaching::Locked { at, cloud, named },
            key,
        }));
        return;
    }
    if let Err(why) = said_sealed(session, cloud.clone(), None) {
        witness::warn(
            channel::CLOUD,
            "the cloud went back to plain and the log could not say so",
            &[("why", Fact::Why(why.code.to_string()))],
        );
    }
    crate::unsealing::say(session, &*plain, home, &cloud, &named);
    {
        let session = held(session);
        session.reach.tried_for(None);
        session.reach.seal_with(None);
    }
    crate::handed::let_go(session, knocking);
    // The sealed copy holds nothing a plain cloud reads, so it goes at once rather than doubling the room.
    if let Err(why) = tisty_carrier::wipe_behind(plain, home, key, &cloud) {
        witness::warn(
            channel::CLOUD,
            "the sealed copy a plain cloud left behind could not be wiped yet",
            &[("why", Fact::Why(why.code().to_string()))],
        );
    }
}

#[cfg(test)]
#[path = "sealing_test.rs"]
mod tests;
