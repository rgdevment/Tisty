use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use tauri::{Emitter, Manager};
use tisty_core::witness::{self, Fact, channel};

use super::storing::{Freeing, Stopping, said_no_longer_held};
use crate::{Answer, OneAtATime, Refusal, Session, held, said};

const A_DAY: u64 = 24 * 60 * 60;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unused {
    files: usize,
    bytes: u64,
}

fn since(days: u64) -> SystemTime {
    SystemTime::now()
        .checked_sub(Duration::from_secs(days.saturating_mul(A_DAY)))
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

/// Only a cloud reached through its API answers: a folder never holds the one copy of anything.
#[tauri::command]
pub async fn unused_attachments(
    session: tauri::State<'_, Mutex<Session>>,
    days: u64,
) -> Answer<Unused> {
    let (carrier, home) = {
        let session = held(&session);
        (session.carrying()?, session.here())
    };
    let could =
        tauri::async_runtime::spawn_blocking(move || carrier.could_free(&home, since(days)))
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
    let _done = alone.inner().taken()?;
    stopping
        .0
        .store(false, std::sync::atomic::Ordering::Relaxed);
    let (carrier, home) = {
        let session = held(&session);
        (session.carrying()?, session.here())
    };

    let telling = app.clone();
    let done = tauri::async_runtime::spawn_blocking(move || {
        let mut said_gone = 0;
        carrier.set_free(&home, since(days), &mut |far| {
            if far.gone != said_gone {
                said_gone = far.gone;
                let _ = telling.emit(
                    "freeing",
                    Freeing {
                        gone: far.gone,
                        freed: far.freed,
                        done: false,
                    },
                );
            }
            !telling
                .state::<Stopping>()
                .0
                .load(std::sync::atomic::Ordering::Relaxed)
        })
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(said)?;

    said_no_longer_held(&session, &done.let_go);
    witness::note(
        channel::CLOUD,
        "attachments unused for a while were let go of here",
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
