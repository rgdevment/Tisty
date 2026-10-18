use super::dated;
use crate::event::{DeviceId, Event, Op, StepAdd, StepRef, TaskAdd};
use crate::state::State;
use ulid::Ulid;

fn at(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

#[test]
fn a_step_tells_when_it_was_added_and_when_it_was_ticked_while_it_stays_ticked() {
    let id = Ulid::generate();
    let one = Ulid::generate();
    let two = Ulid::generate();
    let log = vec![
        at(
            1,
            Op::TaskAdd {
                id,
                d: TaskAdd::new("Rina's booster", "a0"),
            },
        ),
        at(
            2,
            Op::StepAdd {
                id,
                d: StepAdd {
                    step: one,
                    text: "Call the vet".into(),
                    order: "a".into(),
                },
            },
        ),
        at(
            3,
            Op::StepAdd {
                id,
                d: StepAdd {
                    step: two,
                    text: "Bring the card".into(),
                    order: "b".into(),
                },
            },
        ),
        at(
            4,
            Op::StepDone {
                id,
                d: StepRef { step: one },
            },
        ),
        at(
            5,
            Op::StepDone {
                id,
                d: StepRef { step: two },
            },
        ),
        at(
            6,
            Op::StepUndone {
                id,
                d: StepRef { step: two },
            },
        ),
    ];
    let state = State::replay(&log);

    let said = dated(&log, &state.tasks[&id]);

    assert_eq!(said.len(), 2);
    assert_eq!(
        said[0].born,
        Some(jiff::Timestamp::from_millisecond(2).unwrap())
    );
    assert_eq!(
        said[0].done,
        Some(jiff::Timestamp::from_millisecond(4).unwrap())
    );
    assert_eq!(said[1].done, None);
}
