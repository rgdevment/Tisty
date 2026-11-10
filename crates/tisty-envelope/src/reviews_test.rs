use serde_json::Value;
use tisty_contracts::Review;

use super::{LeftOut, decoded, holds};

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
        one["link_key"].as_str(),
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
