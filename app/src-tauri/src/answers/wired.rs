use crate::{Answer, Refusal, waking, wiring};

#[tauri::command]
pub fn wiring() -> Vec<wiring::Seen> {
    wiring::seen()
}

#[tauri::command]
pub fn wire(id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::wire(&id).map_err(stuck)
}

#[tauri::command]
pub fn unwire(id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::unwire(&id).map_err(stuck)
}

fn stuck(why: wiring::Stuck) -> Refusal {
    match why {
        wiring::Stuck::NoSuch => Refusal::of("noSuchAgent"),
        wiring::Stuck::Puzzling(at) => Refusal::about("settingsPuzzling", at),
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
