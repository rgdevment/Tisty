use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter, Manager};

use crate::Session;

const LISTENS: Duration = Duration::from_secs(60);
const AGAIN: Duration = Duration::from_secs(60);
const RENEWED: Duration = Duration::from_secs(10 * 60);
const QUICKEST: Duration = Duration::from_secs(1);

// A provider that can say the other side changed is listened to, so the window brings it in at once instead of at its next glance.
pub fn listen(app: AppHandle) {
    std::thread::spawn(move || {
        loop {
            match heard_from(&app) {
                Some((mut heard, chosen)) => {
                    let began = Instant::now();
                    while began.elapsed() < RENEWED && chosen_now(&app) == chosen {
                        let asked = Instant::now();
                        match heard.told(LISTENS) {
                            Ok(true) => {
                                let _ = app.emit("told", ());
                            }
                            Ok(false) => {}
                            Err(_) => {
                                std::thread::sleep(AGAIN);
                                break;
                            }
                        }
                        if let Some(rest) = QUICKEST.checked_sub(asked.elapsed()) {
                            std::thread::sleep(rest);
                        }
                    }
                }
                None => std::thread::sleep(AGAIN),
            }
        }
    });
}

type Chosen = Option<tisty_core::config::Sync>;

fn chosen_now(app: &AppHandle) -> Chosen {
    app.try_state::<Mutex<Session>>()
        .and_then(|session| session.lock().ok().and_then(|one| one.config.sync.clone()))
}

// Heard together with the way it was chosen, so a listen ends the moment the person picks another.
fn heard_from(app: &AppHandle) -> Option<(Box<dyn tisty_carrier::Watch>, Chosen)> {
    let (carrier, chosen) = {
        let session = app.try_state::<Mutex<Session>>()?;
        let session = session.lock().ok()?;
        (session.carrier()?, session.config.sync.clone())
    };
    Some((carrier.hears()?, chosen))
}
