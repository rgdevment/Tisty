//! The channels this window can speak through, and the watch that feeds them.

use tauri::{Emitter, Manager};
use tisty_core::herald::{Channel, Happening, Heralds, Trouble};

/// A notification handed to the operating system, so it arrives with the window
/// closed, minimised or behind everything else.
pub struct Screen {
    app: tauri::AppHandle,
    words: Words,
}

#[derive(Clone)]
pub struct Words {
    pub due: String,
}

impl Channel for Screen {
    fn named(&self) -> &'static str {
        "screen"
    }

    fn wants(&self, what: &Happening) -> bool {
        on_screen(what)
    }

    fn tell(&self, what: &Happening) -> Result<(), Trouble> {
        use tauri_plugin_notification::NotificationExt;

        self.app
            .notification()
            .builder()
            .title(&self.words.due)
            .body(what.title().unwrap_or_default())
            .show()
            .map_err(|why| Trouble {
                channel: "screen",
                why: why.to_string(),
            })
    }
}

/// A short tone, played by the webview because the sound a desktop application
/// makes is not worth an audio stack in the bundle.
pub struct Chime {
    app: tauri::AppHandle,
}

impl Channel for Chime {
    fn named(&self) -> &'static str {
        "chime"
    }

    fn tell(&self, what: &Happening) -> Result<(), Trouble> {
        let Some(tone) = tone_for(what) else {
            return Ok(());
        };
        self.app.emit("chime", tone).map_err(|why| Trouble {
            channel: "chime",
            why: why.to_string(),
        })
    }
}

/// Filing is already answered by the strip under the field; a second, slower
/// copy from the system would only arrive after you had moved on.
fn on_screen(what: &Happening) -> bool {
    matches!(what, Happening::Due { .. })
}

fn tone_for(what: &Happening) -> Option<&'static str> {
    match what {
        Happening::Filed { .. } => Some("filed"),
        Happening::Due { .. } => Some("due"),
        Happening::Carried { .. } => None,
    }
}

pub fn heralds(app: &tauri::AppHandle, words: Words) -> Heralds {
    Heralds::default()
        .with(Box::new(Screen {
            app: app.clone(),
            words,
        }))
        .with(Box::new(Chime { app: app.clone() }))
}

const EVERY: std::time::Duration = std::time::Duration::from_secs(30);

/// Wakes on its own thread and tells whatever came due since the last look.
///
/// The mark moves even when nothing was owed, so a window left open overnight
/// does not hand the whole night to the lookback at the first reminder.
pub fn watch(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut since = jiff::Timestamp::now();
        loop {
            std::thread::sleep(EVERY);
            let now = jiff::Timestamp::now();
            for due in owed(&app, since, now) {
                told(&app, due);
            }
            since = now;
        }
    });
}

/// Nothing listening is not a failure, and neither is a channel that broke:
/// telling is never the point of the action that caused it.
pub fn told(app: &tauri::AppHandle, what: Happening) {
    if let Some(heralds) = app.try_state::<Heralds>() {
        heralds.tell(&what);
    }
}

/// The terminal writes the same store while the window is open, so a reminder
/// added there has to be picked up before it can be told.
fn owed(app: &tauri::AppHandle, since: jiff::Timestamp, now: jiff::Timestamp) -> Vec<Happening> {
    let Some(session) = app.try_state::<std::sync::Mutex<crate::Session>>() else {
        return Vec::new();
    };
    let Ok(mut session) = session.lock() else {
        return Vec::new();
    };
    let _ = session.reload();
    let here = jiff::tz::TimeZone::system();
    tisty_core::herald::owed(&session.state, since, now, &here)
        .into_iter()
        .map(|one| one.what)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn due() -> Happening {
        Happening::Due {
            title: "tomar la pastilla".into(),
            task: "01T".into(),
        }
    }

    fn filed() -> Happening {
        Happening::Filed {
            title: "comprar pan".into(),
        }
    }

    #[test]
    fn a_reminder_reaches_the_system_and_a_capture_does_not() {
        assert!(on_screen(&due()));
        assert!(!on_screen(&filed()));
    }

    #[test]
    fn both_of_them_sound_and_a_sync_stays_quiet() {
        assert_eq!(tone_for(&filed()), Some("filed"));
        assert_eq!(tone_for(&due()), Some("due"));
        assert_eq!(tone_for(&Happening::Carried { brought: 2 }), None);
    }
}
