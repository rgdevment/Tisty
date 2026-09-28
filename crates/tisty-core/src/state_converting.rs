use super::*;
use crate::event::{DeviceId, Event, LogAdd, TaskPatch};
use crate::model::{STORY_AT, Stays};
use ulid::Ulid;

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

fn ev(ms: i64, who: &str, op: Op) -> Event {
    Event::new(DeviceId(who.into()), at(ms), op)
}

fn converted(id: TaskId, to: Option<Reading>) -> Op {
    Op::TaskUpdate {
        id,
        d: TaskPatch {
            read_as: Some(to),
            ..Default::default()
        },
    }
}

fn a_line(id: TaskId, said: &str) -> Op {
    Op::TaskLog {
        id,
        d: LogAdd::new(Ulid::generate(), said),
    }
}

const LONG: &str = "another long entry about the parcel, the depot, the neighbour and the \
                    courier who never rings twice";

/// A closed errand with nothing written: the trace every archive is mostly made of.
fn closed(state: &mut State, ms: i64, who: &str, title: &str) -> TaskId {
    let id = Ulid::generate();
    state.apply(&ev(
        ms,
        who,
        Op::TaskAdd {
            id,
            d: crate::event::TaskAdd::new(title, "a0"),
        },
    ));
    state.apply(&ev(ms + 1, who, Op::TaskDone { id, filled: false }));
    id
}

fn a_story(state: &mut State, ms: i64, who: &str, title: &str) -> TaskId {
    let id = closed(state, ms, who, title);
    for n in 0..3 {
        state.apply(&ev(ms + 2 + n, who, a_line(id, LONG)));
    }
    assert_eq!(state.tasks[&id].reading(), Reading::Story);
    id
}

#[test]
fn a_conversion_is_applied_and_taken_back_by_null() {
    let mut state = State::default();
    let id = closed(&mut state, 1, "dev_laptop", "comprar pan");

    state.apply(&ev(5, "dev_laptop", converted(id, Some(Reading::Story))));
    assert_eq!(state.tasks[&id].read_as, Some(Reading::Story));
    assert_eq!(state.tasks[&id].reading(), Reading::Story);

    state.apply(&ev(6, "dev_laptop", converted(id, None)));
    assert_eq!(state.tasks[&id].read_as, None);
    assert_eq!(state.tasks[&id].reading(), Reading::Trace);
}

#[test]
fn a_conversion_to_routine_is_ignored_not_a_clearing() {
    let mut state = State::default();
    let id = closed(&mut state, 1, "dev_laptop", "comprar pan");
    state.apply(&ev(5, "dev_laptop", converted(id, Some(Reading::Story))));

    state.apply(&ev(6, "dev_laptop", converted(id, Some(Reading::Routine))));

    assert_eq!(state.tasks[&id].read_as, Some(Reading::Story));
}

#[test]
fn an_assistant_cannot_convert_but_the_rest_of_its_patch_lands() {
    let mut state = State::default();
    state.apply(&ev(
        1,
        "dev_agent",
        Op::DeviceJoin {
            d: DeviceId("dev_agent".into()),
            k: Some(crate::event::DeviceKind::Agent),
        },
    ));
    let id = closed(&mut state, 2, "dev_agent", "comprar pan");

    state.apply(&ev(
        5,
        "dev_agent",
        Op::TaskUpdate {
            id,
            d: TaskPatch {
                read_as: Some(Some(Reading::Story)),
                title: Some("comprar pan integral".into()),
                ..Default::default()
            },
        },
    ));

    let task = &state.tasks[&id];
    assert_eq!(task.read_as, None, "converting is the person's");
    assert_eq!(
        task.title, "comprar pan integral",
        "on what it filed, the rest lands"
    );
}

#[test]
fn reopening_keeps_the_conversion_and_clears_only_hidden() {
    let mut state = State::default();
    let id = closed(&mut state, 1, "dev_laptop", "comprar pan");
    state.apply(&ev(5, "dev_laptop", converted(id, Some(Reading::Story))));
    state.apply(&ev(6, "dev_laptop", Op::TaskHide { id }));

    state.apply(&ev(7, "dev_laptop", Op::TaskReopen { id }));

    let task = &state.tasks[&id];
    assert!(!task.hidden);
    assert_eq!(task.read_as, Some(Reading::Story));
    assert_eq!(task.erasable(), Err(Stays::Open));
}

