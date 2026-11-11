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
use tisty_sharing::{Desk, Gathered, Remembered, Under, Unshared, gathered_under};

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
        maker: None,
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
fn what_it_seals_says_which_terms_its_signer_accepted() {
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
                terms: Some("3"),
                languages_at_most: None,
            },
        )
        .unwrap(),
        vec![],
    );
    let opening = Opening {
        id: GIVEN.to_owned(),
        kind: EnvelopeKind::Main,
        link_key: sealed.remembering[0].1.link_key.clone(),
    };
    let served = Served {
        received: None,
        revocations: None,
        reviewed: None,
        reviewed_received: None,
    };

    let opened = reading::opened(&sealed.going[0].sealed.main, &opening, &served).unwrap();

    let manifest: serde_json::Value = serde_json::from_slice(&opened.manifest).unwrap();
    assert_eq!(manifest["terms"], "3");
}

#[test]
fn a_first_version_goes_under_the_id_folio_gave() {
    let (_tmp, home, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let under = Under {
        mint: &mint,
        editable: false,
        terms: None,
        languages_at_most: None,
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
        terms: None,
        languages_at_most: None,
    };

    let sealed = home.sealed(under_folio(&home, doc, &under).unwrap(), vec![]);

    assert!(sealed.remembering[0].1.edit_token.is_some());
}

#[test]
fn a_later_version_asks_for_no_id_and_keeps_its_tokens() {
    let (_tmp, mut home, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let first = home.sealed(
        under_folio(
            &home,
            doc,
            &Under {
                mint: &mint,
                editable: true,
                terms: None,
                languages_at_most: None,
            },
        )
        .unwrap(),
        vec![],
    );
    home.sent(&first.remembering);

    let later = home.sealed(
        under_folio(
            &home,
            doc,
            &Under {
                mint: &mint,
                editable: false,
                terms: None,
                languages_at_most: None,
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
            terms: None,
            languages_at_most: None,
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
                terms: None,
                languages_at_most: None,
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
        reviewed: None,
        reviewed_received: None,
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
        held_by: None,
        ttl_days: None,
        carried: Some(
            serde_json::json!({
                "lang": "es",
                "languages": [{"lang": "en", "document": "Lang1111", "key": shared.link_key}],
                "cited": null,
            })
            .to_string(),
        ),
        sealed_under: None,
        version_by: None,
        version_root: None,
        left_out: Vec::new(),
    };
    editor.sent(&[(file, entry)]);

    let nobody = || Err(Unshared::Unread("it has its id".to_owned()));
    let second = editor.sealed(
        under_folio(
            &editor,
            copy,
            &Under {
                mint: &nobody,
                editable: false,
                terms: None,
                languages_at_most: None,
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
    assert_eq!(read.document["languages"][0]["document"], "Lang1111");
    assert_eq!(read.document["lang"], "es");
}

fn a_later_minor() -> String {
    let mut said = tisty_contracts::VERSION.split('.');
    let major = said.next().unwrap();
    let minor: u64 = said.next().unwrap().parse().unwrap();
    format!("{major}.{}.0", minor + 1)
}

#[test]
fn what_arrived_under_a_later_minor_is_not_sealed_again_until_tisty_catches_up() {
    let (_tmp, mut home, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let under = Under {
        mint: &mint,
        editable: false,
        terms: None,
        languages_at_most: None,
    };
    let first = home.sealed(under_folio(&home, doc, &under).unwrap(), vec![]);
    let (file, mut shared) = first.remembering[0].clone();

    shared.sealed_under = Some(a_later_minor());
    home.sent(&[(file.clone(), shared.clone())]);
    assert!(matches!(
        under_folio(&home, doc, &under),
        Err(Unshared::Newer)
    ));

    shared.sealed_under = Some(tisty_contracts::VERSION.to_owned());
    home.sent(&[(file, shared)]);
    let again = under_folio(&home, doc, &under).unwrap();
    assert_eq!(
        home.sealed(again, vec![]).remembering[0]
            .1
            .sealed_under
            .as_deref(),
        Some(tisty_contracts::VERSION)
    );
}

#[test]
fn what_sits_under_folios_floor_goes_again_with_no_change_of_its_own() {
    let (_tmp, mut home, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let under = Under {
        mint: &mint,
        editable: false,
        terms: None,
        languages_at_most: None,
    };
    let first = home.sealed(under_folio(&home, doc, &under).unwrap(), vec![]);
    let (file, mut shared) = first.remembering[0].clone();
    home.sent(&[(file.clone(), shared.clone())]);
    let floor = vec![tisty_contracts::VERSION.to_owned()];

    let again = under_folio(&home, doc, &under).unwrap();
    assert!(again.sent_again_below(&floor).same_as_sent());

    shared.sealed_under = Some("1.2.0".to_owned());
    home.sent(&[(file.clone(), shared.clone())]);
    let again = under_folio(&home, doc, &under).unwrap();
    assert!(again.same_as_sent());
    assert!(!again.sent_again_below(&floor).same_as_sent());

    shared.sealed_under = None;
    home.sent(&[(file, shared)]);
    let again = || under_folio(&home, doc, &under).unwrap();
    assert!(again().sent_again_below(&[]).same_as_sent());
    assert!(!again().sent_again_below(&floor).same_as_sent());
}

fn signed_as(home: &Home, doc: DocId, under: &Under<'_>, name: &str) -> Gathered {
    let desk = Desk {
        paths: &home.paths,
        state: &home.state,
        also: None,
        turned: &|_| None,
        maker: None,
    };
    let asked = tisty_sharing::Asked {
        signed_as: Some(name.to_owned()),
        left_out: Vec::new(),
        ..asked(doc)
    };
    gathered_under(&desk, &asked, under).unwrap()
}

fn names_in(document: &serde_json::Value) -> Vec<(String, bool)> {
    document["editors"]
        .as_array()
        .map(|all| {
            all.iter()
                .map(|one| {
                    (
                        one["name"].as_str().unwrap().to_owned(),
                        one["owned"] == true,
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn every_version_names_who_edited_it_and_the_owner_keeps_them_when_it_sends_again() {
    let (_tmp, mut owner, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let editable = Under {
        mint: &mint,
        editable: true,
        terms: None,
        languages_at_most: None,
    };
    let first = owner.sealed(signed_as(&owner, doc, &editable, "Mario"), vec![]);
    let (owner_file, shared) = first.remembering[0].clone();
    owner.sent(&[(owner_file.clone(), shared.clone())]);
    let opening = Opening {
        id: GIVEN.to_owned(),
        kind: EnvelopeKind::Main,
        link_key: shared.link_key.clone(),
    };
    let served = Served {
        received: None,
        revocations: None,
        reviewed: None,
        reviewed_received: None,
    };
    let main = &first.going[0].sealed.main;
    let one = reading::reading(main, &opening, &served).unwrap();
    assert_eq!(names_in(&one.document), [("Mario".to_owned(), true)]);

    let tmp = tempfile::tempdir().unwrap();
    let mut editor = Home::at(tmp.path());
    another_person(&mut editor);
    let body = one.document["body"].as_str().unwrap().to_owned();
    let copy = editor.wrote(&format!("{body}Y otra cosa.\n"), None);
    let file = editor.state.docs[&copy].file.clone();
    let arrived = Remembered {
        admin_token: None,
        owner: reading::owner_block(&reading::opened(main, &opening, &served).unwrap()),
        owner_name: Some("Mario".to_owned()),
        carried: Some(serde_json::json!({ "editors": one.document["editors"] }).to_string()),
        ..shared.clone()
    };
    editor.sent(&[(file, arrived)]);
    let nobody = || Err(Unshared::Unread("it has its id".to_owned()));
    let later = Under {
        mint: &nobody,
        editable: false,
        terms: None,
        languages_at_most: None,
    };
    let second = editor.sealed(signed_as(&editor, copy, &later, "Ana"), vec![]);
    let two = reading::reading(&second.going[0].sealed.main, &opening, &served).unwrap();
    let remembered: serde_json::Value =
        serde_json::from_str(second.remembering[0].1.carried.as_deref().unwrap()).unwrap();
    assert_eq!(remembered["editors"], two.document["editors"]);
    assert_eq!(
        names_in(&two.document),
        [("Mario".to_owned(), true), ("Ana".to_owned(), false)]
    );

    let brought = tisty_sharing::editors_brought(
        shared.carried.as_deref(),
        Some(&serde_json::json!({ "editors": two.document["editors"] }).to_string()),
    );
    owner.sent(&[(
        owner_file,
        Remembered {
            version: 2,
            carried: brought,
            ..shared
        },
    )]);
    let third = owner.sealed(signed_as(&owner, doc, &later, "Mario Hidalgo"), vec![]);
    let three = reading::reading(&third.going[0].sealed.main, &opening, &served).unwrap();
    assert_eq!(
        names_in(&three.document),
        [
            ("Mario Hidalgo".to_owned(), true),
            ("Ana".to_owned(), false)
        ]
    );
}

#[test]
fn whoever_edits_sends_the_owners_reviews_as_they_arrived() {
    let (_tmp, owner, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let editable = Under {
        mint: &mint,
        editable: true,
        terms: None,
        languages_at_most: None,
    };
    let first = owner.sealed(signed_as(&owner, doc, &editable, "Mario"), vec![]);
    let (_, shared) = first.remembering[0].clone();
    let opening = Opening {
        id: GIVEN.to_owned(),
        kind: EnvelopeKind::Main,
        link_key: shared.link_key.clone(),
    };
    let served = Served {
        received: None,
        revocations: None,
        reviewed: None,
        reviewed_received: None,
    };
    let main = &first.going[0].sealed.main;
    let one = reading::reading(main, &opening, &served).unwrap();
    let key = "ylsrD8-8Gn6kHk09Wmw65fL0pN3T9qT3yS4NH3-b0sQ";
    let signed =
        "eDlbK8-hHnTnO3RxmU9NSgdjN6kaHvRwQkfbkFmCnf5cXpGpKjYyQnH6u4L0vT2sN1rE8wZ7dA3bMfI9oUxKDw";
    let reviews = serde_json::json!([
        { "of": key, "kept": true, "document": "enGlish1", "lang": "en", "key": key, "print": key, "seq": 1,
          "root": key, "machine": key, "since": "2026-11-03T12:00:00Z", "certificate": signed, "signed": signed },
        { "of": key, "kept": false, "seq": 2,
          "root": key, "machine": key, "since": "2026-11-03T12:00:00Z", "certificate": signed, "signed": signed },
    ]);

    let tmp = tempfile::tempdir().unwrap();
    let mut editor = Home::at(tmp.path());
    another_person(&mut editor);
    let body = one.document["body"].as_str().unwrap().to_owned();
    let copy = editor.wrote(&format!("{body}Y otra cosa.\n"), None);
    let file = editor.state.docs[&copy].file.clone();
    let arrived = Remembered {
        admin_token: None,
        owner: reading::owner_block(&reading::opened(main, &opening, &served).unwrap()),
        owner_name: Some("Mario".to_owned()),
        carried: Some(
            serde_json::json!({
                "reviews": reviews,
                "set_aside": [{ "lang": "en", "document": "aparte01", "key": key }],
            })
            .to_string(),
        ),
        ..shared.clone()
    };
    editor.sent(&[(file, arrived)]);
    let nobody = || Err(Unshared::Unread("it has its id".to_owned()));
    let later = Under {
        mint: &nobody,
        editable: false,
        terms: None,
        languages_at_most: None,
    };
    let second = editor.sealed(signed_as(&editor, copy, &later, "Ana"), vec![]);
    let two = reading::reading(&second.going[0].sealed.main, &opening, &served).unwrap();

    assert_eq!(two.document["reviews"], reviews);
    assert!(
        !two.document.to_string().contains("aparte01"),
        "what the owner set aside never travels"
    );

    editor.sent(&second.remembering);
    let (file, now) = second.remembering[0].clone();
    let mut newer: Vec<tisty_contracts::Review> = serde_json::from_value(reviews.clone()).unwrap();
    newer[1].seq = 3;
    let carried = tisty_sharing::with_reviews(now.carried.as_deref(), &newer);
    editor.sent(&[(file, Remembered { carried, ..now })]);
    let third = editor
        .sealed(signed_as(&editor, copy, &later, "Ana"), vec![])
        .only_what_changed();
    assert!(
        third.going.is_empty(),
        "someone who edits only carries the owner's word"
    );
}

#[test]
fn what_is_public_names_no_editor() {
    let (_tmp, home, doc) = written();
    let minted = Cell::new(0);
    let mint = given(&minted);
    let under = Under {
        mint: &mint,
        editable: false,
        terms: None,
        languages_at_most: None,
    };
    let desk = Desk {
        paths: &home.paths,
        state: &home.state,
        also: None,
        turned: &|_| None,
        maker: None,
    };
    let asked = tisty_sharing::Asked {
        public: true,
        signed_as: Some("Mario".to_owned()),
        left_out: Vec::new(),
        ..asked(doc)
    };
    let sealed = home.sealed(gathered_under(&desk, &asked, &under).unwrap(), vec![]);
    let opening = Opening {
        id: GIVEN.to_owned(),
        kind: EnvelopeKind::Main,
        link_key: None,
    };
    let served = Served {
        received: None,
        revocations: None,
        reviewed: None,
        reviewed_received: None,
    };
    let read = reading::reading(&sealed.going[0].sealed.main, &opening, &served).unwrap();
    assert!(read.document.get("editors").is_none());
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
                terms: None,
                languages_at_most: None,
            },
        )
        .unwrap(),
        vec![],
    );

    assert_eq!(sealed.remembering[0].1.held_by, None);
}

#[test]
fn the_pages_of_whoever_edits_through_a_link_go_with_their_version() {
    let (_tmp, mut home, doc) = written();
    let body = "# Una página\n\nPalabras.\n";
    let made = tisty_core::docs::create(&home.paths.docs(), &home.device, body).unwrap();
    home.commit(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            file: made.id,
            order: "a9".to_owned(),
            said: Some(tisty_core::event::Said::of(body)),
            guest: true,
            sown: false,
            wrote: None,
            made: None,
            by: None,
            folder: None,
            page_of: Some(doc),
            task_of: None,
            edited: None,
        },
    });
    let file = home.state.docs[&doc].file.clone();
    let mut entry = Remembered {
        id: GIVEN.to_owned(),
        version: 1,
        link_key: Some("a".repeat(43)),
        files_print: None,
        files_hash: None,
        files_key: None,
        admin_token: None,
        edit_token: Some("b".repeat(43)),
        words_print: None,
        signed_as: None,
        owner: None,
        owner_name: None,
        cited: Vec::new(),
        held_by: None,
        ttl_days: None,
        carried: None,
        sealed_under: None,
        version_by: None,
        version_root: None,
        left_out: Vec::new(),
    };
    entry.link_key = reading_key();
    home.sent(&[(file, entry)]);
    let nobody = || Err(Unshared::Unread("it has its id".to_owned()));

    let gathered = under_folio(
        &home,
        doc,
        &Under {
            mint: &nobody,
            editable: false,
            terms: None,
            languages_at_most: None,
        },
    )
    .unwrap();

    assert!(gathered.told().is_empty(), "{:?}", gathered.told());
}

fn reading_key() -> Option<String> {
    tisty_envelope::sealing::link_key().ok()
}

fn counted(minted: &Cell<u32>) -> impl Fn() -> Result<String, Unshared> + '_ {
    || {
        minted.set(minted.get() + 1);
        Ok(format!("Language{}", minted.get()))
    }
}

fn two_languages() -> (tempfile::TempDir, Home, DocId) {
    let tmp = tempfile::tempdir().unwrap();
    let mut home = Home::at(tmp.path());
    home.with_a_name();
    let es = home.wrote("---\nlang: es\n---\n\n# Viaje\n", None);
    let en = home.wrote("---\nlang: en\n---\n\n# Trip\n", None);
    home.joined(en, es);
    (tmp, home, es)
}

fn the_map(sealed: &tisty_sharing::Sealed, anchor: &Remembered) -> serde_json::Value {
    let going = sealed.going.iter().find(|one| one.id == anchor.id).unwrap();
    let opening = Opening {
        id: anchor.id.clone(),
        kind: EnvelopeKind::Main,
        link_key: anchor.link_key.clone(),
    };
    let served = Served {
        received: None,
        revocations: None,
        reviewed: None,
        reviewed_received: None,
    };
    reading::reading(&going.sealed.main, &opening, &served)
        .unwrap()
        .document["languages"]
        .clone()
}

#[test]
fn each_language_shared_to_edit_has_its_token_and_only_the_anchor_s_opens_it() {
    let (_tmp, home, es) = two_languages();
    let minted = Cell::new(0);
    let mint = counted(&minted);
    let under = Under {
        mint: &mint,
        editable: true,
        terms: None,
        languages_at_most: None,
    };

    let sealed = home.sealed(under_folio(&home, es, &under).unwrap(), vec![]);
    let anchor = &sealed
        .remembering
        .iter()
        .find(|(_, one)| one.held_by.is_none())
        .unwrap()
        .1;
    let other = &sealed
        .remembering
        .iter()
        .find(|(_, one)| one.held_by.is_some())
        .unwrap()
        .1;
    let map = the_map(&sealed, anchor);

    let token = other
        .edit_token
        .as_deref()
        .expect("the language has its token");
    assert_ne!(Some(token), anchor.edit_token.as_deref());
    let edit = map[0]["edit"].as_str().expect("the map carries it sealed");
    assert_eq!(
        tisty_envelope::tokens::opened_with(anchor.edit_token.as_deref().unwrap(), &other.id, edit)
            .as_deref(),
        Some(token)
    );
    assert!(sealed.granted.is_empty(), "nothing was in Folio yet");
}

#[test]
fn a_language_shared_before_its_token_gets_one_and_folio_is_told_its_permit() {
    let (_tmp, mut home, es) = two_languages();
    let minted = Cell::new(0);
    let mint = counted(&minted);
    let first = home.sealed(
        under_folio(
            &home,
            es,
            &Under {
                mint: &mint,
                editable: true,
                terms: None,
                languages_at_most: None,
            },
        )
        .unwrap(),
        vec![],
    );
    let without: Vec<(String, Remembered)> = first
        .remembering
        .iter()
        .map(|(file, one)| {
            let mut one = one.clone();
            if one.held_by.is_some() {
                one.edit_token = None;
            }
            (file.clone(), one)
        })
        .collect();
    home.sent(&without);

    let later = home.sealed(
        under_folio(
            &home,
            es,
            &Under {
                mint: &mint,
                editable: false,
                terms: None,
                languages_at_most: None,
            },
        )
        .unwrap(),
        vec![],
    );
    let other = &later
        .remembering
        .iter()
        .find(|(_, one)| one.held_by.is_some())
        .unwrap()
        .1;

    let anchor = &later
        .remembering
        .iter()
        .find(|(_, one)| one.held_by.is_none())
        .unwrap()
        .1;

    assert!(other.edit_token.is_some());
    assert_eq!(
        later.granted,
        std::collections::BTreeSet::from([other.id.clone()])
    );
    assert!(later.changed.contains(&other.id));
    assert!(
        later.changed.contains(&anchor.id),
        "the map with its token stayed behind"
    );
}

#[test]
fn a_sync_that_changes_one_language_sends_only_that_one() {
    let (_tmp, mut home, es) = two_languages();
    let minted = Cell::new(0);
    let mint = counted(&minted);
    let first = home.sealed(
        under_folio(
            &home,
            es,
            &Under {
                mint: &mint,
                editable: true,
                terms: None,
                languages_at_most: None,
            },
        )
        .unwrap(),
        vec![],
    );
    home.sent(&first.remembering);
    let english = first
        .remembering
        .iter()
        .find(|(_, one)| one.held_by.is_some())
        .unwrap()
        .clone();
    std::fs::write(
        home.paths.docs().join(format!("{}.md", english.0)),
        "---
lang: en
---

# Trip

More words.
",
    )
    .unwrap();

    let later = home.sealed(
        under_folio(
            &home,
            es,
            &Under {
                mint: &mint,
                editable: false,
                terms: None,
                languages_at_most: None,
            },
        )
        .unwrap(),
        vec![],
    );

    assert_eq!(
        later.changed,
        std::collections::BTreeSet::from([english.1.id.clone()])
    );
}

#[test]
fn what_went_out_to_read_is_not_made_editable_by_sharing_it_again() {
    let (_tmp, mut home, es) = two_languages();
    let minted = Cell::new(0);
    let mint = counted(&minted);
    let first = home.sealed(
        under_folio(
            &home,
            es,
            &Under {
                mint: &mint,
                editable: false,
                terms: None,
                languages_at_most: None,
            },
        )
        .unwrap(),
        vec![],
    );
    home.sent(&first.remembering);

    let again = home.sealed(
        under_folio(
            &home,
            es,
            &Under {
                mint: &mint,
                editable: true,
                terms: None,
                languages_at_most: None,
            },
        )
        .unwrap(),
        vec![],
    );

    assert!(
        again
            .remembering
            .iter()
            .all(|(_, one)| one.edit_token.is_none())
    );
    assert!(again.granted.is_empty());
}
