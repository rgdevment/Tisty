use super::*;
use tisty_core::config::Sync;

fn folder(at: &str) -> Option<Sync> {
    Some(Sync::folder(at))
}

#[test]
fn a_round_that_stopped_keeps_saying_why_while_the_folder_is_the_same() {
    let why = Refusal::about("syncNewer", "dev_b");

    let stuck = stuck_after(Some(&why), &folder("G:/shared"), &folder("G:/shared"))
        .expect("the stop went unsaid");

    assert_eq!(
        (stuck.code, stuck.name.as_deref()),
        ("syncNewer", Some("dev_b"))
    );
}

#[test]
fn a_round_that_got_through_clears_the_stop() {
    assert!(stuck_after(None, &folder("G:/shared"), &folder("G:/shared")).is_none());
}

#[test]
fn a_stop_from_a_folder_left_while_the_round_ran_is_not_raised() {
    let why = Refusal::of("syncNewer");

    assert!(stuck_after(Some(&why), &folder("G:/shared"), &folder("D:/other")).is_none());
    assert!(stuck_after(Some(&why), &folder("G:/shared"), &Some(Sync::Local)).is_none());
}

#[test]
fn having_no_folder_is_no_stop_to_report() {
    assert!(stuck_after(Some(&Refusal::of("noRemote")), &None, &None).is_none());
}

#[test]
fn a_store_restored_apart_from_its_folder_says_so() {
    let why = Refusal::about("restoredApart", "G:/shared");

    assert!(stuck_after(Some(&why), &folder("G:/shared"), &folder("G:/shared")).is_some());
}

#[test]
fn a_folder_no_round_ever_finished_with_can_be_left_freely() {
    let kept = tempfile::tempdir().unwrap();
    let away = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    session.config.sync = Some(Sync::folder(away.path().to_path_buf()));

    assert!(stranded_by_leaving(&session, &Sync::alone()).is_none());

    std::fs::create_dir_all(session.paths.cache()).unwrap();
    std::fs::write(
        session.paths.cache().join("carried-to"),
        tisty_core::paths::told_of(away.path()),
    )
    .unwrap();

    let stopped = stranded_by_leaving(&session, &Sync::alone())
        .expect("a folder holding what was let go was left");
    assert_eq!(
        (stopped.code, stopped.name),
        ("sharedAwayToLeave", Some(away.path().display().to_string()))
    );
}

fn later() -> Sync {
    Sync::Unknown(toml::from_str("how = \"nube\"").unwrap())
}

#[test]
fn a_way_a_later_build_chose_is_never_left_from_here() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    session.config.sync = Some(later());
    session.config.holds = Some(tisty_core::config::Holds::Everywhere);

    for toward in [Sync::alone(), Sync::folder("G:/otra")] {
        let stopped = stranded_by_leaving(&session, &toward)
            .expect("a choice a later build made was replaced, whatever the setting says");
        assert_eq!(stopped.code, "syncLaterToLeave");
    }
}

#[test]
fn staying_on_the_way_a_later_build_chose_is_not_leaving_it() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    session.config.sync = Some(later());

    assert!(stranded_by_leaving(&session, &later()).is_none());
}

#[test]
fn leaving_nothing_is_always_free() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();

    assert!(stranded_by_leaving(&session, &Sync::folder("G:/otra")).is_none());
    session.config.sync = Some(Sync::alone());
    assert!(stranded_by_leaving(&session, &Sync::folder("G:/otra")).is_none());
}

#[test]
fn a_way_a_newer_build_wrote_after_this_window_opened_is_not_replaced() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    session.config.sync = Some(Sync::folder("G:/compartida"));
    session
        .keep(|c| c.sync = Some(Sync::folder("G:/compartida")))
        .unwrap();
    let mut written = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    written.sync = Some(later());
    written.save(&paths).unwrap();

    let stopped = session
        .keep_unless(choosing(Sync::folder("G:/otra")))
        .expect_err("what a newer build wrote was written over");

    assert_eq!(stopped.code, "syncLaterToLeave");
    let on_disk = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    assert_eq!(on_disk.sync, Some(later()));
    assert_eq!(
        session.config.sync,
        Some(later()),
        "the window went on believing in a way that is no longer written"
    );
}

fn written_since_the_window_opened(paths: &tisty_core::Paths, way: Option<Sync>) {
    let mut written = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    written.sync = way;
    written.save(paths).unwrap();
}

#[test]
fn choosing_a_folder_over_a_way_a_newer_build_wrote_meanwhile_is_refused_and_writes_nothing() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    session
        .keep(|c| c.sync = Some(Sync::folder("G:/compartida")))
        .unwrap();
    written_since_the_window_opened(&paths, Some(later()));

    let stopped = choose(&mut session, Some("G:/otra".to_string())).expect_err("replaced");

    assert_eq!(stopped.code, "syncLaterToLeave");
    let on_disk = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    assert_eq!(on_disk.sync, Some(later()));
    assert_eq!(session.config.sync, Some(later()));
}

#[test]
fn a_window_that_still_believes_in_a_later_way_is_not_stuck_on_it() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    session.keep(|c| c.sync = Some(later())).unwrap();
    written_since_the_window_opened(&paths, None);

    choose(&mut session, Some("G:/otra".to_string()))
        .expect("a way that is no longer written refused the change");

    let on_disk = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    assert_eq!(on_disk.sync, Some(Sync::folder("G:/otra")));
    assert_eq!(session.config.sync, Some(Sync::folder("G:/otra")));
}

#[test]
fn settings_this_version_cannot_read_are_never_written_over() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();
    let said = std::fs::read_to_string(paths.config_file()).unwrap();
    let unreadable = format!("holds = \"cloudonly\"\n{said}");
    std::fs::write(paths.config_file(), &unreadable).unwrap();

    let stopped = session
        .keep(|c| c.locale = Some("es".into()))
        .expect_err("settings written by a newer build were overwritten");
    let chosen = choose(&mut session, Some("G:/otra".to_string())).expect_err("overwritten");

    assert_eq!(stopped.code, "settingsUnreadable");
    assert_eq!(chosen.code, "settingsUnreadable");
    assert_eq!(
        std::fs::read_to_string(paths.config_file()).unwrap(),
        unreadable
    );
}

#[test]
fn a_change_that_is_not_refused_is_written_onto_what_is_on_disk() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths.clone()).unwrap();

    session
        .keep_unless(choosing(Sync::folder("G:/compartida")))
        .unwrap();

    let on_disk = tisty_core::Config::load(&paths.config_file())
        .unwrap()
        .unwrap();
    assert_eq!(on_disk.sync, Some(Sync::folder("G:/compartida")));
}

#[test]
fn a_way_a_later_build_chose_pins_no_stop_on_the_window() {
    let why = Refusal::of("syncLater");

    assert!(stuck_after(Some(&why), &Some(later()), &Some(later())).is_none());
}

#[test]
fn a_command_that_needs_a_carrier_says_why_there_is_none() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();

    assert_eq!(
        session.carrying().err().map(|one| one.code),
        Some("noRemote")
    );
    session.config.sync = Some(later());
    assert_eq!(
        session.carrying().err().map(|one| one.code),
        Some("syncLater")
    );
    session.config.sync = Some(Sync::folder("G:/compartida"));
    assert!(session.carrying().is_ok());
}
