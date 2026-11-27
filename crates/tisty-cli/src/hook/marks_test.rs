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
        asked: None,
        published: None,
        asked_published: None,
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
fn a_turn_that_was_asked_about_is_not_handed_back_but_an_earlier_question_does_not_cover_it() {
    let asked_now = Marks {
        edited: Some(12),
        ended: Some(10),
        asked: Some(14),
        ..Marks::default()
    };
    assert_eq!(verdict(&asked_now, false), Verdict::LetGo);
    let asked_before = Marks {
        asked: Some(8),
        ..asked_now
    };
    assert_eq!(verdict(&asked_before, false), Verdict::HandBack);
    let first_turn = Marks {
        edited: Some(12),
        asked: Some(3),
        ..Marks::default()
    };
    assert_eq!(verdict(&first_turn, false), Verdict::LetGo);
}

#[test]
fn a_hand_back_let_pass_stays_quiet_until_the_agent_calls_tisty_again() {
    let let_pass = Marks {
        edited: Some(20),
        called: None,
        wrote: None,
        ended: Some(10),
        handed: Some(8),
        asked: None,
        published: None,
        asked_published: None,
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

fn kept() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn a_session_remembers_the_task_last_written() {
    let cache = kept();
    let session = Session::of(cache.path(), "s-1");
    assert_eq!(session.last_touched(), None);

    session.touched("01J00000000000000000000001");
    assert_eq!(
        session.last_touched().as_deref(),
        Some("01J00000000000000000000001")
    );

    session.touched("01J00000000000000000000002");
    assert_eq!(
        session.last_touched().as_deref(),
        Some("01J00000000000000000000002")
    );
}

#[test]
fn an_empty_task_name_touches_nothing() {
    let cache = kept();
    let session = Session::of(cache.path(), "s-1");
    session.touched("01J00000000000000000000001");

    session.touched("");

    assert_eq!(
        session.last_touched().as_deref(),
        Some("01J00000000000000000000001")
    );
}

#[test]
fn the_task_last_written_is_kept_per_session() {
    let cache = kept();
    Session::of(cache.path(), "one").touched("aaa");
    Session::of(cache.path(), "two").touched("bbb");

    assert_eq!(
        Session::of(cache.path(), "one").last_touched().as_deref(),
        Some("aaa")
    );
    assert_eq!(
        Session::of(cache.path(), "two").last_touched().as_deref(),
        Some("bbb")
    );
    assert_eq!(Session::of(cache.path(), "three").last_touched(), None);
}

#[test]
fn a_task_name_with_odd_characters_is_kept_as_a_safe_file_name() {
    let cache = kept();
    let session = Session::of(cache.path(), "s-1");

    session.touched("a/b c");

    assert_eq!(session.last_touched().as_deref(), Some("a-b-c"));
}

#[test]
fn the_published_and_asked_marks_are_read_back() {
    let cache = kept();
    let session = Session::of(cache.path(), "s-1");
    assert_eq!(session.marks().published, None);
    assert_eq!(session.marks().asked_published, None);

    session.mark(Mark::Published, 7);
    session.mark(Mark::AskedPublished, 9);

    let marks = session.marks();
    assert_eq!(marks.published, Some(7));
    assert_eq!(marks.asked_published, Some(9));
}

fn published(published: Option<i128>, asked: Option<i128>, ended: Option<i128>) -> Marks {
    Marks {
        published,
        asked_published: asked,
        ended,
        ..Marks::default()
    }
}

#[test]
fn a_push_nobody_asked_about_is_unasked() {
    assert!(published(Some(5), None, None).published_unasked());
    assert!(published(Some(5), None, Some(3)).published_unasked());
}

#[test]
fn nothing_is_unasked_without_a_push() {
    assert!(!published(None, None, None).published_unasked());
    assert!(!published(None, Some(4), Some(3)).published_unasked());
}

#[test]
fn a_push_from_before_the_turn_ended_is_not_unasked() {
    assert!(!published(Some(5), None, Some(8)).published_unasked());
    assert!(!published(Some(5), None, Some(5)).published_unasked());
}

#[test]
fn a_push_already_asked_about_is_not_unasked_until_another_one() {
    assert!(!published(Some(5), Some(6), None).published_unasked());
    assert!(!published(Some(5), Some(5), None).published_unasked());
    assert!(published(Some(8), Some(6), None).published_unasked());
}
