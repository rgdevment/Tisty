#[allow(dead_code)]
mod common;

use std::cell::Cell;

use common::{Home, asked};
use jiff::Timestamp;
use tisty_contracts::{EnvelopeKind, Opening, Served};
use tisty_core::model::DocId;
use tisty_core::{DeviceKind, Op, person};
use tisty_envelope::reading;
use tisty_share_proto::seed::Seed;
use tisty_sharing::{Desk, Gathered, Remembered, Under, Unshared, gathered_under, sent};

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

fn another_person(home: &mut Home) {
    let device = home.device.clone();
    home.commit(Op::DeviceJoin {
        d: device.clone(),
        k: Some(DeviceKind::Machine),
        p: None,
    });
    let seed = Seed::from_bytes(&[0x44; 32]);
    let now = "2026-11-06T12:00:00Z".parse::<Timestamp>().unwrap();
    let ops = person::begun(&home.paths, &device, &seed, &home.state.persons, now).unwrap();
    for op in ops {
        home.commit(op);
    }
}

#[test]
fn whoever_holds_the_edit_link_sends_a_version_under_the_same_owner() {
    let (_tmp, owner, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let first = owner.sealed(
        under_folio(
            &owner,
            doc,
            &Under {
                mint: &mint,
                editable: true,
            },
        )
        .unwrap(),
        vec![],
    );
    let (_, shared) = first.remembering[0].clone();
    let opening = Opening {
        id: GIVEN.to_owned(),
        kind: EnvelopeKind::Main,
        link_key: shared.link_key.clone(),
    };
    let served = Served {
        received: None,
        revocations: None,
    };
    let main = &first.going[0].sealed.main;
    let block = reading::owner_block(&reading::opened(main, &opening, &served).unwrap());
    let arrived = reading::reading(main, &opening, &served).unwrap();
    assert!(block.is_some());

    let tmp = tempfile::tempdir().unwrap();
    let mut editor = Home::at(tmp.path());
    another_person(&mut editor);
    let body = arrived.document["body"].as_str().unwrap().to_owned();
    let copy = editor.wrote(&format!("{body}Y otra cosa.\n"), None);
    let file = editor.state.docs[&copy].file.clone();
    let entry = Remembered {
        id: GIVEN.to_owned(),
        version: 1,
        link_key: shared.link_key.clone(),
        files_print: None,
        files_hash: None,
        files_key: None,
        admin_token: None,
        edit_token: shared.edit_token.clone(),
        words_print: None,
        signed_as: None,
        owner: block,
        owner_name: arrived.document["owner_name"].as_str().map(str::to_owned),
        cited: Vec::new(),
        held: false,
    };
    sent(&editor.paths, &[(file, entry)]).unwrap();

    let nobody = || Err(Unshared::Unread("it has its id".to_owned()));
    let second = editor.sealed(
        under_folio(
            &editor,
            copy,
            &Under {
                mint: &nobody,
                editable: false,
            },
        )
        .unwrap(),
        vec![],
    );

    let read = reading::reading(&second.going[0].sealed.main, &opening, &served).unwrap();
    assert_eq!(read.owner, arrived.owner);
    assert_ne!(read.signer, arrived.signer);
    assert_eq!(second.remembering[0].1.version, 2);
    assert_eq!(second.remembering[0].1.admin_token, None);
}

#[test]
fn the_document_that_was_shared_is_held_by_nothing() {
    let (_tmp, home, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);

    let sealed = home.sealed(
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

    assert!(!sealed.remembering[0].1.held);
}
