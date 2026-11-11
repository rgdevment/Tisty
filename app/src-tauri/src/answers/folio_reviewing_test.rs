use std::collections::BTreeSet;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use tisty_envelope::reviews::{self, LeftOut, Signer, Weighing};
use tisty_share_proto::identity::{Certificate, Root, ShareKey};
use tisty_sharing::Remembered;

use super::{
    Decided, as_shown, foreign_entry, named_here, pictures_of, proposals_of, reviewed_now,
};

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

    let now = reviewed_now(&signer, &was, "anaEngl1", Some(&read_as_shown()), 1).unwrap();

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

    let now = reviewed_now(&signer, &anchor(), "anaEngl1", None, 1).unwrap();

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
    let kept = reviewed_now(&signer, &anchor(), "anaEngl1", Some(&read_as_shown()), 1).unwrap();

    let left = reviewed_now(&signer, &kept, "anaEngl1", None, 1).unwrap();

    let reviews = tisty_sharing::reviews_carried(left.carried.as_deref());
    assert_eq!(reviews.len(), 1);
    assert_eq!((reviews[0].seq, reviews[0].kept), (2, false));
}

#[test]
fn a_word_counts_above_every_word_seen_for_the_anchor() {
    let (key, certificate) = owners_machine(0x30, 0x31);
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };

    let now = reviewed_now(&signer, &anchor(), "anaEngl1", None, 7).unwrap();

    let reviews = tisty_sharing::reviews_carried(now.carried.as_deref());
    assert_eq!(reviews[0].seq, 7);
}

#[test]
fn the_owner_s_word_goes_only_on_a_language_others_added_that_the_map_names() {
    let was = anchor();

    assert_eq!(foreign_entry(&was, &ours(), "anaEngl1").unwrap().lang, "en");
    assert_eq!(
        foreign_entry(&was, &ours(), "marioFr1").err().unwrap().code,
        "notOneOwnDoc"
    );
    assert_eq!(
        foreign_entry(&was, &ours(), "nobody12").err().unwrap().code,
        "folioGone"
    );
}

fn opened_with(
    said: serde_json::Value,
    print: Option<&str>,
) -> crate::answers::folio_receiving::Opened {
    crate::answers::folio_receiving::Opened {
        document: said,
        pictures: vec![vec![1, 2, 3]],
        files: Vec::new(),
        owner: None,
        owner_root: Some("ana".to_owned()),
        signer: "ana".to_owned(),
        version: 1,
        verdict: tisty_contracts::Verdict::Holds,
        contract: None,
        served: tisty_contracts::Served {
            received: None,
            revocations: None,
            reviewed: None,
            reviewed_received: None,
        },
        print: print.map(str::to_owned),
    }
}

#[test]
fn a_document_that_names_another_anchor_takes_no_word_of_this_one() {
    let its = |anchor: &str| {
        opened_with(
            serde_json::json!({ "anchor": anchor, "lang": "en" }),
            Some("P"),
        )
    };

    assert!(named_here(its("anclaDoc"), "anclaDoc").is_ok());
    assert_eq!(
        named_here(its("otroDoc1"), "anclaDoc").err().unwrap().code,
        "folioUnopened"
    );
}

#[test]
fn what_is_kept_is_the_version_shown_and_one_that_changed_is_shown_again() {
    let opened = opened_with(
        serde_json::json!({ "lang": "en", "title": "Walks" }),
        Some("P1"),
    );

    let decided = as_shown(&opened, "k".to_owned(), Some("P1")).unwrap();
    assert_eq!(
        (decided.lang.as_str(), decided.print.as_str()),
        ("en", "P1")
    );
    assert_eq!(
        as_shown(&opened, "k".to_owned(), Some("P0"))
            .err()
            .unwrap()
            .code,
        "folioLanguageChanged"
    );
    assert_eq!(
        as_shown(&opened, "k".to_owned(), None).err().unwrap().code,
        "folioLanguageChanged"
    );
}

#[test]
fn the_pictures_go_with_the_words_shown() {
    let opened = opened_with(
        serde_json::json!({ "pictures": [{ "at": 0, "name": "a", "kind": "png" }] }),
        Some("P"),
    );

    let pictures = pictures_of(&opened);

    assert_eq!(pictures.len(), 1);
    assert_eq!(serde_json::to_value(&pictures[0]).unwrap()["data"], "AQID");
}