#[test]
fn the_trace_is_what_the_layer_shows_and_no_more() {
    let mut state = State::default();
    let seen = closed(&mut state, 1, "dev_laptop", "comprar pan");
    let hidden = closed(&mut state, 10, "dev_laptop", "regar");
    state.apply(&ev(12, "dev_laptop", Op::TaskHide { id: hidden }));
    let dropped = closed(&mut state, 20, "dev_laptop", "llamar");
    state.apply(&ev(22, "dev_laptop", Op::TaskReopen { id: dropped }));
    state.apply(&ev(23, "dev_laptop", Op::TaskDrop { id: dropped }));
    let story = a_story(&mut state, 30, "dev_laptop", "la mudanza");
    let kept = closed(&mut state, 40, "dev_laptop", "el certificado");
    state.apply(&ev(42, "dev_laptop", converted(kept, Some(Reading::Story))));
    let demoted = a_story(&mut state, 50, "dev_laptop", "el seguro");
    state.apply(&ev(
        56,
        "dev_laptop",
        converted(demoted, Some(Reading::Trace)),
    ));
    let turn = Ulid::generate();
    let mut d = crate::event::TaskAdd::new("pastillas", "a1");
    d.after = Some(Ulid::generate());
    state.apply(&ev(60, "dev_laptop", Op::TaskAdd { id: turn, d }));
    state.apply(&ev(
        61,
        "dev_laptop",
        Op::TaskDone {
            id: turn,
            filled: false,
        },
    ));
    let open = Ulid::generate();
    state.apply(&ev(
        70,
        "dev_laptop",
        Op::TaskAdd {
            id: open,
            d: crate::event::TaskAdd::new("pendiente", "a2"),
        },
    ));

    let mut listed: Vec<TaskId> = state.the_trace().map(|t| t.id).collect();
    listed.sort();
    let mut wanted = vec![seen, demoted];
    wanted.sort();
    assert_eq!(listed, wanted, "{listed:?}");
    for (id, why) in [
        (hidden, "hidden is already out of sight"),
        (dropped, "dropped is folded away"),
        (story, "a story stays"),
        (kept, "kept as a story"),
        (turn, "a routine's turn"),
        (open, "still open"),
    ] {
        assert!(!listed.contains(&id), "{why}");
    }

    assert!(listed.contains(&demoted));
}

/// A root whose repeat was taken off reads as a trace, but turns still hang from it:
/// erasing it would cut the series, so it stays with the routines.
#[test]
fn a_root_other_turns_hang_from_is_a_routine_however_bare() {
    let mut state = State::default();
    let root = closed(&mut state, 1, "dev_laptop", "pastillas");
    let turn = Ulid::generate();
    let mut d = crate::event::TaskAdd::new("pastillas", "a1");
    d.after = Some(root);
    state.apply(&ev(10, "dev_laptop", Op::TaskAdd { id: turn, d }));
    state.apply(&ev(
        11,
        "dev_laptop",
        Op::TaskDone {
            id: turn,
            filled: false,
        },
    ));

    assert_eq!(
        state.tasks[&root].reading(),
        Reading::Trace,
        "bare, by itself"
    );
    assert_eq!(
        state.tasks[&root].erasable(),
        Ok(()),
        "the task alone cannot tell"
    );
    assert_eq!(state.erasable(root), Err(Stays::Routine), "the state can");
    assert_eq!(state.erasable(turn), Err(Stays::Routine));
    assert!(
        state.the_trace().any(|t| t.id == root) && state.the_trace().all(|t| t.id != turn),
        "the root shows in the trace as the layer shows it; the turn is a routine's"
    );
    assert_eq!(
        state.erasable(Ulid::generate()),
        Err(Stays::Open),
        "nothing there"
    );
}

#[test]
fn a_delete_that_arrives_for_a_task_kept_as_a_story_is_let_go() {
    let mut state = State::default();
    let kept = closed(&mut state, 1, "dev_a", "el certificado");
    state.apply(&ev(100, "dev_a", converted(kept, Some(Reading::Story))));
    state.apply(&ev(110, "dev_b", Op::TaskDelete { id: kept }));
    assert!(
        state.tasks.contains_key(&kept),
        "the word the person gave outlives the delete"
    );
    assert!(!state.is_erased(kept));

    let gone = closed(&mut state, 200, "dev_a", "comprar pan");
    state.apply(&ev(210, "dev_a", converted(gone, Some(Reading::Trace))));
    state.apply(&ev(220, "dev_b", Op::TaskDelete { id: gone }));
    assert!(!state.tasks.contains_key(&gone), "read as a trace, it goes");

    let unpinned = a_story(&mut state, 300, "dev_a", "la mudanza");
    state.apply(&ev(320, "dev_b", Op::TaskDelete { id: unpinned }));
    assert!(
        !state.tasks.contains_key(&unpinned),
        "only the pin guards: a story by weight was a trace somewhere, and the delete stands"
    );
}

