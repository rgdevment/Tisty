use super::*;

fn wrote(paths: &tisty_core::Paths, title: &str) {
    let mut store = tisty_core::Store::open(
        paths.store(),
        tisty_core::DeviceId("dev_someone_else".into()),
    )
    .unwrap();
    store
        .append(tisty_core::Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::TaskAdd::new(title, "a0"),
        })
        .unwrap();
}

#[test]
fn the_watch_speaks_up_when_the_store_moved_under_it() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let now = jiff::Timestamp::now();
    let mut watching = Watching::default();

    // The store already holds history on the first look, as it does on any real start.
    // Against an empty one the branch under test never runs and this passes for free.
    wrote(&paths, "what was already here");

    let (_, _, first) = watching.owed(&paths, now, now);
    wrote(&paths, "what the agent filed");
    let (_, _, after) = watching.owed(&paths, now, now);
    let (_, _, again) = watching.owed(&paths, now, now);

    assert!(
        !first,
        "the first look is not news, it is the starting point"
    );
    assert!(
        after,
        "someone wrote beside the window and it has to be told"
    );
    assert!(
        !again,
        "nothing changed the second time, so nobody is woken"
    );
}

#[test]
fn the_watch_speaks_up_when_a_document_grew_without_writing_an_event() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let now = jiff::Timestamp::now();
    let mut watching = Watching::default();
    let papers = paths.docs();
    wrote(&paths, "what was already here");
    tisty_core::docs::write(&papers, "dev0-0001", "# Minuta\n").unwrap();

    let (_, _, first) = watching.owed(&paths, now, now);
    tisty_core::docs::append(
        &papers,
        &tisty_core::history::Writer::now(&papers, None),
        "dev0-0001",
        "Un punto mas.",
    )
    .unwrap();
    let (_, _, after) = watching.owed(&paths, now, now);
    let (_, _, again) = watching.owed(&paths, now, now);

    assert!(!first, "the first look is the starting point");
    assert!(
        after,
        "the log did not move, but the document did and the window has to be told"
    );
    assert!(!again, "nothing changed the second time");
}

#[test]
fn the_first_write_after_the_window_opens_is_news() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let now = jiff::Timestamp::now();
    let mut watching = Watching::default();
    wrote(&paths, "what was already here");

    assert!(watching.reread(&paths), "a readable store is read");
    wrote(&paths, "what the agent filed first");
    let (_, _, after) = watching.owed(&paths, now, now);
    let (_, _, again) = watching.owed(&paths, now, now);

    assert!(
        after,
        "the agent's first write is news, not the starting point"
    );
    assert!(!again, "nothing moved since it was told");
}

#[test]
fn a_window_that_opened_with_nothing_new_wakes_nobody() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let now = jiff::Timestamp::now();
    let mut watching = Watching::default();
    wrote(&paths, "what was already here");

    watching.reread(&paths);
    let (_, _, first) = watching.owed(&paths, now, now);

    assert!(!first, "nothing moved since the window opened");
}

#[test]
fn the_first_write_into_an_empty_store_is_news() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let now = jiff::Timestamp::now();
    let mut watching = Watching::default();

    watching.reread(&paths);
    wrote(&paths, "the very first task");
    let (_, _, after) = watching.owed(&paths, now, now);

    assert!(after, "a store that was empty when the window opened grew");
}

#[test]
fn a_write_beside_the_windows_own_is_still_news_after_its_nudge() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let now = jiff::Timestamp::now();
    let mut watching = Watching::default();
    wrote(&paths, "what was already here");

    watching.reread(&paths);
    wrote(&paths, "what the agent filed beside the window");
    assert!(
        watching.reread(&paths),
        "the nudge reads the reminders again"
    );
    let (_, _, after) = watching.owed(&paths, now, now);

    assert!(
        after,
        "a nudge for the window's own write must not swallow the agent's"
    );
}

