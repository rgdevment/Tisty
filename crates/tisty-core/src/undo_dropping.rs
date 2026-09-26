use super::*;
use crate::event::DeviceId;
use ulid::Ulid;

fn ev(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn settled(status: Status) -> (State, Ulid) {
    let mut state = State::default();
    let id = Ulid::generate();
    state.apply(&ev(
        1,
        Op::TaskAdd {
            id,
            d: crate::event::TaskAdd::new("revisar el deploy", "a0"),
        },
    ));
    match status {
        Status::Done => state.apply(&ev(2, Op::TaskDone { id, filled: false })),
        Status::Dropped => state.apply(&ev(2, Op::TaskDrop { id })),
        Status::Open => {}
    }
    (state, id)
}

#[test]
fn discarding_a_completed_task_undoes_back_to_completed() {
    let (before, id) = settled(Status::Done);
    let op = inverse(&ev(3, Op::TaskDrop { id }), &before).unwrap();

    let mut after = before.clone();
    after.apply(&ev(3, Op::TaskDrop { id }));
    after.apply(&ev(4, op[0].clone()));

    assert_eq!(after.tasks[&id].status, Status::Done);
    assert_eq!(after.tasks[&id].hidden, before.tasks[&id].hidden);
}

#[test]
fn discarding_something_put_away_by_hand_leaves_it_put_away() {
    let (mut before, id) = settled(Status::Done);
    before.apply(&ev(3, Op::TaskHide { id }));

    let op = inverse(&ev(4, Op::TaskDrop { id }), &before).unwrap();
    let mut after = before.clone();
    after.apply(&ev(4, Op::TaskDrop { id }));
    after.apply(&ev(5, op[0].clone()));

    assert!(
        after.tasks[&id].hidden,
        "the drawer was the person's choice"
    );
    assert_eq!(after.tasks[&id].status, Status::Done);
}

#[test]
fn discarding_an_open_task_undoes_back_to_open() {
    let (before, id) = settled(Status::Open);
    let op = inverse(&ev(3, Op::TaskDrop { id }), &before).unwrap();

    let mut after = before.clone();
    after.apply(&ev(3, Op::TaskDrop { id }));
    after.apply(&ev(4, op[0].clone()));

    assert_eq!(after.tasks[&id], before.tasks[&id]);
}
