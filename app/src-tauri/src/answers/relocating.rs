use std::sync::Mutex;

use tauri::Emitter;
use tisty_core::config::{CloudAt, Leaving, Sync};
use tisty_core::witness::channel;

use super::storing::{Telling, catching_up, changed_to, folder_way};
use crate::reaching::GOOGLE;
use crate::{Answer, OneAtATime, Refusal, Session, held, said};

const PENDING: &str = "moving.json";

/// Where a move goes: the same three ways a copy can sync.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Toward {
    Alone,
    Folder { at: String },
    Cloud { account: String, email: String },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Weight {
    files: usize,
    bytes: u64,
}

impl From<tisty_carrier::Owed> for Weight {
    fn from(owed: tisty_carrier::Owed) -> Self {
        Self {
            files: owed.files,
            bytes: owed.bytes,
        }
    }
}

/// What a move will bring down from where this copy syncs now, and carry up to where it goes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    down: Weight,
    up: Weight,
    from_cloud: bool,
    to_cloud: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Moving {
    #[serde(rename_all = "camelCase")]
    Done { left_cloud: Option<CloudAt> },
    #[serde(rename_all = "camelCase")]
    Waiting {
        stage: &'static str,
        left: Weight,
        reason: Option<&'static str>,
    },
}

fn way_of(session: &Session, toward: &Toward) -> Answer<Sync> {
    match toward {
        Toward::Alone => Ok(Sync::alone()),
        Toward::Folder { at } => folder_way(session, at.trim().to_string()),
        Toward::Cloud { account, email } => {
            super::clouding::has_access(account)?;
            Ok(Sync::cloud(CloudAt {
                provider: GOOGLE.to_string(),
                account: account.clone(),
                email: email.clone(),
            }))
        }
    }
}

struct Ends {
    from: Option<tisty_carrier::Shared>,
    to: Option<tisty_carrier::Shared>,
    way: Sync,
    here: tisty_carrier::Here,
    alive: Vec<String>,
}

fn ends(session: &mut Session, toward: &Toward) -> Answer<Ends> {
    session.refresh_settings();
    let way = way_of(session, toward)?;
    if session.config.sync.as_ref() == Some(&way) {
        return Err(Refusal::of("moveSamePlace"));
    }
    if session.config.sync.as_ref().map(Sync::leaving) == Some(Leaving::Later) {
        return Err(Refusal::of("syncLaterToLeave"));
    }
    let from = session.carrier();
    if from.is_none() && tisty_carrier::cloud_of(session.config.sync.as_ref()).is_some() {
        return Err(Refusal::of("moveFromUnreachable"));
    }
    let to = tisty_carrier::chosen_with(Some(&way), Some(&session.reach)).carrier;
    if way != Sync::alone() && to.is_none() {
        return Err(Refusal::of("moveToUnreachable"));
    }
    Ok(Ends {
        from,
        to,
        way,
        here: session.here(),
        alive: session.alive(),
    })
}

#[tauri::command(async)]
pub async fn move_plan(session: tauri::State<'_, Mutex<Session>>, toward: Toward) -> Answer<Plan> {
    let ends = ends(&mut held(&session), &toward)?;
    tauri::async_runtime::spawn_blocking(move || {
        let down = ends
            .from
            .as_deref()
            .map(|from| tisty_carrier::homing(from, &ends.here))
            .unwrap_or_default();
        let up = ends
            .to
            .as_deref()
            .map(|to| tisty_carrier::filling(to, &ends.here))
            .unwrap_or_default();
        Plan {
            down: down.into(),
            // What comes down has to go up again too.
            up: Weight {
                files: up.files + if ends.to.is_some() { down.files } else { 0 },
                bytes: up.bytes + if ends.to.is_some() { down.bytes } else { 0 },
            },
            from_cloud: ends.from.as_ref().is_some_and(|from| from.frees_space()),
            to_cloud: ends.to.as_ref().is_some_and(|to| to.frees_space()),
        }
    })
    .await
    .map_err(|_| Refusal::of("internal"))
}

