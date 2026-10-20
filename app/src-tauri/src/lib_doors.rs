use super::{DoorDue, knocking, offering};

fn due(why: &'static str, by: Option<&str>) -> Option<DoorDue> {
    Some(DoorDue {
        why,
        by: by.map(str::to_string),
    })
}

fn knock(at: &str, by: Option<&str>) -> Option<tisty_core::agent::Knock> {
    Some(tisty_core::agent::Knock {
        at: at.parse().unwrap(),
        by: by.map(str::to_string),
    })
}

#[test]
fn a_machine_without_an_assistant_is_never_offered_one() {
    assert_eq!(offering(0, 0, false, false), None);
    assert_eq!(offering(0, 0, true, false), None);
}

#[test]
fn an_assistant_that_is_found_and_not_yet_wired_is_worth_offering() {
    assert_eq!(offering(1, 0, false, false), due("installed", None));
    assert_eq!(offering(6, 0, true, false), due("installed", None));
}

#[test]
fn somebody_who_wired_one_and_opened_the_door_knows_where_it_is() {
    assert_eq!(offering(1, 1, true, false), None);
    assert_eq!(offering(6, 6, true, false), None);
}

#[test]
fn one_wired_assistant_speaks_for_the_rest() {
    assert_eq!(offering(6, 1, true, false), None);
}

#[test]
fn a_wired_assistant_behind_a_closed_door_is_mute() {
    assert_eq!(offering(1, 1, false, false), due("mute", None));
    assert_eq!(offering(6, 1, false, false), due("mute", None));
}

#[test]
fn nobody_knocking_is_nothing_to_tell() {
    assert_eq!(knocking(false, None, None), None);
}

#[test]
fn a_knock_on_a_closed_door_is_told_with_who_knocked() {
    assert_eq!(
        knocking(
            false,
            knock("2026-10-19T10:00:00Z", Some("claude-code")),
            None
        ),
        due("knocked", Some("claude-code"))
    );
    assert_eq!(
        knocking(false, knock("2026-10-19T10:00:00Z", None), None),
        due("knocked", None)
    );
}

#[test]
fn a_knock_already_heard_is_not_told_again() {
    let heard = "2026-10-19T10:00:05Z".parse().ok();
    assert_eq!(
        knocking(false, knock("2026-10-19T10:00:00Z", Some("codex")), heard),
        None
    );
    assert_eq!(
        knocking(false, knock("2026-10-19T10:01:00Z", Some("codex")), heard),
        due("knocked", Some("codex"))
    );
}

#[test]
fn a_knock_on_a_door_since_opened_is_nothing_to_tell() {
    assert_eq!(
        knocking(true, knock("2026-10-19T10:00:00Z", Some("codex")), None),
        None
    );
}

#[test]
fn a_door_tisty_opened_on_its_own_is_told_once_there_is_an_assistant_to_use_it() {
    assert_eq!(offering(1, 0, true, true), due("opened", None));
    assert_eq!(offering(3, 2, true, true), due("opened", None));
    assert_eq!(offering(0, 0, true, true), None);
}

#[test]
fn a_door_opened_by_default_and_closed_since_goes_back_to_the_usual_offers() {
    assert_eq!(offering(1, 0, false, true), due("installed", None));
    assert_eq!(offering(1, 1, false, true), due("mute", None));
}
