use std::sync::Mutex;

use crate::{Answer, Refusal, Session, held, waking, wiring};

#[tauri::command]
pub fn wiring() -> Vec<wiring::Seen> {
    wiring::seen()
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
pub fn wire(id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::wire(&id).map_err(stuck)
}

#[tauri::command(async)]
pub fn unwire(id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::unwire(&id).map_err(stuck)
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

#[tauri::command]
pub fn wake_due(session: tauri::State<'_, Mutex<Session>>) -> Answer<Option<waking::Waking>> {
    let mut session = held(&session);
    let asked = session.config.asked_to_wake == Some(true);
    match wake_offer(asked, reminded(&session.state), waking::waking) {
        WakeOffer::Ask(now) => Ok(Some(now)),
        WakeOffer::Settled => session
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
