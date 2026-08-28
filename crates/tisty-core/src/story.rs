use std::collections::HashMap;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::event::{DeviceId, Event, Op};
use crate::model::{DateSpec, ListId, Priority, Repeat, StepId, Tag, TaskId};

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
    described: bool,
    steps: HashMap<StepId, String>,
}

pub fn story(events: &[Event], id: TaskId) -> Story {
    let mut mine: Vec<&Event> = events
        .iter()
        .filter(|event| event.entity_id() == Some(id))
        .collect();
    mine.sort_by_key(|event| event.sort_key());

    let mut standing = Standing::default();
    let mut pages = Vec::new();

    for event in mine {
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
mod tests {
    use super::*;
    use crate::event::{Body, LogAdd, StepAdd, StepRef, TaskAdd, TaskMove, TaskPatch};
    use ulid::Ulid;

    fn at(seconds: i64) -> Timestamp {
        Timestamp::from_second(1_770_000_000 + seconds).unwrap()
    }

    fn here() -> DeviceId {
        DeviceId("dev_here".into())
    }

    fn event(seconds: i64, op: Op) -> Event {
        Event::new(here(), at(seconds), op)
    }

    fn day(text: &str) -> DateSpec {
        DateSpec::all_day(text.parse().unwrap(), "UTC")
    }

    fn born(id: TaskId, title: &str) -> Event {
        event(
            0,
            Op::TaskAdd {
                id,
                d: TaskAdd::new(title, "a0"),
            },
        )
    }

    fn patched(seconds: i64, id: TaskId, d: TaskPatch) -> Event {
        event(seconds, Op::TaskUpdate { id, d })
    }

    fn chapters(story: &Story) -> Vec<&Chapter> {
        story.pages.iter().map(|page| &page.chapter).collect()
    }

    #[test]
    fn a_deadline_that_moves_twice_keeps_both_moves() {
        let id = Ulid::generate();
        let log = vec![
            born(id, "ship the release"),
            patched(
                10,
                id,
                TaskPatch {
                    deadline: Some(Some(day("2026-08-12"))),
                    ..Default::default()
                },
            ),
            patched(
                20,
                id,
                TaskPatch {
                    deadline: Some(Some(day("2026-08-19"))),
                    ..Default::default()
                },
            ),
        ];

        let told = story(&log, id);

        assert!(matches!(
            chapters(&told)[2],
            Chapter::Bounded {
                from: Some(_),
                to: Some(_)
            }
        ));
        let Chapter::Bounded { from, to } = chapters(&told)[2] else {
            unreachable!()
        };
        assert_eq!(from.as_ref().unwrap().date(), "2026-08-12".parse().unwrap());
        assert_eq!(to.as_ref().unwrap().date(), "2026-08-19".parse().unwrap());
    }

    #[test]
    fn what_the_task_carries_now_never_overwrites_what_it_carried_then() {
        let id = Ulid::generate();
        let log = vec![
            born(id, "call the plumber"),
            patched(
                10,
                id,
                TaskPatch {
                    title: Some("call the plumber back".into()),
                    ..Default::default()
                },
            ),
        ];

        let told = story(&log, id);

        assert!(
            matches!(chapters(&told)[0], Chapter::Born { title } if title == "call the plumber")
        );
        assert!(matches!(chapters(&told)[1], Chapter::Retitled { from, to }
                if from == "call the plumber" && to == "call the plumber back"));
    }

    #[test]
    fn reordering_is_not_a_chapter() {
        let id = Ulid::generate();
        let log = vec![
            born(id, "buy bread"),
            event(
                10,
                Op::TaskMove {
                    id,
                    d: TaskMove {
                        list: None,
                        order: Some("a5".into()),
                    },
                },
            ),
        ];

        assert_eq!(story(&log, id).pages.len(), 1);
    }

    #[test]
    fn a_step_is_told_with_the_words_it_had_when_it_was_ticked() {
        let id = Ulid::generate();
        let step = Ulid::generate();
        let log = vec![
            born(id, "ship the release"),
            event(
                10,
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step,
                        text: "sign the installer".into(),
                        order: "a0".into(),
                    },
                },
            ),
            event(
                20,
                Op::StepDone {
                    id,
                    d: StepRef { step },
                },
            ),
        ];

        let told = story(&log, id);

        assert!(matches!(
            chapters(&told)[2],
            Chapter::Ticked { text } if text == "sign the installer"
        ));
    }

    #[test]
    fn another_task_never_leaks_into_this_one() {
        let mine = Ulid::generate();
        let theirs = Ulid::generate();
        let log = vec![
            born(mine, "mine"),
            born(theirs, "theirs"),
            event(
                10,
                Op::TaskLog {
                    id: theirs,
                    d: LogAdd::new(Ulid::generate(), "not for this story"),
                },
            ),
        ];

        assert_eq!(story(&log, mine).pages.len(), 1);
    }

    #[test]
    fn undoing_is_a_move_of_its_own_and_says_so() {
        let id = Ulid::generate();
        let mut undone = event(20, Op::TaskReopen { id });
        undone.undo = true;

        let log = vec![
            born(id, "water the plants"),
            event(10, Op::TaskDone { id, filled: false }),
            undone,
        ];

        let told = story(&log, id);

        assert_eq!(told.pages.len(), 3, "nothing is hidden from a record");
        assert!(!told.pages[1].undoing);
        assert!(told.pages[2].undoing);
    }

    #[test]
    fn a_story_reads_in_the_same_order_from_either_machine() {
        let id = Ulid::generate();
        let theirs = DeviceId("dev_other".into());
        let mut late = Event::new(theirs, at(10), Op::TaskDone { id, filled: false });
        late.seq = 1;

        let log = vec![late.clone(), born(id, "ship the release")];
        let backwards = vec![born(id, "ship the release"), late];

        assert_eq!(story(&log, id), story(&backwards, id));
        assert!(matches!(
            story(&log, id).pages[0].chapter,
            Chapter::Born { .. }
        ));
    }

    #[test]
    fn every_kind_of_move_becomes_the_chapter_that_names_it() {
        let id = Ulid::generate();
        let step = Ulid::generate();
        let list = Ulid::generate();
        let log = vec![
            born(id, "ship the release"),
            patched(
                10,
                id,
                TaskPatch {
                    priority: Some(crate::model::Priority::Do),
                    ..Default::default()
                },
            ),
            event(
                20,
                Op::TaskMove {
                    id,
                    d: TaskMove {
                        list: Some(Some(list)),
                        order: None,
                    },
                },
            ),
            patched(
                30,
                id,
                TaskPatch {
                    tags: Some(vec![crate::model::Tag::new("release").unwrap()]),
                    ..Default::default()
                },
            ),
            patched(
                40,
                id,
                TaskPatch {
                    repeat: Some(Some(crate::model::Repeat::due(crate::model::Cadence {
                        every: 1,
                        unit: crate::model::Unit::Week,
                    }))),
                    ..Default::default()
                },
            ),
            event(
                50,
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step,
                        text: "sign it".into(),
                        order: "a0".into(),
                    },
                },
            ),
            event(
                60,
                Op::StepDone {
                    id,
                    d: StepRef { step },
                },
            ),
            event(
                70,
                Op::StepUndone {
                    id,
                    d: StepRef { step },
                },
            ),
            event(
                80,
                Op::StepText {
                    id,
                    d: crate::event::StepText {
                        step,
                        text: "sign the installer".into(),
                    },
                },
            ),
            event(
                90,
                Op::StepRemove {
                    id,
                    d: StepRef { step },
                },
            ),
            event(
                100,
                Op::TaskLogEdit {
                    id,
                    d: crate::event::LogEdit {
                        entry: Ulid::generate(),
                        body: "rewritten".into(),
                    },
                },
            ),
            event(110, Op::TaskDrop { id }),
            event(120, Op::TaskReopen { id }),
        ];

        let told = story(&log, id);
        let kinds: Vec<&Chapter> = chapters(&told);

        for wanted in [
            "Placed",
            "Filed",
            "Tagged",
            "Cadenced",
            "Planned",
            "Ticked",
            "Unticked",
            "Reworded",
            "Unplanned",
            "Rewrote",
            "Dropped",
            "Reopened",
        ] {
            assert!(
                kinds
                    .iter()
                    .any(|one| format!("{one:?}").starts_with(wanted)),
                "{wanted} never made it into the trail: {kinds:?}"
            );
        }
    }

    #[test]
    fn a_step_ticked_without_ever_being_added_still_reads_as_a_chapter() {
        let id = Ulid::generate();
        let stray = Ulid::generate();
        let log = vec![
            born(id, "ship the release"),
            event(
                10,
                Op::StepDone {
                    id,
                    d: StepRef { step: stray },
                },
            ),
        ];

        let told = story(&log, id);

        assert!(
            matches!(chapters(&told)[1], Chapter::Ticked { text } if text.is_empty()),
            "a partial log still has to read, even if it reads thin"
        );
    }

    #[test]
    fn an_emptied_description_is_a_chapter_but_an_empty_one_was_never_written() {
        let id = Ulid::generate();
        let log = vec![
            born(id, "renew the certificate"),
            event(
                10,
                Op::TaskDescribe {
                    id,
                    d: Body {
                        body: Some("   ".into()),
                    },
                },
            ),
        ];

        assert_eq!(story(&log, id).pages.len(), 1);
    }
}
