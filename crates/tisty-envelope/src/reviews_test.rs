use serde_json::Value;
use tisty_contracts::{EnvelopeKind, Opening, Review, Served};

use super::{Joining, LeftOut, Weighing, decoded, holds, joined, keeps, kept, print_of};

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

#[test]
fn the_owner_takes_in_only_what_its_root_signed_and_keeps_one_review_for_each_language() {
    let (one, mine) = scenario("a language its owner keeps, as its owner read it");
    let (_, later) = scenario("a language its owner left out");
    let (_, by_someone_else) = scenario("a review signed by someone who edits");
    let joining = Joining {
        anchor: one["anchor"].as_str().unwrap(),
        owner: one["owner"].as_str().unwrap(),
        anchor_key: one["link_key"].as_str().unwrap(),
    };

    let after = joined(&joining, &mine, &later);
    assert_eq!(after.len(), 1);
    assert!(!after[0].kept);
    assert_eq!(after[0].seq, 2);

    let kept_still = joined(&joining, &mine, &by_someone_else);
    assert_eq!(kept_still, mine);
}

fn owners_machine(
    root: u8,
    machine: u8,
) -> (
    tisty_share_proto::identity::ShareKey,
    tisty_share_proto::identity::Certificate,
) {
    use tisty_share_proto::identity::{Root, ShareKey};
    let key = ShareKey::from_bytes(&[machine; 32]);
    let since = "2026-11-03T12:00:00Z".parse().unwrap();
    let certificate = Root::from_bytes(&[root; 32]).certify(&key.public(), since);
    (key, certificate)
}

fn encoded(byte: u8) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([byte; 32])
}

#[test]
fn what_the_owner_signs_is_what_a_reader_counts() {
    use super::{Read, Signer, review_signed, word_signed};
    let (key, certificate) = owners_machine(0x30, 0x31);
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };
    let anchor_key = encoded(0x11);
    let (its_key, print) = (encoded(0x12), encoded(0x13));
    let read = Read {
        lang: "en",
        title: Some("Walks"),
        key: &its_key,
        print: &print,
    };
    let keeping =
        review_signed(&signer, "anclaDoc", &anchor_key, "anaEngl1", Some(&read), 1).unwrap();
    let leaving = review_signed(&signer, "anclaDoc", &anchor_key, "anaPort1", None, 1).unwrap();
    let reviews = vec![keeping.clone(), leaving];
    let served = Served {
        received: None,
        revocations: None,
        reviewed: Some(word_signed(&signer, "anclaDoc", 2, &reviews).unwrap()),
        reviewed_received: None,
    };
    let weighing = Weighing {
        anchor: "anclaDoc",
        owner: &certificate.root,
        anchor_key: &anchor_key,
        served: &served,
        seen: 0,
    };

    let counted = kept(&weighing, &reviews).unwrap();

    assert_eq!(counted.seq, 2);
    assert_eq!(counted.reviews, [keeping]);
    assert!(keeps(&counted.reviews, "anaEngl1", Some("en"), &print));
    let gone = LeftOut::by(&anchor_key, "anclaDoc", &certificate.root, &reviews).unwrap();
    assert!(gone.names("anaPort1"));
}

#[test]
fn a_review_signed_again_by_another_machine_of_the_owner_still_holds() {
    use super::{Signer, review_signed, signed_again};
    let (old_key, old_certificate) = owners_machine(0x30, 0x31);
    let (new_key, new_certificate) = owners_machine(0x30, 0x32);
    let anchor_key = encoded(0x11);
    let old = Signer {
        key: &old_key,
        certificate: &old_certificate,
    };
    let first = review_signed(&old, "anclaDoc", &anchor_key, "anaPort1", None, 3).unwrap();

    let again = signed_again(
        &Signer {
            key: &new_key,
            certificate: &new_certificate,
        },
        "anclaDoc",
        &first,
    )
    .unwrap();

    assert_ne!(again.machine, first.machine);
    assert_eq!(
        (again.of.as_str(), again.seq, again.kept),
        (first.of.as_str(), 3, false)
    );
    assert!(holds(
        "anclaDoc",
        &old_certificate.root,
        &decoded(&anchor_key).unwrap(),
        &again
    ));
}

#[test]
fn the_next_word_on_a_language_follows_the_highest_one_said_of_it() {
    use super::{next_seq, with_review};
    let (_, reviews) = scenario("kept and then left out: the higher seq counts");
    let of = reviews[0].of.clone();

    assert_eq!(next_seq(&reviews, &of), 3);
    assert_eq!(next_seq(&reviews, "otro"), 1);
    let one = with_review(&reviews, reviews[1].clone());
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].seq, 2);
}

fn word_of(of: &str, kept: bool, seq: u64) -> Review {
    serde_json::from_value(serde_json::json!({
        "of": of,
        "kept": kept,
        "seq": seq,
        "root": "r",
        "machine": "m",
        "since": "2026-11-03T12:00:00Z",
        "certificate": "c",
        "signed": "s",
    }))
    .unwrap()
}

#[test]
fn past_what_an_anchor_carries_only_the_oldest_words_that_left_a_language_out_go() {
    let mut reviews: Vec<Review> = (1..=63)
        .map(|seq| word_of(&format!("fuera{seq}"), false, seq))
        .collect();
    reviews.push(word_of("guardado", true, 1));

    let after = super::with_review(&reviews, word_of("nuevo", false, 100));

    assert_eq!(after.len(), super::AT_MOST);
    assert!(after.iter().all(|one| one.of != "fuera1"));
    assert!(after.iter().any(|one| one.of == "fuera2"));
    assert!(after.iter().any(|one| one.of == "guardado"));
    assert!(after.iter().any(|one| one.of == "nuevo"));
}
