//! The channels this window can speak through, and the watch that feeds them.

use tauri::{Emitter, Manager};
use tisty_core::herald::{Channel, Due, Happening, Heralds, Told, Trouble};

/// A notification handed to the operating system, so it arrives with the window
/// closed, minimised or behind everything else.
pub struct Screen {
    app: tauri::AppHandle,
    words: Words,
}

#[derive(Clone)]
pub struct Words {
    pub due: String,
    pub missed: String,
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

        let body = match what {
            Happening::Missed { count } => self.words.missed.replace("{n}", &count.to_string()),
            _ => what.title().unwrap_or_default().to_string(),
        };
        self.app
            .notification()
            .builder()
            .title(&self.words.due)
            .body(body)
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
    matches!(what, Happening::Due { .. } | Happening::Missed { .. })
}

fn tone_for(what: &Happening) -> Option<&'static str> {
    match what {
        Happening::Filed { .. } => Some("filed"),
        Happening::Due { .. } | Happening::Missed { .. } => Some("due"),
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
/// does not hand the whole night to the lookback at the first reminder — but
/// it never moves past something no channel could deliver.
pub fn watch(app: tauri::AppHandle, paths: tisty_core::Paths) {
    std::thread::spawn(move || {
        let mut watching = Watching::default();
        let mut since = jiff::Timestamp::now();
        loop {
            std::thread::sleep(EVERY);
            let now = jiff::Timestamp::now();
            let mut kept: Option<jiff::Timestamp> = None;

            let (owed, read) = watching.owed(&paths, since, now);
            // Grouped happenings lose their own timestamps, so the batch is
            // owed from its oldest — but only the batch that actually failed.
            let oldest = owed.iter().map(|one| one.at).min();
            for what in tisty_core::herald::gathered(owed) {
                if told(&app, what).lost() {
                    kept = match (kept, oldest) {
                        (Some(had), Some(at)) => Some(had.min(at)),
                        (had, at) => had.or(at),
                    };
                }
            }
            // Left just before the oldest one that failed, so the next round
            // picks it up again. The lookback still caps how long it is worth
            // retrying, which is what keeps this from running for ever.
            // A half-written segment — the shape of a sync still coming down —
            // leaves the projection frozen. Moving the mark then walks over
            // reminders that were never read, which is the very loss this
            // whole retry exists to close.
            since = if read { onward(now, kept) } else { since };
        }
    });
}

/// The watch keeps a projection of its own and never touches the session lock.
///
/// It used to `reload()` with the lock held, and a reprojection of a long log
/// is not cheap: Tauri's synchronous commands run on the main thread, so
/// `snapshot`, `capture`, `complete` and the close handler all waited on it.
#[derive(Default)]
struct Watching {
    print: String,
    state: tisty_core::State,
}

impl Watching {
    /// The second half of the answer is whether the store could be read at all.
    fn owed(
        &mut self,
        paths: &tisty_core::Paths,
        since: jiff::Timestamp,
        now: jiff::Timestamp,
    ) -> (Vec<Due>, bool) {
        // Cheap: it only stats the segment files. The replay below happens at
        // most once per tick, and never with anyone waiting on it.
        let print = tisty_core::cache::fingerprint(&paths.store());
        let mut read = true;
        if print != self.print {
            match tisty_core::store::read_all(paths.store()) {
                // Deliberately not `cache::project`: that writes the shared
                // cache, and the window is the only thing that should.
                Ok(events) => {
                    self.state = tisty_core::State::replay(&events);
                    self.print = print;
                }
                Err(_) => read = false,
            }
        }
        let owed = tisty_core::herald::owed(&self.state, since, now, &jiff::tz::TimeZone::system());
        (owed, read)
    }
}

/// Where the next round starts: `now`, unless something could not be delivered
/// — then just before the oldest of those, so it is picked up again.
fn onward(now: jiff::Timestamp, kept: Option<jiff::Timestamp>) -> jiff::Timestamp {
    kept.map_or(now, |at| at - jiff::SignedDuration::from_secs(1))
}

/// Nothing listening is not a failure; every channel failing is, and the caller
/// decides what to do about it.
pub fn told(app: &tauri::AppHandle, what: Happening) -> Told {
    match app.try_state::<Heralds>() {
        Some(heralds) => heralds.tell(&what),
        None => Told::default(),
    }
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

    fn moment(secs: i64) -> jiff::Timestamp {
        jiff::Timestamp::from_second(secs).unwrap()
    }

    /// Nothing failed: the mark moves, so a window open all night does not hand
    /// the whole night to the lookback at the first reminder of the morning.
    #[test]
    fn the_mark_moves_when_everything_was_delivered() {
        assert_eq!(onward(moment(1000), None), moment(1000));
    }

    /// A reminder no channel could deliver is not written off as said.
    #[test]
    fn the_mark_waits_behind_what_could_not_be_told() {
        let kept = moment(940);

        let next = onward(moment(1000), Some(kept));

        assert!(next < kept, "the failed reminder would never come up again");
    }

    #[test]
    fn several_failures_wait_behind_the_oldest() {
        let oldest = moment(900);
        let newer = moment(980);

        let kept = [newer, oldest].into_iter().reduce(|a, b| a.min(b));

        assert!(onward(moment(1000), kept) < oldest);
    }

    /// A lid closed at ten and opened at eight owes a dozen at once.
    #[test]
    fn a_nights_worth_arrives_as_one_line() {
        let owed: Vec<Due> = (0..12)
            .map(|n| Due {
                at: jiff::Timestamp::from_second(1000 + n).unwrap(),
                what: due(),
            })
            .collect();

        let said = tisty_core::herald::gathered(owed);

        assert_eq!(said.len(), 1);
        assert!(matches!(said[0], Happening::Missed { count: 12 }));
    }

    /// Two or three still deserve their own titles.
    #[test]
    fn a_few_are_still_told_one_by_one() {
        let owed: Vec<Due> = (0..3)
            .map(|n| Due {
                at: jiff::Timestamp::from_second(1000 + n).unwrap(),
                what: due(),
            })
            .collect();

        assert_eq!(tisty_core::herald::gathered(owed).len(), 3);
    }

    #[test]
    fn the_gathered_line_still_reaches_the_system_and_still_sounds() {
        let many = Happening::Missed { count: 9 };

        assert!(on_screen(&many));
        assert_eq!(tone_for(&many), Some("due"));
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
