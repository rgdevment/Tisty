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
        let carrier = session.carrying()?;
        let Some(dest) = carrier.place().map(std::path::Path::to_path_buf) else {
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

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Peeked {
    /// This machine has never written a history of its own.
    fresh: bool,
    /// Unknown when the place could not be asked, so the window decides again once a round has run.
    holds: Option<bool>,
}

/// Whether a place already holds a store, in one look and never a pull, so a first round is told before it starts.
#[tauri::command(async)]
pub async fn place_peek(
    session: tauri::State<'_, Mutex<Session>>,
    toward: Option<super::relocating::Toward>,
) -> Answer<Peeked> {
    let (carrier, store) = {
        let session = held(&session);
        let carrier = match &toward {
            Some(toward) => super::relocating::carrier_toward(&session, toward)?,
            None => session.carrying()?,
        };
        (carrier, session.paths.store())
    };
    let ours = toward.is_none();
    elsewhere(move || peeked(&*carrier, &store, ours)).await
}

pub(crate) fn peeked(
    carrier: &dyn tisty_carrier::Carrier,
    store: &std::path::Path,
    ours: bool,
) -> Peeked {
    Peeked {
        fresh: !tisty_core::store::inhabited(store),
        holds: carrier.peeks_a_store(ours),
    }
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
    let kin = kinned(carrier.kin(&here));
    carrier.refuses().map_err(said)?;
    let store = here.data.join(tisty_carrier::STORE);
    Ok(Joining {
        kin,
        fresh: !tisty_core::store::inhabited(&store),
        holds: carrier.holds_a_store(),
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

/// What a place answers when it holds another history: the two that open the choice of three doors.
pub(crate) fn apart(code: &str) -> bool {
    matches!(code, "wouldReset" | "otherStore")
}

/// A new machine takes a place's history on its own only when the place names whose history it is,
/// that history is not one this machine's names clash with or cannot read, and it was not taken once already.
pub(crate) fn may_adopt(
    code: &str,
    theirs: Option<&str>,
    kin: &tisty_carrier::Kin,
    took: Option<&str>,
) -> bool {
    let theirs = theirs.map(str::trim).filter(|one| !one.is_empty());
    code == "otherStore"
        && theirs.is_some_and(|one| took.map(str::trim) != Some(one))
        && !matches!(
            kin,
            tisty_carrier::Kin::Clash(_) | tisty_carrier::Kin::Unsure(_)
        )
}

pub(crate) fn new_here(paths: &tisty_core::Paths) -> bool {
    tisty_core::paths::profile().is_none()
        && tisty_core::store::inhabited(paths.store())
        && tisty_core::untouched::untouched(&paths.store(), &paths.docs())
}

/// The place's history a round held apart will be taken without asking, if it will; read with the session let go of.
pub(crate) async fn will_adopt(
    session: &tauri::State<'_, Mutex<Session>>,
    toward: Option<tisty_carrier::Shared>,
    code: &'static str,
    theirs: Option<String>,
) -> Option<String> {
    if !apart(code) {
        return None;
    }
    let (carrier, here, paths, took) = {
        let session = held(session);
        (
            toward.or_else(|| session.carrying().ok())?,
            session.here(),
            session.paths.clone(),
            session.config.took_quietly.clone(),
        )
    };
    elsewhere(move || {
        let theirs = theirs?;
        let may = new_here(&paths)
            && may_adopt(code, Some(&theirs), &carrier.kin(&here), took.as_deref());
        may.then_some(theirs)
    })
    .await
    .ok()
    .flatten()
}

/// A machine holding only what its first opening sowed takes the place's history without asking,
/// once a backup of what it had has landed. Nothing happens to one that holds anything of the person's.
pub(crate) fn quietly_joined(paths: &tisty_core::Paths) -> Option<std::path::PathBuf> {
    if !new_here(paths) {
        return None;
    }
    let into = tisty_core::backup::kept_before_joining(paths, jiff::Timestamp::now()).ok()?;
    let landed = tisty_core::backup::write(paths.data(), &into, paths.cache(), None);
    // Asked again once the backup has landed: a command line or agent that wrote meanwhile is never reset away.
    if landed.is_ok() && !new_here(paths) {
        return None;
    }
    match landed.and_then(|_| tisty_core::backup::set_aside(paths)) {
        Ok(()) => {
            witness::note(
                channel::SYNC,
                "this machine held only what it starts with, so it took the place's history without asking",
                &[("backup", Fact::Path(into.clone()))],
            );
            Some(into)
        }
        Err(e) => {
            witness::warn(
                channel::BACKUP,
                "a new machine was left as it was because its backup did not land",
                &[("why", Fact::Why(e.to_string()))],
            );
            None
        }
    }
}

// Under the session's lock, so nothing written through this window slips between the check and the reset.
pub(crate) async fn adopted_quietly(
    session: &tauri::State<'_, Mutex<Session>>,
    theirs: String,
) -> bool {
    let joined = {
        let mut session = held(session);
        let _ = session.keep(|c| c.took_quietly = Some(theirs));
        quietly_joined(&session.paths).is_some()
    };
    if !joined {
        return false;
    }
    match elsewhere(Session::open).await {
        Ok(Ok(fresh)) => {
            *held(session) = fresh;
            let _ = held(session).keep(|c| c.restored_at = None);
            true
        }
        _ => {
            witness::error(
                channel::BACKUP,
                "the session would not reopen after a new machine took the place's history",
                &[],
            );
            false
        }
    }
}

#[cfg(test)]
#[path = "joining_test.rs"]
mod tests;
