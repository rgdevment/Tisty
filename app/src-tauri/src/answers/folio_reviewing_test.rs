use std::collections::BTreeSet;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use tisty_carrier::folio_client::{Answered, Asked, Door};
use tisty_carrier::folio_reviewed::{self, Copy};
use tisty_core::merge::Changed;
use tisty_envelope::Translation;
use tisty_envelope::reviews::{self, LeftOut, Signer, Weighing};
use tisty_share_proto::identity::{Certificate, Root, ShareKey};
use tisty_sharing::Remembered;

use super::{
    Decided, as_shown, foreign_entry, named_here, pictures_kept, proposals_of, reviewed_now,
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
        serde_json::json!({ "pictures": [{ "at": 0, "name": "a", "kind": "image/jpeg" }] }),
        Some("P"),
    );
    let cache = tempfile::tempdir().unwrap();
    let folder = super::folder(cache.path());
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("left.png"), b"old").unwrap();

    let pictures = pictures_kept(&opened, &folder).unwrap();

    let path = serde_json::to_value(&pictures[0]).unwrap()["path"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(std::fs::read(&path).unwrap(), [1, 2, 3]);
    assert!(path.ends_with("0.jpg"));
    assert!(!folder.join("left.png").exists());
}

fn shelf_of(entries: Vec<Remembered>) -> std::collections::BTreeMap<String, Remembered> {
    entries
        .into_iter()
        .map(|one| (format!("{}.md", one.id), one))
        .collect()
}

fn words_said(words: &[super::LanguageWord]) -> Vec<(String, String, bool)> {
    let said = serde_json::to_value(words).unwrap();
    said.as_array()
        .unwrap()
        .iter()
        .map(|one| {
            (
                one["document"].as_str().unwrap().to_owned(),
                one["word"].as_str().unwrap().to_owned(),
                one["mine"].as_bool().unwrap(),
            )
        })
        .collect()
}

#[test]
fn the_owner_s_card_names_the_languages_others_added_with_its_word() {
    let (key, certificate) = owners_machine(0x30, 0x31);
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };
    let own_french = Remembered {
        id: "marioFr1".to_owned(),
        ..anchor()
    };
    let was = anchor();
    let home = tempfile::tempdir().unwrap();

    let before = super::words_on_card(
        home.path(),
        &shelf_of(vec![was.clone(), own_french.clone()]),
        &was,
    );
    let kept = reviewed_now(&signer, &was, "anaEngl1", Some(&read_as_shown()), 1).unwrap();
    let after = super::words_on_card(
        home.path(),
        &shelf_of(vec![kept.clone(), own_french]),
        &kept,
    );

    assert_eq!(
        words_said(&before),
        [("anaEngl1".to_owned(), "waiting".to_owned(), false)]
    );
    assert_eq!(
        words_said(&after),
        [("anaEngl1".to_owned(), "kept".to_owned(), false)]
    );
}

#[test]
fn someone_who_edits_hears_the_owner_s_word_on_the_language_they_added() {
    let (key, certificate) = owners_machine(0x30, 0x31);
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };
    let as_edited = |was: Remembered| Remembered {
        admin_token: None,
        edit_token: Some("token".to_owned()),
        owner: Some(serde_json::json!({ "root": certificate.root }).to_string()),
        ..was
    };
    let mine = Remembered {
        id: "anaEngl1".to_owned(),
        held_by: Some("anclaDoc".to_owned()),
        carried: None,
        ..anchor()
    };
    let home = tempfile::tempdir().unwrap();
    let word = |anchor: Remembered| {
        let shelf = shelf_of(vec![as_edited(anchor), mine.clone()]);
        words_said(&super::words_on_card(home.path(), &shelf, &mine))
    };
    let kept = reviewed_now(&signer, &anchor(), "anaEngl1", Some(&read_as_shown()), 1).unwrap();
    let left = reviewed_now(&signer, &kept, "anaEngl1", None, 1).unwrap();

    assert_eq!(
        word(anchor()),
        [("anaEngl1".to_owned(), "waiting".to_owned(), true)]
    );
    assert_eq!(
        word(kept),
        [("anaEngl1".to_owned(), "kept".to_owned(), true)]
    );
    assert_eq!(
        word(left),
        [("anaEngl1".to_owned(), "leftOut".to_owned(), true)]
    );
    let kept_again =
        reviewed_now(&signer, &anchor(), "anaEngl1", Some(&read_as_shown()), 2).unwrap();
    known(home.path(), "anaEngl1", 5, encoded(0x15));
    assert_eq!(
        word(kept_again),
        [("anaEngl1".to_owned(), "changed".to_owned(), true)]
    );
}

