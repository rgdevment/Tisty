use super::Session;
use tisty_core::{Config, Paths};

fn named(session: &Session) -> Option<String> {
    session
        .state
        .named
        .get(&session.config.device_id)
        .map(|one| one.name.clone())
}

#[test]
fn a_name_the_person_gave_is_written_and_kept_over_the_systems_on_every_opening() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths.clone()).unwrap();

    session.rename("  Roble 42 ").unwrap();

    assert_eq!(named(&session).as_deref(), Some("Roble 42"));
    assert_eq!(
        Config::load_or_init(&paths).unwrap().called.as_deref(),
        Some("Roble 42")
    );
    drop(session);
    let again = Session::at(paths.clone()).unwrap();
    assert_eq!(
        named(&again).as_deref(),
        Some("Roble 42"),
        "opening again put the system's name back"
    );
}

#[test]
fn an_empty_name_gives_this_computer_back_its_systems_name() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths.clone()).unwrap();
    session.rename("Roble 42").unwrap();

    session.rename("   ").unwrap();

    assert_eq!(Config::load_or_init(&paths).unwrap().called, None);
    if let Some(system) = tisty_core::called::here() {
        assert_eq!(named(&session), Some(system.name));
    }
}
