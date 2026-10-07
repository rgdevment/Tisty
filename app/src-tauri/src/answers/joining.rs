use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

use super::storing::let_go_to;
use crate::{Answer, OneAtATime, Refusal, Session, blamed, elsewhere, held, said};

#[tauri::command]
pub async fn join_them(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<u64> {
    let _done = alone.inner().taken()?;
    if tisty_core::paths::profile().is_some() {
        return Err(Refusal::of("sandboxCannotJoin"));
    }
    let (paths, aside) = {
        let session = held(&session);
        (session.paths.clone(), session.paths.cache().to_path_buf())
    };

    let at = std::path::PathBuf::from(&into);
    let made = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::backup::reset(&paths, &at, &aside, None)
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::error(
            channel::BACKUP,
            "nothing was reset because the backup did not land",
            &[("why", Fact::Why(e.to_string()))],
        );
        match e {
            tisty_core::Error::TooBig => Refusal::of("tooBig"),
            _ => Refusal::about("cannotWrite", into),
        }
    })?;

    let fresh = elsewhere(Session::open).await?.map_err(|e| {
        blamed(
            channel::BACKUP,
            "the session would not reopen after being reset",
            e,
        )
    })?;
    *held(&session) = fresh;
    held(&session).keep(|c| c.restored_at = None)?;
    Ok(made.bytes)
}

#[tauri::command]
pub async fn take_over(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<u64> {
    let _done = alone.inner().taken()?;
    if tisty_core::paths::profile().is_some() {
        return Err(Refusal::of("sandboxCannotJoin"));
    }
    let (dest, aside, ours) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        let ours = tisty_core::store::identity(session.paths.store())
            .map_err(|e| blamed(channel::SYNC, "this machine has no name of its own", e))?;
        (dest, session.paths.cache().to_path_buf(), ours)
    };

    let at = std::path::PathBuf::from(&into);
    let made = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::backup::take_over(&dest, &ours, &at, &aside)
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::error(
            channel::BACKUP,
            "the folder was left alone because its backup did not land",
            &[("why", Fact::Why(e.to_string()))],
        );
        match e {
            tisty_core::Error::TooBig => Refusal::of("tooBig"),
            _ => Refusal::about("cannotWrite", into),
        }
    })?;

    held(&session).keep(|c| c.restored_at = None)?;
    Ok(made.bytes)
}

fn kinned(store: &std::path::Path, dest: &std::path::Path) -> &'static str {
    match tisty_sync::kinship(store, dest) {
        tisty_sync::Kin::SameLineage => "sameLineage",
        tisty_sync::Kin::Clash(_) => "clash",
        tisty_sync::Kin::Unsure(_) => "unsure",
        tisty_sync::Kin::Strangers => "strangers",
    }
}

/// The folder is walked with the session let go of: a cloud folder can take its time, and every
/// other command waits behind whoever holds it.
#[tauri::command(async)]
pub fn folder_astir(session: tauri::State<'_, Mutex<Session>>) -> Answer<String> {
    let dest = {
        let session = held(&session);
        match session.config.sync.clone() {
            Some(tisty_core::config::Sync::Folder(dest)) => dest,
            _ => return Err(Refusal::of("noRemote")),
        }
    };
    Ok(tisty_sync::stirring(&dest).to_string())
}

#[tauri::command(async)]
pub fn sync_kin(session: tauri::State<'_, Mutex<Session>>) -> Answer<&'static str> {
    let (store, dest) = folder_and_store(&session)?;
    Ok(kinned(&store, &dest))
}

/// Read after the lock is let go: a slow folder must not hold up every other command.
fn folder_and_store(
    session: &tauri::State<'_, Mutex<Session>>,
) -> Answer<(std::path::PathBuf, std::path::PathBuf)> {
    let session = held(session);
    let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
        return Err(Refusal::of("noRemote"));
    };
    Ok((session.paths.store(), dest))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Joining {
    kin: &'static str,
    fresh: bool,
    holds: bool,
    alias: Option<String>,
    coming: bool,
}

#[tauri::command(async)]
pub fn joining(session: tauri::State<'_, Mutex<Session>>) -> Answer<Joining> {
    let (store, dest) = folder_and_store(&session)?;
    let signed = tisty_sync::signed_here(&dest);
    Ok(Joining {
        kin: kinned(&store, &dest),
        fresh: !tisty_core::store::inhabited(&store),
        holds: tisty_core::store::inhabited(dest.join(tisty_sync::STORE)),
        alias: signed.alias,
        coming: signed.coming,
    })
}

#[tauri::command]
pub async fn merge_stores(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<bool> {
    let _done = alone.inner().taken()?;
    if tisty_core::paths::profile().is_some() {
        return Err(Refusal::of("sandboxCannotJoin"));
    }
    let (data, dest, aside, device, also, key) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        (
            session.paths.data().to_path_buf(),
            dest,
            session.paths.cache().to_path_buf(),
            session.config.device_id.0.clone(),
            let_go_to(&session),
            session.store.signs(),
        )
    };

    let at = std::path::PathBuf::from(&into);
    let seam = tauri::async_runtime::spawn_blocking(move || -> Answer<tisty_sync::Stitched> {
        tisty_core::backup::write(&data, &at, &aside, also.as_deref()).map_err(|e| {
            witness::error(
                channel::BACKUP,
                "nothing was joined because the backup did not land",
                &[("why", Fact::Why(e.to_string()))],
            );
            match e {
                tisty_core::Error::TooBig => Refusal::of("tooBig"),
                _ => Refusal::about("cannotWrite", into),
            }
        })?;
        tisty_sync::stitch(&data, &device, &dest, key).map_err(|trouble| {
            let refusal = said(trouble);
            witness::warn(
                channel::SYNC,
                "the two histories were left apart",
                &[("code", Fact::Code(refusal.code))],
            );
            refusal
        })
    })
    .await
    .map_err(|_| Refusal::of("internal"))??;

    let fresh = elsewhere(Session::open).await?.map_err(|e| {
        blamed(
            channel::BACKUP,
            "the session would not reopen after joining",
            e,
        )
    })?;
    *held(&session) = fresh;
    Ok(seam.stitch.is_some())
}
