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
