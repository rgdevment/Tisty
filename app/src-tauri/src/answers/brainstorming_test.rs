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
    assert!(written.contains("> [!IDEA] 2026-11-11 09:07 · pink\n> llamar al vet"));
    assert!(
        written.contains("> [!IDEA] 2026-11-11 09:09 · coral\n> probar el alias"),
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

    filing(
        &mut session,
        &made.id,
        "  Probar con harina integral  ",
        "2026-11-12 09:40 · yellow",
    )
    .unwrap();

    assert_eq!(
        body(&session, &made.id),
        "# Recetas\n\nPan amasado.\n\n> [!IDEA] 2026-11-12 09:40 · yellow · right\n> Probar con harina integral\n"
    );
    assert!(
        body(&session, &brainstorm.file).contains("una idea"),
        "the brainstorm is left to the window"
    );
}

#[test]
fn an_idea_is_not_filed_in_a_document_that_is_not_there() {
    let (_tmp, mut session) = session();

    let refused = filing(
        &mut session,
        "nadie-0001",
        "una idea",
        "2026-11-12 09:40 · blue",
    )
    .unwrap_err();

    assert_eq!(refused.code, "noSuchDoc");
}

#[test]
fn the_person_takes_an_agents_mark_off_one_idea_and_leaves_the_rest() {
    let (_tmp, mut session) = session();
    let made = jotting(
        &mut session,
        "entradas del concierto",
        "Lluvia de ideas",
        at(7),
        0,
    )
    .unwrap();
    jotting(
        &mut session,
        "pan con masa madre",
        "Lluvia de ideas",
        at(9),
        5,
    )
    .unwrap();
    let on = tisty_core::brainstorm::ideas_on(&body(&session, &made.file));
    let id = session
        .state
        .docs
        .values()
        .find(|one| one.file == made.file)
        .unwrap()
        .id;
    for idea in &on {
        session
            .commit(Op::IdeaFlag {
                id,
                idea: idea.clone(),
                d: tisty_core::event::Flag::new("ya fue").because(Some("obsolete".into()), None),
            })
            .unwrap();
    }

    unflagging(&mut session, &made.file, on[0].clone()).unwrap();
    unflagging(&mut session, &made.file, on[0].clone()).unwrap();

    let left = &session.state.docs[&id].ideas_flagged;
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].idea, on[1]);
}

#[test]
fn opening_ideas_before_any_idea_makes_the_sheet_once_and_the_first_idea_lands_in_it() {
    let (_tmp, mut session) = session();

    let made = beginning(&mut session, "Ideas").unwrap();
    let again = beginning(&mut session, "Ideas").unwrap();
    let jotted = jotting(&mut session, "llamar al vet", "Ideas", at(7), 0).unwrap();

    assert_eq!(made, again);
    assert_eq!(jotted.file, made);
    assert!(!jotted.made);
    assert_eq!(session.state.docs.len(), 1, "one sheet, never one per ask");
    let kept = session.state.docs.values().next().unwrap();
    assert_eq!(kept.title.as_deref(), Some("Ideas"));
    assert!(body(&session, &made).contains("> llamar al vet"));
}

fn unmarked_sheet(session: &mut Session, file: &str) {
    let body = tisty_core::brainstorm::opening("Ideas");
    tisty_core::docs::create_named(&session.paths.docs(), file, &body).unwrap();
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
                file: file.into(),
                order: "a0".into(),
                folder: None,
                page_of: None,
                task_of: None,
                edited: None,
            },
        })
        .unwrap();
}

#[test]
fn a_session_starts_without_having_looked_through_the_library_for_a_sheet() {
    let (_tmp, session) = session();

    assert!(!session.brainstorm_looked);
}

#[test]
fn looking_without_the_whole_library_misses_a_sheet_nobody_marked_and_never_remembers_it() {
    let (_tmp, mut session) = session();
    unmarked_sheet(&mut session, "mine-1");

    assert!(found(&mut session, false).is_none());
    assert!(session.brainstorm.is_none());
    assert_eq!(found(&mut session, true).unwrap().0, "mine-1");
    assert_eq!(session.brainstorm.as_deref(), Some("mine-1"));
}

#[test]
fn opening_ideas_finds_the_old_unmarked_sheet_after_a_quick_look_missed_it() {
    let (_tmp, mut session) = session();
    assert!(found(&mut session, false).is_none());
    unmarked_sheet(&mut session, "mine-1");

    assert!(found(&mut session, false).is_none());
    let opened = beginning(&mut session, "Ideas").unwrap();

    assert_eq!(opened, "mine-1");
    assert_eq!(session.state.docs.len(), 1, "no second sheet was made");
}

#[test]
fn an_idea_jotted_after_a_quick_look_lands_in_the_old_unmarked_sheet() {
    let (_tmp, mut session) = session();
    unmarked_sheet(&mut session, "mine-1");
    assert!(found(&mut session, false).is_none());

    let jotted = jotting(&mut session, "llamar al vet", "Ideas", at(7), 0).unwrap();

    assert_eq!(jotted.file, "mine-1");
    assert!(!jotted.made);
    assert_eq!(session.state.docs.len(), 1);
}

#[test]
fn the_first_look_for_the_pinned_sheet_reads_the_whole_library_and_finds_an_old_unmarked_one() {
    let (_tmp, mut session) = session();
    unmarked_sheet(&mut session, "mine-1");

    let pinned = pinned(&mut session).unwrap();

    assert_eq!(pinned.file, "mine-1");
    assert!(session.brainstorm_looked);
}

#[test]
fn a_second_look_for_the_pinned_sheet_never_reads_the_library_again() {
    let (_tmp, mut session) = session();
    assert!(pinned(&mut session).is_none());
    unmarked_sheet(&mut session, "mine-1");

    assert!(pinned(&mut session).is_none(), "the library was read again");
    assert_eq!(beginning(&mut session, "Ideas").unwrap(), "mine-1");
    assert_eq!(session.state.docs.len(), 1, "no second sheet was made");
}
