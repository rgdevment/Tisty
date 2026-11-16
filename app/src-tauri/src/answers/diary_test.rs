use super::*;

fn session(locale: &str) -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    session.locale = Some(locale.to_string());
    (tmp, session)
}

fn day(said: &str) -> Date {
    said.parse().unwrap()
}

fn body(session: &Session, file: &str) -> String {
    tisty_core::docs::read(&session.paths.docs(), file).unwrap()
}

#[test]
fn opening_the_diary_makes_the_year_and_the_month_but_no_empty_day() {
    let (_tmp, mut session) = session("es-CL");

    let today = opening(&mut session, day("2026-11-15")).unwrap();

    assert_eq!(body(&session, &today.page), "# Noviembre\n");
    let book = body(&session, &today.book);
    assert!(
        book.starts_with("---\ndiary: on\n---\n\n# Diario 2026\n"),
        "{book}"
    );
    assert!(
        book.contains(&format!("tisty:doc/{}", today.page)),
        "{book}"
    );
    let shown = kept(&session, day("2026-11-15"));
    assert_eq!(shown.book, Some(today.book.clone()));
    assert_eq!(shown.page, Some(today.page.clone()));
    assert_eq!(kept(&session, day("2026-12-01")).page, None);
    assert_eq!(opening(&mut session, day("2026-11-20")).unwrap(), today);
    assert_eq!(session.state.docs.len(), 2);
}

#[test]
fn what_is_written_lands_in_today_on_top_in_the_app_s_language() {
    let (_tmp, mut session) = session("en-US");

    writing(&mut session, "Walked Rina.", day("2026-11-14")).unwrap();
    let today = writing(&mut session, "Called Jaime.", day("2026-11-15")).unwrap();
    writing(&mut session, "Dinner with Marisol.", day("2026-11-15")).unwrap();

    assert_eq!(
        body(&session, &today.page),
        "# November\n\n## Sunday 15\n\nCalled Jaime.\n\nDinner with Marisol.\n\n## Saturday 14\n\nWalked Rina.\n"
    );
    let refused = writing(&mut session, "  \n", day("2026-11-15")).unwrap_err();
    assert_eq!(refused.code, "untitled");
}

#[test]
fn a_note_is_stuck_in_today_with_its_hour_and_a_long_one_writes_nothing() {
    let (_tmp, mut session) = session("es");
    let at = day("2026-11-15").at(14, 30, 0, 0);

    let today = sticking(&mut session, "Comprar pan", at, 0).unwrap();

    let written = body(&session, &today.page);
    assert!(
        written.starts_with("# Noviembre\n\n## Domingo 15\n\n> [!IDEA] 2026-11-15 14:30 · "),
        "{written}"
    );
    assert!(written.contains(" · right\n> Comprar pan\n"), "{written}");
    let long = "a".repeat(tisty_core::brainstorm::IDEA_AT_MOST + 1);
    let refused = sticking(&mut session, &long, at, 0).unwrap_err();
    assert_eq!(refused.code, "ideaTooLong");
    assert_eq!(body(&session, &today.page), written);
}

#[test]
fn a_locked_book_is_not_written_by_the_diary() {
    let (_tmp, mut session) = session("es");
    let today = opening(&mut session, day("2026-11-15")).unwrap();
    let id = session
        .state
        .docs
        .values()
        .find(|one| one.file == today.book)
        .unwrap()
        .id;
    session.commit(Op::DocLock { id }).unwrap();

    let refused = writing(&mut session, "Algo", day("2026-11-15")).unwrap_err();
    let reopened = opening(&mut session, day("2026-11-20")).unwrap();
    let december = opening(&mut session, day("2026-12-01")).unwrap_err();

    assert_eq!(refused.code, "documentLocked");
    assert_eq!(reopened, today, "a locked diary still opens to read");
    assert_eq!(december.code, "documentLocked");
    assert_eq!(body(&session, &today.page), "# Noviembre\n");
}

#[test]
fn the_door_is_said_once_and_only_when_it_changes() {
    let (_tmp, mut session) = session("es");
    assert!(!kept(&session, day("2026-11-15")).open);
    session.commit(Op::DiaryAgents { open: true }).unwrap();
    assert!(kept(&session, day("2026-11-15")).open);
}

#[test]
fn a_year_opens_on_its_newest_month_and_a_month_on_none() {
    let (_tmp, mut session) = session("es");
    let november = opening(&mut session, day("2026-11-15")).unwrap();
    let december = opening(&mut session, day("2026-12-01")).unwrap();

    assert_eq!(newest(&session, &november.book), Some(december.page));
    assert_eq!(newest(&session, &november.page), None);
    assert_eq!(newest(&session, "a3f1-9999"), None);
}
