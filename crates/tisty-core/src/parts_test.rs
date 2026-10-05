use super::*;
use crate::event::{DeviceId, DeviceKind, TaskAdd, TaskMove, TaskPatch};
use crate::model::{Cadence, Repeat, Stays, Unit};
use ulid::Ulid;

fn ev(ms: i64, who: &str, op: Op) -> Event {
    Event::new(
        DeviceId(who.into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn with_an_agent() -> State {
    let mut state = State::default();
    state.apply(&ev(
        1,
        "dev_agent",
        Op::DeviceJoin {
            d: DeviceId("dev_agent".into()),
            k: Some(DeviceKind::Agent),
            p: None,
        },
    ));
    state
}

fn written(state: &mut State, ms: i64, who: &str, title: &str, part_of: Option<TaskId>) -> TaskId {
    let id = Ulid::generate();
    let mut d = TaskAdd::new(title, "a0");
    d.part_of = part_of;
    state.apply(&ev(ms, who, Op::TaskAdd { id, d }));
    id
}

fn moved(state: &mut State, ms: i64, id: TaskId, part_of: Option<TaskId>) {
    state.apply(&ev(
        ms,
        "dev_a",
        Op::TaskMove {
            id,
            d: TaskMove {
                part_of: Some(part_of),
                ..Default::default()
            },
        },
    ));
}

fn weekly() -> Repeat {
    Repeat::due(Cadence {
        every: 1,
        unit: Unit::Week,
    })
}

fn whole_of(state: &State, id: TaskId) -> Option<TaskId> {
    state.tasks[&id].part_of
}

#[test]
fn a_task_written_as_a_part_lands_under_its_whole() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "dev_a", "move house", None);
    let part = written(&mut state, 2, "dev_a", "book the van", Some(whole));

    assert_eq!(whole_of(&state, part), Some(whole));
    assert!(state.holds_parts(whole));
    assert_eq!(state.parts_of(whole).count(), 1);
}

#[test]
fn a_part_holds_no_parts_and_a_task_with_parts_is_no_part() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "dev_a", "move house", None);
    let part = written(&mut state, 2, "dev_a", "book the van", Some(whole));
    let other = written(&mut state, 3, "dev_a", "pack", None);

    let deeper = written(&mut state, 4, "dev_a", "find a van", Some(part));
    assert_eq!(whole_of(&state, deeper), None, "one level only");

    moved(&mut state, 5, whole, Some(other));
    assert_eq!(
        whole_of(&state, whole),
        None,
        "a task that holds parts never becomes one"
    );
}

#[test]
fn a_task_is_never_a_part_of_itself_nor_of_one_that_is_not_there() {
    let mut state = State::default();
    let one = written(&mut state, 1, "dev_a", "move house", None);

    moved(&mut state, 2, one, Some(one));
    assert_eq!(whole_of(&state, one), None);

    let loose = written(&mut state, 3, "dev_a", "pack", Some(Ulid::generate()));
    assert_eq!(whole_of(&state, loose), None);
}

#[test]
fn something_that_comes_back_neither_holds_parts_nor_is_one() {
    let mut state = State::default();
    let routine = Ulid::generate();
    let mut d = TaskAdd::new("water the plants", "a0");
    d.repeat = Some(weekly());
    state.apply(&ev(1, "dev_a", Op::TaskAdd { id: routine, d }));
    let task = written(&mut state, 2, "dev_a", "buy soil", Some(routine));
    assert_eq!(whole_of(&state, task), None, "a routine never ends");

    let whole = written(&mut state, 3, "dev_a", "move house", None);
    let part = written(&mut state, 4, "dev_a", "book the van", Some(whole));
    state.apply(&ev(
        5,
        "dev_a",
        Op::TaskUpdate {
            id: part,
            d: TaskPatch {
                repeat: Some(Some(weekly())),
                ..Default::default()
            },
        },
    ));
    assert_eq!(
        whole_of(&state, part),
        None,
        "a part that turns into a routine leaves its whole"
    );
}

#[test]
fn the_later_move_is_where_the_part_ends_up() {
    let mut state = State::default();
    let first = written(&mut state, 1, "dev_a", "move house", None);
    let second = written(&mut state, 2, "dev_a", "paint the flat", None);
    let part = written(&mut state, 3, "dev_a", "buy brushes", None);

    moved(&mut state, 4, part, Some(first));
    moved(&mut state, 5, part, Some(second));
    assert_eq!(whole_of(&state, part), Some(second));

    moved(&mut state, 6, part, None);
    assert_eq!(whole_of(&state, part), None, "null lets it go");
}