#[test]
fn reopening_lets_a_trace_pin_go_and_keeps_a_story_pin() {
    let mut state = State::default();
    let demoted = a_story(&mut state, 1, "dev_laptop", "la mudanza");
    state.apply(&ev(
        10,
        "dev_laptop",
        converted(demoted, Some(Reading::Trace)),
    ));
    let kept = closed(&mut state, 20, "dev_laptop", "el certificado");
    state.apply(&ev(22, "dev_laptop", converted(kept, Some(Reading::Story))));

    for (n, op) in state.reopening(demoted).into_iter().enumerate() {
        state.apply(&ev(30 + n as i64, "dev_laptop", op));
    }
    for (n, op) in state.reopening(kept).into_iter().enumerate() {
        state.apply(&ev(40 + n as i64, "dev_laptop", op));
    }

    assert_eq!(
        state.tasks[&demoted].read_as, None,
        "work starts again, judged again"
    );
    assert_eq!(
        state.tasks[&kept].read_as,
        Some(Reading::Story),
        "a finish taken back"
    );
    assert!(state.tasks[&demoted].is_open() && state.tasks[&kept].is_open());
}

#[test]
fn searching_orders_by_what_the_person_read_it_as() {
    let mut state = State::default();
    let plain = closed(&mut state, 1, "dev_laptop", "pagar la luz");
    let kept = closed(&mut state, 10, "dev_laptop", "pagar el agua");
    state.apply(&ev(12, "dev_laptop", converted(kept, Some(Reading::Story))));
    assert!(state.tasks[&kept].weight() < state.tasks[&plain].weight() + 1);

    let (found, _) = state.searching("pagar", crate::view::Scope::Archived, 10);
    let ids: Vec<TaskId> = found.iter().map(|t| t.id).collect();

    assert_eq!(
        ids[0], kept,
        "what was kept as a story comes first: {ids:?}"
    );
}

/// Two machines: one converts, the other reopens and writes without having seen it. Whatever
/// order the events arrive in, the person's conversion holds.
#[test]
fn a_conversion_holds_whatever_the_other_machine_writes_and_in_any_order() {
    let mut seed = State::default();
    let id = a_story(&mut seed, 1, "dev_a", "la mudanza");
    let mut later = vec![
        ev(100, "dev_a", converted(id, Some(Reading::Trace))),
        ev(110, "dev_b", Op::TaskReopen { id }),
        ev(120, "dev_b", a_line(id, LONG)),
        ev(130, "dev_b", Op::TaskDone { id, filled: false }),
    ];

    for round in 0..2 {
        if round == 1 {
            later.reverse();
        }
        let mut sorted = later.clone();
        sorted.sort_by(|one, two| one.sort_key().cmp(&two.sort_key()));
        let mut state = seed.clone();
        for one in &sorted {
            state.apply(one);
        }
        let task = &state.tasks[&id];
        assert_eq!(task.reading(), Reading::Trace, "round {round}");
        assert_eq!(task.erasable(), Ok(()), "round {round}");
        assert!(
            task.weight() >= STORY_AT,
            "the weight still tells what was written"
        );
    }

    let mut state = seed.clone();
    for one in &later {
        state.apply(one);
    }
    state.apply(&ev(140, "dev_b", converted(id, Some(Reading::Story))));
    assert_eq!(
        state.tasks[&id].reading(),
        Reading::Story,
        "the last word wins"
    );

    let mut state = seed;
    state.apply(&ev(100, "dev_a", converted(id, Some(Reading::Trace))));
    state.apply(&ev(101, "dev_a", Op::TaskDelete { id }));
    state.apply(&ev(120, "dev_b", a_line(id, LONG)));
    assert!(
        !state.tasks.contains_key(&id),
        "what arrives after the tombstone is let go"
    );
}
