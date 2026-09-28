use ulid::Ulid;

use crate::{
    State,
    event::{ListAdd, Op, TaskAdd},
    model::{DateSpec, ListId, Priority, Tag, TaskId},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Filing {
    Marked(String),
    Named(String),
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
    pub source: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Rejected {
    #[error("a title is required")]
    Untitled,
    #[error("no list matches «{0}»")]
    NoSuchList(String),
    #[error("several lists match «{0}»")]
    AmbiguousList(String),
    #[error("the list «{0}» is put away")]
    ArchivedList(String),
    #[error("a series cannot end before today")]
    EndedAlready,
}

pub struct Plan {
    pub task: TaskId,
    pub ops: Vec<Op>,
}

pub fn plan(state: &State, draft: Draft) -> Result<Plan, Rejected> {
    let title = draft.title.trim();
    if title.is_empty() {
        return Err(Rejected::Untitled);
    }
    if draft
        .repeat
        .is_some_and(|over| over.ended(jiff::Zoned::now().date()))
    {
        return Err(Rejected::EndedAlready);
    }

    let mut ops = Vec::with_capacity(2);
    let list = match &draft.filing {
        None => None,
        Some(Filing::Kept(id)) => Some(*id),
        Some(Filing::Named(name)) => Some(existing(state, name)?),
        Some(Filing::Marked(name)) => Some(match state.find_list(name).as_slice() {
            [one] if one.archived => return Err(Rejected::ArchivedList(name.clone())),
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
            tags: crate::tagging::worth_keeping(&draft.tags),
            list,
            repeat: draft.repeat,
            source: draft.source,
            ..TaskAdd::new(title, state.next_task_order())
        },
    });

    Ok(Plan { task, ops })
}

fn existing(state: &State, name: &str) -> Result<ListId, Rejected> {
    match state.find_list(name).as_slice() {
        [one] if one.archived => Err(Rejected::ArchivedList(name.to_string())),
        [one] => Ok(one.id),
        [] => Err(Rejected::NoSuchList(name.to_string())),
        _ => Err(Rejected::AmbiguousList(name.to_string())),
    }
}

#[cfg(test)]
#[path = "capture_test.rs"]
mod tests;
