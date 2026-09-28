use super::*;
use crate::{
    event::{DeviceId, LogAdd, TaskAdd},
    model::{DateSpec, Priority},
};
use ulid::Ulid;

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

fn ev(ms: i64, op: Op) -> Event {
    Event::new(DeviceId("dev_a".into()), at(ms), op)
}

fn round_trip(setup: Vec<Event>, action: Event) -> (State, State) {
    let before = State::replay(&setup);

    let mut after = before.clone();
    after.apply(&action);

    let undo = inverse(&action, &before).expect("no inverse");
    let mut undone = after.clone();
    for (n, op) in undo.into_iter().enumerate() {
        undone.apply(&ev(999 + n as i64, op));
    }

    (before, undone)
}

fn a_task(id: Ulid) -> Event {
    ev(
        1,
        Op::TaskAdd {
            id,
            d: TaskAdd::new("ship it", "a0"),
        },
    )
}

fn converted(id: Ulid, to: Option<crate::Reading>) -> Op {
    Op::TaskUpdate {
        id,
        d: TaskPatch {
            read_as: Some(to),
            ..Default::default()
        },
    }
}

fn retitled(id: Ulid, title: &str) -> Op {
    Op::TaskUpdate {
        id,
        d: TaskPatch {
            title: Some(title.into()),
            ..Default::default()
        },
    }
}

#[test]
fn converting_is_undone_to_what_it_read_as_before() {
    let id = Ulid::generate();
    let closed = vec![a_task(id), ev(2, Op::TaskDone { id, filled: false })];

    let (before, undone) = round_trip(
        closed.clone(),
        ev(3, converted(id, Some(crate::Reading::Story))),
    );
    assert_eq!(before, undone);
    assert_eq!(undone.tasks[&id].read_as, None);

    let mut kept = closed;
    kept.push(ev(3, converted(id, Some(crate::Reading::Story))));
    let (before, undone) = round_trip(kept, ev(4, converted(id, Some(crate::Reading::Trace))));
    assert_eq!(before, undone);
    assert_eq!(undone.tasks[&id].read_as, Some(crate::Reading::Story));
}

fn opened(id: Ulid, open: bool) -> Op {
    Op::TaskUpdate {
        id,
        d: TaskPatch {
            open_to_agents: Some(open),
            ..Default::default()
        },
    }
}

#[test]
fn a_mark_taken_off_and_put_back_keeps_the_client_that_spoke() {
    let id = Ulid::generate();
    let entry = Ulid::generate();
    let mut said = ev(
        2,
        Op::TaskResolve {
            id,
            d: crate::event::Resolve::new(entry),
        },
    );
    said.via = Some("codex".into());
    let setup = vec![a_task(id), said];

    let (before, undone) = round_trip(setup, ev(3, Op::TaskUnresolve { id }));

    assert_eq!(before, undone);
    assert_eq!(
        undone.tasks[&id].resolved.as_ref().unwrap().via.as_deref(),
        Some("codex")
    );
}

#[test]
fn opening_to_agents_is_undone_to_how_the_door_stood() {
    let id = Ulid::generate();

    let (before, undone) = round_trip(vec![a_task(id)], ev(2, opened(id, true)));
    assert_eq!(before, undone);
    assert!(!undone.tasks[&id].open_to_agents);

    let (before, undone) = round_trip(
        vec![a_task(id), ev(2, opened(id, true))],
        ev(3, opened(id, false)),
    );
    assert_eq!(before, undone);
    assert!(undone.tasks[&id].open_to_agents);
}

/// A conversion and an edit, interleaved: each undo takes back its own and leaves the other.
#[test]
fn undoing_an_edit_keeps_the_conversion_and_the_other_way_round() {
    let id = Ulid::generate();
    let mut setup = vec![a_task(id), ev(2, Op::TaskDone { id, filled: false })];
    setup.push(ev(3, converted(id, Some(crate::Reading::Story))));

    let (before, undone) = round_trip(setup.clone(), ev(4, retitled(id, "ship it, later")));
    assert_eq!(before, undone);
    assert_eq!(undone.tasks[&id].read_as, Some(crate::Reading::Story));
    assert_eq!(undone.tasks[&id].title, "ship it");

    let mut setup = vec![a_task(id), ev(2, Op::TaskDone { id, filled: false })];
    setup.push(ev(3, retitled(id, "ship it, later")));
    let (before, undone) = round_trip(setup, ev(4, converted(id, Some(crate::Reading::Trace))));
    assert_eq!(before, undone);
    assert_eq!(undone.tasks[&id].title, "ship it, later");
    assert_eq!(undone.tasks[&id].read_as, None);
}

#[test]
fn completing_is_undone() {
    let id = Ulid::generate();
    let (before, undone) = round_trip(vec![a_task(id)], ev(2, Op::TaskDone { id, filled: false }));
    assert_eq!(before, undone);
}