#[test]
fn the_first_document_that_grows_after_the_window_opens_is_news() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let now = jiff::Timestamp::now();
    let mut watching = Watching::default();
    let papers = paths.docs();
    wrote(&paths, "what was already here");
    tisty_core::docs::write(&papers, "dev0-0001", "# Minuta\n").unwrap();

    watching.reread(&paths);
    tisty_core::docs::append(
        &papers,
        &tisty_core::history::Writer::now(&papers, None),
        "dev0-0001",
        "Un punto mas.",
    )
    .unwrap();
    let (_, _, after) = watching.owed(&paths, now, now);

    assert!(after, "the document grew after the window opened");
}

#[test]
fn a_store_that_cannot_be_read_at_first_still_speaks_once_it_can() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let now = jiff::Timestamp::now();
    let mut watching = Watching::default();
    std::fs::create_dir_all(paths.data()).unwrap();
    std::fs::write(paths.store(), "a file where the store goes").unwrap();

    let read = watching.reread(&paths);
    std::fs::remove_file(paths.store()).unwrap();
    wrote(&paths, "what the agent filed");
    let (_, _, after) = watching.owed(&paths, now, now);

    assert!(!read, "the first look could not read the store");
    assert!(after, "the window is told rather than left behind");
}

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

    let broke = quietly(|| {
        survived(
            || panic!("the store went sideways"),
            channel::HERALD,
            "the watch broke",
            &[],
        )
    });

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
    assert_eq!(
        survived(|| 41 + 1, channel::HERALD, "unused", &[]),
        Some(42)
    );
}

#[test]
fn a_reminder_the_window_sets_is_read_again_for_the_system_at_once() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let mut session = crate::Session::at(paths.clone()).unwrap();
    let mut watching = Watching::default();
    assert!(watching.reread(&paths));
    let mut add = tisty_core::event::TaskAdd::new("comprar pan", "a0");
    add.reminders = vec![tisty_core::DateSpec::floating(
        "2026-08-12T09:00:00".parse().unwrap(),
        "America/Santiago",
    )];

    session
        .commit(tisty_core::Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: add,
        })
        .unwrap();

    assert!(session.nudge.taken(), "the commit nudges the watch");
    assert!(watching.reread(&paths));
    let coming = tisty_core::herald::coming(
        &watching.state,
        "2026-08-11T12:00:00Z".parse().unwrap(),
        &jiff::tz::TimeZone::get("America/Santiago").unwrap(),
        64,
    );
    assert_eq!(coming.len(), 1, "{coming:?}");
}

#[test]
fn what_the_system_holds_leaves_the_window_before_it_is_gathered() {
    struct Takes;
    impl crate::booking::Book for Takes {
        fn shows(&self) -> bool {
            true
        }
        fn pending(&self) -> Option<Vec<String>> {
            Some(Vec::new())
        }
        fn cancel(&self, _keys: &[String]) {}
        fn add(&self, _ring: &crate::booking::Ring) -> bool {
            true
        }
    }
    let at: jiff::Timestamp = "2026-08-12T13:00:00Z".parse().unwrap();
    let due = |task: &str| Due {
        at,
        what: Happening::Due {
            title: "comprar pan".into(),
            task: task.into(),
        },
    };
    let owed: Vec<Due> = ["A", "B", "C", "D"].into_iter().map(due).collect();
    let booking = crate::booking::Booking::new(Some(Box::new(Takes)), 64);
    let held = crate::booking::rings(owed[..3].to_vec(), "Recordatorio", true, false);
    booking.keep(&held, "2026-08-12T10:00:00Z".parse().unwrap());
    let now = "2026-08-12T13:00:20Z".parse().unwrap();

    let left = unheld(owed.clone(), Some(&booking), now);

    assert_eq!(left, vec![due("D")]);
    assert_eq!(
        tisty_core::herald::gathered(left),
        vec![due("D").what],
        "one left rings by name, never as a missed four"
    );
    assert_eq!(unheld(owed.clone(), None, now), owed);
}
