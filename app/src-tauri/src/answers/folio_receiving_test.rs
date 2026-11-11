use std::cell::Cell;

use tisty_carrier::folio_client::{Answered, Asked, Door, Fetched};
use tisty_envelope::Translation;

use super::languages_brought;
use super::{
    Link, Opened, belongs, editable, editable_language, granted_entry, kept_reviews, language_by,
    language_link, link_of, looked_for, owner_apart, reached, reviewed, saying_its_language,
    served_by, signer_alias, speaks,
};

#[derive(Default)]
struct CountingFolio(Cell<u32>);

impl Door for CountingFolio {
    fn ask(&self, _: &Asked<'_>) -> Option<Answered> {
        self.0.set(self.0.get() + 1);
        None
    }
}

fn some_link() -> Link {
    parsed("https://tisty.cl/Ab3xK9pQ#7Zq2").unwrap()
}

#[test]
fn nothing_reaches_folio_until_the_person_says_yes() {
    let folio = CountingFolio::default();

    let told = looked_for(&folio, &some_link(), || false);

    assert!(matches!(told, Ok(None)));
    assert_eq!(folio.0.get(), 0);
}

#[test]
fn on_a_yes_it_asks_folio_for_that_link() {
    let folio = CountingFolio::default();

    let told = looked_for(&folio, &some_link(), || true);

    assert!(told.is_err());
    assert_eq!(folio.0.get(), 1);
}

#[test]
fn a_language_folio_could_not_hand_over_is_named_as_left_behind() {
    let folio = CountingFolio::default();
    let named = |lang: &str, key: Option<&str>| serde_json::json!({ "lang": lang, "document": format!("{lang}Docum1"), "key": key });
    let anchor = Opened {
        document: serde_json::json!({
            "id": "Anchor12",
            "languages": [named("en", Some("k")), named("fr", Some("k")), named("de", None)],
        }),
        ..opened_by(Some("mario"), "mario")
    };

    let (brought, left) = languages_brought(&folio, &anchor, &["fr".to_owned()].into(), &[]);

    assert!(brought.is_empty());
    assert_eq!(left, vec!["en".to_owned()]);
}

fn parsed(said: &str) -> Option<Link> {
    link_of(&url::Url::parse(said).unwrap())
}

#[test]
fn a_link_to_read_gives_its_id_and_key() {
    assert_eq!(
        parsed("https://tisty.cl/Ab3xK9pQ#key"),
        Some(Link {
            id: "Ab3xK9pQ".to_owned(),
            key: "key".to_owned(),
            token: None,
        })
    );
}

#[test]
fn a_link_to_edit_also_gives_its_token() {
    assert_eq!(
        parsed("https://tisty.cl/Ab3xK9pQ#key.token").and_then(|one| one.token),
        Some("token".to_owned())
    );
}

#[test]
fn what_has_no_key_or_no_id_of_folio_is_no_link() {
    for said in [
        "https://tisty.cl/Ab3xK9pQ",
        "https://tisty.cl/Ab3xK9pQ#",
        "https://tisty.cl/short#key",
        "https://tisty.cl/Ab3x/K9pQ#key",
        "https://tisty.cl/#key",
    ] {
        assert_eq!(parsed(said), None, "{said}");
    }
}

#[test]
fn a_name_that_is_no_domain_asks_nothing_and_does_not_wait() {
    let home = tempfile::tempdir().unwrap();
    let began = std::time::Instant::now();
    let checked = super::checked_within(
        home.path(),
        &[
            ("R".repeat(43).as_str(), Some("Ana")),
            ("O".repeat(43).as_str(), None),
        ],
    );
    assert!(checked.is_empty());
    assert!(began.elapsed() < super::DOMAIN_PATIENCE);
}

fn opened_by(owner_root: Option<&str>, signer: &str) -> Opened {
    Opened {
        document: serde_json::json!({}),
        pictures: Vec::new(),
        files: Vec::new(),
        owner: owner_root.map(|root| format!("{{\"root\":\"{root}\",\"signed\":\"x\"}}")),
        owner_root: owner_root.map(str::to_owned),
        signer: signer.to_owned(),
        version: 1,
        verdict: tisty_contracts::Verdict::Holds,
        contract: None,
        served: tisty_contracts::Served {
            received: None,
            revocations: None,
            reviewed: None,
            reviewed_received: None,
        },
        print: None,
    }
}

