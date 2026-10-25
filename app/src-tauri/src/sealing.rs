use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tisty_carrier::{Remote, Sealing, Stepped, Turn, Turning};
use tisty_core::Op;
use tisty_core::config::CloudAt;
use tisty_core::witness::{self, Fact, channel};
use tisty_share_proto::cloud::CloudKey;

use crate::{Answer, Refusal, Session, elsewhere, held};

const ORPHANED_AFTER: u64 = 7 * 24 * 60 * 60;
const SETTLING_FOR: Duration = Duration::from_secs(30);
const LOOKING_EVERY: Duration = Duration::from_secs(5);

/// Every drive or cancel moves `driven` on, so a thread left from before sees it no longer carries the turn.
#[derive(Default)]
pub struct Turns {
    driven: AtomicU64,
    last: Mutex<Option<Told>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Told {
    pub on: bool,
    pub step: &'static str,
    pub done: usize,
    pub left: usize,
    pub refused: usize,
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

fn told_last(turns: &Turns) -> Option<Told> {
    turns.last.lock().ok().and_then(|last| last.clone())
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
        told_last(&turns)
            .filter(|told| told.on == left.on)
            .unwrap_or(Told {
                on: left.on,
                step: "stopped",
                done: 0,
                left: 0,
                refused: 0,
                why: None,
            })
    });
    let orphan = state == "elsewhere"
        && tisty_carrier::claim_age(&word).is_some_and(|age| age >= ORPHANED_AFTER);
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
fn plain_of(session: &Mutex<Session>) -> Answer<(Arc<dyn Remote>, PathBuf, CloudAt)> {
    let (at, home) = turned_at(&held(session)).ok_or_else(|| Refusal::of("sealingNoCloud"))?;
    let plain = crate::reaching::plain_at(&home, &at).map_err(|why| {
        witness::warn(
            channel::CLOUD,
            "the cloud could not be reached to turn it",
            &[("why", Fact::Why(why))],
        );
        Refusal::of("cloudNotNow")
    })?;
    Ok((plain, home, at))
}

fn refused_by(hitch: tisty_carrier::Hitch, provider: &str) -> Refusal {
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

fn key_of(session: &Mutex<Session>) -> Answer<CloudKey> {
    crate::identity::phrase_key(session)?.ok_or_else(|| Refusal::of("sealingPhrase"))
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

#[tauri::command(async)]
pub async fn sealing_begin(
    app: AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    turns: tauri::State<'_, Turns>,
    on: bool,
) -> Answer<()> {
    let (kept, confirmed) = crate::identity::standing(&session)?;
    if !kept || !confirmed {
        return Err(Refusal::of("sealingPhrase"));
    }
    let (cloud, device) = {
        let session = held(&session);
        (
            chosen_cloud(&session, on)?,
            session.config.device_id.0.clone(),
        )
    };
    let key = key_of(&session)?;
    let (plain, home, at) = plain_of(&session)?;
    let named = key.named_for(&cloud);
    if let Ok(mut last) = turns.last.lock() {
        *last = None;
    }
    let turn = if on { Turn::On } else { Turn::Off };
    let (said, there, giving) = (cloud.clone(), home.clone(), plain.clone());
    let begun = elsewhere(move || Turning::begin(plain, &there, key, &said, &device, turn)).await?;
    if let Err(hitch) = begun {
        if Turning::left(&home).is_some() {
            drive(app);
        }
        return Err(refused_by(hitch, &at.provider));
    }
    // Turning on names the key now, while every machine still reads plain; turning off waits until the place is plain.
    if on && let Err(refusal) = said_sealed(&session, cloud, Some(named)) {
        let key = key_of(&session)?;
        let _ = elsewhere(move || Turning::resume(giving, &home, key).map(Turning::abort)).await;
        return Err(refusal);
    }
    drive(app);
    Ok(())
}

#[tauri::command(async)]
pub fn sealing_resume(app: AppHandle) {
    drive(app);
}

#[tauri::command(async)]
pub async fn sealing_cancel(
    app: AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    turns: tauri::State<'_, Turns>,
) -> Answer<()> {
    turns.driven.fetch_add(1, Ordering::AcqRel);
    let key = key_of(&session)?;
    let (plain, home, at) = plain_of(&session)?;
    let store = tisty_core::store::peek_identity(held(&session).paths.store())
        .ok_or_else(|| Refusal::of("sealingNoCloud"))?;
    let given_up = match Turning::left(&home) {
        Some(left) => {
            let turning =
                Turning::resume(plain, &home, key).ok_or_else(|| Refusal::of("sealingPhrase"))?;
            elsewhere(move || turning.abort())
                .await?
                .map_err(|hitch| refused_by(hitch, &at.provider))?;
            left.on.then_some(left.cloud)
        }
        None => {
            let on = tisty_carrier::turn_of(&word_here(&home)) == Some(Turn::On);
            let cloud = store.clone();
            let dropped =
                elsewhere(move || Turning::orphaned(plain, &home, key, &cloud, ORPHANED_AFTER))
                    .await?
                    .map_err(|hitch| refused_by(hitch, &at.provider))?;
            (dropped && on).then_some(store)
        }
    };
    // A sealing given up leaves the place plain, so the key its claim named is taken back too.
    if let Some(cloud) = given_up {
        said_sealed(&session, cloud, None)?;
    }
    told(&app, &turns, None);
    Ok(())
}

#[tauri::command(async)]
pub async fn sealing_wipe(session: tauri::State<'_, Mutex<Session>>) -> Answer<usize> {
    let key = key_of(&session)?;
    let (plain, home, at) = plain_of(&session)?;
    let cloud = match tisty_carrier::sealing_in(&word_here(&home)) {
        Sealing::Sealed(cloud) => cloud,
        _ => return Err(Refusal::of("sealingNoCloud")),
    };
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
    let Some((_, home)) = turned_at(&held(&session)) else {
        return;
    };
    if Turning::left(&home).is_none() {
        return;
    }
    let Ok(key) = key_of(&session) else {
        return;
    };
    let Ok((plain, home, at)) = plain_of(&session) else {
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
                if turning.turn() == Turn::On {
                    let _ = said_sealed(&session, turning.said().0.to_string(), None);
                }
                told(app, &turns, None);
                return;
            }
            Ok(Stepped::Settling { left: 0 }) => {
                told(
                    app,
                    &turns,
                    Some(told_of(&turning, &Stepped::Settling { left: 0 })),
                );
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
                    let refusal = crate::said(trouble);
                    let mut said = told_of(&turning, &Stepped::Claimed);
                    said.step = "stopped";
                    said.why = Some(refusal.code.to_string());
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