/// Brings everything home, fills the new place, and only then points there; a move that has to wait resumes later.
#[tauri::command(async)]
pub async fn move_to(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    toward: Toward,
) -> Answer<Moving> {
    let _done = alone.inner().taken()?;
    let (ends, paths, left) = {
        let mut session = held(&session);
        let ends = ends(&mut session, &toward)?;
        let left = tisty_carrier::cloud_of(session.config.sync.as_ref()).cloned();
        (ends, session.paths.clone(), left)
    };
    pending_kept(&paths, Some(&toward));

    let mut telling = Telling::new(app.clone(), false);
    let Ends {
        from,
        to,
        way,
        here,
        alive,
    } = ends;
    let shifted = tauri::async_runtime::spawn_blocking(move || {
        let mut saying = |far| telling.hear(far);
        let done = tisty_carrier::shift(from.as_deref(), to.as_deref(), &here, &alive, &mut saying);
        telling.flush();
        done
    })
    .await
    .map_err(|_| Refusal::of("internal"))?;
    // Whatever came home already sits in the store, so it is read even when the move has to wait.
    catching_up(
        &session,
        &paths,
        "the store would not project after a move brought things home",
        channel::SYNC,
    )
    .await?;

    match shifted.map_err(said)? {
        tisty_carrier::Shift::Homing { owed, deferred } => Ok(waiting("homing", owed, deferred)),
        tisty_carrier::Shift::Filling { owed, deferred } => Ok(waiting("filling", owed, deferred)),
        tisty_carrier::Shift::Ready => {
            changed_to(&mut held(&session), way.clone(), moving_in)?;
            pending_kept(&paths, None);
            let _ = app.emit("unstuck", ());
            let left_cloud = left.filter(|was| tisty_carrier::cloud_of(Some(&way)) != Some(was));
            Ok(Moving::Done { left_cloud })
        }
    }
}

fn waiting(
    stage: &'static str,
    owed: tisty_carrier::Owed,
    deferred: Option<tisty_carrier::Deferred>,
) -> Moving {
    Moving::Waiting {
        stage,
        left: owed.into(),
        reason: deferred.map(|one| match one.reason {
            tisty_carrier::Reason::Budget => "budget",
            tisty_carrier::Reason::Limit => "limit",
            tisty_carrier::Reason::Offline => "offline",
            tisty_carrier::Reason::Busy => "busy",
            tisty_carrier::Reason::Paused => "paused",
            tisty_carrier::Reason::Changed => "changed",
        }),
    }
}

/// Everything is already in both places by now, so leaving any way, a cloud included, strands nothing.
pub(crate) fn moving_in(chosen: Sync) -> impl FnOnce(&mut tisty_core::Config) -> Option<Refusal> {
    move |config| match config.sync.as_ref().map(Sync::leaving) {
        Some(Leaving::Later) => Some(Refusal::of("syncLaterToLeave")),
        _ => {
            config.sync = Some(chosen);
            None
        }
    }
}

fn pending_kept(paths: &tisty_core::Paths, toward: Option<&Toward>) {
    let at = paths.private().join(PENDING);
    let written = match toward {
        Some(toward) => serde_json::to_vec(toward)
            .map_err(std::io::Error::other)
            .and_then(|bytes| {
                std::fs::create_dir_all(paths.private())?;
                tisty_core::store::write_atomic(&at, &bytes).map_err(std::io::Error::other)
            }),
        None => match std::fs::remove_file(&at) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        },
    };
    if let Err(why) = written {
        tisty_core::witness::warn(
            channel::SYNC,
            "where a move was going could not be written down",
            &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
        );
    }
}

/// A move that had to wait, so the window can offer to carry on with it.
#[tauri::command]
pub fn move_pending(session: tauri::State<'_, Mutex<Session>>) -> Option<Toward> {
    let at = held(&session).paths.private().join(PENDING);
    std::fs::read(at)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

/// Nothing was pointed anywhere yet, so dropping a move only forgets where it was going.
#[tauri::command]
pub fn move_dropped(session: tauri::State<'_, Mutex<Session>>) {
    pending_kept(&held(&session).paths.clone(), None);
}

/// After moving off a cloud: its copies go to the provider's bin, then its access is let go of.
#[tauri::command(async)]
pub async fn cloud_wipe(
    session: tauri::State<'_, Mutex<Session>>,
    account: String,
    email: String,
) -> Answer<usize> {
    let at = CloudAt {
        provider: GOOGLE.to_string(),
        account,
        email,
    };
    let carrier = {
        let session = held(&session);
        if tisty_carrier::cloud_of(session.config.sync.as_ref())
            .is_some_and(|now| now.account == at.account)
        {
            return Err(Refusal::of("wipeChosen"));
        }
        tisty_carrier::chosen_with(Some(&Sync::cloud(at.clone())), Some(&session.reach)).carrier
    };
    let carrier = carrier.ok_or_else(|| Refusal::about("cloudLost", "Google Drive".to_string()))?;
    let gone = tauri::async_runtime::spawn_blocking(move || carrier.wipe())
        .await
        .map_err(|_| Refusal::of("internal"))?
        .map_err(said)?;
    super::clouding::let_go_of(&at.account);
    Ok(gone)
}

#[cfg(test)]
#[path = "relocating_test.rs"]
mod tests;
