use super::*;

fn marks(edited: Option<i128>, called: Option<i128>, wrote: Option<i128>) -> Marks {
    Marks {
        edited,
        called,
        wrote,
        ..Marks::default()
    }
}

#[test]
fn a_turn_that_edited_and_wrote_nothing_in_tisty_is_handed_back() {
    assert_eq!(
        verdict(&marks(Some(5), None, None), false),
        Verdict::HandBack
    );
}

#[test]
fn a_turn_without_edits_ends_quietly() {
    assert_eq!(verdict(&Marks::default(), false), Verdict::LetGo);
    let earlier = Marks {
        edited: Some(5),
        ended: Some(9),
        ..Marks::default()
    };
    assert_eq!(verdict(&earlier, false), Verdict::LetGo);
}

#[test]
fn a_turn_that_wrote_in_tisty_after_editing_ends_quietly() {
    assert_eq!(
        verdict(&marks(Some(5), Some(7), Some(7)), false),
        Verdict::LetGo
    );
}

#[test]
fn a_write_in_tisty_before_this_turn_does_not_cover_it() {
    let then = Marks {
        edited: Some(12),
        called: Some(3),
        wrote: Some(3),
        ended: Some(10),
        handed: None,
    };
    assert_eq!(verdict(&then, false), Verdict::HandBack);
}

#[test]
fn the_client_saying_it_already_continued_once_lets_the_turn_end() {
    assert_eq!(verdict(&marks(Some(5), None, None), true), Verdict::LetGo);
}

#[test]
fn a_turn_already_handed_back_is_never_handed_back_again() {
    let handed = Marks {
        edited: Some(5),
        called: Some(9),
        handed: Some(8),
        ..Marks::default()
    };
    assert_eq!(verdict(&handed, false), Verdict::LetGo);
}

#[test]
fn a_hand_back_let_pass_stays_quiet_until_the_agent_calls_tisty_again() {
    let let_pass = Marks {
        edited: Some(20),
        called: None,
        wrote: None,
        ended: Some(10),
        handed: Some(8),
    };
    assert_eq!(verdict(&let_pass, false), Verdict::LetGo);
    let called_again = Marks {
        called: Some(15),
        ..let_pass
    };
    assert_eq!(verdict(&called_again, false), Verdict::HandBack);
}

#[test]
fn marks_are_kept_per_session() {
    let home = tempfile::tempdir().unwrap();
    let one = Session::of(home.path(), "a1");
    let other = Session::of(home.path(), "b2");
    one.mark(Mark::Edited, 41);
    one.mark(Mark::Edited, 42);
    assert_eq!(one.marks().edited, Some(42));
    assert_eq!(other.marks(), Marks::default());
}

#[test]
fn only_the_first_claim_on_a_key_wins() {
    let home = tempfile::tempdir().unwrap();
    let session = Session::of(home.path(), "s");
    assert!(session.claim("stop-42"));
    assert!(!session.claim("stop-42"));
    assert!(session.claim("stop-43"));
}

#[test]
fn a_session_id_never_leaves_its_folder() {
    let home = tempfile::tempdir().unwrap();
    let session = Session::of(home.path(), "..\\..\\evil/../id");
    session.mark(Mark::Called, 1);
    let made: Vec<_> = std::fs::read_dir(home.path().join(FOLDER))
        .unwrap()
        .flatten()
        .map(|one| one.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(made, vec!["------evil----id".to_string()]);
    assert_eq!(named(""), "unnamed");
    assert_eq!(named("../"), "unnamed");
}

#[test]
fn a_sweep_takes_only_sessions_older_than_a_week() {
    let home = tempfile::tempdir().unwrap();
    Session::of(home.path(), "fresh").mark(Mark::Ended, 1);
    let later =
        std::time::SystemTime::now() + std::time::Duration::from_secs(KEPT_FOR_SECONDS + 60);
    sweep(home.path(), std::time::SystemTime::now());
    assert!(home.path().join(FOLDER).join("fresh").is_dir());
    sweep(home.path(), later);
    assert!(!home.path().join(FOLDER).join("fresh").exists());
}