#[test]
fn the_owner_is_named_apart_only_when_another_root_signed_this_version() {
    assert_eq!(owner_apart(&opened_by(Some("mario"), "mario")), None);
    assert_eq!(owner_apart(&opened_by(Some("mario"), "ana")), Some("mario"));
    assert_eq!(owner_apart(&opened_by(None, "ana")), None);
}

#[test]
fn a_token_the_owner_s_block_does_not_name_is_kept_as_a_link_to_read() {
    let token = tisty_envelope::sealing::link_key().unwrap();
    let permit = tisty_envelope::sealing::permit_shown(&token).unwrap();
    let opened = |named: &str| Opened {
        owner: Some(format!(r#"{{"root":"r","permit":"{named}"}}"#)),
        ..opened_by(Some("r"), "r")
    };
    let link = |token: Option<&str>| Link {
        id: "abcd1234".into(),
        key: "key".into(),
        token: token.map(str::to_owned),
    };

    assert!(editable(&link(Some(&token)), &opened(&permit)));
    assert!(!editable(&link(Some("anything")), &opened(&permit)));
    assert!(!editable(&link(Some(&token)), &opened("anotherPermit")));
    assert!(!editable(&link(None), &opened(&permit)));
}

#[test]
fn the_anchor_s_edit_link_opens_each_language_s_token_its_owner_names() {
    use tisty_envelope::sealing::{link_key, permit_shown};
    let (anchor, token) = (link_key().unwrap(), link_key().unwrap());
    let key = link_key().unwrap();
    let one = tisty_envelope::Translation {
        lang: "en".into(),
        document: "enGlish1".into(),
        title: None,
        key: Some(key.clone()),
        edit: Some(tisty_envelope::tokens::sealed_for(&anchor, "enGlish1", &token).unwrap()),
        pending: None,
    };
    let its = |permit: &str| Opened {
        owner: Some(format!(r#"{{"root":"r","permit":"{permit}"}}"#)),
        ..opened_by(Some("r"), "r")
    };
    let link = |token: Option<&str>| Link {
        id: "anchor12".into(),
        key: "key".into(),
        token: token.map(str::to_owned),
    };
    let named = permit_shown(&token).unwrap();

    let opened = language_link(&link(Some(&anchor)), &one, &its(&named)).unwrap();
    assert_eq!(opened.id, "enGlish1");
    assert_eq!(opened.key, key);
    assert_eq!(opened.token.as_deref(), Some(token.as_str()));
    assert!(language_link(&link(Some(&anchor)), &one, &its("anotherPermit")).is_none());
    assert!(language_link(&link(None), &one, &its(&named)).is_none());
    assert!(language_link(&link(Some(&token)), &one, &its(&named)).is_none());
}

#[test]
fn a_language_of_another_root_is_never_edited_through_the_anchor_s_link() {
    use tisty_envelope::sealing::{link_key, permit_shown};
    let (anchor_token, token) = (link_key().unwrap(), link_key().unwrap());
    let one = tisty_envelope::Translation {
        lang: "en".into(),
        document: "enGlish1".into(),
        title: None,
        key: Some(link_key().unwrap()),
        edit: Some(tisty_envelope::tokens::sealed_for(&anchor_token, "enGlish1", &token).unwrap()),
        pending: None,
    };
    let anchor = Opened {
        owner: Some(format!(
            r#"{{"root":"mario","permit":"{}"}}"#,
            permit_shown(&anchor_token).unwrap()
        )),
        ..opened_by(Some("mario"), "mario")
    };
    let its = |root: &str| Opened {
        owner: Some(format!(
            r#"{{"root":"{root}","permit":"{}"}}"#,
            permit_shown(&token).unwrap()
        )),
        ..opened_by(Some(root), root)
    };
    let link = Link {
        id: "anchor12".into(),
        key: "key".into(),
        token: Some(anchor_token.clone()),
    };

    assert!(editable_language(&link, &anchor, &one, &its("mario")).is_some());
    assert!(editable_language(&link, &anchor, &one, &its("ana")).is_none());
}

fn edited_anchor(token: &str) -> tisty_sharing::Remembered {
    tisty_sharing::Remembered {
        id: "anchor12".into(),
        version: 4,
        link_key: Some("key".into()),
        files_print: None,
        files_hash: None,
        files_key: None,
        admin_token: None,
        edit_token: Some(token.to_owned()),
        words_print: None,
        signed_as: None,
        owner: Some(r#"{"root":"mario","permit":"p"}"#.into()),
        owner_name: None,
        cited: Vec::new(),
        held_by: None,
        ttl_days: None,
        carried: None,
        sealed_under: None,
        version_by: None,
        version_root: None,
        left_out: Vec::new(),
    }
}

#[test]
fn only_a_copy_alone_in_its_language_whose_token_opens_here_is_looked_for_in_folio() {
    use tisty_envelope::sealing::link_key;
    let (anchor, token, stranger) = (
        link_key().unwrap(),
        link_key().unwrap(),
        link_key().unwrap(),
    );
    let sealed = |under: &str, id: &str| tisty_envelope::tokens::sealed_for(under, id, &token).ok();
    let one = |lang: &str, id: &str, edit: Option<String>| serde_json::json!({ "lang": lang, "document": id, "key": "k", "edit": edit });
    let was = tisty_sharing::Remembered {
        carried: Some(
            serde_json::json!({ "languages": [
                one("en", "enGlish1", sealed(&anchor, "enGlish1")),
                one("fr", "fraNcais", sealed(&stranger, "fraNcais")),
                one("de", "deUtsch1", None),
                one("pt", "portUgu1", sealed(&anchor, "portUgu1")),
            ]})
            .to_string(),
        ),
        ..edited_anchor(&anchor)
    };
    let here = |lang: &str, file: &str| (lang.to_owned(), file.to_owned());
    let to_read = [
        here("en", "en.md"),
        here("fr", "fr.md"),
        here("de", "de.md"),
        here("pt", "pt.md"),
        here("pt", "pt-2.md"),
    ];

    let found: Vec<(String, String)> = super::grantable(&was, &super::carried_map(&was), &to_read)
        .into_iter()
        .map(|(file, one)| (file, one.document))
        .collect();

    assert_eq!(found, vec![("en.md".to_owned(), "enGlish1".to_owned())]);
    let opened_again = super::carried_map(&was);
    assert_eq!(
        super::grantable(&edited_anchor(&anchor), &opened_again, &to_read).len(),
        1,
        "a link opened again hands its map over before the anchor here holds it"
    );
    let under_another: Vec<String> =
        super::grantable(&edited_anchor(&stranger), &opened_again, &to_read)
            .into_iter()
            .map(|(file, _)| file)
            .collect();
    assert_eq!(under_another, vec!["fr.md".to_owned()]);
    assert!(super::grantable(&edited_anchor(&anchor), &[], &to_read).is_empty());
}

#[test]
fn a_new_edit_link_its_owner_names_is_the_one_the_anchor_goes_on_with() {
    use tisty_envelope::sealing::{link_key, permit_shown};
    let (old, new) = (link_key().unwrap(), link_key().unwrap());
    let block = |token: &str| {
        format!(
            r#"{{"root":"mario","permit":"{}"}}"#,
            permit_shown(token).unwrap()
        )
    };
    let opened = Opened {
        owner: Some(block(&new)),
        ..opened_by(Some("mario"), "mario")
    };
    let link = |token: &str| Link {
        id: "anchor12".into(),
        key: "key".into(),
        token: Some(token.to_owned()),
    };

    let moved = super::moved_to(&edited_anchor(&old), &link(&new), &opened).unwrap();
    assert_eq!(moved.edit_token.as_deref(), Some(new.as_str()));
    assert_eq!(moved.owner, Some(block(&new)));

    assert!(super::moved_to(&edited_anchor(&new), &link(&new), &opened).is_none());
    assert!(super::moved_to(&edited_anchor(&old), &link(&old), &opened).is_none());
    let owned = tisty_sharing::Remembered {
        admin_token: Some("admin".into()),
        ..edited_anchor(&old)
    };
    assert!(super::moved_to(&owned, &link(&new), &opened).is_none());
}

#[test]
fn a_granted_language_without_its_print_is_never_kept_and_the_rest_always_is() {
    let entry = |print: Option<&str>| tisty_sharing::Remembered {
        words_print: print.map(str::to_owned),
        ..edited_anchor("t")
    };
    let granted = vec![
        ("en.md".to_owned(), entry(None)),
        ("fr.md".to_owned(), entry(None)),
    ];
    let printed = vec![
        ("anchor.md".to_owned(), entry(None)),
        ("en.md".to_owned(), entry(Some("abc"))),
        ("fr.md".to_owned(), entry(None)),
    ];
    let files = |all: Vec<(String, tisty_sharing::Remembered)>| -> Vec<String> {
        all.into_iter().map(|(file, _)| file).collect()
    };

    let (kept, left) = super::printed_apart(printed, &granted);

    assert_eq!(
        files(kept),
        vec!["anchor.md".to_owned(), "en.md".to_owned()]
    );
    assert_eq!(files(left), vec!["fr.md".to_owned()]);
}

#[test]
fn a_language_kept_to_read_is_edited_once_the_anchor_s_map_holds_its_token() {
    use tisty_envelope::sealing::{link_key, permit_shown};
    let (anchor, token) = (link_key().unwrap(), link_key().unwrap());
    let one = tisty_envelope::Translation {
        lang: "en".into(),
        document: "enGlish1".into(),
        title: None,
        key: Some("enKey".into()),
        edit: Some(tisty_envelope::tokens::sealed_for(&anchor, "enGlish1", &token).unwrap()),
        pending: None,
    };
    let its = |said: serde_json::Value, owner: &str, permit: &str| Opened {
        document: said,
        owner: Some(format!(r#"{{"root":"{owner}","permit":"{permit}"}}"#)),
        version: 7,
        ..opened_by(Some(owner), owner)
    };
    let named = permit_shown(&token).unwrap();
    let ours = serde_json::json!({ "anchor": "anchor12", "files_hash": "h" });

    let entry = granted_entry(
        &edited_anchor(&anchor),
        &one,
        &its(ours.clone(), "mario", &named),
    )
    .unwrap();
    assert_eq!(entry.id, "enGlish1");
    assert_eq!(entry.edit_token.as_deref(), Some(token.as_str()));
    assert_eq!(entry.held_by.as_deref(), Some("anchor12"));
    assert_eq!(entry.version, 0);
    assert_eq!(entry.files_hash, None);

    let elsewhere = serde_json::json!({ "anchor": "another1" });
    assert!(
        granted_entry(
            &edited_anchor(&anchor),
            &one,
            &its(elsewhere, "mario", &named)
        )
        .is_none()
    );
    assert!(
        granted_entry(
            &edited_anchor(&anchor),
            &one,
            &its(ours.clone(), "ana", &named)
        )
        .is_none()
    );
    assert!(
        granted_entry(
            &edited_anchor(&anchor),
            &one,
            &its(ours.clone(), "mario", "p")
        )
        .is_none()
    );
    assert!(granted_entry(&edited_anchor(&token), &one, &its(ours, "mario", &named)).is_none());
}

#[test]
fn the_language_shown_is_the_one_tisty_speaks() {
    assert!(speaks(Some("en"), Some("en-US")));
    assert!(speaks(Some("pt-BR"), Some("pt")));
    assert!(!speaks(Some("es"), Some("en")));
    assert!(!speaks(None, Some("en")));
}

#[test]
fn a_language_that_arrived_without_saying_it_says_it_in_its_words() {
    let arrived = |body: &str| tisty_sharing::Received {
        title: None,
        body: body.to_owned(),
        pages: Vec::new(),
        owner_name: None,
        signed_as: None,
        carried: None,
    };
    let said = saying_its_language(
        arrived(
            "# Trip
",
        ),
        Some("en"),
    );
    assert_eq!(
        tisty_markdown::front::lang(&said.body).as_deref(),
        Some("en")
    );
    let kept = saying_its_language(
        arrived(
            "---
lang: es
---
# Viaje
",
        ),
        Some("en"),
    );
    assert_eq!(
        tisty_markdown::front::lang(&kept.body).as_deref(),
        Some("es")
    );
}

#[test]
fn a_language_whose_owner_is_not_the_anchor_s_is_not_reviewed() {
    assert!(reviewed(
        &opened_by(Some("mario"), "mario"),
        &opened_by(Some("mario"), "ana")
    ));
    assert!(!reviewed(
        &opened_by(Some("mario"), "mario"),
        &opened_by(Some("ana"), "ana")
    ));
    assert!(!reviewed(
        &opened_by(None, "mario"),
        &opened_by(None, "ana")
    ));
}

#[test]
fn a_language_is_brought_only_when_it_names_this_anchor() {
    let anchor = Opened {
        document: serde_json::json!({ "id": "Anchor12" }),
        ..opened_by(Some("mario"), "mario")
    };
    let naming = |said: serde_json::Value, owner: &str| Opened {
        document: said,
        ..opened_by(Some(owner), owner)
    };

    assert!(belongs(
        &anchor,
        &naming(serde_json::json!({ "anchor": "Anchor12" }), "mario"),
        &[]
    ));
    assert!(!belongs(
        &anchor,
        &naming(serde_json::json!({ "anchor": "Another1" }), "mario"),
        &[]
    ));
    assert!(!belongs(
        &anchor,
        &naming(serde_json::json!({}), "mario"),
        &[]
    ));
    assert!(!belongs(
        &anchor,
        &naming(serde_json::json!({ "anchor": "Anchor12" }), "ana"),
        &[]
    ));
}

fn kept_review(document: &str, lang: &str, print: &str) -> tisty_contracts::Review {
    serde_json::from_value(serde_json::json!({
        "of": "o", "kept": true, "document": document, "lang": lang, "title": "Walks", "key": "enKey",
        "print": print, "seq": 1, "root": "mario", "machine": "m", "since": "2026-11-03T12:00:00Z",
        "certificate": "c", "signed": "s",
    }))
    .unwrap()
}

#[test]
fn a_language_of_someone_who_edits_is_brought_only_as_its_owner_kept_it() {
    let anchor = Opened {
        document: serde_json::json!({ "id": "Anchor12" }),
        ..opened_by(Some("mario"), "mario")
    };
    let anas = |lang: &str, print: &str| Opened {
        document: serde_json::json!({ "anchor": "Anchor12", "id": "anaEngl1", "lang": lang }),
        print: Some(print.to_owned()),
        ..opened_by(Some("ana"), "ana")
    };
    let kept = [kept_review("anaEngl1", "en", "P1")];

    assert!(belongs(&anchor, &anas("en", "P1"), &kept));
    assert!(!belongs(&anchor, &anas("en", "P2"), &kept));
    assert!(!belongs(&anchor, &anas("en-GB", "P1"), &kept));
    assert!(!belongs(&anchor, &anas("en", "P1"), &[]));
}

#[test]
fn a_language_its_owner_keeps_is_reached_by_what_the_owner_signed() {
    let map: Vec<Translation> = serde_json::from_value(serde_json::json!([
        { "lang": "fr", "document": "marioFr1", "key": "frKey" },
        { "lang": "en", "document": "anaEngl1", "title": "Otra cosa", "key": "wrong", "edit": "e" },
    ]))
    .unwrap();

    let reached = reached(
        map,
        &[
            kept_review("anaEngl1", "en", "P1"),
            kept_review("anaPort1", "pt-BR", "P2"),
        ],
    );

    assert_eq!(reached.len(), 3);
    assert_eq!(reached[0].document, "marioFr1");
    assert_eq!(reached[1].key.as_deref(), Some("enKey"));
    assert_eq!(reached[1].title.as_deref(), Some("Walks"));
    assert_eq!(reached[1].edit.as_deref(), Some("e"));
    assert_eq!(
        (reached[2].document.as_str(), reached[2].lang.as_str()),
        ("anaPort1", "pt-BR")
    );
    assert_eq!(reached[2].edit, None);
}

#[test]
fn what_folio_serves_beside_an_anchor_names_its_owners_word_and_when_it_arrived() {
    let fetched = Fetched {
        bytes: Vec::new(),
        version: 2,
        arrived_ms: 2_000,
        revocations: None,
        reviewed: Some(
            serde_json::json!({
                "anchor": "Anchor12", "seq": 3, "reviews": "h", "root": "r", "machine": "m",
                "since": "2026-11-03T12:00:00Z", "certificate": "c", "signed": "s",
            })
            .to_string(),
        ),
        reviewed_arrived_ms: Some(1_000),
    };

    let served = served_by(&fetched);

    assert_eq!(served.reviewed.unwrap().seq, 3);
    assert_eq!(
        served.reviewed_received.as_deref(),
        Some("1970-01-01T00:00:01Z")
    );
    assert_eq!(served.received.as_deref(), Some("1970-01-01T00:00:02Z"));
}

#[test]
fn a_version_is_signed_by_its_own_name_and_never_by_an_owner_who_did_not_sign_it() {
    let by_ana = opened_by(Some("mario"), "ana");
    let by_mario = opened_by(Some("mario"), "mario");

    assert_eq!(
        signer_alias(&by_ana, Some("Ana"), Some("Mario")),
        Some("Ana")
    );
    assert_eq!(signer_alias(&by_mario, None, Some("Mario")), Some("Mario"));
    assert_eq!(signer_alias(&by_ana, None, Some("Mario")), None);
    assert_eq!(signer_alias(&by_mario, None, None), None);
}

struct ServingFolio(Vec<(String, Vec<u8>)>);

impl Door for ServingFolio {
    fn ask(&self, asked: &Asked<'_>) -> Option<Answered> {
        let (_, body) = self.0.iter().find(|(path, _)| *path == asked.path)?;
        Some(Answered {
            status: 200,
            headers: vec![
                ("folio-version".to_owned(), "1".to_owned()),
                ("folio-arrived".to_owned(), "1800000000000".to_owned()),
            ],
            body: body.clone(),
        })
    }
}

struct Signer(
    tisty_share_proto::identity::ShareKey,
    tisty_share_proto::identity::Certificate,
);

fn signer(byte: u8) -> Signer {
    use tisty_share_proto::identity::{Root, ShareKey};
    let key = ShareKey::from_bytes(&[byte + 1; 32]);
    let certificate = Root::from_bytes(&[byte; 32])
        .certify(&key.public(), "2026-11-03T12:00:00Z".parse().unwrap());
    Signer(key, certificate)
}

const ANCHOR: &str = "Anchor12";

fn b64(said: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(said)
}

fn unb64(said: &str) -> Vec<u8> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(said)
        .unwrap()
}

fn sealed_language(by: &Signer, id: &str, lang: &str, key: &str) -> Vec<u8> {
    let document = serde_json::from_value(serde_json::json!({
        "id": id, "lang": lang, "anchor": ANCHOR, "html": "<p>hi</p>", "body": "hi\n"
    }))
    .unwrap();
    let sealing = tisty_contracts::Sealing {
        id: id.to_owned(),
        version: 1,
        from: None,
        link_key: Some(key.to_owned()),
        files_hash: None,
        files_key: None,
        owner: None,
        token: None,
        terms: None,
    };
    tisty_envelope::sealing::Writer::new(&by.0, &by.1)
        .sealed(document, vec![], vec![], &sealing)
        .unwrap()
        .main
}

fn print_of_sealed(main: &[u8], id: &str, key: &str) -> String {
    let opening = tisty_contracts::Opening {
        id: id.to_owned(),
        kind: tisty_contracts::EnvelopeKind::Main,
        link_key: Some(key.to_owned()),
    };
    let opened = tisty_envelope::reading::opened(main, &opening, &nothing_served()).unwrap();
    tisty_envelope::reviews::print_of(&opened.parts)
}

fn nothing_served() -> tisty_contracts::Served {
    tisty_contracts::Served {
        received: None,
        revocations: None,
        reviewed: None,
        reviewed_received: None,
    }
}

fn keeping(
    owner: &Signer,
    anchor_key: &str,
    id: &str,
    lang: &str,
    key: &str,
    print: &str,
) -> tisty_contracts::Review {
    use tisty_share_proto::reviewing::{self, Kept, Said};
    let of = reviewing::of(&unb64(anchor_key), id).unwrap();
    let (key_bytes, print_bytes) = (unb64(key), unb64(print));
    let said = Said {
        anchor: ANCHOR,
        of: &of,
        seq: 1,
        kept: Some(Kept {
            document: id,
            lang,
            title: Some("Walks"),
            key: &key_bytes,
            print: &print_bytes,
        }),
    };
    let signed = reviewing::signed(&owner.0, &owner.1, &said).unwrap();
    serde_json::from_value(serde_json::json!({
        "of": b64(&of), "kept": true, "document": id, "lang": lang, "title": "Walks", "key": key, "print": print,
        "seq": 1, "root": signed.certificate.root, "machine": signed.certificate.share,
        "since": "2026-11-03T12:00:00Z", "certificate": signed.certificate.sig, "signed": signed.sig,
    }))
    .unwrap()
}

fn owners_word(
    owner: &Signer,
    seq: u64,
    reviews: &[tisty_contracts::Review],
) -> tisty_contracts::Reviewed {
    use tisty_share_proto::reviewing;
    let hash = tisty_envelope::reviews::reviews_hash(ANCHOR, reviews).unwrap();
    let signed = owner
        .0
        .sign(
            &owner.1,
            reviewing::REVIEWED,
            &reviewing::reviewed(ANCHOR, seq, &hash),
        )
        .unwrap();
    serde_json::from_value(serde_json::json!({
        "anchor": ANCHOR, "seq": seq, "reviews": b64(&hash), "root": signed.certificate.root,
        "machine": signed.certificate.share, "since": "2026-11-03T12:00:00Z",
        "certificate": signed.certificate.sig, "signed": signed.sig,
    }))
    .unwrap()
}

#[test]
fn a_link_brings_the_language_its_owner_kept_by_the_key_the_owner_signed_and_one_for_each_lang() {
    use tisty_envelope::sealing::link_key;
    let (mario, ana) = (signer(0x30), signer(0x40));
    let anchor_key = link_key().unwrap();
    let (first_key, second_key) = (link_key().unwrap(), link_key().unwrap());
    let first = sealed_language(&ana, "anaEngl1", "en", &first_key);
    let second = sealed_language(&ana, "anaEngl2", "en", &second_key);
    let reviews = vec![
        keeping(
            &mario,
            &anchor_key,
            "anaEngl1",
            "en",
            &first_key,
            &print_of_sealed(&first, "anaEngl1", &first_key),
        ),
        keeping(
            &mario,
            &anchor_key,
            "anaEngl2",
            "en",
            &second_key,
            &print_of_sealed(&second, "anaEngl2", &second_key),
        ),
    ];
    let anchor = Opened {
        document: serde_json::json!({ "id": ANCHOR, "languages": [], "reviews": reviews }),
        served: tisty_contracts::Served {
            reviewed: Some(owners_word(&mario, 2, &reviews)),
            reviewed_received: Some("2026-11-05T00:00:00Z".into()),
            ..nothing_served()
        },
        ..opened_by(Some(&mario.1.root), &mario.1.root)
    };
    let folio = ServingFolio(vec![
        ("/docs/anaEngl1".to_owned(), first),
        ("/docs/anaEngl2".to_owned(), second),
    ]);

    let kept = kept_reviews(&anchor, &anchor_key, 0).unwrap();
    let (brought, left) = super::languages_brought(
        &folio,
        &anchor,
        &std::collections::BTreeSet::new(),
        &kept.reviews,
    );

    assert_eq!(kept.seq, 2);
    assert_eq!(brought.len(), 1);
    assert_eq!(brought[0].0.title.as_deref(), Some("Walks"));
    assert!(left.is_empty());
    assert!(kept_reviews(&anchor, &anchor_key, 3).is_none());
}

#[test]
fn a_language_of_the_anchor_s_root_goes_under_the_anchor_s_name_and_another_s_under_its_own() {
    let name = |said: &str| Some(said.to_owned());

    assert_eq!(
        language_by(Some("mario"), Some("mario"), name("Mario"), name("Ana")),
        name("Mario")
    );
    assert_eq!(
        language_by(Some("mario"), Some("ana"), name("Mario"), name("Ana")),
        name("Ana")
    );
    assert_eq!(
        language_by(None, None, name("Mario"), name("Ana")),
        name("Ana")
    );
}
