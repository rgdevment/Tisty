use super::Session;
use tisty_core::{Op, Paths};

fn desk() -> (tempfile::TempDir, Paths) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    (tmp, paths)
}

fn wrote(session: &mut Session, body: &str, page_of: Option<tisty_core::model::DocId>) -> String {
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
                order: tisty_core::order::last_of(
                    session.state.docs.values().map(|one| one.order.as_str()),
                ),
                folder: None,
                page_of,
            },
        })
        .unwrap();
    made.id
}

fn id_of(session: &Session, file: &str) -> tisty_core::model::DocId {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .id
}

fn held(session: &Session, up: tisty_core::model::DocId) -> Vec<String> {
    session
        .state
        .pages_of(up)
        .into_iter()
        .map(|one| one.file.clone())
        .collect()
}

#[test]
fn a_page_hung_again_is_read_where_the_book_still_names_it() {
    let (_tmp, paths) = desk();
    let mut session = Session::at(paths).unwrap();
    let book = wrote(&mut session, "# Libro\n\nintro\n", None);
    let up = id_of(&session, &book);
    let one = wrote(&mut session, "# Uno\n", Some(up));
    let two = wrote(&mut session, "# Dos\n", Some(up));
    tisty_core::docs::write(
        &session.paths.docs(),
        &book,
        &format!("# Libro\n\nintro\n\n![Uno](tisty:doc/{one})\n\n![Dos](tisty:doc/{two})\n"),
    )
    .unwrap();
    let body = tisty_core::docs::read(&session.paths.docs(), &book).unwrap();
    session.retell(&book, &body, None);
    let was = held(&session, up);
    assert_eq!(was, vec![one.clone(), two.clone()]);

    // What dropping it on the book again commits: membership, and a key of its own at the end.
    let mine = id_of(&session, &one);
    let last =
        tisty_core::order::last_of(session.state.docs.values().map(|one| one.order.as_str()));
    session
        .commit(Op::DocMove {
            id: mine,
            d: tisty_core::event::Filed {
                folder: None,
                page_of: Some(Some(up)),
                order: Some(last),
            },
        })
        .unwrap();
    let body = tisty_core::docs::read(&session.paths.docs(), &book).unwrap();
    session.retell(&book, &body, None);

    assert_eq!(
        held(&session, up),
        was,
        "the text did not change, so neither did the order"
    );
}
