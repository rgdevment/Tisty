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

pub struct Speaking {
    words: Words,
    now: std::sync::Mutex<Heralds>,
}

impl Speaking {
    pub fn new(app: &tauri::AppHandle, words: Words, quiet: &[String]) -> Self {
        Self {
            now: std::sync::Mutex::new(built(app, &words, quiet)),
            words,
        }
    }

    fn tell(&self, what: &Happening) -> Told {
        match self.now.lock() {
            Ok(heralds) => heralds.tell(what),
            Err(held) => held.into_inner().tell(what),
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

pub fn watch(app: tauri::AppHandle, paths: tisty_core::Paths) {
    std::thread::spawn(move || {
        let mut watching = Watching::default();
        let mut since = jiff::Timestamp::now();
        loop {
            std::thread::sleep(EVERY);
            let now = jiff::Timestamp::now();
            let mut kept: Option<jiff::Timestamp> = None;

            let Some((owed, read)) = survived(
                || watching.owed(&paths, since, now),
                "the watch could not read what was owed",
            ) else {
                continue;
            };
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
    state: tisty_core::State,
}

impl Watching {
    fn owed(
        &mut self,
        paths: &tisty_core::Paths,
        since: jiff::Timestamp,
        now: jiff::Timestamp,
    ) -> (Vec<Due>, bool) {
        let print = tisty_core::cache::fingerprint(&paths.store());
        let mut read = true;
        if print != self.print {
            match tisty_core::store::read_all(paths.store()) {
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

    #[test]
    fn the_mark_moves_when_everything_was_delivered() {
        assert_eq!(onward(moment(1000), None), moment(1000));
    }

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
    fn a_muted_channel_is_not_registered() {
        assert_eq!(would_speak(&[]), vec!["screen", "chime"]);
        assert_eq!(would_speak(&["screen".to_string()]), vec!["chime"]);
        assert_eq!(would_speak(&["chime".to_string()]), vec!["screen"]);
        assert!(would_speak(&["screen".to_string(), "chime".to_string()]).is_empty());
    }

    #[test]
    fn a_channel_nobody_has_heard_of_mutes_nothing() {
        assert_eq!(
            would_speak(&["telegram".to_string()]),
            vec!["screen", "chime"]
        );
    }

    fn would_speak(quiet: &[String]) -> Vec<&'static str> {
        ["screen", "chime"]
            .into_iter()
            .filter(|one| speaks(one, quiet))
            .collect()
    }

    #[test]
    fn a_reminder_reaches_the_system_and_a_capture_does_not() {
        assert!(on_screen(&due()));
        assert!(!on_screen(&filed()));
    }

    fn finished() -> Happening {
        Happening::Done {
            title: "regar las plantas".into(),
        }
    }

    #[test]
    fn finishing_something_sounds_like_finishing_and_not_like_filing() {
        assert_eq!(tone_for(&finished()), Some("done"));
        assert_ne!(tone_for(&finished()), tone_for(&filed()));
    }

    #[test]
    fn finishing_something_never_interrupts_with_a_system_notice() {
        assert!(
            !on_screen(&finished()),
            "avisar en pantalla de algo que acabas de hacer es ruido"
        );
    }

    #[test]
    fn both_of_them_sound_and_a_sync_stays_quiet() {
        assert_eq!(tone_for(&filed()), Some("filed"));
        assert_eq!(tone_for(&due()), Some("due"));
        assert_eq!(tone_for(&Happening::Carried { brought: 2 }), None);
    }

    static ALONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn quietly<T>(work: impl FnOnce() -> T) -> T {
        let loud = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let done = work();
        std::panic::set_hook(loud);
        done
    }

    #[test]
    fn a_round_that_panics_does_not_take_the_watch_with_it() {
        let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().unwrap();
        let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        witness::keeps(witness::file(&paths), false);

        let broke = quietly(|| survived(|| panic!("the store went sideways"), "the watch broke"));

        assert!(broke.is_none());
        let seen = witness::recent(&paths, 10);
        assert!(
            seen.iter().any(|line| line.contains("the watch broke")),
            "{seen:?}"
        );
        assert!(
            !seen.iter().any(|line| line.contains("sideways")),
            "the panic message reached the file: {seen:?}"
        );
    }

    #[test]
    fn a_round_that_finishes_hands_back_what_it_found() {
        assert_eq!(survived(|| 41 + 1, "unused"), Some(42));
    }
}
