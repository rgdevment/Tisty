use crate::{
    event::{Body, Event, LogEdit, Op, Resolve, StepAdd, StepRef, StepText, TaskMove, TaskPatch},
    model::Status,
    state::State,
};

/// What puts an event back, in the order it has to be applied. Some of them take more than one
/// operation: reopening a task clears more than its status, and one op cannot say all of it.
pub fn inverse(event: &Event, before: &State) -> Option<Vec<Op>> {
    let one = undoing(event, before)?;
    let mut back = vec![one];
    if let Op::TaskReopen { id } = &event.op
        && before.tasks.get(id).is_some_and(|was| was.hidden)
    {
        back.push(Op::TaskHide { id: *id });
    }
    Some(back)
}

pub fn unhung(events: &[Event], now: &State, id: crate::model::DocId) -> crate::event::Filed {
    let hung = |op: &Op| {
        matches!(op, Op::DocMove { id: which, d }
            if which == &id && matches!(d.page_of, Some(Some(_))))
    };
    if let Some(at) = events.iter().rposition(|one| hung(&one.op))
        && let Some(Op::DocMove { d, .. }) = undoing(&events[at], &State::replay(&events[..at]))
        && matches!(d.page_of, Some(None))
        && d.folder
            .is_some_and(|home| home.is_none_or(|home| now.folders.contains_key(&home)))
    {
        return d;
    }

    let here = now.docs.get(&id);
    let folder = match here.and_then(|one| one.page_of) {
        Some(up) => now.docs.get(&up).and_then(|one| one.folder),
        None => here.and_then(|one| one.folder),
    };
    crate::event::Filed {
        folder: Some(folder),
        page_of: Some(None),
        order: Some(crate::order::last_of(
            now.docs
                .values()
                .filter(|one| one.id != id && one.page_of.is_none() && one.folder == folder)
                .map(|one| one.order.as_str()),
        )),
    }
}

