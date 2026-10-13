use std::sync::Mutex;

use tauri::Emitter;
use tisty_core::config::{CloudAt, Leaving, Sync};
use tisty_core::witness::channel;

use super::storing::{
    After, Joins, Telling, catching_up, changed_with, cloud_joined, folder_way, forget_word_in,
    joined_on, went_back_on,
};
use crate::reaching::GOOGLE;
use crate::{Answer, OneAtATime, Refusal, Session, held, said};

const PENDING: &str = "moving.json";
const WAITED: std::time::Duration = std::time::Duration::from_secs(30);

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
    Done {
        left_cloud: Option<CloudAt>,
        told: bool,
    },
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
        Toward::Cloud { account, email } => Ok(Sync::cloud(CloudAt {
            provider: GOOGLE.to_string(),
            account: account.clone(),
            email: email.clone(),
        })),
    }
}

struct Ends {
    from: Option<tisty_carrier::Shared>,
    to: Option<tisty_carrier::Shared>,
    way: Sync,
    here: tisty_carrier::Here,
    alive: Vec<String>,
    joins: Joins,
}

fn ends(session: &mut Session, toward: &Toward) -> Answer<Ends> {
    session.refresh_settings();
    let way = way_of(session, toward)?;
    let now = session.config.sync.clone();
    if now
        .as_ref()
        .is_some_and(|now| crate::leaving::same_place(now, &way))
        || (now.is_none() && way == Sync::alone())
    {
        return Err(Refusal::of("moveSamePlace"));
    }
    if now.as_ref().map(Sync::leaving) == Some(Leaving::Later) {
        return Err(Refusal::of("syncLaterToLeave"));
    }
    if tisty_carrier::cloud_of(now.as_ref()).is_some() && !cloud_joined(session) {
        return Err(Refusal::of("moveFromUnused"));
    }
    let from = session.carrier();
    if from.is_none() && tisty_carrier::cloud_of(now.as_ref()).is_some() {
        return Err(Refusal::of("moveFromUnreachable"));
    }
    let to = match tisty_carrier::cloud_of(Some(&way)) {
        Some(at) => session.reach.apart(at),
        None => tisty_carrier::chosen_with(Some(&way), Some(&session.reach)).carrier,
    };
    if way != Sync::alone() && to.is_none() {
        return Err(Refusal::of("moveToUnreachable"));
    }
    if session.config.restored_at.is_some()
        && let Some(to) = &to
        && went_back_on(session, to.theirs())
    {
        return Err(Refusal::about("restoredApart", towards(toward)));
    }
    let joins = Joins {
        rejoined: joined_on(session, Some(&way)),
        left: true,
    };
    Ok(Ends {
        from,
        to,
        way,
        here: session.here(),
        alive: session.alive(),
        joins,
    })
}

async fn accessible(toward: &Toward) -> Answer<()> {
    let Toward::Cloud { account, .. } = toward else {
        return Ok(());
    };
    let account = account.clone();
    tauri::async_runtime::spawn_blocking(move || super::clouding::has_access(&account))
        .await
        .map_err(|_| Refusal::of("internal"))?
}

fn towards(toward: &Toward) -> String {
    match toward {
        Toward::Alone => String::new(),
        Toward::Folder { at } => at.clone(),
        Toward::Cloud { email, .. } => email.clone(),
    }
}

#[tauri::command(async)]
pub async fn move_plan(session: tauri::State<'_, Mutex<Session>>, toward: Toward) -> Answer<Plan> {
    accessible(&toward).await?;
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
        let also = match (&ends.from, &ends.to) {
            (Some(from), Some(to)) if to.frees_space() => {
                tisty_carrier::homing_up_to(&**from, &ends.here, tisty_core::attach::HELD_UP_TO)
            }
            (_, Some(_)) => down,
            _ => tisty_carrier::Owed::default(),
        };
        Plan {
            down: down.into(),
            up: Weight {
                files: up.files + also.files,
                bytes: up.bytes + also.bytes,
            },
            from_cloud: ends.from.as_ref().is_some_and(|from| from.frees_space()),
            to_cloud: ends.to.as_ref().is_some_and(|to| to.frees_space()),
        }
    })
    .await
    .map_err(|_| Refusal::of("internal"))
}

