use super::*;
use tisty_core::event::{DeviceId, Event, TaskAdd};

fn ev(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn written(state: &mut State, ms: i64, title: &str, part_of: Option<TaskId>) -> TaskId {
    let id = ulid::Ulid::generate();
    let mut d = TaskAdd::new(title, "a0");
    d.part_of = part_of;
    state.apply(&ev(ms, Op::TaskAdd { id, d }));
    id
}

fn applied(state: &mut State, ms: i64, ops: Vec<Op>) {
    for one in ops {
        state.apply(&ev(ms, one));
    }
}

#[test]
fn a_whole_is_counted_and_named_for_the_rows_that_show_its_parts() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "move house", None);
    let open = written(&mut state, 2, "pack", Some(whole));
    let done = written(&mut state, 3, "book the van", Some(whole));
    written(&mut state, 4, "loose", None);
    applied(
        &mut state,
        5,
        vec![Op::TaskDone {
            id: done,
            filled: false,
        }],
    );
    let _ = open;

    let all = wholes(&state);
    assert_eq!(all.len(), 1);
    assert_eq!(
        all[&whole.to_string()],
        Whole {
            title: "move house".into(),
            open: 1,
            closed: 1,
            away: 0
        }
    );
}

#[test]
fn hanging_says_why_instead_of_letting_nothing_land() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "move house", None);
    let part = written(&mut state, 2, "pack", Some(whole));
    let other = written(&mut state, 3, "paint", None);

    assert_eq!(hanging(&state, other, Some(other)), Err("partOfItself"));
    assert_eq!(hanging(&state, other, Some(part)), Err("partOfAPart"));
    assert_eq!(hanging(&state, whole, Some(other)), Err("wholeIsNoPart"));
    assert!(hanging(&state, other, Some(whole)).is_ok());
    assert!(
        hanging(&state, part, None).is_ok(),
        "letting go always lands"
    );

    applied(
        &mut state,
        4,
        vec![Op::TaskDone {
            id: whole,
            filled: false,
        }],
    );
    assert_eq!(hanging(&state, other, Some(whole)), Err("wholeClosed"));
}

#[test]
fn a_part_put_out_of_sight_still_names_its_whole_and_still_counts_as_left() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "move house", None);
    let part = written(&mut state, 2, "pack", Some(whole));
    applied(&mut state, 3, vec![Op::TaskHide { id: part }]);

    let all = wholes(&state);
    let counted = &all[&whole.to_string()];
    assert_eq!(counted.title, "move house");
    assert_eq!((counted.open, counted.closed, counted.away), (0, 0, 1));
}

#[test]
fn a_part_added_lands_in_its_whole_and_its_list() {
    let mut state = State::default();
    let list = ulid::Ulid::generate();
    state.apply(&ev(
        1,
        Op::ListAdd {
            id: list,
            d: tisty_core::event::ListAdd {
                name: "Casa".into(),
                order: "a0".into(),
                color: None,
            },
        },
    ));
    let whole = ulid::Ulid::generate();
    let mut d = TaskAdd::new("move house", "a0");
    d.list = Some(list);
    state.apply(&ev(2, Op::TaskAdd { id: whole, d }));

    let (born, ops) = part_added(&state, whole, " book the van ").unwrap();
    applied(&mut state, 3, ops);

    let part = &state.tasks[&born];
    assert_eq!(part.part_of, Some(whole));
    assert_eq!(part.list, Some(list));
    assert_eq!(part.title, "book the van");
}

#[test]
fn a_step_turned_into_a_part_leaves_the_steps_in_the_same_breath() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "move house", None);
    let step = ulid::Ulid::generate();
    applied(
        &mut state,
        2,
        vec![Op::StepAdd {
            id: whole,
            d: tisty_core::event::StepAdd {
                step,
                text: "call the landlord".into(),
                order: "a0".into(),
            },
        }],
    );

    let ops = step_turned(&state, whole, &step.to_string()).unwrap();
    applied(&mut state, 3, ops);

    assert!(state.tasks[&whole].steps.is_empty());
    let part = state
        .parts_of(whole)
        .next()
        .expect("the step is a part now");
    assert_eq!(part.title, "call the landlord");
}

#[test]
fn only_open_tasks_that_could_hold_parts_are_offered() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "move house", None);
    let part = written(&mut state, 2, "pack", Some(whole));
    let loose = written(&mut state, 3, "paint", None);
    let closed = written(&mut state, 4, "old plan", None);
    applied(
        &mut state,
        5,
        vec![Op::TaskDone {
            id: closed,
            filled: false,
        }],
    );

    let titles = |id| -> Vec<String> {
        offered(&state, id)
            .into_iter()
            .map(|one| one.title)
            .collect()
    };
    assert_eq!(titles(loose), vec!["move house".to_string()]);
    assert_eq!(
        titles(part),
        vec!["paint".to_string()],
        "its own whole is not offered again"
    );
    assert!(titles(whole).is_empty(), "a task with parts is no part");
}

#[test]
fn a_whole_put_out_of_sight_takes_no_new_parts_however_they_come() {
    let mut state = State::default();
    let whole = written(&mut state, 1, "move house", None);
    let loose = written(&mut state, 2, "paint", None);
    applied(&mut state, 3, vec![Op::TaskHide { id: whole }]);

    assert!(part_added(&state, whole, "pack").is_err());
    assert_eq!(hanging(&state, loose, Some(whole)), Err("wholeClosed"));
}
