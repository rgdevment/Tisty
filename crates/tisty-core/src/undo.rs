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
    if let Op::DocArchive { id } = &event.op
        && !before.assistants.contains(&event.device)
        && let Some(said) = before.docs.get(id).and_then(|one| one.flagged.clone())
    {
        back.push(Op::DocFlag {
            id: *id,
            d: crate::event::Flag::new(said.body)
                .said_by(said.at, said.by)
                .through(said.via),
        });
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

        Op::DocLock { .. }
        | Op::DocUnlock { .. }
        | Op::DocSaid { .. }
        | Op::DocFlag { .. }
        | Op::DocUnflag { .. } => None,

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
#[path = "undo_test.rs"]
mod tests;

#[cfg(test)]
#[path = "undo_dropping.rs"]
mod dropping;

#[cfg(test)]
#[path = "undo_hanging.rs"]
mod hanging;
