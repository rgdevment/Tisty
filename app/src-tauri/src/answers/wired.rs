use crate::{Answer, Refusal, waking, wiring};

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
