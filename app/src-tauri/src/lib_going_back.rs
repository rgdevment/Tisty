use super::{Session, one_step_back, went_back};
use tisty_core::{Op, Paths};

struct Desk {
    _tmp: tempfile::TempDir,
    paths: Paths,
}

fn desk() -> Desk {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    Desk { _tmp: tmp, paths }
}

fn wrote(session: &mut Session, body: &str) -> String {
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body).unwrap();
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: made.id.clone(),
                order: tisty_core::order::first(),
                folder: None,
                page_of: None,
            },
        })
        .unwrap();
    made.id
}

fn agent_wrote(session: &Session, id: &str, body: &str) {
    let was = tisty_core::docs::read(&session.paths.docs(), id).unwrap();
    let print = tisty_core::attach::printed(was.as_bytes());
    tisty_core::docs::rewrite(
        &session.paths.docs(),
        session.paths.data(),
        id,
        body,
        &print,
    )
    .unwrap();
}

#[test]
fn a_document_nothing_wrote_over_has_nothing_to_go_back_to() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let doc = wrote(&mut session, "# Acta\n\nlo que hay\n");

    assert!(one_step_back(&session, &doc).is_none());
    let why = went_back(&mut session, &doc).unwrap_err();
    assert_eq!(why.code, "nothingKeptBeside");
}

#[test]
fn going_back_puts_the_body_back_and_keeps_the_one_it_wrote_over() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let doc = wrote(&mut session, "# Acta\n\nlo que hay\n");
    agent_wrote(&session, &doc, "# Acta\n\nlo que dejo el agente\n");

    let said = went_back(&mut session, &doc).unwrap();

    assert_eq!(said.title, "Acta");
    let now = tisty_core::docs::read(&session.paths.docs(), &doc).unwrap();
    assert!(now.contains("lo que hay"), "{now}");
    let kept = tisty_core::docs::read_before(session.paths.data(), &doc).unwrap();
    assert!(kept.contains("lo que dejo el agente"), "{kept}");
}

#[test]
fn going_back_twice_leaves_the_document_where_it_started() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let doc = wrote(&mut session, "# Acta\n\nuno\n");
    agent_wrote(&session, &doc, "# Acta\n\ndos\n");

    went_back(&mut session, &doc).unwrap();
    went_back(&mut session, &doc).unwrap();

    let now = tisty_core::docs::read(&session.paths.docs(), &doc).unwrap();
    assert!(now.contains("dos"), "{now}");
}

#[test]
fn a_body_written_since_the_copy_was_set_aside_is_not_taken_with_it() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let doc = wrote(&mut session, "# Acta\n\nuno\n");
    agent_wrote(&session, &doc, "# Acta\n\ndos\n");
    tisty_core::docs::write(&session.paths.docs(), &doc, "# Acta\n\ntres\n").unwrap();

    assert!(
        one_step_back(&session, &doc).is_none(),
        "what is kept is two writes back, so it is not one step"
    );
    let why = went_back(&mut session, &doc).unwrap_err();
    assert_eq!(why.code, "nothingKeptBeside");
    let now = tisty_core::docs::read(&session.paths.docs(), &doc).unwrap();
    assert!(now.contains("tres"), "nothing was written over: {now}");
}

#[test]
fn a_document_the_archive_holds_does_not_go_back() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let doc = wrote(&mut session, "# Acta\n\nuno\n");
    agent_wrote(&session, &doc, "# Acta\n\ndos\n");
    let id = session
        .state
        .docs
        .values()
        .find(|one| one.file == doc)
        .unwrap()
        .id;
    session.commit(Op::DocArchive { id }).unwrap();

    let why = went_back(&mut session, &doc).unwrap_err();
    assert_eq!(why.code, "documentAway");
    let now = tisty_core::docs::read(&session.paths.docs(), &doc).unwrap();
    assert!(now.contains("dos"), "nothing was written: {now}");
}
