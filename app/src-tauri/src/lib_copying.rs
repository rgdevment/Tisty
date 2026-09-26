use super::Session;
use tisty_core::model::DocId;
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

fn wrote(session: &mut Session, title: &str, up: Option<DocId>) -> DocId {
    let made = tisty_core::docs::create(
        &session.paths.docs(),
        &session.config.device_id,
        &format!("# {title}"),
    )
    .unwrap();
    let id = ulid::Ulid::generate();
    session
        .commit(Op::DocAdd {
            id,
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: made.id,
                order: tisty_core::order::last_of(
                    session
                        .state
                        .docs
                        .values()
                        .filter(|one| one.page_of == up)
                        .map(|one| one.order.as_str()),
                ),
                folder: None,
                page_of: up,
            },
        })
        .unwrap();
    id
}

fn twin_of(session: &Session, made: &tisty_core::docs::Doc) -> DocId {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == made.id)
        .unwrap()
        .id
}

#[test]
fn a_copy_of_a_page_in_the_archive_is_in_the_archive_too() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let book = wrote(&mut session, "Book", None);
    let page = wrote(&mut session, "Page", Some(book));
    session.commit(Op::DocArchive { id: page }).unwrap();

    let made = session.copy_doc(&page.to_string()).unwrap();
    let twin = twin_of(&session, &made);

    assert!(
        session.state.docs[&twin].archived,
        "a copy of what was put away comes back awake and loses what the person had decided"
    );
    assert_eq!(session.state.docs[&twin].page_of, Some(book));
}

#[test]
fn copying_a_book_keeps_each_page_where_the_person_left_it() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let book = wrote(&mut session, "Book", None);
    let first = wrote(&mut session, "First", Some(book));
    wrote(&mut session, "Second", Some(book));
    session.commit(Op::DocArchive { id: first }).unwrap();

    let made = session.copy_doc(&book.to_string()).unwrap();
    let twin = twin_of(&session, &made);

    assert!(!session.state.docs[&twin].archived);
    let pages: Vec<bool> = session
        .state
        .pages_of(twin)
        .iter()
        .map(|one| one.archived)
        .collect();
    assert_eq!(
        pages,
        vec![true, false],
        "the copy has to hold the same two pages, one of them put away"
    );
}

#[test]
fn a_page_of_a_document_in_the_archive_is_not_copied_into_it() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let book = wrote(&mut session, "Book", None);
    let page = wrote(&mut session, "Page", Some(book));
    session.commit(Op::DocArchive { id: book }).unwrap();

    assert!(
        session.copy_doc(&page.to_string()).is_err(),
        "a copy is a new page, and nothing new goes into the archive"
    );
    assert_eq!(session.state.pages_of(book).len(), 1);
}

#[test]
fn copying_a_document_in_the_archive_leaves_the_copy_there() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let book = wrote(&mut session, "Book", None);
    wrote(&mut session, "Page", Some(book));
    session.commit(Op::DocArchive { id: book }).unwrap();

    let made = session.copy_doc(&book.to_string()).unwrap();
    let twin = twin_of(&session, &made);

    assert!(session.state.docs[&twin].archived);
    let page = session.state.pages_of(twin)[0];
    assert!(
        !page.archived && session.state.held_away(page),
        "the page is covered by the copy, not marked on its own"
    );
}
