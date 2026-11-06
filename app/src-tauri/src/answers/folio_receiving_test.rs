use super::{Link, link_of};

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
