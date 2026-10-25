use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tisty_carrier::{Hitch, Remote, Sealing, Stepped, Turn, Turning};
use tisty_core::Op;
use tisty_core::config::CloudAt;
use tisty_core::witness::{self, Fact, channel};
use tisty_share_proto::cloud::CloudKey;

use crate::{Answer, Refusal, Session, elsewhere, held};

const ORPHANED_AFTER: u64 = 7 * 24 * 60 * 60;
const SETTLING_FOR: Duration = Duration::from_secs(30);
const LOOKING_EVERY: Duration = Duration::from_secs(5);

/// Every drive or cancel moves `driven` on, so a thread left from before sees it no longer carries the turn.
/// `keying` keeps the key named at a claim from landing after a cancel has taken it back.
#[derive(Default)]
pub struct Turns {
    driven: AtomicU64,
    last: Mutex<Option<Told>>,
    keying: Mutex<()>,
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
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    pub cloud: Option<String>,
    pub state: &'static str,
    pub held: bool,
    pub confirmed: bool,
    pub phrase_elsewhere: bool,
    pub behind: Vec<String>,
    pub turning: Option<Told>,
    pub orphan: bool,
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

fn word_here(home: &Path) -> String {
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
            held: kept,
            confirmed,
            phrase_elsewhere,
            behind: Vec::new(),
            turning: None,
            orphan: false,
        });
    };
    let word = word_here(&home);
    let ours = Turning::left(&home);
    let state = match (tisty_carrier::sealing_in(&word), &ours) {
        (_, Some(_)) => "turning",
        (Sealing::Sealed(_), None) => "sealed",
        (Sealing::Copying(_) | Sealing::Closed, None) => "elsewhere",
        (Sealing::Plain | Sealing::Unsaid, None) => "plain",
    };
    let turning = ours.map(|left| {
        told_last(&turns, left.on).unwrap_or(Told {
            on: left.on,
            step: if left.closed { "closed" } else { "copying" },
            done: 0,
            left: 0,
            refused: 0,
            stopped: true,
            why: None,
        })
    });
    let orphan = state == "elsewhere"
        && tisty_carrier::idle_for(&word).is_some_and(|idle| idle >= ORPHANED_AFTER);
    Ok(Shown {
        cloud: Some(at.provider),
        state,
        held: kept,
        confirmed,
        phrase_elsewhere,
        behind: behind(&session),
        turning,
        orphan,
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
    if let Ok(mut last) = turns.last.lock() {
        *last = None;
    }
    let turn = if on { Turn::On } else { Turn::Off };
    let (said, there) = (cloud.clone(), home.clone());
    let begun = elsewhere(move || Turning::begin(plain, &there, key, &said, &device, turn)).await?;
    if let Err(hitch) = begun {
        if Turning::left(&home).is_some() {
            drive(app);
        }
        return Err(refused_by(hitch, &at.provider));
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
pub async fn sealing_wipe(session: tauri::State<'_, Mutex<Session>>) -> Answer<usize> {
    let (plain, home, at) = plain_of(&session)?;
    let cloud = match tisty_carrier::sealing_in(&word_here(&home)) {
        Sealing::Sealed(cloud) => cloud,
        _ => return Err(Refusal::of("sealingNoCloud")),
    };
    let key = key_of(&session, &cloud)?;
    elsewhere(move || tisty_carrier::wipe_behind(plain, &home, key, &cloud))
        .await?
        .map_err(|hitch| refused_by(hitch, &at.provider))
}

fn told(app: &AppHandle, turns: &Turns, said: Option<Told>) {
    if let Ok(mut last) = turns.last.lock() {
        last.clone_from(&said);
    }
    let _ = app.emit("sealing", said);
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
    let (Ok(key), Ok(plain)) = (key_of(&session, &left.cloud), reached(&home, &at)) else {
        return;
    };
    let Some(mut turning) = Turning::resume(plain.clone(), &home, key.clone()) else {
        return;
    };
    while still(&turns, ours) {
        match turning.step() {
            Ok(Stepped::Done) => {
                told(app, &turns, Some(told_of(&turning, &Stepped::Done)));
                ended(&session, &turning, plain, &home, key, at);
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
            Err(hitch) => match tisty_carrier::pause_for(hitch, &at.provider) {
                Ok(wait) => {
                    if !rested(&turns, ours, wait) {
                        return;
                    }
                }
                Err(trouble) => {
                    let on = turning.turn() == Turn::On;
                    let mut said = told_last(&turns, on)
                        .unwrap_or_else(|| told_of(&turning, &Stepped::Claimed));
                    said.stopped = true;
                    said.why = Some(crate::said(trouble).code.to_string());
                    told(app, &turns, Some(said));
                    return;
                }
            },
        }
    }
}

// The carrier is built from the turn itself, since the mirror only learns the new word at its next pull.
fn ended(
    session: &Mutex<Session>,
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
    {
        let session = held(session);
        session.reach.tried_for(None);
        session.reach.seal_with(None);
    }
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
