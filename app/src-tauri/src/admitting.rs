use std::sync::{Mutex, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tisty_carrier::knock;
use tisty_core::event::DeviceId;
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::handing::{Ask, Giver};

use crate::handed::{Here, Knocking, NEW_FOR, fits, here_now, hitched, now, seated, signer};
use crate::{Answer, Refusal, Session, elsewhere, held};

// A fresh ask is kept rather than made again while this much of its hour is left, so reopening the window never voids a code.
const KEPT_FOR: i64 = NEW_FOR - 10 * 60;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Admitted {
    pub step: &'static str,
    pub code: Option<String>,
    pub by: Option<String>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Newcomer {
    pub device: String,
    pub name: String,
    pub at: i64,
    pub taken: bool,
}

type Worked<T> = Result<T, Refusal>;

fn trouble(why: impl ToString) -> Refusal {
    tisty_core::witness::note(
        tisty_core::witness::channel::CLOUD,
        "a new computer could not be let into the sealed cloud",
        &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
    );
    Refusal::of("admitNotNow")
}

fn up(hitch: tisty_carrier::Hitch) -> Refusal {
    trouble(hitched(hitch))
}

async fn off_the_lock<T: Send + 'static>(
    app: &AppHandle,
    work: impl FnOnce(&Mutex<Session>, &Knocking, &Here, Option<CloudKey>) -> Worked<T> + Send + 'static,
) -> Answer<T> {
    let app = app.clone();
    elsewhere(move || {
        let (session, knocking) = (app.state::<Mutex<Session>>(), app.state::<Knocking>());
        let (here, key) = here_now(&session, &knocking).map_err(trouble)?;
        work(&session, &knocking, &here, key)
    })
    .await?
}

fn trusted_now(knocking: &Knocking) -> Option<String> {
    knocking
        .trusted
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

fn trust(knocking: &Knocking, giver: Option<String>) {
    *knocking
        .trusted
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = giver;
}

fn asked_now(knocking: &Knocking, here: &Here) -> Option<(Ask, String)> {
    let asked = knocking
        .asked
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    asked
        .as_ref()
        .filter(|(ask, _)| ask.cloud == here.cloud)
        .map(|(ask, one_use)| (ask.clone(), one_use.revealed()))
}

fn ask_again(session: &Mutex<Session>, knocking: &Knocking, here: &Here) -> Worked<()> {
    trust(knocking, None);
    let mut asked = knocking
        .asked
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    crate::handed::ask_now(session, &mut asked, here).map_err(trouble)
}

fn ask(
    session: &Mutex<Session>,
    knocking: &Knocking,
    here: &Here,
    key: Option<CloudKey>,
) -> Worked<()> {
    if key.is_some() {
        return Err(Refusal::of("admitAlreadyIn"));
    }
    // One on the list is handed the key on its own, and asking again would take away an answer on its way.
    if seated(session, &here.device.0) {
        return Err(Refusal::of("admitOnList"));
    }
    let standing = asked_now(knocking, here).filter(|(ask, _)| now() - ask.at < KEPT_FOR);
    if let Some((ask, _)) = standing {
        let up_there = knock::ask_of(&*here.plain, &here.home, &here.device.0).map_err(up)?;
        if up_there.as_ref() == Some(&ask) {
            return Ok(());
        }
    }
    ask_again(session, knocking, here)
}

fn shown(
    session: &Mutex<Session>,
    knocking: &Knocking,
    here: &Here,
    key: Option<CloudKey>,
) -> Worked<Admitted> {
    let said = |step: &'static str, giver: Option<&Giver>, code: Option<String>| Admitted {
        step,
        code,
        by: giver.map(|one| one.name.clone()),
    };
    if key.is_some() {
        return Ok(said("done", None, None));
    }
    let Some((ask, reveal)) = asked_now(knocking, here) else {
        return Ok(said("idle", None, None));
    };
    // Past its hour no computer answers it any more, so a new ask goes up in its place.
    if now() - ask.at > NEW_FOR {
        ask_again(session, knocking, here)?;
        return Ok(said("waiting", None, None));
    }
    let Some(giver) = knock::giver_of(&*here.plain, &here.home, &here.device.0).map_err(up)? else {
        return Ok(said("waiting", None, None));
    };
    if knock::reveal_of(&*here.plain, &here.home, &here.device.0).map_err(up)?
        != Some(reveal.clone())
    {
        knock::revealed(&*here.plain, &here.home, &here.device.0, &reveal).map_err(up)?;
    }
    if trusted_now(knocking).as_deref() != Some(giver.key.as_str()) {
        let code = code_of(knocking, &ask, &giver, &reveal);
        return Ok(said("compare", Some(&giver), code));
    }
    match crate::handed::asking(session, knocking, here).map_err(trouble)? {
        true => Ok(said("done", Some(&giver), None)),
        false => Ok(said("trusted", Some(&giver), None)),
    }
}

// Ground once an ask and giver, since the window asks every few seconds and scrypt takes its time and memory.
fn code_of(knocking: &Knocking, ask: &Ask, giver: &Giver, reveal: &str) -> Option<String> {
    let mut coded = knocking
        .coded
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    if let Some((sig, key, code)) = coded.as_ref()
        && *sig == ask.sig
        && *key == giver.key
    {
        return Some(code.clone());
    }
    let code = ask.code(&giver.key()?, reveal)?;
    *coded = Some((ask.sig.clone(), giver.key.clone(), code.clone()));
    Some(code)
}

// The code travels back from the window, so a giver that changed since it was shown is never trusted.
fn trusted(knocking: &Knocking, here: &Here, code: &str) -> Worked<()> {
    let (ask, reveal) = asked_now(knocking, here).ok_or_else(|| Refusal::of("admitGone"))?;
    let giver = knock::giver_of(&*here.plain, &here.home, &here.device.0)
        .map_err(up)?
        .ok_or_else(|| Refusal::of("admitGone"))?;
    if code_of(knocking, &ask, &giver, &reveal).as_deref() != Some(code) {
        return Err(Refusal::of("admitChanged"));
    }
    trust(knocking, Some(giver.key));
    Ok(())
}

fn cancelled(knocking: &Knocking, here: &Here) -> Worked<()> {
    *knocking
        .asked
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = None;
    trust(knocking, None);
    knock::withdrawn(&*here.plain, &here.home, &here.device.0).map_err(up)
}

fn fresh(session: &Mutex<Session>, ask: &Ask, here: &Here) -> bool {
    !seated(session, &ask.device)
        && fits(ask, here)
        && (-NEW_FOR..=NEW_FOR).contains(&(now() - ask.at))
}

fn looked(session: &Mutex<Session>, here: &Here, key: Option<CloudKey>) -> Worked<Vec<Newcomer>> {
    key.ok_or_else(|| Refusal::of("admitNoKey"))?;
    let mine = signer(here).map_err(trouble)?.verifying_key();
    let mut found = Vec::new();
    for one in knock::knocks(&*here.plain, &here.home).map_err(up)? {
        let Some(ask) = one.ask.filter(|ask| fresh(session, ask, here)) else {
            continue;
        };
        let taken = one.offered && !ours(here, &one.device, &mine)?;
        found.push(Newcomer {
            device: one.device,
            name: ask.name,
            at: ask.at,
            taken,
        });
    }
    Ok(found)
}

fn standing_ask(session: &Mutex<Session>, here: &Here, device: &str) -> Worked<Ask> {
    knock::ask_of(&*here.plain, &here.home, device)
        .map_err(up)?
        .filter(|ask| fresh(session, ask, here) && ask.holds())
        .ok_or_else(|| Refusal::of("admitGone"))
}

fn ours(here: &Here, device: &str, mine: &tisty_share_proto::VerifyingKey) -> Worked<bool> {
    Ok(knock::giver_of(&*here.plain, &here.home, device)
        .map_err(up)?
        .is_some_and(|one| one.key().as_ref() == Some(mine)))
}

// Read back after the put: on Drive the first writer is not sure to stand, so only the copy read tells who took it.
fn offer(session: &Mutex<Session>, here: &Here, key: Option<CloudKey>, device: &str) -> Worked<()> {
    key.ok_or_else(|| Refusal::of("admitNoKey"))?;
    standing_ask(session, here, device)?;
    let mine = signer(here).map_err(trouble)?.verifying_key();
    let giver = Giver::of(
        &here.device.0,
        &crate::handed::name_here(session, &here.device),
        &mine,
    );
    knock::offered(&*here.plain, &here.home, device, &giver).map_err(up)?;
    match ours(here, device, &mine)? {
        true => Ok(()),
        false => Err(Refusal::of("admitTaken")),
    }
}

// Its signing key came with an ask the person checked, so its history is let through without a second code.
fn vouch_for(session: &Mutex<Session>, ask: &Ask) -> Worked<()> {
    let key = ask.key().ok_or_else(|| Refusal::of("admitGone"))?;
    let who = DeviceId(ask.device.clone());
    // Under the lock, as when a machine is confirmed by hand, so a round never rewrites the waiting list meanwhile.
    let session = held(session);
    let data = session.paths.data();
    if !tisty_core::vouched::confirm(data, &who, &tisty_core::signing::shown_of(&key)) {
        return Err(trouble("the new computer's key could not be confirmed"));
    }
    tisty_carrier::turned::let_through(data, &ask.device);
    Ok(())
}

fn digits(said: &str) -> String {
    said.chars().filter(char::is_ascii_digit).collect()
}

// The person types what the new computer shows: an ask made up up there leaves nothing to type, and gets nothing.
fn granted(
    session: &Mutex<Session>,
    here: &Here,
    key: Option<CloudKey>,
    device: &str,
    typed: &str,
) -> Worked<()> {
    let key = key.ok_or_else(|| Refusal::of("admitNoKey"))?;
    let ask = standing_ask(session, here, device)?;
    let signer = signer(here).map_err(trouble)?;
    let mine = signer.verifying_key();
    if !ours(here, device, &mine)? {
        return Err(Refusal::of("admitTaken"));
    }
    let reveal = knock::reveal_of(&*here.plain, &here.home, device)
        .map_err(up)?
        .ok_or_else(|| Refusal::of("admitNotYet"))?;
    let expected = ask.code(&mine, &reveal).map(|code| digits(&code));
    if expected.is_none_or(|code| code != digits(typed)) {
        return Err(Refusal::of("admitWrongCode"));
    }
    vouch_for(session, &ask)?;
    let answer = crate::handed::sealed_for(session, here, &ask, &key, &signer).map_err(trouble)?;
    let standing = knock::answered(&*here.plain, &here.home, device).map_err(up)?;
    if standing.is_some_and(|one| one != answer) {
        knock::unanswered(&*here.plain, &here.home, device).map_err(up)?;
    }
    knock::answer(&*here.plain, &here.home, device, &answer).map_err(up)?;
    match knock::answered(&*here.plain, &here.home, device).map_err(up)? {
        Some(one) if one == answer => Ok(()),
        _ => Err(trouble("another answer stands where this one was put")),
    }
}

#[tauri::command(async)]
pub async fn admitting_ask(app: AppHandle) -> Answer<()> {
    off_the_lock(&app, ask).await
}

/// Asked again and again by the window while a new computer waits, since the person stands by both.
#[tauri::command(async)]
pub async fn admitting_shown(app: AppHandle) -> Answer<Admitted> {
    off_the_lock(&app, shown).await
}

#[tauri::command(async)]
pub async fn admitting_trust(app: AppHandle, code: String) -> Answer<()> {
    off_the_lock(&app, move |_, knocking, here, _| {
        trusted(knocking, here, &code)
    })
    .await
}

#[tauri::command(async)]
pub async fn admitting_cancel(app: AppHandle) -> Answer<()> {
    off_the_lock(&app, |_, knocking, here, _| cancelled(knocking, here)).await
}

#[tauri::command(async)]
pub async fn admitting_look(app: AppHandle) -> Answer<Vec<Newcomer>> {
    off_the_lock(&app, |session, _, here, key| looked(session, here, key)).await
}

#[tauri::command(async)]
pub async fn admitting_offer(app: AppHandle, device: String) -> Answer<()> {
    off_the_lock(&app, move |session, _, here, key| {
        offer(session, here, key, &device)
    })
    .await
}

#[tauri::command(async)]
pub async fn admitting_grant(app: AppHandle, device: String, code: String) -> Answer<()> {
    off_the_lock(&app, move |session, _, here, key| {
        granted(session, here, key, &device, &code)
    })
    .await
}

#[tauri::command(async)]
pub async fn admitting_refuse(app: AppHandle, device: String) -> Answer<()> {
    off_the_lock(&app, move |_, _, here, _| {
        knock::withdrawn(&*here.plain, &here.home, &device).map_err(up)
    })
    .await
}

#[cfg(test)]
#[path = "admitting_test.rs"]
mod tests;
