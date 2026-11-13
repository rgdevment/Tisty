use super::*;
use tisty_core::Op;

fn session() -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    (tmp, session)
}

fn filed(session: &mut Session, body: &str) -> String {
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body).unwrap();
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
    made.id
}

#[test]
fn the_line_a_search_shows_is_cut_around_the_word_it_was_found_by() {
    let (_tmp, mut session) = session();
    filed(
        &mut session,
        &format!(
            "# Keys

{}Idempotency keys are kept 90 days{}
",
            "word ".repeat(60),
            " tail".repeat(60)
        ),
    );

    let found = sought(&mut session, "idempotency", None);

    let line = &found.papers[0].line;
    assert!(line.contains("Idempotency keys are kept 90 days"), "{line}");
    assert!(line.starts_with('…') && line.ends_with('…'), "{line}");
    assert_eq!(line.chars().count(), tisty_core::docs::SAID_AT_MOST);
}
