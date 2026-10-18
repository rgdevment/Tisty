use crate::State;
use crate::event::{DeviceId, Event, LogAdd, Op, StepAdd, StepRef, TaskAdd};
use crate::model::{Reading, TaskId};
use jiff::Timestamp;
use ulid::Ulid;

const HOUR: i64 = 60 * 60;

fn at(seconds: i64, op: Op) -> Event {
    let when = Timestamp::from_second(1_770_000_000 + seconds).unwrap();
    Event::new(DeviceId("dev_a".into()), when, op)
}

fn planned(seconds: i64, id: TaskId, text: &str) -> (Event, Ulid) {
    let step = Ulid::generate();
    let d = StepAdd {
        step,
        text: text.into(),
        order: "a0".into(),
    };
    (at(seconds, Op::StepAdd { id, d }), step)
}

fn noted(seconds: i64, id: TaskId) -> Event {
    let d = LogAdd::new(Ulid::generate(), "Called, they open at nine.");
    at(seconds, Op::TaskLog { id, d })
}

fn born(id: TaskId) -> Event {
    at(
        0,
        Op::TaskAdd {
            id,
            d: TaskAdd::new("Groceries", "a0"),
        },
    )
}

#[test]
fn a_list_written_at_once_and_only_ticked_after_stays_a_trace() {
    let id = Ulid::generate();
    let mut log = vec![born(id)];
    for item in [
        "milk", "bread", "eggs", "rice", "salt", "oil", "tea", "soap",
    ] {
        let (added, step) = planned(5, id, item);
        log.push(added);
        log.push(at(
            3 * HOUR,
            Op::StepDone {
                id,
                d: StepRef { step },
            },
        ));
    }

    let state = State::replay(&log);

    assert_eq!(state.tasks[&id].volume.grew, 1);
    assert_eq!(state.tasks[&id].reading(), Reading::Trace);
}

#[test]
fn a_task_come_back_to_four_times_reads_as_a_story() {
    let id = Ulid::generate();
    let log = vec![
        born(id),
        planned(10, id, "ask for the quote").0,
        noted(HOUR, id),
        planned(2 * HOUR, id, "compare two offers").0,
        noted(3 * HOUR, id),
    ];

    let state = State::replay(&log);

    assert_eq!(state.tasks[&id].volume.grew, 4);
    assert_eq!(state.tasks[&id].reading(), Reading::Story);
}

#[test]
fn three_visits_are_still_a_trace_and_one_long_sitting_is_one_visit() {
    let id = Ulid::generate();
    let log = vec![
        born(id),
        noted(10, id),
        noted(20 * 60, id),
        planned(HOUR, id, "pay").0,
        noted(2 * HOUR, id),
    ];

    let state = State::replay(&log);

    assert_eq!(state.tasks[&id].volume.grew, 3);
    assert_eq!(state.tasks[&id].reading(), Reading::Trace);
}

#[test]
fn one_long_sitting_is_one_visit_and_undo_is_none() {
    let id = Ulid::generate();
    let mut log = vec![born(id)];
    for minute in (0..120).step_by(10) {
        log.push(noted(minute * 60, id));
    }
    let (mut again, _) = planned(5 * HOUR, id, "again");
    again.undo = true;
    log.push(again);

    let state = State::replay(&log);

    assert_eq!(state.tasks[&id].volume.grew, 1);
}
