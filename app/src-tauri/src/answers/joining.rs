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
        let Some(dest) = session.place() else {
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

fn kinned(kin: tisty_carrier::Kin) -> &'static str {
    match kin {
        tisty_carrier::Kin::SameLineage => "sameLineage",
        tisty_carrier::Kin::Clash(_) => "clash",
        tisty_carrier::Kin::Unsure(_) => "unsure",
        tisty_carrier::Kin::Strangers => "strangers",
    }
}

/// The folder is walked with the session let go of: a cloud folder can take its time, and every
/// other command waits behind whoever holds it.
#[tauri::command(async)]
pub fn folder_astir(session: tauri::State<'_, Mutex<Session>>) -> Answer<String> {
    let carrier = held(&session).carrying()?;
    Ok(carrier.stirring().to_string())
}

#[tauri::command(async)]
pub fn sync_kin(session: tauri::State<'_, Mutex<Session>>) -> Answer<&'static str> {
    let (carrier, here) = carrier_and_here(&session)?;
    Ok(kinned(carrier.kin(&here)))
}

/// Read after the lock is let go: a slow folder must not hold up every other command.
fn carrier_and_here(
    session: &tauri::State<'_, Mutex<Session>>,
) -> Answer<(tisty_carrier::Shared, tisty_carrier::Here)> {
    let session = held(session);
    Ok((session.carrying()?, session.here()))
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
    let (carrier, here) = carrier_and_here(&session)?;
    let signed = carrier.signed();
    let store = here.data.join(tisty_carrier::STORE);
    Ok(Joining {
        kin: kinned(carrier.kin(&here)),
        fresh: !tisty_core::store::inhabited(&store),
        holds: carrier
            .place()
            .is_some_and(|dest| tisty_core::store::inhabited(dest.join(tisty_carrier::STORE))),
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
    let (carrier, here, also, key) = {
        let session = held(&session);
        (
            session.carrying()?,
            session.here(),
            let_go_to(&session),
            session.store.signs(),
        )
    };

    let at = std::path::PathBuf::from(&into);
    let seam = tauri::async_runtime::spawn_blocking(move || -> Answer<tisty_carrier::Stitched> {
        tisty_core::backup::write(&here.data, &at, &here.aside, also.as_deref()).map_err(|e| {
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
        carrier.stitch(&here, key).map_err(|trouble| {
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
