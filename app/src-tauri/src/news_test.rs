use super::{behind, decide, told};

fn said(all: &[&str]) -> Vec<String> {
    all.iter().map(|one| (*one).to_owned()).collect()
}

#[test]
fn a_first_install_is_left_to_the_welcome() {
    assert!(decide(true, None, "1.30.0", &said(&["1.30.0"])).is_empty());
}

#[test]
fn a_machine_told_nothing_before_hears_everything_up_to_what_it_runs() {
    let all = said(&["1.29.0", "1.30.0", "1.30.1", "1.30.2", "1.31.0"]);
    assert_eq!(
        decide(false, None, "1.30.2", &all),
        said(&["1.30.2", "1.30.1", "1.30.0", "1.29.0"])
    );
}

#[test]
fn an_update_shows_only_what_was_not_seen_newest_first() {
    let all = said(&["1.30.0", "1.30.1", "1.31.0", "1.32.0"]);
    assert_eq!(
        decide(false, Some("1.30.0"), "1.31.0", &all),
        said(&["1.31.0", "1.30.1"])
    );
}

#[test]
fn the_same_version_twice_says_nothing() {
    assert!(decide(false, Some("1.30.0\n"), "1.30.0", &said(&["1.30.0"])).is_empty());
}

#[test]
fn going_back_a_version_says_nothing() {
    assert!(decide(false, Some("1.31.0"), "1.30.0", &said(&["1.30.0"])).is_empty());
}

#[test]
fn a_patch_tells_again_what_its_minor_brought() {
    assert_eq!(
        decide(
            false,
            Some("1.30.1"),
            "1.30.2",
            &said(&["1.29.0", "1.30.0"])
        ),
        said(&["1.30.0"])
    );
}

#[test]
fn a_patch_with_its_own_entry_does_not_repeat_its_minor() {
    assert_eq!(
        decide(
            false,
            Some("1.30.1"),
            "1.30.2",
            &said(&["1.30.0", "1.30.2"])
        ),
        said(&["1.30.2"])
    );
}

#[test]
fn an_update_with_nothing_written_for_it_nor_its_minor_says_nothing() {
    assert!(decide(false, Some("1.30.0"), "1.31.1", &said(&["1.30.0"])).is_empty());
}

#[test]
fn a_candidate_counts_as_its_own_version() {
    assert_eq!(
        decide(
            false,
            Some("1.30.0"),
            "1.31.0-rc.1",
            &said(&["1.31.0-rc.1", "1.31.0"])
        ),
        said(&["1.31.0-rc.1"])
    );
}

#[test]
fn a_version_nobody_can_read_is_treated_as_never_told() {
    assert_eq!(
        decide(false, Some("ayer"), "1.30.0", &said(&["1.30.0"])),
        said(&["1.30.0"])
    );
    assert!(decide(false, None, "nada", &said(&["1.30.0"])).is_empty());
}

#[test]
fn only_a_version_older_than_the_running_one_is_behind() {
    assert!(behind(None, "1.30.0"));
    assert!(behind(Some("1.29.9"), "1.30.0"));
    assert!(!behind(Some("1.30.0"), "1.30.0"));
    assert!(!behind(Some("1.31.0"), "1.30.0"));
}

#[test]
fn the_news_file_names_versions_that_parse() {
    let all = told();
    assert!(!all.is_empty(), "the news file says nothing");
    for one in all {
        assert!(semver::Version::parse(&one).is_ok(), "{one}");
    }
}
