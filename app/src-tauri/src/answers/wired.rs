use std::collections::BTreeSet;
use std::sync::Mutex;

use tisty_core::agents::Ledger;

use crate::wiring::agents::Homes;
use crate::{Answer, Refusal, Session, held, waking, wiring};

const PLUGGED_CLIENTS: [(&str, &str); 1] = [("claude-code", "claude")];

// The plugin's records are agent files, so they are read only after this computer's yes to Agents or to Tisty's skills.
fn plugged_here(session: &Mutex<Session>) -> BTreeSet<&'static str> {
    let data = held(session).paths.data().to_path_buf();
    let machine = tisty_core::machine::here().unwrap_or_default();
    plugged_with(&Ledger::open(&data, &machine), Homes::here)
}

pub(crate) fn plugged_with(
    ledger: &Ledger,
    homes: impl FnOnce() -> Option<Homes>,
) -> BTreeSet<&'static str> {
    match ledger.consented() || ledger.tisty_skills() {
        true => homes().map(|homes| homes.plugged()).unwrap_or_default(),
        false => BTreeSet::new(),
    }
}

// Only this computer's own folders hold the plugin Tisty reads; a WSL distro has its own.
pub(crate) fn marked(mut seen: Vec<wiring::Seen>, plugged: &BTreeSet<&str>) -> Vec<wiring::Seen> {
    for one in seen.iter_mut().filter(|one| one.place.is_none()) {
        let agent = PLUGGED_CLIENTS
            .iter()
            .find(|(client, _)| *client == one.id)
            .map(|(_, agent)| *agent);
        if let (Some(hooks), Some(agent)) = (one.hooks.as_mut(), agent) {
            hooks.plugged = plugged.contains(agent);
        }
    }
    seen
}

fn shown(session: &Mutex<Session>, seen: Vec<wiring::Seen>) -> Vec<wiring::Seen> {
    marked(seen, &plugged_here(session))
}

// It waits on the session while a round runs, so it never holds the window's thread.
#[tauri::command(async)]
pub fn wiring(session: tauri::State<'_, Mutex<Session>>) -> Vec<wiring::Seen> {
    shown(&session, wiring::seen())
}

#[tauri::command(async)]
pub fn wiring_known() -> Vec<wiring::Seen> {
    wiring::known()
}

#[tauri::command]
pub fn wsl_distros() -> Vec<String> {
    wiring::distros()
}

#[derive(serde::Serialize)]
pub struct InDistro {
    distro: wiring::Distro,
    seen: Vec<wiring::Seen>,
}

/// Reading a distro's files starts it, so it is looked into only when the person asks.
#[tauri::command(async)]
pub fn wsl_seen(name: String) -> Answer<InDistro> {
    let (distro, seen) =
        wiring::seen_in_distro(&name).ok_or_else(|| Refusal::about("wslUnread", name))?;
    Ok(InDistro { distro, seen })
}

#[tauri::command(async)]
pub fn wire(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::wire(&id)
        .map(|seen| shown(&session, seen))
        .map_err(stuck)
}

#[tauri::command(async)]
pub fn unwire(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::unwire(&id)
        .map(|seen| shown(&session, seen))
        .map_err(stuck)
}

#[tauri::command(async)]
pub fn hook(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::hook(&id)
        .map(|seen| shown(&session, seen))
        .map_err(stuck)
}

#[tauri::command(async)]
pub fn unhook(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::unhook(&id)
        .map(|seen| shown(&session, seen))
        .map_err(stuck)
}

fn stuck(why: wiring::Stuck) -> Refusal {
    match why {
        wiring::Stuck::NoSuch => Refusal::of("noSuchAgent"),
        wiring::Stuck::Shut => Refusal::of("wslShut"),
        wiring::Stuck::Puzzling(at) => Refusal::about("settingsPuzzling", at),
        wiring::Stuck::Unborn(at) => Refusal::about("settingsUnborn", at),
        wiring::Stuck::Cannot(why) => Refusal::about("cannotWrite", why),
    }
}

#[tauri::command]
pub fn waking() -> waking::Waking {
    waking::waking()
}

#[tauri::command]
pub fn wake_for(wanted: bool) -> Answer<waking::Waking> {
    waking::wake(wanted).map_err(|e| Refusal::about("cannotWrite", e.to_string()))
}

#[derive(Debug, PartialEq, Eq)]
pub enum WakeOffer {
    Not,
    Ask(waking::Waking),
    Settled,
}

pub fn wake_offer(
    asked: bool,
    reminded: bool,
    looked: impl FnOnce() -> waking::Waking,
) -> WakeOffer {
    if asked || !reminded {
        return WakeOffer::Not;
    }
    let now = looked();
    match (now.offered, now.wakes) {
        (true, false) if now.theirs => WakeOffer::Not,
        (true, false) => WakeOffer::Ask(now),
        (true, true) => WakeOffer::Settled,
        (false, _) => WakeOffer::Not,
    }
}

pub fn reminded(state: &tisty_core::State) -> bool {
    state
        .tasks
        .values()
        .any(|task| task.status == tisty_core::model::Status::Open && !task.reminders.is_empty())
}

// Asking the machine can wait on Windows, so it is done off the window's thread and without holding the session.
#[tauri::command(async)]
pub fn wake_due(session: tauri::State<'_, Mutex<Session>>) -> Answer<Option<waking::Waking>> {
    let (asked, reminded) = {
        let session = held(&session);
        (
            session.config.asked_to_wake == Some(true),
            reminded(&session.state),
        )
    };
    match wake_offer(asked, reminded, waking::waking) {
        WakeOffer::Ask(now) => Ok(Some(now)),
        WakeOffer::Settled => held(&session)
            .keep(|c| c.asked_to_wake = Some(true))
            .map(|()| None),
        WakeOffer::Not => Ok(None),
    }
}

#[tauri::command]
pub fn wake_asked(session: tauri::State<'_, Mutex<Session>>) -> Answer<()> {
    held(&session).keep(|c| c.asked_to_wake = Some(true))
}

#[cfg(test)]
#[path = "wired_test.rs"]
mod tests;
