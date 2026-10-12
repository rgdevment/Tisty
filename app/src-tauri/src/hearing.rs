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
                Some(mut heard) => {
                    let began = Instant::now();
                    while began.elapsed() < RENEWED {
                        let asked = Instant::now();
                        match heard.told(LISTENS) {
                            Ok(true) => {
                                let _ = app.emit("told", ());
                            }
                            Ok(false) => {}
                            Err(_) => break,
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

fn heard_from(app: &AppHandle) -> Option<Box<dyn tisty_carrier::Watch>> {
    let carrier = app.try_state::<Mutex<Session>>()?.lock().ok()?.carrier()?;
    carrier.hears()
}
