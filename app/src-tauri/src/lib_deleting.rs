use super::Session;
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

fn a_page_under(session: &mut Session, up: Option<tisty_core::model::DocId>) -> String {
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, "# A note")
            .unwrap();
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
                page_of: up,
            },
        })
        .unwrap();
    made.id
}

fn ledgered(desk: &Desk, files: &[&str]) {
    let mut said = tisty_core::docs::Carried::read(desk.paths.data());
    for file in files {
        said.keep(file, "a print");
    }
    said.save(desk.paths.data()).unwrap();
}

fn there(desk: &Desk, file: &str) -> bool {
    tisty_core::docs::resolve(&desk.paths.docs(), file).is_ok_and(|at| at.exists())
}

fn named(session: &Session, file: &str) -> tisty_core::model::DocId {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id
}

#[test]
fn deleting_a_document_takes_its_pages_off_the_disk_and_out_of_the_log() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let parent = a_page_under(&mut session, None);
    let up = named(&session, &parent);
    let page = a_page_under(&mut session, Some(up));
    ledgered(&desk, &[&parent, &page]);

    session.drop_doc(&up.to_string()).unwrap();

    assert!(session.state.docs.is_empty(), "neither is named any more");
    for file in [&parent, &page] {
        assert!(!there(&desk, file), "{file} stayed on the disk");
    }
    assert!(
        tisty_core::docs::Carried::read(desk.paths.data())
            .of(&parent)
            .is_none()
    );
}

#[test]
fn a_page_the_archive_holds_through_its_document_is_not_deleted() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let parent = a_page_under(&mut session, None);
    let up = named(&session, &parent);
    let page = a_page_under(&mut session, Some(up));
    let leaf = named(&session, &page);
    session.commit(Op::DocArchive { id: leaf }).unwrap();
    session.commit(Op::DocArchive { id: up }).unwrap();
    ledgered(&desk, &[&parent, &page]);

    assert!(
        session.drop_doc(&leaf.to_string()).is_err(),
        "deleting has no undo, and the archive keeps what it holds"
    );
    assert!(there(&desk, &page), "and the file is still on the disk");
}

#[test]
fn a_file_that_will_not_go_leaves_the_rest_deleted_and_is_swept_at_the_next_opening() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let parent = a_page_under(&mut session, None);
    let up = named(&session, &parent);
    let page = a_page_under(&mut session, Some(up));

    ledgered(&desk, &[&parent, &page]);

    let at = tisty_core::docs::resolve(&desk.paths.docs(), &parent).unwrap();
    std::fs::remove_file(&at).unwrap();
    std::fs::create_dir(&at).unwrap();

    session
        .drop_doc(&up.to_string())
        .expect("the log is the truth, so a file left behind is not a refusal");
    assert!(session.state.docs.is_empty(), "both are gone from the log");
    assert!(
        !there(&desk, &page),
        "the run carried on past the one that would not go"
    );
    assert!(
        session.state.shed.contains(&parent),
        "the one that stayed is still swept for"
    );
    let said = tisty_core::docs::Carried::read(desk.paths.data());
    assert!(
        said.of(&parent).is_none() && said.of(&page).is_none(),
        "and the ledger forgets both either way"
    );

    std::fs::remove_dir(&at).unwrap();
    std::fs::write(&at, b"# A note").unwrap();
    drop(session);
    let session = Session::at(desk.paths.clone()).unwrap();
    assert!(
        !there(&desk, &parent),
        "the shed is taken out when the window opens"
    );
    drop(session);
}

#[test]
fn a_name_that_is_not_a_document_deletes_nothing() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let file = a_page_under(&mut session, None);

    assert!(session.drop_doc(&file).is_err(), "a file name is not an id");
    assert!(session.drop_doc("nonsense").is_err());
    assert!(there(&desk, &file));
    assert_eq!(session.state.docs.len(), 1);
}

#[test]
fn a_stray_file_is_taken_in_and_then_cannot_be_taken_in_twice() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, "# Minuta")
            .unwrap();

    assert_eq!(
        tisty_core::docs::strayed(&desk.paths.docs(), &[]).len(),
        1,
        "nothing names it yet"
    );
    let took = session.take_in(&made.id).unwrap();
    assert_eq!(took.title, "Minuta");
    assert!(session.state.docs.values().any(|one| one.file == made.id));
    assert!(session.take_in(&made.id).is_err(), "not twice");
}

#[test]
fn a_file_the_log_still_names_is_not_let_go_of() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let file = a_page_under(&mut session, None);

    assert!(
        session.let_go_of(&file).is_err(),
        "the log names it, so it stays"
    );
    assert!(there(&desk, &file));
}

#[test]
fn a_stray_file_nobody_names_is_let_go_of() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let made =
        tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, "# Suelto")
            .unwrap();

    session.let_go_of(&made.id).unwrap();
    assert!(!there(&desk, &made.id));
    assert!(session.state.docs.is_empty(), "and no event was written");
}

#[test]
fn a_file_the_log_already_shed_is_not_taken_in() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let file = a_page_under(&mut session, None);
    let id = named(&session, &file);
    session.drop_doc(&id.to_string()).unwrap();

    std::fs::write(
        tisty_core::docs::resolve(&desk.paths.docs(), &file).unwrap(),
        b"# Vuelve",
    )
    .unwrap();
    assert!(
        session.take_in(&file).is_err(),
        "the next sweep would only take it again"
    );
    assert!(session.state.docs.is_empty());
}
