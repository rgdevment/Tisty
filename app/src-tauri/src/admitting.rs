use std::sync::{Mutex, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tisty_carrier::knock;
use tisty_core::event::DeviceId;
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::handing::{Ask, Giver, handed};

use crate::handed::{Here, Knocking, NEW_FOR, fits, here_now, hitched, now, seated, signer};
use crate::{Answer, Refusal, Session, elsewhere, held};

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

type Worked<T> = Result<T, String>;

fn refused(why: String) -> Refusal {
    tisty_core::witness::note(
        tisty_core::witness::channel::CLOUD,
        "a new computer could not be let into the sealed cloud",
        &[("why", tisty_core::witness::Fact::Why(why))],
    );
    Refusal::of("admitNotNow")
}

async fn off_the_lock<T: Send + 'static>(
    app: &AppHandle,
    work: impl FnOnce(&Mutex<Session>, &Knocking, &Here, Option<CloudKey>) -> Worked<T> + Send + 'static,
) -> Answer<T> {
    let app = app.clone();
    elsewhere(move || {
        let (session, knocking) = (app.state::<Mutex<Session>>(), app.state::<Knocking>());
        let (here, key) = here_now(&session, &knocking)?;
        work(&session, &knocking, &here, key)
    })
    .await?
    .map_err(refused)
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

fn asked_now(knocking: &Knocking, here: &Here) -> Worked<Option<Ask>> {
    let asked = knocking.asked.lock().map_err(|e| e.to_string())?;
    Ok(asked
        .as_ref()
        .map(|(ask, _)| ask.clone())
        .filter(|ask| ask.cloud == here.cloud))
}

fn ask(
    session: &Mutex<Session>,
    knocking: &Knocking,
    here: &Here,
    key: Option<CloudKey>,
) -> Worked<()> {
    if key.is_some() {
        return Err("this computer already reads the sealed cloud".into());
    }
    trust(knocking, None);
    let mut asked = knocking.asked.lock().map_err(|e| e.to_string())?;
    crate::handed::ask_now(session, &mut asked, here)
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
    let Some(ask) = asked_now(knocking, here)? else {
        return Ok(said("idle", None, None));
    };
    let Some(giver) = knock::giver_of(&*here.plain, &here.home, &here.device.0).map_err(hitched)?
    else {
        return Ok(said("waiting", None, None));
    };
    let code = giver.key().and_then(|key| ask.code(&key));
    if trusted_now(knocking).as_deref() != Some(giver.key.as_str()) {
        return Ok(said("compare", Some(&giver), code));
    }
    match crate::handed::asking(session, knocking, here)? {
        true => Ok(said("done", Some(&giver), None)),
        false => Ok(said("trusted", Some(&giver), code)),
    }
}

// The code travels back from the window, so a giver that changed since it was shown is never trusted.
fn trusted(knocking: &Knocking, here: &Here, code: &str) -> Worked<()> {
    let ask = asked_now(knocking, here)?.ok_or("this computer asked for nothing")?;
    let giver = knock::giver_of(&*here.plain, &here.home, &here.device.0)
        .map_err(hitched)?
        .ok_or("no computer took the ask")?;
    let key = giver
        .key()
        .ok_or("the computer that took the ask names no key")?;
    if ask.code(&key).as_deref() != Some(code) {
        return Err("the code changed since it was shown".into());
    }
    trust(knocking, Some(giver.key));
    Ok(())
}

fn cancelled(knocking: &Knocking, here: &Here) -> Worked<()> {
    *knocking.asked.lock().map_err(|e| e.to_string())? = None;
    trust(knocking, None);
    knock::withdrawn(&*here.plain, &here.home, &here.device.0).map_err(hitched)
}

fn fresh(session: &Mutex<Session>, ask: &Ask, here: &Here) -> bool {
    !seated(session, &ask.device)
        && fits(ask, here)
        && (-NEW_FOR..=NEW_FOR).contains(&(now() - ask.at))
}

fn looked(session: &Mutex<Session>, here: &Here, key: Option<CloudKey>) -> Worked<Vec<Newcomer>> {
    key.ok_or("this computer cannot open the sealed cloud")?;
    Ok(knock::knocks(&*here.plain, &here.home)
        .map_err(hitched)?
        .into_iter()
        .filter_map(|one| {
            let ask = one.ask.filter(|ask| fresh(session, ask, here))?;
            Some(Newcomer {
                device: one.device,
                name: ask.name,
                at: ask.at,
                taken: one.offered,
            })
        })
        .collect())
}

fn standing_ask(session: &Mutex<Session>, here: &Here, device: &str) -> Worked<Ask> {
    knock::ask_of(&*here.plain, &here.home, device)
        .map_err(hitched)?
        .filter(|ask| fresh(session, ask, here) && ask.holds())
        .ok_or_else(|| "that computer no longer asks".into())
}

fn ours(here: &Here, device: &str, mine: &tisty_share_proto::VerifyingKey) -> Worked<bool> {
    Ok(knock::giver_of(&*here.plain, &here.home, device)
        .map_err(hitched)?
        .is_some_and(|one| one.key().as_ref() == Some(mine)))
}

fn offer(
    session: &Mutex<Session>,
    here: &Here,
    key: Option<CloudKey>,
    device: &str,
) -> Worked<String> {
    key.ok_or("this computer cannot open the sealed cloud")?;
    let ask = standing_ask(session, here, device)?;
    let mine = signer(here)?.verifying_key();
    let giver = Giver::of(
        &here.device.0,
        &crate::handed::name_here(session, &here.device),
        &mine,
    );
    let taken = knock::offered(&*here.plain, &here.home, device, &giver).map_err(hitched)?;
    if !taken && !ours(here, device, &mine)? {
        return Err("another computer is letting it in".into());
    }
    ask.code(&mine)
        .ok_or_else(|| "no code could be made".into())
}

// Its signing key came with an ask the person checked, so its history is let through without a second code.
fn vouch_for(session: &Mutex<Session>, ask: &Ask) -> Worked<()> {
    let key = ask.key().ok_or("the ask names no key")?;
    let who = DeviceId(ask.device.clone());
    let data = held(session).paths.data().to_path_buf();
    if !tisty_core::vouched::confirm(&data, &who, &tisty_core::signing::shown_of(&key)) {
        return Err("the new computer's key could not be confirmed".into());
    }
    tisty_carrier::turned::let_through(&data, &ask.device);
    Ok(())
}

fn granted(
    session: &Mutex<Session>,
    here: &Here,
    key: Option<CloudKey>,
    device: &str,
) -> Worked<()> {
    let key = key.ok_or("this computer cannot open the sealed cloud")?;
    let ask = standing_ask(session, here, device)?;
    let signer = signer(here)?;
    if !ours(here, device, &signer.verifying_key())? {
        return Err("this computer is not the one letting it in".into());
    }
    let answer = handed(&ask, &key, &here.device.0, &signer)
        .ok_or("the cloud's key could not be sealed for the new computer")?;
    knock::answer(&*here.plain, &here.home, device, &answer).map_err(hitched)?;
    vouch_for(session, &ask)
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
pub async fn admitting_offer(app: AppHandle, device: String) -> Answer<String> {
    off_the_lock(&app, move |session, _, here, key| {
        offer(session, here, key, &device)
    })
    .await
}

#[tauri::command(async)]
pub async fn admitting_grant(app: AppHandle, device: String) -> Answer<()> {
    off_the_lock(&app, move |session, _, here, key| {
        granted(session, here, key, &device)
    })
    .await
}

#[tauri::command(async)]
pub async fn admitting_refuse(app: AppHandle, device: String) -> Answer<()> {
    off_the_lock(&app, move |_, _, here, _| {
        knock::withdrawn(&*here.plain, &here.home, &device).map_err(hitched)
    })
    .await
}

#[cfg(test)]
#[path = "admitting_test.rs"]
mod tests;
