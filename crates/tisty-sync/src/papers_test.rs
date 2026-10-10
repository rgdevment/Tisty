use super::*;

fn said(newest: &str, others: &[&str]) -> Answers {
    Answers {
        newest: Some(newest.into()),
        own: None,
        others: others.iter().map(|one| one.to_string()).collect(),
        behind: false,
    }
}

fn held_of<'a>(waiting: &'a [&'a str]) -> impl Fn(&str) -> bool + 'a {
    move |print| waiting.contains(&print)
}

#[test]
fn a_body_held_by_a_waiting_machine_waits() {
    let says = said("la-ultima", &[]);
    assert_eq!(
        answered_for(
            Some(&"la-del-otro".to_string()),
            Some(&says),
            &held_of(&["la-del-otro"])
        ),
        Answer::Waits
    );
}

#[test]
fn a_body_nobody_answers_for_is_still_put_to_the_person() {
    let says = said("la-ultima", &["una-vieja"]);
    assert_eq!(
        answered_for(
            Some(&"de-nadie".to_string()),
            Some(&says),
            &held_of(&["la-del-otro"])
        ),
        Answer::No
    );
}

#[test]
fn what_a_trusted_history_says_comes_first_and_the_waiting_one_is_never_read() {
    let says = said("la-ultima", &["una-vieja"]);
    let untouched = |_: &str| -> bool { panic!("the waiting history was read") };
    assert_eq!(
        answered_for(Some(&"la-ultima".to_string()), Some(&says), &untouched),
        Answer::Yes
    );
    assert_eq!(
        answered_for(Some(&"una-vieja".to_string()), Some(&says), &untouched),
        Answer::Doubtful
    );
}

#[test]
fn a_body_the_log_has_no_print_for_comes_in_with_what_was_here_set_aside() {
    let silent = Answers {
        newest: None,
        own: None,
        others: Default::default(),
        behind: false,
    };
    let untouched = |_: &str| -> bool { panic!("the waiting history was read") };
    assert_eq!(
        answered_for(Some(&"cualquiera".to_string()), Some(&silent), &untouched),
        Answer::Doubtful,
        "a log with no print vouched for whatever the folder held"
    );
    assert_eq!(
        answered_for(Some(&"cualquiera".to_string()), None, &untouched),
        Answer::Doubtful,
        "a document the log never named was vouched for"
    );
}

#[test]
fn a_body_held_by_a_machine_later_removed_is_asked_about_at_once() {
    use tisty_core::docs::Move;
    let mut awaited = Awaited::default();
    let hour = crate::awaited::LANDING;

    assert_eq!(
        landing(
            &mut awaited,
            "doc-0001",
            Move::Bring,
            Answer::Waits,
            false,
            1_000
        ),
        None
    );
    assert_eq!(
        landing(
            &mut awaited,
            "doc-0001",
            Move::Bring,
            Answer::No,
            false,
            1_000 + hour
        ),
        None,
        "removing the machine hid the document for another hour"
    );
}

#[test]
fn a_locked_body_is_timed_while_it_is_put_to_the_person() {
    use tisty_core::docs::Move;
    let mut awaited = Awaited::default();
    let hour = crate::awaited::LANDING;

    assert_eq!(
        landing(
            &mut awaited,
            "doc-0001",
            Move::Bring,
            Answer::No,
            true,
            1_000
        ),
        None
    );
    assert_eq!(
        landing(
            &mut awaited,
            "doc-0001",
            Move::Bring,
            Answer::No,
            false,
            1_000 + hour
        ),
        None,
        "unlocking it hid the document for another hour"
    );
}

#[test]
fn only_the_first_sight_of_a_waiting_body_is_told() {
    use tisty_core::docs::Move;
    let mut awaited = Awaited::default();

    assert_eq!(
        landing(
            &mut awaited,
            "doc-0001",
            Move::Bring,
            Answer::No,
            false,
            1_000
        ),
        Some(true)
    );
    assert_eq!(
        landing(
            &mut awaited,
            "doc-0001",
            Move::Bring,
            Answer::No,
            false,
            1_060
        ),
        Some(false)
    );
    assert_eq!(
        landing(
            &mut awaited,
            "doc-0002",
            Move::TheyDecide,
            Answer::No,
            false,
            1_000
        ),
        None
    );
}

fn tried(at: &Path, checked: &str) -> (Option<Vec<u8>>, Moved) {
    let mut done = Moved::default();
    let mut prints = tisty_core::docs::Prints::default();
    let got = taken(&mut prints, at, checked, "dev_a-0001", &mut done);
    (got, done)
}

#[test]
fn a_body_as_it_was_checked_is_handed_over_untouched() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("acta.md");
    std::fs::write(&at, "# Uno").unwrap();
    let checked = tisty_core::docs::print_of(&at).unwrap().unwrap();

    let (got, done) = tried(&at, &checked);

    assert_eq!(got, Some(b"# Uno".to_vec()));
    assert!(done.coming.is_empty() && done.astray.is_empty());
}

#[test]
fn a_body_that_changed_since_it_was_checked_waits_for_the_next_round() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("acta.md");
    std::fs::write(&at, "# Uno\n").unwrap();
    let checked = tisty_core::docs::print_of(&at).unwrap().unwrap();
    std::fs::write(&at, "# Dos\n").unwrap();

    let (got, done) = tried(&at, &checked);

    assert_eq!(got, None);
    assert_eq!(done.coming, vec!["dev_a-0001".to_string()]);
    assert!(done.astray.is_empty());
}

#[test]
fn a_body_that_went_away_waits_for_the_next_round() {
    let room = tempfile::tempdir().unwrap();

    let (got, done) = tried(&room.path().join("nada.md"), "whatever");

    assert_eq!(got, None);
    assert_eq!(done.coming, vec!["dev_a-0001".to_string()]);
    assert!(done.astray.is_empty());
}

#[test]
fn a_body_that_cannot_be_read_is_left_astray_where_the_person_hears_of_it() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("acta.md");
    std::fs::create_dir(&at).unwrap();

    let (got, done) = tried(&at, "whatever");

    assert_eq!(got, None);
    assert_eq!(done.astray, vec!["dev_a-0001".to_string()]);
    assert!(
        done.coming.is_empty(),
        "a body that cannot be read was said to be coming"
    );
}

#[test]
fn a_body_that_turned_into_a_link_is_left_astray_instead_of_stopping_the_round() {
    let room = tempfile::tempdir().unwrap();
    let real = room.path().join("real.md");
    std::fs::write(&real, "# Real\n").unwrap();
    let checked = tisty_core::docs::print_of(&real).unwrap().unwrap();
    let link = room.path().join("link.md");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &link).unwrap();
    #[cfg(windows)]
    if std::os::windows::fs::symlink_file(&real, &link).is_err() {
        return;
    }

    let (got, done) = tried(&link, &checked);

    assert_eq!(got, None);
    assert_eq!(done.astray, vec!["dev_a-0001".to_string()]);
}
