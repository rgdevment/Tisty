use super::{opened_with, sealed_for};
use crate::sealing::link_key;

#[test]
fn only_the_anchor_s_token_opens_a_language_s_token_and_only_for_that_language() {
    let (anchor, token, stranger) = (
        link_key().unwrap(),
        link_key().unwrap(),
        link_key().unwrap(),
    );
    let sealed = sealed_for(&anchor, "enGlish1", &token).unwrap();

    assert_eq!(sealed.len(), 80);
    assert_eq!(opened_with(&anchor, "enGlish1", &sealed), Some(token));
    assert_eq!(opened_with(&stranger, "enGlish1", &sealed), None);
    assert_eq!(opened_with(&anchor, "fraNcais", &sealed), None);
    assert_eq!(opened_with(&anchor, "enGlish1", "not sealed"), None);
}

// Sealed apart with Node's crypto: HKDF-SHA256 (salt the language, info `tisty-language-token`), then AES-256-GCM.
#[test]
fn a_token_sealed_elsewhere_opens_to_the_same_token() {
    let anchor = "ERERERERERERERERERERERERERERERERERERERERERE";
    let token = "IiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiIiI";
    let sealed = "AAECAwQFBgcICQoLzVI-cQ68BEJL0wh0xtTzL63nJCgtNpZkYolhE2wElGRRyrm9vm--GCTKLXw2Yh0Q";

    assert_eq!(
        opened_with(anchor, "enGlish1", sealed).as_deref(),
        Some(token)
    );
    assert_eq!(opened_with(anchor, "fraNcais", sealed), None);
}

#[test]
fn sealing_twice_never_says_the_same_thing() {
    let (anchor, token) = (link_key().unwrap(), link_key().unwrap());
    assert_ne!(
        sealed_for(&anchor, "enGlish1", &token).unwrap(),
        sealed_for(&anchor, "enGlish1", &token).unwrap()
    );
}
