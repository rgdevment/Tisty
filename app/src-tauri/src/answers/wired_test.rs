use super::{WakeOffer, reminded, wake_offer};
use crate::waking::Waking;

fn standing(offered: bool, wakes: bool) -> Waking {
    Waking {
        offered,
        wakes,
        theirs: false,
    }
}

#[test]
fn the_first_reminder_on_a_copy_that_does_not_start_with_the_machine_asks() {
    assert_eq!(
        wake_offer(false, true, || standing(true, false)),
        WakeOffer::Ask(standing(true, false))
    );
}

#[test]
fn with_no_reminder_yet_nothing_is_asked_and_the_machine_is_not_read() {
    assert_eq!(
        wake_offer(false, false, || unreachable!("nothing to ask about")),
        WakeOffer::Not
    );
}

#[test]
fn once_asked_it_is_never_asked_again() {
    assert_eq!(
        wake_offer(true, true, || unreachable!("already asked")),
        WakeOffer::Not
    );
}

#[test]
fn a_copy_that_already_starts_with_the_machine_settles_the_question() {
    assert_eq!(
        wake_offer(false, true, || standing(true, true)),
        WakeOffer::Settled
    );
}

#[test]
fn a_copy_that_cannot_start_with_the_machine_is_not_asked() {
    assert_eq!(
        wake_offer(false, true, || standing(false, false)),
        WakeOffer::Not
    );
}

#[test]
fn only_an_open_task_with_a_reminder_counts() {
    let mut state = tisty_core::State::default();
    let id = ulid::Ulid::generate();
    let mut add = tisty_core::event::TaskAdd::new("comprar pan", "a0");
    add.reminders = vec![tisty_core::DateSpec::floating(
        "2026-08-12T09:00:00".parse().unwrap(),
        "America/Santiago",
    )];
    let fired = |op| {
        tisty_core::Event::new(
            tisty_core::DeviceId("a".into()),
            jiff::Timestamp::UNIX_EPOCH,
            op,
        )
    };
    assert!(!reminded(&state));

    state.apply(&fired(tisty_core::Op::TaskAdd { id, d: add }));
    assert!(reminded(&state));

    state.apply(&fired(tisty_core::Op::TaskDone { id, filled: false }));
    assert!(!reminded(&state));
}
