use std::cell::Cell;

use tisty_carrier::folio_client::{Answered, Asked, Door};

use super::languages_brought;
use super::{
    Link, Opened, belongs, editable, language_link, link_of, looked_for, owner_apart, reviewed,
    saying_its_language, signer_alias, speaks,
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

    let (brought, left) = languages_brought(&folio, &anchor, &["fr".to_owned()].into());

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
        &naming(serde_json::json!({ "anchor": "Anchor12" }), "mario")
    ));
    assert!(!belongs(
        &anchor,
        &naming(serde_json::json!({ "anchor": "Another1" }), "mario")
    ));
    assert!(!belongs(&anchor, &naming(serde_json::json!({}), "mario")));
    assert!(!belongs(
        &anchor,
        &naming(serde_json::json!({ "anchor": "Anchor12" }), "ana")
    ));
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
