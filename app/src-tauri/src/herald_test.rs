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
    tisty_core::docs::append(&papers, "dev0-0001", "Un punto mas.").unwrap();
    let (_, _, after) = watching.owed(&paths, now, now);
    let (_, _, again) = watching.owed(&paths, now, now);

    assert!(!first, "the first look is the starting point");
    assert!(
        after,
        "the log did not move, but the document did and the window has to be told"
    );
    assert!(!again, "nothing changed the second time");
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