fn undoing(event: &Event, before: &State) -> Option<Op> {
    match &event.op {
        Op::TaskAdd { id, .. } => Some(Op::TaskDelete { id: *id }),

        Op::TaskDone { id, .. } | Op::TaskDrop { id } => {
            let was = before.tasks.get(id)?;
            match was.status {
                Status::Open => Some(Op::TaskReopen { id: *id }),
                Status::Done => Some(Op::TaskDone {
                    id: *id,
                    filled: was.filled,
                }),
                Status::Dropped => Some(Op::TaskDrop { id: *id }),
            }
        }
        Op::TaskHide { id } => (!before.tasks.get(id)?.hidden).then_some(Op::TaskShow { id: *id }),
        Op::TaskShow { id } => before
            .tasks
            .get(id)?
            .hidden
            .then_some(Op::TaskHide { id: *id }),
        Op::TaskReopen { id } => {
            let was = before.tasks.get(id)?;
            match was.status {
                Status::Done => Some(Op::TaskDone {
                    id: *id,
                    filled: was.filled,
                }),
                Status::Dropped => Some(Op::TaskDrop { id: *id }),
                Status::Open => None,
            }
        }

        Op::TaskUpdate { id, d } => {
            let task = before.tasks.get(id)?;
            Some(Op::TaskUpdate {
                id: *id,
                d: TaskPatch {
                    title: d.title.as_ref().map(|_| task.title.clone()),
                    read_as: d.read_as.as_ref().map(|_| task.read_as),
                    open_to_agents: d.open_to_agents.map(|_| task.open_to_agents),
                    date: d.date.as_ref().map(|_| task.date.clone()),
                    deadline: d.deadline.as_ref().map(|_| task.deadline.clone()),
                    priority: d.priority.map(|_| task.priority),
                    tags: d.tags.as_ref().map(|_| task.tags.clone()),
                    reminders: d.reminders.as_ref().map(|_| task.reminders.clone()),
                    repeat: d.repeat.as_ref().map(|_| task.repeat),
                },
            })
        }

        Op::TaskMove { id, d } => {
            let task = before.tasks.get(id)?;
            Some(Op::TaskMove {
                id: *id,
                d: TaskMove {
                    list: d.list.as_ref().map(|_| task.list),
                    order: d.order.as_ref().map(|_| task.order.clone()),
                },
            })
        }

        Op::TaskDescribe { id, .. } => Some(Op::TaskDescribe {
            id: *id,
            d: Body {
                body: before.tasks.get(id)?.description.clone(),
            },
        }),

        Op::TaskLog { id, d } => Some(Op::TaskLogEdit {
            id: *id,
            d: LogEdit {
                entry: d.entry,
                body: String::new(),
            },
        }),
        Op::TaskResolve { id, .. } => match &before.tasks.get(id)?.resolved {
            Some(was) => Some(Op::TaskResolve {
                id: *id,
                d: Resolve::new(was.entry)
                    .said_by(was.at, was.by.clone())
                    .through(was.via.clone()),
            }),
            None => Some(Op::TaskUnresolve { id: *id }),
        },
        Op::TaskUnresolve { id } => {
            let was = before.tasks.get(id)?.resolved.as_ref()?;
            Some(Op::TaskResolve {
                id: *id,
                d: Resolve::new(was.entry)
                    .said_by(was.at, was.by.clone())
                    .through(was.via.clone()),
            })
        }
        Op::TaskLogEdit { id, d } => Some(Op::TaskLogEdit {
            id: *id,
            d: LogEdit {
                entry: d.entry,
                body: before.tasks.get(id)?.entry(d.entry)?.body.clone(),
            },
        }),

        Op::StepAdd { id, d } => Some(Op::StepRemove {
            id: *id,
            d: StepRef { step: d.step },
        }),
        Op::StepDone { id, d } => Some(Op::StepUndone {
            id: *id,
            d: StepRef { step: d.step },
        }),
        Op::StepUndone { id, d } => Some(Op::StepDone {
            id: *id,
            d: StepRef { step: d.step },
        }),
        Op::StepText { id, d } => Some(Op::StepText {
            id: *id,
            d: StepText {
                step: d.step,
                text: before.tasks.get(id)?.step(d.step)?.text.clone(),
            },
        }),
        Op::StepRemove { id, d } => {
            let step = before.tasks.get(id)?.step(d.step)?;
            Some(Op::StepAdd {
                id: *id,
                d: StepAdd {
                    step: step.id,
                    text: step.text.clone(),
                    order: step.order.clone(),
                },
            })
        }
        Op::StepReorder { id, d } => Some(Op::StepReorder {
            id: *id,
            d: crate::event::StepReorder {
                step: d.step,
                order: before.tasks.get(id)?.step(d.step)?.order.clone(),
            },
        }),

        Op::ListArchive { id } => Some(Op::ListUnarchive { id: *id }),
        Op::ListUnarchive { id } => Some(Op::ListArchive { id: *id }),
        Op::ListRename { id, .. } => Some(Op::ListRename {
            id: *id,
            d: crate::event::Name {
                name: before.lists.get(id)?.name.clone(),
            },
        }),

        Op::ListLook { id, d } => {
            let was = before.lists.get(id)?;
            Some(Op::ListLook {
                id: *id,
                d: crate::event::Look {
                    icon: d.icon.as_ref().map(|_| was.icon.clone()),
                    color: d.color.as_ref().map(|_| was.color.clone()),
                },
            })
        }

        Op::ListAdd { id, .. } => Some(Op::ListDelete { id: *id }),

        Op::FolderAdd { id, .. } => Some(Op::FolderDelete { id: *id }),
        Op::FolderRename { id, .. } => Some(Op::FolderRename {
            id: *id,
            d: crate::event::Name {
                name: before.folders.get(id)?.name.clone(),
            },
        }),
        Op::FolderLook { id, d } => {
            let was = before.folders.get(id)?;
            Some(Op::FolderLook {
                id: *id,
                d: crate::event::Look {
                    icon: d.icon.as_ref().map(|_| was.icon.clone()),
                    color: None,
                },
            })
        }
        Op::FolderMove { id, .. } => {
            let was = before.folders.get(id)?;
            Some(Op::FolderMove {
                id: *id,
                d: crate::event::Filed {
                    folder: Some(was.parent),
                    page_of: None,
                    order: Some(was.order.clone()),
                },
            })
        }
        Op::DocAdd { id, .. } => Some(Op::DocDelete { id: *id }),
        Op::DocMove { id, d } => {
            let was = before.docs.get(id)?;
            Some(Op::DocMove {
                id: *id,
                d: crate::event::Filed {
                    // Hanging it took the folder of what it hangs from, so unhanging hands it back.
                    folder: (d.folder.is_some() || d.page_of.is_some()).then_some(was.folder),
                    page_of: d.page_of.map(|_| was.page_of),
                    order: Some(was.order.clone()),
                },
            })
        }

        Op::DocSigned { .. } => None,
        Op::DocArchive { id } => Some(Op::DocUnarchive { id: *id }),
        Op::DocUnarchive { id } => Some(Op::DocArchive { id: *id }),
        Op::FolderArchive { id } => Some(Op::FolderUnarchive { id: *id }),
        Op::FolderUnarchive { id } => Some(Op::FolderArchive { id: *id }),
        Op::FolderDelete { .. } | Op::DocDelete { .. } => None,

        Op::DocLock { .. } | Op::DocUnlock { .. } | Op::DocSaid { .. } => None,

        Op::TaskDelete { .. } | Op::ListDelete { .. } => None,

        Op::DeviceJoin { .. }
        | Op::DeviceHost { .. }
        | Op::DeviceRemove { .. }
        | Op::Signed { .. }
        | Op::AttachRetire { .. }
        | Op::StoresJoined { .. } => None,
    }
}