fn known(home: &std::path::Path, document: &str, version: u64, print: String) {
    folio_reviewed::print_known(home, document, folio_reviewed::Printed { version, print });
}

#[test]
fn a_kept_language_whose_print_changed_since_says_so_on_the_owner_s_card() {
    let (key, certificate) = owners_machine(0x30, 0x31);
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };
    let home = tempfile::tempdir().unwrap();
    let kept = reviewed_now(&signer, &anchor(), "anaEngl1", Some(&read_as_shown()), 1).unwrap();
    let own_french = Remembered {
        id: "marioFr1".to_owned(),
        ..anchor()
    };
    let shelf = shelf_of(vec![kept.clone(), own_french]);
    let said = || words_said(&super::words_on_card(home.path(), &shelf, &kept));

    known(home.path(), "anaEngl1", 4, encoded(0x13));
    assert_eq!(said(), [("anaEngl1".to_owned(), "kept".to_owned(), false)]);

    known(home.path(), "anaEngl1", 5, encoded(0x15));
    assert_eq!(
        said(),
        [("anaEngl1".to_owned(), "changed".to_owned(), false)]
    );
}

fn copy(print: u8, body: &str) -> Copy {
    Copy {
        print: encoded(print),
        title: Some("Walks".to_owned()),
        body: body.to_owned(),
    }
}

#[test]
fn what_changed_is_told_against_the_copy_kept_with_the_word_that_stands() {
    let (key, certificate) = owners_machine(0x30, 0x31);
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };
    let home = tempfile::tempdir().unwrap();
    let kept = reviewed_now(&signer, &anchor(), "anaEngl1", Some(&read_as_shown()), 1).unwrap();
    let now = copy(
        0x15,
        "One.

Three.",
    );
    assert_eq!(
        super::changes_since(home.path(), &kept, "anaEngl1", &now),
        None
    );

    folio_reviewed::copy_kept(
        home.path(),
        "anaEngl1",
        &copy(
            0x13,
            "One.

Two.",
        ),
    );

    assert_eq!(
        super::changes_since(home.path(), &kept, "anaEngl1", &now).unwrap(),
        [
            Changed::Same("Walks".to_owned()),
            Changed::Same("One.".to_owned()),
            Changed::Gone("Two.".to_owned()),
            Changed::Came("Three.".to_owned()),
        ]
    );
    folio_reviewed::copy_kept(home.path(), "anaEngl1", &copy(0x14, "One."));
    assert_eq!(
        super::changes_since(home.path(), &kept, "anaEngl1", &now),
        None
    );
}

struct LatestOnly(std::cell::Cell<u32>);

impl Door for LatestOnly {
    fn ask(&self, asked: &Asked<'_>) -> Option<Answered> {
        self.0.set(self.0.get() + 1);
        asked.path.ends_with("/latest").then(|| Answered {
            status: 200,
            headers: Vec::new(),
            body: br#"{"version":4}"#.to_vec(),
        })
    }
}

#[test]
fn a_version_already_printed_here_is_not_brought_again() {
    let home = tempfile::tempdir().unwrap();
    let folio = LatestOnly(std::cell::Cell::new(0));
    let entry: Translation = serde_json::from_value(serde_json::json!({
        "lang": "en", "document": "anaEngl1", "key": encoded(0x12),
    }))
    .unwrap();
    known(home.path(), "anaEngl1", 4, encoded(0x13));

    assert!(!super::looked_again(&folio, home.path(), &entry));

    assert_eq!(folio.0.get(), 1);
    known(home.path(), "anaEngl1", 3, encoded(0x12));
    assert_eq!(
        folio_reviewed::printed(home.path(), "anaEngl1")
            .unwrap()
            .print,
        encoded(0x13)
    );
}
