use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tisty_carrier::{Remote, Sealing, Stepped, Turn, Turning};
use tisty_core::Op;
use tisty_core::config::CloudAt;
use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Refusal, Session, elsewhere, held};

const ORPHANED_AFTER: u64 = 7 * 24 * 60 * 60;
const SETTLING_FOR: Duration = Duration::from_secs(30);
const LOOKING_EVERY: Duration = Duration::from_secs(5);

// Every drive or cancel moves this on, so a thread left from before sees it is no longer the one carrying the turn.
static DRIVEN: AtomicU64 = AtomicU64::new(0);
static LAST: Mutex<Option<Told>> = Mutex::new(None);

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

fn word_here(home: &std::path::Path) -> String {
    std::fs::read_to_string(tisty_carrier::mirrored_at(home).join("tisty.toml")).unwrap_or_default()
}

fn orphan(word: &str) -> bool {
    tisty_carrier::claim_in(word)
        .and_then(|claim| {
            let (_, nonce) = claim.split_once(':')?;
            nonce.parse::<u128>().ok()
        })
        .is_some_and(|nanos| {
            let claimed = (nanos / 1_000_000_000) as u64;
            tisty_carrier::policy::now().saturating_sub(claimed) >= ORPHANED_AFTER
        })
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

#[tauri::command(async)]
pub fn sealing_shown(session: tauri::State<'_, Mutex<Session>>) -> Answer<Shown> {
    let (held, confirmed) = crate::identity::standing(&session)?;
    let session = held_session(&session);
    let Some((at, home)) = turned_at(&session) else {
        return Ok(Shown {
            cloud: None,
            state: "none",
            held,
            confirmed,
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
        LAST.lock()
            .ok()
            .and_then(|last| last.clone())
            .unwrap_or(Told {
                on: left.on,
                step: "copying",
                done: 0,
                left: 0,
                refused: 0,
                why: None,
            })
    });
    Ok(Shown {
        cloud: Some(at.provider),
        state,
        held,
        confirmed,
        behind: behind(&session),
        turning,
        orphan: state == "elsewhere" && orphan(&word),
    })
}

fn held_session<'a>(
    session: &'a tauri::State<'_, Mutex<Session>>,
) -> std::sync::MutexGuard<'a, Session> {
    held(session)
}

