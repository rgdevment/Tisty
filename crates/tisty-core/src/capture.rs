use ulid::Ulid;

use crate::{
    State,
    event::{ListAdd, Op, TaskAdd},
    model::{DateSpec, ListId, Priority, Tag, TaskId},
};

/// `Marked` creates the list if it is missing; `Named` demands it exists.
#[derive(Debug, Clone, PartialEq)]
pub enum Filing {
    Marked(String),
    Named(String),
    /// The window already knows which list: it is the one being looked at.
    Kept(ListId),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Draft {
    pub title: String,
    pub date: Option<DateSpec>,
    pub deadline: Option<DateSpec>,
    pub priority: Option<Priority>,
    pub tags: Vec<Tag>,
    pub filing: Option<Filing>,
    pub repeat: Option<crate::model::Repeat>,
}

#[derive(Debug, thiserror::Error)]
pub enum Rejected {
    #[error("a title is required")]
    Untitled,
    #[error("no list matches «{0}»")]
    NoSuchList(String),
    #[error("several lists match «{0}»")]
    AmbiguousList(String),
}

/// Applied and undone as one batch.
pub struct Plan {
    pub task: TaskId,
    pub ops: Vec<Op>,
}

pub fn plan(state: &State, draft: Draft) -> Result<Plan, Rejected> {
    let title = draft.title.trim();
    if title.is_empty() {
        return Err(Rejected::Untitled);
    }

    let mut ops = Vec::with_capacity(2);
    let list = match &draft.filing {
        None => None,
        Some(Filing::Kept(id)) => Some(*id),
        Some(Filing::Named(name)) => Some(existing(state, name)?),
        Some(Filing::Marked(name)) => Some(match state.find_list(name).as_slice() {
            [one] => one.id,
            [] => {
                let id = Ulid::generate();
                ops.push(Op::ListAdd {
                    id,
                    d: ListAdd {
                        name: name.clone(),
                        order: state.next_list_order(),
                        color: None,
                    },
                });
                id
            }
            _ => return Err(Rejected::AmbiguousList(name.clone())),
        }),
    };

    let task = Ulid::generate();
    ops.push(Op::TaskAdd {
        id: task,
        d: TaskAdd {
            date: draft.date,
            deadline: draft.deadline,
            priority: draft.priority,
            tags: draft.tags,
            list,
            repeat: draft.repeat,
            ..TaskAdd::new(title, state.next_task_order())
        },
    });

    Ok(Plan { task, ops })
}

fn existing(state: &State, name: &str) -> Result<ListId, Rejected> {
    match state.find_list(name).as_slice() {
        [one] => Ok(one.id),
        [] => Err(Rejected::NoSuchList(name.to_string())),
        _ => Err(Rejected::AmbiguousList(name.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{DeviceId, Event};

    fn with_lists(names: &[&str]) -> State {
        let events: Vec<Event> = names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                Event::new(
                    DeviceId("dev_a".into()),
                    jiff::Timestamp::from_millisecond(i as i64 + 1).unwrap(),
                    Op::ListAdd {
                        id: Ulid::generate(),
                        d: ListAdd {
                            name: (*name).to_string(),
                            order: format!("a{i}"),
                            color: None,
                        },
                    },
                )
            })
            .collect();
        State::replay(&events)
    }

    fn drafted(title: &str, filing: Option<Filing>) -> Draft {
        Draft {
            title: title.to_string(),
            filing,
            ..Draft::default()
        }
    }

    fn listed(state: &State, plan: &Plan) -> Option<ListId> {
        let applied = replayed(state, plan);
        applied.tasks[&plan.task].list
    }

    fn replayed(state: &State, plan: &Plan) -> State {
        let mut applied = state.clone();
        for (i, op) in plan.ops.iter().enumerate() {
            applied.apply(&Event::new(
                DeviceId("dev_a".into()),
                jiff::Timestamp::from_millisecond(1_000 + i as i64).unwrap(),
                op.clone(),
            ));
        }
        applied
    }

    #[test]
    fn a_marked_list_that_is_missing_is_created_alongside_the_task() {
        let state = with_lists(&[]);
        let plan = plan(
            &state,
            drafted("book a haircut", Some(Filing::Marked("home".into()))),
        )
        .unwrap();

        assert_eq!(plan.ops.len(), 2, "the list and the task travel together");
        let applied = replayed(&state, &plan);
        assert_eq!(applied.lists.len(), 1);
        assert_eq!(
            applied.tasks[&plan.task].list,
            applied.lists.keys().next().copied()
        );
    }

    #[test]
    fn a_marked_list_that_exists_is_reused() {
        let state = with_lists(&["work"]);
        let plan = plan(
            &state,
            drafted("fix the checkout", Some(Filing::Marked("work".into()))),
        )
        .unwrap();

        assert_eq!(plan.ops.len(), 1);
        assert_eq!(listed(&state, &plan), state.lists.keys().next().copied());
    }

    #[test]
    fn a_named_list_that_is_missing_is_refused_instead_of_created() {
        let state = with_lists(&["work"]);
        let outcome = plan(
            &state,
            drafted("fix the checkout", Some(Filing::Named("home".into()))),
        );

        assert!(matches!(outcome, Err(Rejected::NoSuchList(_))));
    }

    #[test]
    fn an_ambiguous_marker_is_refused_rather_than_creating_another() {
        let state = with_lists(&["work trip", "work notes"]);
        let outcome = plan(
            &state,
            drafted("book a flight", Some(Filing::Marked("work".into()))),
        );

        assert!(matches!(outcome, Err(Rejected::AmbiguousList(_))));
    }

    #[test]
    fn an_exact_name_wins_over_the_lists_that_merely_contain_it() {
        let state = with_lists(&["work", "work notes"]);
        let plan = plan(
            &state,
            drafted("fix the checkout", Some(Filing::Marked("work".into()))),
        )
        .unwrap();

        assert_eq!(plan.ops.len(), 1);
        let applied = replayed(&state, &plan);
        assert_eq!(
            applied.lists[&applied.tasks[&plan.task].list.unwrap()].name,
            "work"
        );
    }

    #[test]
    fn a_capture_that_is_all_markers_has_no_title_and_is_refused() {
        let state = with_lists(&[]);
        let outcome = plan(&state, drafted("   ", Some(Filing::Marked("home".into()))));

        assert!(matches!(outcome, Err(Rejected::Untitled)));
    }

    #[test]
    fn a_task_is_ordered_after_the_ones_already_there() {
        let state = with_lists(&[]);
        let first = plan(&state, drafted("one", None)).unwrap();
        let after = replayed(&state, &first);
        let second = plan(&after, drafted("two", None)).unwrap();

        let ordered = replayed(&after, &second);
        assert!(ordered.tasks[&first.task].order < ordered.tasks[&second.task].order);
    }
}
