use super::*;

fn said(newest: &str, others: &[&str]) -> Answers {
    Answers {
        newest: Some(newest.into()),
        own: None,
        others: others.iter().map(|one| one.to_string()).collect(),
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
