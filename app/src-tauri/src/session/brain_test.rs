use super::*;

#[test]
fn a_write_waits_for_the_writing_to_settle() {
    assert_eq!(due(true, Duration::from_secs(1)), None);
    assert_eq!(due(true, SETTLES), Some(Round::Nudged));
}

#[test]
fn a_write_after_a_long_quiet_is_never_taken_on_trust() {
    assert_eq!(due(true, EVERY), Some(Round::Nudged));
}

#[test]
fn quiet_rounds_come_on_their_own_and_only_then() {
    assert_eq!(due(false, SETTLES), None);
    assert_eq!(due(false, EVERY), Some(Round::Looking));
}

#[test]
fn a_pass_with_meanings_left_to_read_comes_back_at_once() {
    let left = brain::Passed::Meaning {
        written: 10,
        left: 5,
    };
    assert_eq!(next_look(Some(left)), READS_ON_AFTER);
    assert_eq!(next_look(Some(brain::Passed::Idle)), WEIGHS_EVERY);
    assert_eq!(next_look(None), WEIGHS_EVERY);
}
