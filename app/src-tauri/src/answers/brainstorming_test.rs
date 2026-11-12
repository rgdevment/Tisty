use super::*;

fn session() -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    (tmp, session)
}

fn at(minute: i8) -> jiff::civil::DateTime {
    jiff::civil::date(2026, 11, 11).at(9, minute, 0, 0)
}

fn body(session: &Session, file: &str) -> String {
    tisty_core::docs::read(&session.paths.docs(), file).unwrap()
}

#[test]
fn the_first_idea_makes_the_brainstorm_and_every_later_one_lands_in_it() {
    let (_tmp, mut session) = session();

    let first = jotting(&mut session, "llamar al vet", "Lluvia de ideas", at(7)).unwrap();
    let second = jotting(&mut session, "probar el alias", "Lluvia de ideas", at(9)).unwrap();

    assert!(first.made);
    assert!(!second.made);
    assert_eq!(first.file, second.file);
    assert_eq!(
        session.state.docs.len(),
        1,
        "one brainstorm, never one per idea"
    );
    let written = body(&session, &first.file);
    assert!(written.contains("## 2026-11-11 09:07\n\nllamar al vet"));
    assert!(written.contains("## 2026-11-11 09:09\n\nprobar el alias"));
    assert!(
        written.find("09:07") < written.find("09:09"),
        "ideas keep the order they came in"
    );
    let kept = session.state.docs.values().next().unwrap();
    assert_eq!(kept.title.as_deref(), Some("Lluvia de ideas"));
}

#[test]
fn an_idea_with_no_words_is_refused_and_makes_nothing() {
    let (_tmp, mut session) = session();

    let refused = jotting(&mut session, "  \n ", "Lluvia de ideas", at(7)).unwrap_err();

    assert_eq!(refused.code, "untitled");
    assert!(session.state.docs.is_empty());
}

#[test]
fn an_archived_brainstorm_is_left_alone_and_a_new_one_takes_the_ideas() {
    let (_tmp, mut session) = session();
    let old = jotting(&mut session, "una idea", "Lluvia de ideas", at(7)).unwrap();
    let id = session
        .state
        .docs
        .values()
        .find(|one| one.file == old.file)
        .unwrap()
        .id;
    session.commit(Op::DocArchive { id }).unwrap();

    let new = jotting(&mut session, "otra idea", "Lluvia de ideas", at(9)).unwrap();

    assert!(new.made);
    assert_ne!(new.file, old.file);
    assert!(!body(&session, &old.file).contains("otra idea"));
}
