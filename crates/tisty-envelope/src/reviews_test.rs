use serde_json::Value;
use tisty_contracts::{EnvelopeKind, Opening, Review, Served};

use super::{LeftOut, Weighing, decoded, holds, keeps, kept, print_of};

const VECTORS: &str = include_str!("../../../contracts/vectors/vectors.json");

fn vectors() -> Value {
    serde_json::from_str(VECTORS).unwrap()
}

#[test]
fn a_review_holds_as_the_contracts_vectors_say() {
    let all = vectors();
    let said = &all["reviews"];
    let (anchor, owner) = (
        said["anchor"].as_str().unwrap(),
        said["owner"].as_str().unwrap(),
    );
    let key = decoded(said["link_key"].as_str().unwrap()).unwrap();
    for one in said["cases"].as_array().unwrap() {
        let review: Review = serde_json::from_value(one["review"].clone()).unwrap();
        assert_eq!(
            holds(anchor, owner, &key, &review),
            one["holds"].as_bool().unwrap(),
            "{}",
            one["says"]
        );
    }
}

fn scenario(says: &str) -> (Value, Vec<Review>) {
    let all = vectors();
    let one = all["languages_shown"]
        .as_array()
        .unwrap()
        .iter()
        .find(|one| one["says"] == says)
        .unwrap()
        .clone();
    let reviews = serde_json::from_value(one["reviews"].clone()).unwrap();
    (one, reviews)
}

fn left_out(says: &str, owner: Option<&str>, reviews: Option<Vec<Review>>) -> Option<LeftOut> {
    let (one, theirs) = scenario(says);
    LeftOut::by(
        one["link_key"].as_str().unwrap(),
        one["anchor"].as_str().unwrap(),
        owner.unwrap_or(one["owner"].as_str().unwrap()),
        &reviews.unwrap_or(theirs),
    )
}

#[test]
fn the_latest_word_of_each_language_decides_what_was_left_out() {
    let gone = left_out("a language its owner left out", None, None).unwrap();
    assert!(gone.names("anaEnglish"));
    assert!(!gone.names("anaPortugues"));
    let later = left_out("kept and then left out: the higher seq counts", None, None).unwrap();
    assert!(later.names("anaEnglish"));
    assert!(left_out("left out and then kept again", None, None).is_none());
}

#[test]
fn a_left_out_its_owner_did_not_sign_leaves_nothing_out() {
    let (_, mut reviews) = scenario("a language its owner left out");
    reviews[0].signed = reviews[0].certificate.clone();
    assert!(left_out("a language its owner left out", None, Some(reviews)).is_none());
    let anothers = Some("F8t5-ytBIPKx7GXkGY1uCLKOgT_rAeSkAIObheGAgM5");
    assert!(left_out("a language its owner left out", anothers, None).is_none());
}

fn shown(one: &Value) -> Vec<String> {
    let (anchor, owner) = (
        one["anchor"].as_str().unwrap(),
        one["owner"].as_str().unwrap(),
    );
    let reviews: Vec<Review> = serde_json::from_value(one["reviews"].clone()).unwrap();
    let served: Served = serde_json::from_value(one["served"].clone()).unwrap();
    let weighing = Weighing {
        anchor,
        owner,
        anchor_key: one["link_key"].as_str().unwrap(),
        served: &served,
        seen: one["seen"].as_u64().unwrap_or(0),
    };
    let kept = kept(&weighing, &reviews)
        .map(|counted| counted.reviews)
        .unwrap_or_default();
    let map = one["map"].as_array().unwrap();
    let own = |language: &Value| language["root"] == owner && map.contains(&language["document"]);
    let text = |language: &Value, name: &str| language[name].as_str().unwrap().to_owned();
    one["languages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|language| {
            own(language)
                || keeps(
                    &kept,
                    &text(language, "document"),
                    language["lang"].as_str(),
                    &text(language, "print"),
                )
        })
        .map(|language| text(language, "document"))
        .collect()
}

#[test]
fn a_reader_shows_what_the_contracts_vectors_say() {
    for one in vectors()["languages_shown"].as_array().unwrap() {
        assert_eq!(Value::from(shown(one)), one["shown"], "{}", one["says"]);
    }
}

#[test]
fn a_languages_print_is_the_one_the_contract_shows() {
    let all = vectors();
    let opening = Opening {
        id: "k7Qm2xPa".into(),
        kind: EnvelopeKind::Main,
        link_key: all["link_key"].as_str().map(str::to_owned),
    };
    let served = Served {
        received: None,
        revocations: None,
        reviewed: None,
        reviewed_received: None,
    };
    let hex = all["main"]["envelope"].as_str().unwrap();
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect();
    let opened = crate::reading::opened(&bytes, &opening, &served).unwrap();
    assert_eq!(
        print_of(&opened.parts),
        all["main"]["print"].as_str().unwrap()
    );
}