#[cfg(test)]
mod tests {
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
        let (before, undone) =
            round_trip(vec![a_task(id)], ev(2, Op::TaskDone { id, filled: false }));
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
}

#[cfg(test)]
mod dropping {
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
}

#[cfg(test)]
mod hanging {
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

    fn doc(
        events: &mut Vec<Event>,
        ms: i64,
        folder: Option<crate::model::FolderId>,
    ) -> crate::model::DocId {
        let id = Ulid::generate();
        let told = State::replay(events);
        let order = crate::order::last_of(
            told.docs
                .values()
                .filter(|one| one.page_of.is_none() && one.folder == folder)
                .map(|one| one.order.as_str()),
        );
        events.push(ev(
            ms,
            Op::DocAdd {
                id,
                d: crate::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    said: None,
                    file: format!("dev_a-{ms:04}"),
                    order,
                    folder,
                    page_of: None,
                },
            },
        ));
        id
    }

    fn folder(events: &mut Vec<Event>, ms: i64) -> crate::model::FolderId {
        let id = Ulid::generate();
        events.push(ev(
            ms,
            Op::FolderAdd {
                id,
                d: crate::event::FolderAdd {
                    name: format!("Folder {ms}"),
                    parent: None,
                    order: crate::order::first(),
                    icon: None,
                    color: None,
                },
            },
        ));
        id
    }

    fn hang(events: &mut Vec<Event>, ms: i64, id: crate::model::DocId, up: crate::model::DocId) {
        let told = State::replay(events);
        let order = crate::order::last_of(
            told.docs
                .values()
                .filter(|one| one.page_of == Some(up))
                .map(|one| one.order.as_str()),
        );
        events.push(ev(
            ms,
            Op::DocMove {
                id,
                d: crate::event::Filed {
                    folder: None,
                    page_of: Some(Some(up)),
                    order: Some(order),
                },
            },
        ));
    }

    fn unhang(events: &mut Vec<Event>, ms: i64, id: crate::model::DocId) -> State {
        let told = State::replay(events);
        events.push(ev(
            ms,
            Op::DocMove {
                id,
                d: unhung(events, &told, id),
            },
        ));
        State::replay(events)
    }

    #[test]
    fn unhanging_puts_it_back_in_the_folder_and_the_place_it_came_from() {
        let mut events = Vec::new();
        let home = folder(&mut events, 1);
        let work = folder(&mut events, 2);
        let one = doc(&mut events, 3, Some(home));
        let two = doc(&mut events, 4, Some(home));
        let up = doc(&mut events, 5, Some(work));

        let was = State::replay(&events).docs[&one].clone();
        hang(&mut events, 6, one, up);
        assert_eq!(State::replay(&events).docs[&one].folder, Some(work));

        let after = unhang(&mut events, 7, one);
        assert_eq!(after.docs[&one].page_of, None);
        assert_eq!(after.docs[&one].folder, Some(home));
        assert_eq!(
            after.docs[&one].order, was.order,
            "and in the place it held"
        );
        assert!(after.docs[&two].order > after.docs[&one].order);
    }

    #[test]
    fn a_document_born_a_page_lands_beside_the_one_it_hung_from() {
        let mut events = Vec::new();
        let work = folder(&mut events, 1);
        let up = doc(&mut events, 2, Some(work));
        let beside = doc(&mut events, 3, Some(work));
        let page = Ulid::generate();
        events.push(ev(
            4,
            Op::DocAdd {
                id: page,
                d: crate::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    said: None,
                    file: "dev_a-0004".into(),
                    order: crate::order::first(),
                    folder: None,
                    page_of: Some(up),
                },
            },
        ));

        let after = unhang(&mut events, 5, page);
        assert_eq!(after.docs[&page].page_of, None);
        assert_eq!(after.docs[&page].folder, Some(work));
        assert!(
            after.docs[&page].order > after.docs[&beside].order,
            "at the end of the folder it lands in, not in front of it"
        );
    }

    #[test]
    fn unhanging_into_a_folder_that_is_gone_lands_beside_what_it_hung_from() {
        let mut events = Vec::new();
        let home = folder(&mut events, 1);
        let work = folder(&mut events, 2);
        let one = doc(&mut events, 3, Some(home));
        let up = doc(&mut events, 4, Some(work));
        hang(&mut events, 5, one, up);
        events.push(ev(6, Op::FolderDelete { id: home }));

        let after = unhang(&mut events, 7, one);
        assert_eq!(after.docs[&one].page_of, None);
        assert_eq!(after.docs[&one].folder, Some(work));
    }

    #[test]
    fn unhanging_something_that_is_not_a_page_leaves_the_folder_it_is_in() {
        let mut events = Vec::new();
        let home = folder(&mut events, 1);
        let one = doc(&mut events, 2, Some(home));

        let told = State::replay(&events);
        let d = unhung(&events, &told, one);
        assert_eq!(d.folder, Some(Some(home)), "not adrift in unfiled");

        events.push(ev(3, Op::DocMove { id: one, d }));
        let after = State::replay(&events);
        assert_eq!(after.docs[&one].folder, Some(home));
        assert_eq!(after.docs[&one].page_of, None);
    }

    #[test]
    fn a_page_moved_straight_from_one_document_to_another_is_still_unhung() {
        let mut events = Vec::new();
        let home = folder(&mut events, 1);
        let one = doc(&mut events, 2, Some(home));
        let up = doc(&mut events, 3, Some(home));
        let other = doc(&mut events, 4, Some(home));

        hang(&mut events, 5, one, up);
        hang(&mut events, 6, one, other);
        assert_eq!(State::replay(&events).docs[&one].page_of, Some(other));

        let after = unhang(&mut events, 7, one);
        assert_eq!(
            after.docs[&one].page_of, None,
            "inverting the last hang would have re-hung it under the first"
        );
        assert_eq!(after.docs[&one].folder, Some(home));
    }
}