fn marked(ms: i64, id: Ulid, entry: Ulid) -> Vec<Event> {
    vec![
        ev(
            ms,
            Op::TaskLog {
                id,
                d: LogAdd::new(entry, "0 errores"),
            },
        ),
        ev(
            ms,
            Op::TaskResolve {
                id,
                d: Resolve::new(entry),
            },
        ),
    ]
}

#[test]
fn undoing_the_first_word_that_it_is_done_takes_the_mark_off() {
    let id = Ulid::generate();
    let entry = Ulid::generate();
    let mut setup = vec![a_task(id)];
    setup.push(marked(2, id, entry).remove(0));

    let (before, undone) = round_trip(
        setup,
        ev(
            3,
            Op::TaskResolve {
                id,
                d: Resolve::new(entry),
            },
        ),
    );

    assert!(undone.tasks[&id].resolved.is_none());
    assert_eq!(before, undone);
}

#[test]
fn undoing_a_second_word_puts_the_first_one_back() {
    let id = Ulid::generate();
    let first = Ulid::generate();
    let again = Ulid::generate();
    let mut setup = vec![a_task(id)];
    setup.extend(marked(2, id, first));
    setup.push(ev(
        3,
        Op::TaskLog {
            id,
            d: LogAdd::new(again, "y esta vez de verdad"),
        },
    ));

    let (_, undone) = round_trip(
        setup,
        ev(
            4,
            Op::TaskResolve {
                id,
                d: Resolve::new(again),
            },
        ),
    );

    assert_eq!(
        undone.tasks[&id].resolved.as_ref().map(|one| one.entry),
        Some(first),
        "the mark it had before the second word is the one that comes back"
    );
}

#[test]
fn undoing_the_person_rejecting_it_puts_the_mark_back() {
    let id = Ulid::generate();
    let entry = Ulid::generate();
    let mut setup = vec![a_task(id)];
    setup.extend(marked(2, id, entry));

    let (_, undone) = round_trip(setup, ev(3, Op::TaskUnresolve { id }));

    let back = undone.tasks[&id]
        .resolved
        .as_ref()
        .expect("the mark is back");
    assert_eq!(back.entry, entry);
    assert_eq!(back.by, DeviceId("dev_a".into()));
}

#[test]
fn taking_back_a_rejection_gives_the_word_back_to_whoever_said_it() {
    let id = Ulid::generate();
    let entry = Ulid::generate();
    let agent = DeviceId("dev_agent".into());
    let spoke = Event::new(
        agent.clone(),
        at(2),
        Op::TaskResolve {
            id,
            d: Resolve::new(entry),
        },
    );
    let setup = vec![
        a_task(id),
        ev(
            2,
            Op::TaskLog {
                id,
                d: LogAdd::new(entry, "0 errores"),
            },
        ),
        spoke,
    ];

    let (_, undone) = round_trip(setup, ev(3, Op::TaskUnresolve { id }));

    let back = undone.tasks[&id]
        .resolved
        .as_ref()
        .expect("la marca vuelve");
    assert_eq!(
        back.by, agent,
        "quien la puso fue el agente, no quien deshace"
    );
    assert_eq!(
        back.at,
        at(2),
        "y la hora es la de entonces, no la de ahora"
    );
}

#[test]
fn there_is_nothing_to_undo_in_rejecting_what_was_never_marked() {
    let id = Ulid::generate();
    let before = State::replay(&[a_task(id)]);

    assert!(inverse(&ev(2, Op::TaskUnresolve { id }), &before).is_none());
}

#[test]
fn dropping_is_undone() {
    let id = Ulid::generate();
    let (before, undone) = round_trip(vec![a_task(id)], ev(2, Op::TaskDrop { id }));
    assert_eq!(before, undone);
}

#[test]
fn reopening_restores_the_status_it_had() {
    let id = Ulid::generate();
    let (_, undone) = round_trip(
        vec![a_task(id), ev(2, Op::TaskDrop { id })],
        ev(3, Op::TaskReopen { id }),
    );
    assert_eq!(undone.tasks[&id].status, Status::Dropped);
}

#[test]
fn undoing_a_reopen_folds_the_task_away_again_if_that_is_where_it_was() {
    let id = Ulid::generate();
    let (before, undone) = round_trip(
        vec![
            a_task(id),
            ev(2, Op::TaskDone { id, filled: false }),
            ev(3, Op::TaskHide { id }),
        ],
        ev(4, Op::TaskReopen { id }),
    );

    assert!(before.tasks[&id].hidden);
    assert_eq!(undone.tasks[&id].status, Status::Done);
    assert!(
        undone.tasks[&id].hidden,
        "it was folded away before, so undoing has to fold it away again"
    );
}

#[test]
fn undoing_a_reopen_restamps_the_completion_time() {
    let id = Ulid::generate();
    let (before, undone) = round_trip(
        vec![a_task(id), ev(2, Op::TaskDone { id, filled: false })],
        ev(3, Op::TaskReopen { id }),
    );

    assert_eq!(before.tasks[&id].completed_at, Some(at(2)));
    assert_eq!(undone.tasks[&id].completed_at, Some(at(999)));
    assert_eq!(undone.tasks[&id].status, Status::Done);
}

