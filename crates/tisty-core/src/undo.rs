use crate::{
    event::{Body, Event, LogEdit, Op, StepAdd, StepRef, StepText, TaskMove, TaskPatch},
    model::Status,
    state::State,
};

pub fn inverse(event: &Event, before: &State) -> Option<Op> {
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
        Op::FolderMove { id, .. } => Some(Op::FolderMove {
            id: *id,
            d: crate::event::Filed {
                folder: Some(before.folders.get(id)?.parent),
            },
        }),
        Op::DocAdd { id, .. } => Some(Op::DocDelete { id: *id }),
        Op::DocMove { id, .. } => Some(Op::DocMove {
            id: *id,
            d: crate::event::Filed {
                folder: Some(before.docs.get(id)?.folder),
            },
        }),

        Op::DocArchive { id } => Some(Op::DocUnarchive { id: *id }),
        Op::DocUnarchive { id } => Some(Op::DocArchive { id: *id }),

        Op::FolderDelete { .. } | Op::DocDelete { .. } => None,

        Op::TaskDelete { .. } | Op::ListDelete { .. } => None,

        Op::DeviceJoin { .. }
        | Op::DeviceRemove { .. }
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
        undone.apply(&ev(999, undo));

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

    #[test]
    fn completing_is_undone() {
        let id = Ulid::generate();
        let (before, undone) =
            round_trip(vec![a_task(id)], ev(2, Op::TaskDone { id, filled: false }));
        assert_eq!(before, undone);
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
        state.apply(&ev(2, undo));

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
        state.apply(&ev(3, undo));

        assert_eq!(state.tasks[&id].entry(entry).unwrap().body, "");
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
            Some(Op::TaskShow { id })
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
        after.apply(&ev(4, op));

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
        after.apply(&ev(5, op));

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
        after.apply(&ev(4, op));

        assert_eq!(after.tasks[&id], before.tasks[&id]);
    }
}
