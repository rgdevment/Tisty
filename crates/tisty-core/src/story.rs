use std::collections::HashMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::event::{DeviceId, Event, Op};
use crate::model::{DateSpec, ListId, Priority, Reading, Repeat, StepId, Tag, TaskId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "chapter", rename_all = "lowercase")]
pub enum Chapter {
    Born {
        title: String,
    },
    Retitled {
        from: String,
        to: String,
    },
    Dated {
        from: Option<DateSpec>,
        to: Option<DateSpec>,
    },
    Bounded {
        from: Option<DateSpec>,
        to: Option<DateSpec>,
    },
    Placed {
        from: Priority,
        to: Priority,
    },
    Filed {
        from: Option<ListId>,
        to: Option<ListId>,
    },
    Tagged {
        added: Vec<Tag>,
        gone: Vec<Tag>,
    },
    Cadenced {
        from: Option<Repeat>,
        to: Option<Repeat>,
    },
    Described {
        emptied: bool,
    },
    Wrote {
        body: String,
    },
    Rewrote {
        body: String,
    },
    Planned {
        text: String,
    },
    Ticked {
        text: String,
    },
    Unticked {
        text: String,
    },
    Reworded {
        from: String,
        to: String,
    },
    Unplanned {
        text: String,
    },
    Closed,
    Dropped,
    Reopened,
    Converted {
        from: Option<Reading>,
        to: Option<Reading>,
    },
    Opened,
    Shut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub n: usize,
    pub at: Timestamp,
    pub by: DeviceId,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub undoing: bool,
    #[serde(flatten)]
    pub chapter: Chapter,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Story {
    pub id: TaskId,
    pub pages: Vec<Page>,
}

#[derive(Default)]
struct Standing {
    title: String,
    date: Option<DateSpec>,
    deadline: Option<DateSpec>,
    priority: Priority,
    list: Option<ListId>,
    tags: Vec<Tag>,
    repeat: Option<Repeat>,
    read_as: Option<Reading>,
    open_to_agents: bool,
    filed_by_agent: bool,
    described: bool,
    steps: HashMap<StepId, String>,
}

/// The same door `State::apply` keeps for an assistant's hand.
fn lets(op: &Op, standing: &Standing) -> bool {
    let attended = standing.open_to_agents || standing.filed_by_agent;
    match op {
        Op::TaskAdd { .. } | Op::TaskLog { .. } => true,
        Op::TaskUpdate { d, .. } => standing.filed_by_agent || crate::state::only_bells(d),
        Op::TaskResolve { .. }
        | Op::TaskDescribe { .. }
        | Op::StepAdd { .. }
        | Op::StepDone { .. } => attended,
        _ => false,
    }
}

pub fn story(events: &[Event], id: TaskId) -> Story {
    // The same rule `State::apply` keeps: a device says what kind it is itself, and a
    // conversion an assistant wrote never landed, so it is not a chapter either.
    let assistants: std::collections::BTreeSet<&DeviceId> = events
        .iter()
        .filter_map(|event| match &event.op {
            Op::DeviceJoin {
                d,
                k: Some(crate::event::DeviceKind::Agent),
            } if d == &event.device => Some(d),
            _ => None,
        })
        .collect();
    let mut mine: Vec<&Event> = events
        .iter()
        .filter(|event| event.entity_id() == Some(id))
        .collect();
    mine.sort_by_key(|event| event.sort_key());

    let mut standing = Standing::default();
    let mut pages = Vec::new();

    for event in mine {
        // The trail tells what landed: what `State::apply` lets go of an assistant's hand is
        // no chapter, or the trail would tell of steps and marks the task never took.
        let by_assistant = assistants.contains(&event.device);
        if by_assistant && !lets(&event.op, &standing) {
            continue;
        }
        let mut write = |chapter: Chapter| {
            pages.push(Page {
                n: pages.len(),
                at: event.timestamp,
                by: event.device.clone(),
                undoing: event.undo,
                chapter,
            });
        };

        match &event.op {
            Op::TaskAdd { d, .. } => {
                standing.filed_by_agent = by_assistant;
                standing.title = d.title.clone();
                standing.date = d.date.clone();
                standing.deadline = d.deadline.clone();
                standing.priority = d.priority.unwrap_or_default();
                standing.list = d.list;
                standing.tags = d.tags.clone();
                standing.repeat = d.repeat;
                write(Chapter::Born {
                    title: d.title.clone(),
                });
            }

            Op::TaskUpdate { d, .. } => {
                if let Some(title) = &d.title
                    && *title != standing.title
                {
                    write(Chapter::Retitled {
                        from: standing.title.clone(),
                        to: title.clone(),
                    });
                    standing.title = title.clone();
                }
                if let Some(date) = &d.date
                    && *date != standing.date
                {
                    write(Chapter::Dated {
                        from: standing.date.clone(),
                        to: date.clone(),
                    });
                    standing.date = date.clone();
                }
                if let Some(deadline) = &d.deadline
                    && *deadline != standing.deadline
                {
                    write(Chapter::Bounded {
                        from: standing.deadline.clone(),
                        to: deadline.clone(),
                    });
                    standing.deadline = deadline.clone();
                }
                if let Some(priority) = d.priority
                    && priority != standing.priority
                {
                    write(Chapter::Placed {
                        from: standing.priority,
                        to: priority,
                    });
                    standing.priority = priority;
                }
                if let Some(tags) = &d.tags {
                    let added: Vec<Tag> = tags
                        .iter()
                        .filter(|one| !standing.tags.contains(one))
                        .cloned()
                        .collect();
                    let gone: Vec<Tag> = standing
                        .tags
                        .iter()
                        .filter(|one| !tags.contains(one))
                        .cloned()
                        .collect();
                    if !added.is_empty() || !gone.is_empty() {
                        write(Chapter::Tagged { added, gone });
                        standing.tags = tags.clone();
                    }
                }
                if let Some(repeat) = &d.repeat
                    && *repeat != standing.repeat
                {
                    write(Chapter::Cadenced {
                        from: standing.repeat,
                        to: *repeat,
                    });
                    standing.repeat = *repeat;
                }
                if let Some(read_as) = d.read_as
                    && read_as != standing.read_as
                    && read_as != Some(Reading::Routine)
                    && !assistants.contains(&event.device)
                {
                    write(Chapter::Converted {
                        from: standing.read_as,
                        to: read_as,
                    });
                    standing.read_as = read_as;
                }
                if let Some(open) = d.open_to_agents
                    && open != standing.open_to_agents
                    && !assistants.contains(&event.device)
                {
                    write(if open { Chapter::Opened } else { Chapter::Shut });
                    standing.open_to_agents = open;
                }
            }

            Op::TaskMove { d, .. } => {
                if let Some(list) = d.list
                    && list != standing.list
                {
                    write(Chapter::Filed {
                        from: standing.list,
                        to: list,
                    });
                    standing.list = list;
                }
            }

            Op::TaskDescribe { d, .. } => {
                let empty = d.body.as_ref().is_none_or(|body| body.trim().is_empty());
                if !empty || standing.described {
                    write(Chapter::Described { emptied: empty });
                }
                standing.described = !empty;
            }

            Op::TaskLog { d, .. } => write(Chapter::Wrote {
                body: d.body.clone(),
            }),
            Op::TaskLogEdit { d, .. } => write(Chapter::Rewrote {
                body: d.body.clone(),
            }),

            Op::StepAdd { d, .. } => {
                standing.steps.insert(d.step, d.text.clone());
                write(Chapter::Planned {
                    text: d.text.clone(),
                });
            }
            Op::StepDone { d, .. } => write(Chapter::Ticked {
                text: standing.steps.get(&d.step).cloned().unwrap_or_default(),
            }),
            Op::StepUndone { d, .. } => write(Chapter::Unticked {
                text: standing.steps.get(&d.step).cloned().unwrap_or_default(),
            }),
            Op::StepText { d, .. } => {
                let was = standing.steps.get(&d.step).cloned().unwrap_or_default();
                if was != d.text {
                    write(Chapter::Reworded {
                        from: was,
                        to: d.text.clone(),
                    });
                    standing.steps.insert(d.step, d.text.clone());
                }
            }
            Op::StepRemove { d, .. } => {
                let text = standing.steps.remove(&d.step).unwrap_or_default();
                write(Chapter::Unplanned { text });
            }

            Op::TaskDone { .. } => write(Chapter::Closed),
            Op::TaskDrop { .. } => write(Chapter::Dropped),
            Op::TaskReopen { .. } => write(Chapter::Reopened),

            _ => {}
        }
    }

    Story { id, pages }
}

#[cfg(test)]
#[path = "story_test.rs"]
mod tests;