#[test]
fn a_field_returns_to_its_previous_value() {
    let id = Ulid::generate();
    let (before, undone) = round_trip(
        vec![
            a_task(id),
            ev(
                2,
                Op::TaskUpdate {
                    id,
                    d: TaskPatch {
                        priority: Some(Priority::Decide),
                        ..Default::default()
                    },
                },
            ),
        ],
        ev(
            3,
            Op::TaskUpdate {
                id,
                d: TaskPatch {
                    priority: Some(Priority::Do),
                    ..Default::default()
                },
            },
        ),
    );
    assert_eq!(before, undone);
    assert_eq!(undone.tasks[&id].priority, Priority::Decide);
}

#[test]
fn setting_a_date_on_a_task_that_had_none_is_undone_to_none() {
    let id = Ulid::generate();
    let (before, undone) = round_trip(
        vec![a_task(id)],
        ev(
            2,
            Op::TaskUpdate {
                id,
                d: TaskPatch {
                    date: Some(Some(DateSpec::all_day(
                        "2026-08-05".parse().unwrap(),
                        "UTC",
                    ))),
                    ..Default::default()
                },
            },
        ),
    );
    assert_eq!(before, undone);
    assert!(undone.tasks[&id].date.is_none());
}

#[test]
fn adding_a_task_is_undone_by_deleting_it() {
    let id = Ulid::generate();
    let mut state = State::default();
    let add = a_task(id);
    state.apply(&add);

    let undo = inverse(&add, &State::default()).unwrap();
    state.apply(&ev(2, undo[0].clone()));

    assert!(state.tasks.is_empty());
}

#[test]
fn a_step_returns_to_its_previous_text() {
    let (id, step) = (Ulid::generate(), Ulid::generate());
    let (before, undone) = round_trip(
        vec![
            a_task(id),
            ev(
                2,
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step,
                        text: "original".into(),
                        order: "a0".into(),
                    },
                },
            ),
        ],
        ev(
            3,
            Op::StepText {
                id,
                d: StepText {
                    step,
                    text: "changed".into(),
                },
            },
        ),
    );
    assert_eq!(before, undone);
    assert_eq!(undone.tasks[&id].steps[0].text, "original");
}

#[test]
fn a_removed_step_comes_back_whole() {
    let (id, step) = (Ulid::generate(), Ulid::generate());
    let (before, undone) = round_trip(
        vec![
            a_task(id),
            ev(
                2,
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step,
                        text: "reproduce it".into(),
                        order: "a1".into(),
                    },
                },
            ),
        ],
        ev(
            3,
            Op::StepRemove {
                id,
                d: StepRef { step },
            },
        ),
    );
    assert_eq!(before, undone);
    assert_eq!(undone.tasks[&id].steps.len(), 1);
}

#[test]
fn a_journal_entry_is_emptied_not_removed() {
    let (id, entry) = (Ulid::generate(), Ulid::generate());
    let mut state = State::replay(&[a_task(id)]);
    let action = ev(
        2,
        Op::TaskLog {
            id,
            d: LogAdd::new(entry, "written by mistake"),
        },
    );
    state.apply(&action);

    let undo = inverse(&action, &State::replay(&[a_task(id)])).unwrap();
    state.apply(&ev(3, undo[0].clone()));

    assert_eq!(state.tasks[&id].entry(entry).unwrap().body, "");
}

#[test]
fn a_lock_is_not_walked_back_by_undoing_it_from_anywhere_else() {
    let id = Ulid::generate();
    let state = State::replay(&[ev(
        1,
        Op::DocAdd {
            id,
            d: crate::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: "dev0-0001".into(),
                order: "a0".into(),
                folder: None,
                page_of: None,
            },
        },
    )]);

    assert_eq!(inverse(&ev(2, Op::DocLock { id }), &state), None);
    assert_eq!(inverse(&ev(2, Op::DocUnlock { id }), &state), None);
}

#[test]
fn deleting_has_no_inverse() {
    let id = Ulid::generate();
    let state = State::replay(&[a_task(id)]);
    assert!(inverse(&ev(2, Op::TaskDelete { id }), &state).is_none());
}

#[test]
fn undoing_a_redundant_fold_leaves_it_folded() {
    let mut before = State::default();
    let id = Ulid::generate();
    before.apply(&ev(
        1,
        Op::TaskAdd {
            id,
            d: TaskAdd::new("revisar", "a0"),
        },
    ));
    before.apply(&ev(2, Op::TaskHide { id }));

    let again = ev(3, Op::TaskHide { id });
    assert_eq!(inverse(&again, &before), None);

    let mut open = State::default();
    open.apply(&ev(
        1,
        Op::TaskAdd {
            id,
            d: TaskAdd::new("revisar", "a0"),
        },
    ));
    assert_eq!(
        inverse(&ev(2, Op::TaskHide { id }), &open),
        Some(vec![Op::TaskShow { id }])
    );
}
