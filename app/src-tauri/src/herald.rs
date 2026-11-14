use tauri::{Emitter, Manager};
use tisty_core::herald::{Channel, Due, Happening, Heralds, Told, Trouble};
use tisty_core::witness::{self, Fact, channel};

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

fn on_screen(what: &Happening) -> bool {
    matches!(what, Happening::Due { .. } | Happening::Missed { .. })
}

fn tone_for(what: &Happening) -> Option<&'static str> {
    match what {
        Happening::Filed { .. } => Some("filed"),
        Happening::Done { .. } => Some("done"),
        Happening::Due { .. } | Happening::Missed { .. } => Some("due"),
        Happening::Carried { .. } => None,
    }
}

#[derive(Clone, Default)]
pub struct Nudge(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl Nudge {
    pub fn give(&self) {
        self.0.store(true, std::sync::atomic::Ordering::Release);
    }

    fn taken(&self) -> bool {
        self.0.swap(false, std::sync::atomic::Ordering::AcqRel)
    }
}

pub struct Speaking {
    words: Words,
    now: std::sync::Mutex<Heralds>,
    nudge: Nudge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Voices {
    pub shown: bool,
    pub heard: bool,
}

impl Speaking {
    pub fn new(app: &tauri::AppHandle, words: Words, quiet: &[String], nudge: Nudge) -> Self {
        Self {
            now: std::sync::Mutex::new(built(app, &words, quiet)),
            words,
            nudge,
        }
    }

    fn tell(&self, what: &Happening) -> Told {
        match self.now.lock() {
            Ok(heralds) => heralds.tell(what),
            Err(held) => held.into_inner().tell(what),
        }
    }

    fn voices(&self) -> Voices {
        let names = match self.now.lock() {
            Ok(heralds) => heralds.names(),
            Err(held) => held.into_inner().names(),
        };
        Voices {
            shown: names.contains(&"screen"),
            heard: names.contains(&"chime"),
        }
    }
}

pub fn respeak(app: &tauri::AppHandle, quiet: &[String]) {
    let Some(speaking) = app.try_state::<Speaking>() else {
        return;
    };
    let fresh = built(app, &speaking.words, quiet);
    match speaking.now.lock() {
        Ok(mut now) => *now = fresh,
        Err(held) => *held.into_inner() = fresh,
    }
    speaking.nudge.give();
}

fn built(app: &tauri::AppHandle, words: &Words, quiet: &[String]) -> Heralds {
    let mut heralds = Heralds::default();
    let screen = Screen {
        app: app.clone(),
        words: words.clone(),
    };
    if speaks(screen.named(), quiet) {
        heralds = heralds.with(Box::new(screen));
    }
    let chime = Chime { app: app.clone() };
    if speaks(chime.named(), quiet) {
        heralds = heralds.with(Box::new(chime));
    }
    heralds
}

fn speaks(channel: &str, quiet: &[String]) -> bool {
    !quiet.iter().any(|one| one == channel)
}

const EVERY: std::time::Duration = std::time::Duration::from_secs(30);
// An agent's bell is a hint, so the round above still runs whether it rings or not.
const LISTENS: std::time::Duration = std::time::Duration::from_secs(1);

pub fn watch(app: tauri::AppHandle, paths: tisty_core::Paths, nudge: Nudge) {
    std::thread::spawn(move || {
        let mut watching = Watching::default();
        let mut since = jiff::Timestamp::now();
        let mut heard = tisty_core::stirring::rung(&paths);
        let mut round = std::time::Instant::now();
        loop {
            std::thread::sleep(LISTENS);
            let bell = tisty_core::stirring::rung(&paths);
            let nudged = nudge.taken();
            if bell == heard && round.elapsed() < EVERY {
                if nudged {
                    survived(
                        || {
                            watching.reread(&paths);
                            booked(&app, &watching.state, jiff::Timestamp::now());
                        },
                        "the watch could not book what the window changed",
                    );
                }
                continue;
            }
            heard = bell;
            round = std::time::Instant::now();
            let now = jiff::Timestamp::now();
            let mut kept: Option<jiff::Timestamp> = None;

            let Some((mut owed, read, stirred)) = survived(
                || {
                    let found = watching.owed(&paths, since, now);
                    booked(&app, &watching.state, now);
                    found
                },
                "the watch could not read what was owed",
            ) else {
                continue;
            };
            if stirred {
                let _ = app.emit("stirred", ());
            }
            if let Some(booking) = app.try_state::<crate::booking::Booking>() {
                owed.retain(|one| !booking.holds(one, now));
            }
            let oldest = owed.iter().map(|one| one.at).min();
            for what in tisty_core::herald::gathered(owed) {
                if told(&app, what).lost() {
                    kept = match (kept, oldest) {
                        (Some(had), Some(at)) => Some(had.min(at)),
                        (had, at) => had.or(at),
                    };
                }
            }
            since = if read { onward(now, kept) } else { since };
        }
    });
}

fn survived<T>(work: impl FnOnce() -> T, said: &'static str) -> Option<T> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {
        Ok(done) => Some(done),
        Err(_) => {
            witness::error(channel::HERALD, said, &[]);
            None
        }
    }
}

#[derive(Default)]
struct Watching {
    print: String,
    papers: String,
    state: tisty_core::State,
    looked: bool,
}

impl Watching {
    fn owed(
        &mut self,
        paths: &tisty_core::Paths,
        since: jiff::Timestamp,
        now: jiff::Timestamp,
    ) -> (Vec<Due>, bool, bool) {
        let print = tisty_core::cache::fingerprint(&paths.store());
        let mut read = true;
        let mut stirred = false;
        if print != self.print {
            // Reminders need titles and times, not bodies; the window keeps the cache current.
            match tisty_core::cache::summarised(&paths.store(), paths.cache()) {
                Ok(state) => {
                    self.state = state;
                    stirred = self.looked;
                    self.print = print;
                }
                Err(_) => read = false,
            }
        }
        let papers = tisty_core::docs::print(&paths.docs());
        if papers != self.papers {
            stirred = stirred || self.looked;
            self.papers = papers;
        }
        self.looked = true;
        let owed = tisty_core::herald::owed(&self.state, since, now, &jiff::tz::TimeZone::system());
        (owed, read, stirred)
    }

    // The window's own write may leave the print where it was, so its reminders are read again without waking the window.
    fn reread(&mut self, paths: &tisty_core::Paths) {
        if let Ok(state) = tisty_core::cache::summarised(&paths.store(), paths.cache()) {
            self.state = state;
        }
    }
}

fn booked<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    state: &tisty_core::State,
    now: jiff::Timestamp,
) {
    let (Some(booking), Some(speaking)) = (
        app.try_state::<crate::booking::Booking>(),
        app.try_state::<Speaking>(),
    ) else {
        return;
    };
    if !booking.books() {
        return;
    }
    let voices = speaking.voices();
    let wanted = match voices.shown {
        true => crate::booking::rings(
            tisty_core::herald::coming(state, now, &jiff::tz::TimeZone::system(), booking.most()),
            &speaking.words.due,
            voices.heard,
        ),
        false => Vec::new(),
    };
    booking.keep(&wanted, now);
}

pub fn book_before_leaving<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let Some(session) = app.try_state::<std::sync::Mutex<crate::Session>>() else {
        return;
    };
    let session = crate::held(&session);
    survived(
        || booked(app, &session.state, jiff::Timestamp::now()),
        "the reminders could not be booked on the way out",
    );
}

fn onward(now: jiff::Timestamp, kept: Option<jiff::Timestamp>) -> jiff::Timestamp {
    kept.map_or(now, |at| at - jiff::SignedDuration::from_secs(1))
}

pub fn told(app: &tauri::AppHandle, what: Happening) -> Told {
    let Some(speaking) = app.try_state::<Speaking>() else {
        return Told::default();
    };
    let told = speaking.tell(&what);
    if told.lost() {
        let why: Vec<String> = told
            .trouble
            .iter()
            .map(|one| format!("{}: {}", one.channel, one.why))
            .collect();
        witness::warn(
            channel::HERALD,
            "no channel could deliver",
            &[
                ("asked", Fact::Count(told.asked)),
                ("why", Fact::Why(why.join("; "))),
            ],
        );
    }
    told
}

#[cfg(test)]
#[path = "herald_test.rs"]
mod tests;
