use super::*;
use crate::event::{DeviceId, DocAdd, Event, ListAdd, Said, TaskAdd};
use ulid::Ulid;

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

fn ev(ms: i64, op: Op) -> Event {
    Event::new(DeviceId("dev_a".into()), at(ms), op)
}

fn list(state: &mut State, ms: i64, name: &str, order: &str) -> ListId {
    let id = Ulid::generate();
    state.apply(&ev(
        ms,
        Op::ListAdd {
            id,
            d: ListAdd {
                name: name.into(),
                order: order.into(),
                color: None,
            },
        },
    ));
    id
}

fn task_in(state: &mut State, ms: i64, list: ListId) {
    let mut d = TaskAdd::new("algo", "a0");
    d.list = Some(list);
    state.apply(&ev(
        ms,
        Op::TaskAdd {
            id: Ulid::generate(),
            d,
        },
    ));
}

fn guide(state: &mut State, ms: i64, file: &str, title: &str) -> DocId {
    let id = Ulid::generate();
    state.apply(&ev(
        ms,
        Op::DocAdd {
            id,
            d: DocAdd {
                wrote: None,
                guest: true,
                made: Some(at(ms)),
                by: Some(GUIDE_BY.into()),
                said: Some(Said {
                    title: title.into(),
                    ..Default::default()
                }),
                file: file.into(),
                order: "a0".into(),
                folder: None,
                page_of: None,
            },
        },
    ));
    id
}

#[test]
fn two_lists_of_one_name_become_the_one_that_holds_the_tasks() {
    let mut state = State::default();
    let sample = list(&mut state, 1, "Trabajo", "a0");
    let real = list(&mut state, 2, "trabajo ", "b0");
    task_in(&mut state, 3, real);
    task_in(&mut state, 4, sample);
    task_in(&mut state, 5, real);

    for op in doubled(&state) {
        state.apply(&ev(10, op));
    }

    assert!(state.lists.contains_key(&real));
    assert!(!state.lists.contains_key(&sample));
    assert_eq!(
        state
            .tasks
            .values()
            .filter(|one| one.list == Some(real))
            .count(),
        3,
        "a task in the list that went is not left without one"
    );
}

#[test]
fn lists_with_different_names_are_left_alone() {
    let mut state = State::default();
    list(&mut state, 1, "Trabajo", "a0");
    list(&mut state, 2, "Personal", "b0");

    assert!(doubled(&state).is_empty());
}

#[test]
fn the_guide_each_side_brought_is_kept_once() {
    let mut state = State::default();
    let first = guide(&mut state, 1, "aaaa-0001", "Cómo funciona Tisty");
    let second = guide(&mut state, 2, "bbbb-0001", "Cómo funciona Tisty");
    let other = guide(&mut state, 3, "bbbb-0002", "La Rina");

    for op in doubled(&state) {
        state.apply(&ev(10, op));
    }

    assert!(!state.docs[&first].archived);
    assert!(state.docs[&second].archived);
    assert!(!state.docs[&other].archived);
}
