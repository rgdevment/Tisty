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

    let first = jotting(&mut session, "llamar al vet", "Lluvia de ideas", at(7), 0).unwrap();
    let second = jotting(&mut session, "probar el alias", "Lluvia de ideas", at(9), 0).unwrap();

    assert!(first.made);
    assert!(!second.made);
    assert_eq!(first.file, second.file);
    assert_eq!(
        session.state.docs.len(),
        1,
        "one brainstorm, never one per idea"
    );
    let written = body(&session, &first.file);
    assert!(written.contains("## 2026-11-11 09:07 · pink\n\nllamar al vet"));
    assert!(
        written.contains("## 2026-11-11 09:09 · coral\n\nprobar el alias"),
        "the same roll twice still gives the next idea a colour of its own"
    );
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

    let refused = jotting(&mut session, "  \n ", "Lluvia de ideas", at(7), 0).unwrap_err();

    assert_eq!(refused.code, "untitled");
    assert!(session.state.docs.is_empty());
}

#[test]
fn an_archived_brainstorm_is_left_alone_and_a_new_one_takes_the_ideas() {
    let (_tmp, mut session) = session();
    let old = jotting(&mut session, "una idea", "Lluvia de ideas", at(7), 0).unwrap();
    let id = session
        .state
        .docs
        .values()
        .find(|one| one.file == old.file)
        .unwrap()
        .id;
    session.commit(Op::DocArchive { id }).unwrap();

    let new = jotting(&mut session, "otra idea", "Lluvia de ideas", at(9), 0).unwrap();

    assert!(new.made);
    assert_ne!(new.file, old.file);
    assert!(!body(&session, &old.file).contains("otra idea"));
}

#[test]
fn an_idea_past_its_length_is_refused_and_nothing_is_written() {
    let (_tmp, mut session) = session();
    let long = "a".repeat(tisty_core::brainstorm::IDEA_AT_MOST + 1);

    let refused = jotting(&mut session, &long, "Lluvia de ideas", at(7), 0).unwrap_err();

    assert_eq!(refused.code, "ideaTooLong");
    assert!(session.state.docs.is_empty());
    let fits = "a".repeat(tisty_core::brainstorm::IDEA_AT_MOST);
    assert!(jotting(&mut session, &fits, "Lluvia de ideas", at(7), 0).is_ok());
}

#[test]
fn a_locked_brainstorm_refuses_the_idea_rather_than_making_a_second() {
    let (_tmp, mut session) = session();
    let first = jotting(&mut session, "una idea", "Lluvia de ideas", at(7), 0).unwrap();
    let id = session
        .state
        .docs
        .values()
        .find(|one| one.file == first.file)
        .unwrap()
        .id;
    session.commit(Op::DocLock { id }).unwrap();

    let refused = jotting(&mut session, "otra idea", "Lluvia de ideas", at(9), 0).unwrap_err();

    assert_eq!(refused.code, "documentLocked");
    assert_eq!(session.state.docs.len(), 1, "no second brainstorm");
}

#[test]
fn an_idea_filed_in_a_document_lands_at_its_end_and_leaves_the_rest_as_it_was() {
    let (_tmp, mut session) = session();
    let brainstorm = jotting(&mut session, "una idea", "Lluvia de ideas", at(7), 0).unwrap();
    let made = tisty_core::docs::create(
        &session.paths.docs(),
        &session.config.device_id,
        "# Recetas\n\nPan amasado.\n",
    )
    .unwrap();
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                sown: false,
                made: None,
                by: None,
                said: None,
                file: made.id.clone(),
                order: made.id.clone(),
                folder: None,
                page_of: None,
                task_of: None,
                edited: None,
            },
        })
        .unwrap();

    filing(&mut session, &made.id, "  Probar con harina integral  ").unwrap();

    assert_eq!(
        body(&session, &made.id),
        "# Recetas\n\nPan amasado.\n\nProbar con harina integral\n"
    );
    assert!(
        body(&session, &brainstorm.file).contains("una idea"),
        "the brainstorm is left to the window"
    );
}

#[test]
fn an_idea_is_not_filed_in_a_document_that_is_not_there() {
    let (_tmp, mut session) = session();

    let refused = filing(&mut session, "nadie-0001", "una idea").unwrap_err();

    assert_eq!(refused.code, "noSuchDoc");
}