#[test]
fn a_whole_that_is_deleted_leaves_its_parts_standing_on_their_own() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "dev_a", "move house", None);
    let part = written(&mut state, 2, "dev_a", "book the van", Some(whole));

    state.apply(&ev(3, "dev_b", Op::TaskDelete { id: whole }));

    assert!(state.tasks.contains_key(&part));
    assert_eq!(whole_of(&state, part), None);
}

#[test]
fn a_task_with_parts_is_not_for_erasing() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "dev_a", "move house", None);
    written(&mut state, 2, "dev_a", "book the van", Some(whole));
    state.apply(&ev(
        3,
        "dev_a",
        Op::TaskDone {
            id: whole,
            filled: false,
        },
    ));

    assert_eq!(state.erasable(whole), Err(Stays::Parts));
}

#[test]
fn an_assistant_hangs_a_part_only_where_it_is_let_in() {
    let mut state = with_an_agent();
    let persons = written(&mut state, 2, "dev_a", "move house", None);
    let its_own = written(&mut state, 3, "dev_agent", "plan the move", None);

    let refused = written(&mut state, 4, "dev_agent", "book the van", Some(persons));
    let taken = written(&mut state, 5, "dev_agent", "list the boxes", Some(its_own));

    assert!(state.tasks.contains_key(&refused), "the task is kept");
    assert_eq!(whole_of(&state, refused), None, "only its place is let go");
    assert_eq!(whole_of(&state, taken), Some(its_own));
}

#[test]
fn an_assistant_cannot_move_the_persons_task_under_anything() {
    let mut state = with_an_agent();
    let whole = written(&mut state, 2, "dev_agent", "plan the move", None);
    let persons = written(&mut state, 3, "dev_a", "pack", None);

    state.apply(&ev(
        4,
        "dev_agent",
        Op::TaskMove {
            id: persons,
            d: TaskMove {
                part_of: Some(Some(whole)),
                ..Default::default()
            },
        },
    ));

    assert_eq!(whole_of(&state, persons), None);
}

#[test]
fn undoing_a_move_puts_the_part_back_where_it_was() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "dev_a", "move house", None);
    let part = written(&mut state, 2, "dev_a", "book the van", Some(whole));
    let before = state.clone();
    let op = Op::TaskMove {
        id: part,
        d: TaskMove {
            part_of: Some(None),
            ..Default::default()
        },
    };

    let event = ev(3, "dev_a", op);
    let back = crate::undo::inverse(&event, &before).expect("a move has an inverse");
    state.apply(&event);
    for one in back {
        state.apply(&ev(4, "dev_a", one));
    }

    assert_eq!(whole_of(&state, part), Some(whole));
}

#[test]
fn finishing_the_whole_lets_the_open_parts_go_and_keeps_the_closed_ones() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "dev_a", "move house", None);
    let done = written(&mut state, 2, "dev_a", "book the van", Some(whole));
    let open = written(&mut state, 3, "dev_a", "pack", Some(whole));
    state.apply(&ev(
        4,
        "dev_a",
        Op::TaskDone {
            id: done,
            filled: false,
        },
    ));

    let ops = state.completing_with_parts(whole, jiff::Zoned::now());

    assert!(ops.contains(&Op::TaskDrop { id: open }));
    assert!(!ops.contains(&Op::TaskDrop { id: done }));
    assert!(ops.contains(&Op::TaskDone {
        id: whole,
        filled: false
    }));
}

#[test]
fn a_build_that_knows_no_parts_reads_the_line_as_a_plain_task() {
    let whole = Ulid::generate();
    let mut d = TaskAdd::new("book the van", "a0");
    d.part_of = Some(whole);
    let line = serde_json::to_value(&d).unwrap();
    assert_eq!(line["part_of"], whole.to_string());

    let mut older = line.clone();
    older.as_object_mut().unwrap().remove("part_of");
    let read: TaskAdd = serde_json::from_value(older).unwrap();
    assert_eq!(read.part_of, None);

    let plain = serde_json::to_value(TaskAdd::new("pack", "a0")).unwrap();
    assert!(
        plain.get("part_of").is_none(),
        "nothing is written for a task of its own"
    );
}
