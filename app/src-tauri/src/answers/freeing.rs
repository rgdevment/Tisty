use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime};

use tauri::{Emitter, Manager};
use tisty_core::witness::{self, Fact, channel};

use super::storing::said_no_longer_held;
use crate::{Answer, OneAtATime, Refusal, Session, held, said};

const A_DAY: u64 = 24 * 60 * 60;
const MOST_DAYS: u64 = 10 * 365;
const SAID_EVERY: Duration = Duration::from_millis(250);

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Freeing {
    gone: usize,
    freed: u64,
    done: bool,
}

#[derive(Default)]
pub(crate) struct Stopping(AtomicBool);

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unused {
    files: usize,
    bytes: u64,
}

type Letting = Box<
    dyn FnOnce(
            tisty_carrier::Shared,
            &tisty_carrier::Here,
            tisty_carrier::Told,
        ) -> Result<tisty_carrier::LetGo, tisty_carrier::Trouble>
        + Send,
>;

/// Lets go of copies here one at a time, saying how far it got and stopping when asked.
async fn letting_go(
    app: tauri::AppHandle,
    session: &tauri::State<'_, Mutex<Session>>,
    alone: &tauri::State<'_, OneAtATime>,
    stopping: &tauri::State<'_, Stopping>,
    work: Letting,
    noted: &'static str,
) -> Answer<Freeing> {
    let _done = alone.inner().taken()?;
    stopping.0.store(false, Ordering::Relaxed);
    let (carrier, home) = {
        let session = held(session);
        (session.carrying()?, session.here())
    };

    let telling = app.clone();
    let done = tauri::async_runtime::spawn_blocking(move || {
        let mut said_at: Option<Instant> = None;
        work(carrier, &home, &mut |far| {
            if far.gone > 0 && said_at.is_none_or(|at| at.elapsed() >= SAID_EVERY) {
                said_at = Some(Instant::now());
                let _ = telling.emit(
                    "freeing",
                    Freeing {
                        gone: far.gone,
                        freed: far.freed,
                        done: false,
                    },
                );
            }
            !telling.state::<Stopping>().0.load(Ordering::Relaxed)
        })
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(said)?;

    said_no_longer_held(session, &done.let_go);
    witness::note(
        channel::SYNC,
        noted,
        &[
            ("count", Fact::Count(done.gone)),
            ("bytes", Fact::Bytes(done.freed)),
        ],
    );
    let now = Freeing {
        gone: done.gone,
        freed: done.freed,
        done: true,
    };
    let _ = app.emit("freeing", now.clone());
    Ok(now)
}

/// Turning it on is the only change that moves anything, so it is asked for rather than done on
/// the way past: it can take an afternoon, and somebody may want it to stop.
#[tauri::command(async)]
pub async fn free_up(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    stopping: tauri::State<'_, Stopping>,
) -> Answer<Freeing> {
    let (above, held_away) = {
        let session = held(&session);
        let mine = session.store.device().clone();
        let held_away: std::collections::BTreeSet<String> = session
            .state
            .holders
            .iter()
            .filter(|(_, who)| who.iter().any(|one| one != &mine))
            .map(|(at, _)| at.clone())
            .collect();
        (session.config.only_shared_above(), held_away)
    };
    let work: Letting = Box::new(move |carrier, home, told| {
        let elsewhere = |at: &str| held_away.contains(at);
        carrier.let_go(home, above, &elsewhere, told)
    });
    letting_go(
        app,
        &session,
        &alone,
        &stopping,
        work,
        "big attachments were left to the shared folder",
    )
    .await
}

#[tauri::command]
pub fn stop_freeing(stopping: tauri::State<'_, Stopping>) {
    stopping.0.store(true, Ordering::Relaxed);
}

fn since(days: u64) -> Answer<SystemTime> {
    if !(1..=MOST_DAYS).contains(&days) {
        return Err(Refusal::about("notAllowed", days.to_string()));
    }
    Ok(SystemTime::now()
        .checked_sub(Duration::from_secs(days * A_DAY))
        .unwrap_or(SystemTime::UNIX_EPOCH))
}

/// Only a cloud reached through its API answers: a folder never holds the one copy of anything.
#[tauri::command]
pub async fn unused_attachments(
    session: tauri::State<'_, Mutex<Session>>,
    days: u64,
) -> Answer<Unused> {
    let since = since(days)?;
    let (carrier, home) = {
        let session = held(&session);
        (session.carrying()?, session.here())
    };
    let could = tauri::async_runtime::spawn_blocking(move || carrier.could_free(&home, since))
        .await
        .map_err(|_| Refusal::of("internal"))?;
    Ok(Unused {
        files: could.gone,
        bytes: could.freed,
    })
}

#[tauri::command]
pub async fn free_space(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    stopping: tauri::State<'_, Stopping>,
    days: u64,
) -> Answer<Freeing> {
    let since = since(days)?;
    let work: Letting = Box::new(move |carrier, home, told| carrier.set_free(home, since, told));
    letting_go(
        app,
        &session,
        &alone,
        &stopping,
        work,
        "attachments unused for a while were let go of here",
    )
    .await
}

#[cfg(test)]
#[path = "freeing_test.rs"]
mod tests;