fn plain_of(session: &Mutex<Session>) -> Answer<(Arc<dyn Remote>, PathBuf, CloudAt)> {
    let session = held(session);
    let (at, home) = turned_at(&session).ok_or_else(|| Refusal::of("sealingNoCloud"))?;
    let plain = session.reach.plain(&at).map_err(|why| {
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

fn unsealed(session: &Mutex<Session>, cloud: String) -> Answer<()> {
    held(session)
        .commit(Op::CloudSealed { s: cloud, k: None })
        .map_err(|e| {
            crate::blamed(
                channel::STORE,
                "the turn of the cloud could not be written",
                e,
            )
        })
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

#[tauri::command(async)]
pub async fn sealing_begin(
    app: AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    on: bool,
) -> Answer<()> {
    let (held, confirmed) = crate::identity::standing(&session)?;
    if !held || !confirmed {
        return Err(Refusal::of("sealingPhrase"));
    }
    let (cloud, device) = {
        let session = held_session(&session);
        if !behind(&session).is_empty() {
            return Err(Refusal::of("sealingBehind"));
        }
        let (_, home) = turned_at(&session).ok_or_else(|| Refusal::of("sealingNoCloud"))?;
        let cloud = match (on, tisty_carrier::sealing_in(&word_here(&home))) {
            (true, _) => tisty_core::store::peek_identity(session.paths.store()),
            (false, Sealing::Sealed(cloud)) => Some(cloud),
            (false, _) => None,
        }
        .ok_or_else(|| Refusal::of("sealingNoCloud"))?;
        (cloud, session.config.device_id.0.clone())
    };
    let key = crate::identity::phrase_key(&session)?.ok_or_else(|| Refusal::of("sealingPhrase"))?;
    let (plain, home, at) = plain_of(&session)?;
    let named = key.named_for(&cloud);
    let turn = if on { Turn::On } else { Turn::Off };
    let said = cloud.clone();
    elsewhere(move || Turning::begin(plain, &home, key, &said, &device, turn).map(|_| ()))
        .await?
        .map_err(|hitch| refused_by(hitch, &at.provider))?;
    // Turning on names the key now, while every machine still reads plain; turning off waits until the place is plain.
    if on {
        held_session(&session)
            .commit(Op::CloudSealed {
                s: cloud,
                k: Some(named),
            })
            .map_err(|e| {
                crate::blamed(
                    channel::STORE,
                    "the turn of the cloud could not be written",
                    e,
                )
            })?;
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
) -> Answer<()> {
    DRIVEN.fetch_add(1, Ordering::AcqRel);
    let key = crate::identity::phrase_key(&session)?.ok_or_else(|| Refusal::of("sealingPhrase"))?;
    let (plain, home, at) = plain_of(&session)?;
    let store = {
        let session = held_session(&session);
        tisty_core::store::peek_identity(session.paths.store())
            .ok_or_else(|| Refusal::of("sealingNoCloud"))?
    };
    let word = word_here(&home);
    let (on, cloud) = match Turning::left(&home) {
        Some(left) => (left.on, left.cloud),
        None => (
            !word
                .lines()
                .any(|line| line.trim_start().starts_with("sealed")),
            store,
        ),
    };
    let (sealed, ours) = (cloud.clone(), Turning::left(&home).is_some());
    elsewhere(move || match ours {
        true => match Turning::resume(plain, &home, key) {
            Some(turning) => turning.abort(),
            None => Ok(()),
        },
        false => Turning::orphaned(plain, &home, key, &sealed, ORPHANED_AFTER).map(|_| ()),
    })
    .await?
    .map_err(|hitch| refused_by(hitch, &at.provider))?;
    // A sealing given up leaves the place plain, so the key the claim named must be taken back too.
    if on {
        unsealed(&session, cloud)?;
    }
    told(&app, None);
    Ok(())
}

#[tauri::command(async)]
pub async fn sealing_wipe(session: tauri::State<'_, Mutex<Session>>) -> Answer<usize> {
    let key = crate::identity::phrase_key(&session)?.ok_or_else(|| Refusal::of("sealingPhrase"))?;
    let (plain, home, at) = plain_of(&session)?;
    let cloud = {
        let session = held_session(&session);
        match tisty_carrier::sealing_in(&word_here(&home)) {
            Sealing::Sealed(cloud) => Some(cloud),
            _ => session
                .state
                .sealed_cloud()
                .map(|(cloud, _)| cloud.to_string()),
        }
        .or_else(|| tisty_core::store::peek_identity(session.paths.store()))
        .ok_or_else(|| Refusal::of("sealingNoCloud"))?
    };
    elsewhere(move || tisty_carrier::wipe_behind(plain, &home, key, &cloud))
        .await?
        .map_err(|hitch| refused_by(hitch, &at.provider))
}

fn told(app: &AppHandle, said: Option<Told>) {
    if let Ok(mut last) = LAST.lock() {
        last.clone_from(&said);
    }
    let _ = app.emit("sealing", said);
}

fn told_of(turning: &Turning, stepped: &Stepped) -> Told {
    let (step, done, left) = match stepped {
        Stepped::Claimed => ("claimed", 0, 0),
        Stepped::Copying { done, left } => ("copying", *done, *left),
        Stepped::Closed => ("closed", 0, 0),
        Stepped::Settling { left } => ("settling", 0, *left),
        Stepped::Done => ("done", 0, 0),
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
    let ours = DRIVEN.fetch_add(1, Ordering::AcqRel) + 1;
    std::thread::spawn(move || driven(&app, ours));
}

fn still(ours: u64) -> bool {
    DRIVEN.load(Ordering::Acquire) == ours
}

fn rested(ours: u64, wait: Duration) -> bool {
    let mut left = wait;
    while !left.is_zero() {
        if !still(ours) {
            return false;
        }
        let nap = left.min(LOOKING_EVERY);
        std::thread::sleep(nap);
        left -= nap;
    }
    still(ours)
}

fn driven(app: &AppHandle, ours: u64) {
    let session = app.state::<Mutex<Session>>();
    let home = {
        let session = held(&session);
        match turned_at(&session) {
            Some((_, home)) => home,
            None => return,
        }
    };
    if Turning::left(&home).is_none() {
        return;
    }
    let Ok(Some(key)) = crate::identity::phrase_key(&session) else {
        return;
    };
    let Ok((plain, home, at)) = plain_of(&session) else {
        return;
    };
    let Some(mut turning) = Turning::resume(plain.clone(), &home, key.clone()) else {
        return;
    };
    while still(ours) {
        match turning.step() {
            Ok(Stepped::Done) => {
                told(app, Some(told_of(&turning, &Stepped::Done)));
                ended(&session, &turning, plain, &home, key, at);
                let _ = app.emit("told", ());
                return;
            }
            Ok(Stepped::Settling { left: 0 }) => {
                told(app, Some(told_of(&turning, &Stepped::Settling { left: 0 })));
                if !rested(ours, SETTLING_FOR) {
                    return;
                }
            }
            Ok(stepped) => told(app, Some(told_of(&turning, &stepped))),
            Err(hitch) => match tisty_carrier::pause_for(hitch, &at.provider) {
                Ok(wait) => {
                    if !rested(ours, wait) {
                        return;
                    }
                }
                Err(trouble) => {
                    let refusal = crate::said(trouble);
                    let mut said = told_of(&turning, &Stepped::Claimed);
                    said.step = "stopped";
                    said.why = Some(refusal.code.to_string());
                    told(app, Some(said));
                    return;
                }
            },
        }
    }
}

// The carrier is built anew from the turn itself, since the mirror only learns the new word at its next pull.
fn ended(
    session: &Mutex<Session>,
    turning: &Turning,
    plain: Arc<dyn Remote>,
    home: &std::path::Path,
    key: tisty_share_proto::cloud::CloudKey,
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
    if let Err(why) = unsealed(session, cloud.clone()) {
        witness::warn(
            channel::CLOUD,
            "the cloud went back to plain and the log could not say so",
            &[("why", Fact::Why(why.code.to_string()))],
        );
    }
    let session_reach = held(session);
    session_reach.reach.tried_for(None);
    session_reach.reach.seal_with(None);
    drop(session_reach);
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
