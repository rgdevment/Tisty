use super::{saw, seen};

#[test]
fn the_highest_word_seen_for_each_anchor_is_kept() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(seen(home.path(), "anclaDoc"), 0);

    saw(home.path(), "anclaDoc", 3);
    saw(home.path(), "anclaDoc", 2);
    saw(home.path(), "otroDoc1", 1);

    assert_eq!(seen(home.path(), "anclaDoc"), 3);
    assert_eq!(seen(home.path(), "otroDoc1"), 1);
}

#[test]
fn a_home_that_does_not_exist_yet_is_made_on_the_first_word() {
    let home = tempfile::tempdir().unwrap();
    let inner = home.path().join("private");

    saw(&inner, "anclaDoc", 4);

    assert_eq!(seen(&inner, "anclaDoc"), 4);
}

#[test]
fn the_languages_others_added_are_asked_after_once_a_day() {
    let home = tempfile::tempdir().unwrap();
    assert!(super::languages_due(home.path(), "anclaDoc", 1_000));

    super::languages_asked(home.path(), "anclaDoc", 1_000);

    assert!(!super::languages_due(
        home.path(),
        "anclaDoc",
        1_000 + 3_600
    ));
    assert!(super::languages_due(
        home.path(),
        "anclaDoc",
        1_000 + 86_400
    ));
    assert!(super::languages_due(home.path(), "otroDoc1", 1_000));
}
