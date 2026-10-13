use super::{said, said_carrying};
use tisty_carrier::Trouble;
use tisty_core::Paths;

#[test]
fn a_provider_s_own_words_stay_in_the_log_and_the_window_says_a_sentence_of_its_own() {
    let refusal = said(Trouble::Broke(
        "Google Drive refused: 400 badRequest Invalid Value".into(),
    ));

    assert_eq!(refusal.code, "syncFailed");
    assert_eq!(
        refusal.name, None,
        "English from the provider reached a window that may speak Spanish"
    );
}

#[test]
fn a_document_that_holds_the_push_back_is_named_by_its_title() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    tisty_core::docs::write(
        &paths.docs(),
        "dev_a-0001",
        "# Plan de viaje\n\nlo escrito\n",
    )
    .unwrap();

    let doc = said_carrying(
        &paths,
        Trouble::StuckOn {
            name: "docs/dev_a-0001.md".into(),
            why: "landed with other bytes".into(),
        },
    );
    let history = said_carrying(
        &paths,
        Trouble::StuckOn {
            name: "store/dev_a/0001.tisty".into(),
            why: "landed with another hash".into(),
        },
    );

    assert_eq!(doc.code, "syncStuckDoc");
    assert_eq!(doc.name.as_deref(), Some("Plan de viaje"));
    assert_eq!(history.code, "syncStuckHistory");
    assert_eq!(history.name.as_deref(), Some("store/dev_a/0001.tisty"));
}

#[test]
fn a_spent_day_and_a_store_in_the_bin_are_said_as_what_they_are() {
    let spent = said(Trouble::CloudSpent {
        provider: "Google Drive".into(),
    });
    let binned = said(Trouble::CloudBinned {
        provider: "Google Drive".into(),
    });

    assert_eq!(spent.code, "cloudSpent");
    assert_eq!(binned.code, "cloudBinned");
}

#[test]
fn a_round_left_for_later_that_moved_nothing_is_not_said_to_have_just_synced() {
    use super::answers::storing::synced_now;
    let waiting = |reason| tisty_carrier::Deferred {
        reason,
        left: 1,
        retry_after: None,
        names: Vec::new(),
    };

    assert!(!synced_now(
        Some(&waiting(tisty_carrier::Reason::Offline)),
        0,
        false
    ));
    assert!(synced_now(
        Some(&waiting(tisty_carrier::Reason::Offline)),
        1,
        false
    ));
    assert!(synced_now(
        Some(&waiting(tisty_carrier::Reason::Offline)),
        0,
        true
    ));
    assert!(
        synced_now(Some(&waiting(tisty_carrier::Reason::Unsent)), 0, false),
        "a round that ran whole and only left an attachment here did sync"
    );
    assert!(synced_now(None, 0, false));
}
