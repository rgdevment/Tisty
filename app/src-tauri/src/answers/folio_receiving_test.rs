use std::cell::Cell;

use tisty_carrier::folio_client::{Answered, Asked, Door};

use super::{Link, link_of, looked_for};

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
fn a_link_that_failed_says_why_and_only_a_failed_connection_blames_it() {
    use crate::refusing::edit_failed_words;
    for spanish in [true, false] {
        let connection = edit_failed_words(spanish, "folioUnreached");
        for code in [
            "folioGone",
            "folioBlocked",
            "folioPaused",
            "folioOutdated",
            "folioUnopened",
        ] {
            assert_ne!(edit_failed_words(spanish, code), connection, "{code}");
            assert_ne!(
                edit_failed_words(spanish, code),
                edit_failed_words(!spanish, code),
                "{code}"
            );
        }
    }
}

#[test]
fn the_question_before_looking_tells_who_will_be_asked_in_both_languages() {
    for spanish in [true, false] {
        let (words, look, leave) = crate::refusing::edit_look_words(spanish);
        assert!(words.contains("Folio"));
        assert!(words.contains(leave));
        assert_ne!(look, leave);
    }
    assert_ne!(
        crate::refusing::edit_look_words(true).0,
        crate::refusing::edit_look_words(false).0
    );
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
