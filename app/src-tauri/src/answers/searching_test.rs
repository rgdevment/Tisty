use super::sought;
use crate::Session;
use tisty_core::Op;
use tisty_core::docs::SAID_AT_MOST;
use tisty_core::event::DocAdd;

#[test]
fn the_line_a_search_shows_is_cut_around_the_word_it_was_found_by() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let mut session = Session::at(paths).unwrap();
    let body = format!(
        "# Keys\n\n{}Idempotency keys are kept 90 days{}\n",
        "word ".repeat(60),
        " tail".repeat(60)
    );
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, &body).unwrap();
    let filed = DocAdd {
        file: made.id.clone(),
        order: made.id,
        ..DocAdd::default()
    };
    let id = ulid::Ulid::generate();
    session.commit(Op::DocAdd { id, d: filed }).unwrap();

    let found = sought(&mut session, "idempotency", None);

    let line = &found.papers[0].line;
    assert!(line.contains("Idempotency keys are kept 90 days"), "{line}");
    assert!(line.starts_with('…') && line.ends_with('…'), "{line}");
    assert_eq!(line.chars().count(), SAID_AT_MOST);
}
