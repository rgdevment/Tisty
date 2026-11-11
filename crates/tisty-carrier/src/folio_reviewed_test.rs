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

fn printed_at(version: u64, print: &str) -> super::Printed {
    super::Printed {
        version,
        print: print.into(),
    }
}

#[test]
fn the_print_of_the_latest_version_known_is_kept_and_an_older_one_never_takes_its_place() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(super::printed(home.path(), "frances1"), None);

    super::print_known(home.path(), "frances1", printed_at(3, "c"));
    super::print_known(home.path(), "frances1", printed_at(2, "b"));

    assert_eq!(
        super::printed(home.path(), "frances1"),
        Some(printed_at(3, "c"))
    );
    super::print_known(home.path(), "frances1", printed_at(4, "d"));
    assert_eq!(
        super::printed(home.path(), "frances1"),
        Some(printed_at(4, "d"))
    );
}

#[test]
fn a_kept_copy_comes_back_whole_until_it_is_dropped() {
    let home = tempfile::tempdir().unwrap();
    let copy = super::Copy {
        print: "p".into(),
        title: Some("Bonjour".into()),
        body: "Un.

Deux."
            .into(),
    };

    super::copy_kept(home.path(), "frances1", &copy);

    assert_eq!(super::kept_copy(home.path(), "frances1"), Some(copy));
    super::copy_dropped(home.path(), "frances1");
    assert_eq!(super::kept_copy(home.path(), "frances1"), None);
}

#[test]
fn an_id_that_could_reach_outside_the_folder_keeps_no_copy() {
    let home = tempfile::tempdir().unwrap();
    let copy = super::Copy {
        print: "p".into(),
        title: None,
        body: "x".into(),
    };

    super::copy_kept(home.path(), "../fuera", &copy);

    assert_eq!(super::kept_copy(home.path(), "../fuera"), None);
    assert!(!home.path().join("fuera.json").exists());
}