#[tauri::command(async)]
pub async fn move_to(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    toward: Toward,
) -> Answer<Moving> {
    let _done = alone.inner().waited(WAITED).await?;
    accessible(&toward).await?;
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
        joins,
    } = ends;
    let shifting = from.clone();
    let shifted = tauri::async_runtime::spawn_blocking(move || {
        let mut saying = |far| telling.hear(far);
        let done = tisty_carrier::shift(
            shifting.as_deref(),
            to.as_deref(),
            &here,
            &alive,
            &mut saying,
        );
        telling.flush();
        done
    })
    .await
    .map_err(|_| Refusal::of("internal"))?;
    let caught = catching_up(
        &session,
        &paths,
        "the store would not project after a move brought things home",
        channel::SYNC,
    )
    .await;
    let shifted = match shifted {
        Ok(shifted) => shifted,
        Err(trouble) => {
            pending_kept(&paths, None);
            return Err(said(trouble));
        }
    };
    caught?;
    match shifted {
        tisty_carrier::Shift::Homing { owed, deferred } => Ok(waiting("homing", owed, deferred)),
        tisty_carrier::Shift::Filling { owed, deferred } => Ok(waiting("filling", owed, deferred)),
        tisty_carrier::Shift::Unsettled => Ok(Moving::Waiting {
            stage: "homing",
            left: Weight::default(),
            reason: Some("unsettled"),
        }),
        tisty_carrier::Shift::Ready => {
            let after = changed_with(&mut held(&session), way.clone(), moving_in, joins)?;
            pending_kept(&paths, None);
            let at = paths.clone();
            let told = tauri::async_runtime::spawn_blocking(move || delivered(&at, after, from))
                .await
                .map_err(|_| Refusal::of("internal"))?;
            let _ = app.emit("unstuck", ());
            let left_cloud = left.filter(|was| tisty_carrier::cloud_of(Some(&way)) != Some(was));
            Ok(Moving::Done { left_cloud, told })
        }
    }
}

fn delivered(paths: &tisty_core::Paths, after: After, from: Option<tisty_carrier::Shared>) -> bool {
    let told = match after.word {
        Some((left, said, _)) => crate::leaving::left_said(paths, &left, &said, from),
        None => !after.owes_word,
    };
    if let Some(now) = after.unword {
        forget_word_in(&now);
    }
    told
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
            tisty_carrier::Reason::Troubled => "troubled",
            tisty_carrier::Reason::Unsent => "unsent",
        }),
    }
}

pub(crate) fn moving_in(chosen: Sync) -> impl FnOnce(&mut tisty_core::Config) -> Option<Refusal> {
    move |config| match config.sync.as_ref().map(Sync::leaving) {
        Some(Leaving::Later) => Some(Refusal::of("syncLaterToLeave")),
        _ => {
            config.sync = Some(chosen);
            None
        }
    }
}

pub(crate) fn pending_cloud(paths: &tisty_core::Paths) -> Option<String> {
    let toward: Toward =
        serde_json::from_slice(&std::fs::read(paths.private().join(PENDING)).ok()?).ok()?;
    match toward {
        Toward::Cloud { account, .. } => Some(account),
        _ => None,
    }
}

