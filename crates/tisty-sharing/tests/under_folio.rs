#[allow(dead_code)]
mod common;

use std::cell::Cell;

use common::{Home, asked};
use tisty_core::model::DocId;
use tisty_sharing::{Desk, Gathered, Under, Unshared, gathered_under, sent};

const GIVEN: &str = "Ab3xK9pQ";

fn written() -> (tempfile::TempDir, Home, DocId) {
    let tmp = tempfile::tempdir().unwrap();
    let mut home = Home::at(tmp.path());
    home.with_a_name();
    let doc = home.wrote("# Viaje\n\nPalabras.\n", None);
    (tmp, home, doc)
}

fn under_folio(home: &Home, doc: DocId, under: &Under<'_>) -> Result<Gathered, Unshared> {
    let desk = Desk {
        paths: &home.paths,
        state: &home.state,
        also: None,
        turned: &|_| None,
    };
    gathered_under(&desk, &asked(doc), under)
}

fn given(minted: &Cell<u32>) -> impl Fn() -> Result<String, Unshared> + '_ {
    || {
        minted.set(minted.get() + 1);
        Ok(GIVEN.to_owned())
    }
}

#[test]
fn a_first_version_goes_under_the_id_folio_gave() {
    let (_tmp, home, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let under = Under {
        mint: &mint,
        editable: false,
    };

    let sealed = home.sealed(under_folio(&home, doc, &under).unwrap(), vec![]);

    assert_eq!(minted.get(), 1);
    assert_eq!(sealed.going[0].id, GIVEN);
    let (_, remembered) = &sealed.remembering[0];
    assert_eq!(remembered.id, GIVEN);
    assert!(remembered.admin_token.is_some());
    assert_eq!(remembered.edit_token, None);
}

#[test]
fn only_what_may_be_edited_has_a_token() {
    let (_tmp, home, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let under = Under {
        mint: &mint,
        editable: true,
    };

    let sealed = home.sealed(under_folio(&home, doc, &under).unwrap(), vec![]);

    assert!(sealed.remembering[0].1.edit_token.is_some());
}

#[test]
fn a_later_version_asks_for_no_id_and_keeps_its_tokens() {
    let (_tmp, home, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let first = home.sealed(
        under_folio(
            &home,
            doc,
            &Under {
                mint: &mint,
                editable: true,
            },
        )
        .unwrap(),
        vec![],
    );
    sent(&home.paths, &first.remembering).unwrap();

    let later = home.sealed(
        under_folio(
            &home,
            doc,
            &Under {
                mint: &mint,
                editable: false,
            },
        )
        .unwrap(),
        vec![],
    );

    assert_eq!(minted.get(), 1);
    assert_eq!(later.remembering[0].1.version, 2);
    assert_eq!(later.remembering[0].1.id, GIVEN);
    assert_eq!(
        later.remembering[0].1.admin_token,
        first.remembering[0].1.admin_token
    );
    assert_eq!(
        later.remembering[0].1.edit_token,
        first.remembering[0].1.edit_token
    );
}

#[test]
fn an_id_that_does_not_arrive_stops_it_before_anything_is_sealed() {
    let (_tmp, home, doc) = written();
    let mint = || Err(Unshared::Unread("nobody answered".to_owned()));

    let stopped = under_folio(
        &home,
        doc,
        &Under {
            mint: &mint,
            editable: false,
        },
    );

    assert!(matches!(stopped, Err(Unshared::Unread(_))));
}
