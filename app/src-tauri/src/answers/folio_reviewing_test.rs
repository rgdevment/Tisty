use std::collections::BTreeSet;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use tisty_envelope::reviews::{self, LeftOut, Signer, Weighing};
use tisty_share_proto::identity::{Certificate, Root, ShareKey};
use tisty_sharing::Remembered;

use super::{Decided, proposals_of, reviewed_now};

fn owners_machine(root: u8, machine: u8) -> (ShareKey, Certificate) {
    let key = ShareKey::from_bytes(&[machine; 32]);
    let since = "2026-11-03T12:00:00Z".parse().unwrap();
    let certificate = Root::from_bytes(&[root; 32]).certify(&key.public(), since);
    (key, certificate)
}

fn encoded(byte: u8) -> String {
    URL_SAFE_NO_PAD.encode([byte; 32])
}

fn anchor() -> Remembered {
    let map = serde_json::json!({
        "languages": [
            { "lang": "fr", "document": "marioFr1", "key": encoded(0x14) },
            { "lang": "en", "document": "anaEngl1", "key": encoded(0x12), "pending": true },
        ],
    });
    Remembered {
        id: "anclaDoc".to_owned(),
        version: 3,
        link_key: Some(encoded(0x11)),
        files_print: None,
        files_hash: None,
        files_key: None,
        admin_token: Some("admin".to_owned()),
        edit_token: None,
        words_print: None,
        signed_as: None,
        owner: None,
        owner_name: None,
        cited: Vec::new(),
        held_by: None,
        ttl_days: None,
        carried: Some(map.to_string()),
        sealed_under: None,
        version_by: None,
        version_root: None,
        left_out: Vec::new(),
    }
}

fn read_as_shown() -> Decided {
    Decided {
        lang: "en".to_owned(),
        title: Some("Walks".to_owned()),
        key: encoded(0x12),
        print: encoded(0x13),
    }
}

fn ours() -> BTreeSet<&'static str> {
    BTreeSet::from(["anclaDoc", "marioFr1"])
}

#[test]
fn what_the_owner_keeps_is_signed_so_a_reader_counts_it_and_the_list_says_so() {
    let (key, certificate) = owners_machine(0x30, 0x31);
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };
    let was = anchor();
    assert_eq!(proposals_of(&was, &ours()).len(), 1);
    assert!(!proposals_of(&was, &ours())[0].kept);

    let now = reviewed_now(&signer, &was, "anaEngl1", Some(&read_as_shown())).unwrap();

    let reviews = tisty_sharing::reviews_carried(now.carried.as_deref());
    let word = reviews::word_signed(&signer, "anclaDoc", 1, &reviews).unwrap();
    let served = tisty_contracts::Served {
        received: None,
        revocations: None,
        reviewed: Some(word),
        reviewed_received: None,
    };
    let anchor_key = encoded(0x11);
    let weighing = Weighing {
        anchor: "anclaDoc",
        owner: &certificate.root,
        anchor_key: &anchor_key,
        served: &served,
        seen: 0,
    };
    let counted = reviews::kept(&weighing, &reviews).unwrap();
    assert!(reviews::keeps(
        &counted.reviews,
        "anaEngl1",
        Some("en"),
        &encoded(0x13)
    ));
    let listed = proposals_of(&now, &ours());
    assert_eq!(listed.len(), 1);
    assert!(listed[0].kept);
}

#[test]
fn a_language_left_out_leaves_the_map_and_its_word_stays_behind() {
    let (key, certificate) = owners_machine(0x30, 0x31);
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };

    let now = reviewed_now(&signer, &anchor(), "anaEngl1", None).unwrap();

    assert!(proposals_of(&now, &ours()).is_empty());
    let reviews = tisty_sharing::reviews_carried(now.carried.as_deref());
    assert_eq!(reviews.len(), 1);
    let gone = LeftOut::by(&encoded(0x11), "anclaDoc", &certificate.root, &reviews).unwrap();
    assert!(gone.names("anaEngl1"));
}

#[test]
fn a_second_word_on_one_language_takes_the_place_of_the_first_and_counts_higher() {
    let (key, certificate) = owners_machine(0x30, 0x31);
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };
    let kept = reviewed_now(&signer, &anchor(), "anaEngl1", Some(&read_as_shown())).unwrap();

    let left = reviewed_now(&signer, &kept, "anaEngl1", None).unwrap();

    let reviews = tisty_sharing::reviews_carried(left.carried.as_deref());
    assert_eq!(reviews.len(), 1);
    assert_eq!((reviews[0].seq, reviews[0].kept), (2, false));
}