pub(crate) fn pending_kept(paths: &tisty_core::Paths, toward: Option<&Toward>) {
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

#[tauri::command]
pub fn move_pending(session: tauri::State<'_, Mutex<Session>>) -> Option<Toward> {
    let at = held(&session).paths.private().join(PENDING);
    std::fs::read(at)
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

#[tauri::command]
pub fn move_dropped(session: tauri::State<'_, Mutex<Session>>) {
    let session = held(&session);
    let dropped = pending_cloud(&session.paths).filter(|account| {
        tisty_carrier::cloud_of(session.config.sync.as_ref())
            .is_none_or(|now| &now.account != account)
            && !crate::leaving::owed_to(&session.paths, account)
    });
    pending_kept(&session.paths, None);
    if let Some(account) = dropped {
        super::clouding::let_go_of(&account);
    }
}

#[tauri::command(async)]
pub async fn cloud_wipe(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    account: String,
    email: String,
) -> Answer<usize> {
    let _done = alone.inner().waited(WAITED).await?;
    let at = CloudAt {
        provider: GOOGLE.to_string(),
        account,
        email,
    };
    let (carrier, paths, here, alive) = {
        let session = held(&session);
        if tisty_carrier::cloud_of(session.config.sync.as_ref())
            .is_some_and(|now| now.account == at.account)
        {
            return Err(Refusal::of("wipeChosen"));
        }
        (
            session.reach.apart(&at),
            session.paths.clone(),
            session.here(),
            session.alive(),
        )
    };
    let carrier = carrier.ok_or_else(|| Refusal::about("cloudLost", "Google Drive".to_string()))?;
    if crate::leaving::owed_to(&paths, &at.account) {
        return Err(Refusal::of("wipeUntold"));
    }
    let gone = tauri::async_runtime::spawn_blocking(move || wiped(&*carrier, &here, &alive))
        .await
        .map_err(|_| Refusal::of("internal"))?;
    catching_up(
        &session,
        &paths,
        "the store would not project after a cloud left behind brought something home",
        channel::SYNC,
    )
    .await?;
    let gone = gone?;
    super::clouding::let_go_of(&at.account);
    Ok(gone)
}

/// Asked of the cloud itself, never of the mirror here: emptied only once all is home and the word is up there.
pub(crate) fn wiped(
    carrier: &dyn tisty_carrier::Carrier,
    here: &tisty_carrier::Here,
    alive: &[String],
) -> Answer<usize> {
    if !tisty_carrier::all_home(carrier, here, alive).map_err(said)? {
        return Err(Refusal::of("wipeNotHome"));
    }
    carrier.wipe().map_err(|trouble| match trouble {
        tisty_carrier::Trouble::NotThere(name) if name == tisty_core::moved::NAMED => {
            Refusal::of("wipeUntold")
        }
        other => said(other),
    })
}

/// Only a cloud this build can never reach again is left without bringing everything home first.
fn lost_cloud(session: &Session) -> Answer<CloudAt> {
    let at = tisty_carrier::cloud_of(session.config.sync.as_ref())
        .cloned()
        .ok_or_else(|| Refusal::of("noRemote"))?;
    match super::clouding::lost_for_good_at(&session.reach.home_of(&at), &at.account) {
        true => Ok(at),
        false => Err(Refusal::of("cloudNotLost")),
    }
}

/// What would stay only in Drive, read off this disk, so the person knows before letting it go.
#[tauri::command(async)]
pub async fn cloud_stranded(session: tauri::State<'_, Mutex<Session>>) -> Answer<Weight> {
    let (home, here) = {
        let session = held(&session);
        let at = lost_cloud(&session)?;
        (session.reach.home_of(&at), session.here())
    };
    tauri::async_runtime::spawn_blocking(move || tisty_carrier::only_up_there(&home, &here).into())
        .await
        .map_err(|_| Refusal::of("internal"))
}

#[tauri::command(async)]
pub async fn cloud_abandon(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
) -> Answer<()> {
    let _done = alone.inner().waited(WAITED).await?;
    {
        let mut session = held(&session);
        session.refresh_settings();
        lost_cloud(&session)?;
        left_for_alone(&mut session)?;
    }
    let _ = app.emit("unstuck", ());
    Ok(())
}

// Nobody up there can be told and its access is let go of: the copy here keeps every task and document.
pub(crate) fn left_for_alone(session: &mut Session) -> Answer<()> {
    let joins = Joins {
        rejoined: false,
        left: false,
    };
    changed_with(session, Sync::alone(), moving_in, joins)?;
    pending_kept(&session.paths, None);
    Ok(())
}

#[cfg(test)]
#[path = "relocating_test.rs"]
mod tests;
