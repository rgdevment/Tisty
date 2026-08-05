use std::collections::{BTreeMap, BTreeSet};

use ulid::Ulid;

use crate::{
    event::{Event, LogAdd, LogEdit, Op, StepAdd, TaskAdd, TaskMove, TaskPatch},
    model::{List, ListId, LogEntry, Status, Step, StepId, Tag, Task, TaskId},
};

/// Replaying in canonical order is what yields last-write-wins per field.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    pub tasks: BTreeMap<TaskId, Task>,
    pub lists: BTreeMap<ListId, List>,
    tombstones: BTreeSet<Ulid>,
}

impl State {
    pub fn replay(events: &[Event]) -> Self {
        let mut state = Self::default();
        for event in events {
            state.apply(event);
        }
        state
    }

    pub fn apply(&mut self, event: &Event) {
        if self.tombstones.contains(&event.entity_id()) {
            return;
        }

        match &event.op {
            Op::TaskAdd { id, d } => {
                self.tasks.insert(*id, task_from(*id, d));
            }
            Op::TaskUpdate { id, d } => self.with_task(*id, |t| patch(t, d)),
            Op::TaskDone { id } => self.with_task(*id, |t| {
                t.status = Status::Done;
                t.completed_at = Some(event.timestamp);
            }),
            Op::TaskReopen { id } => self.with_task(*id, |t| {
                t.status = Status::Open;
                t.completed_at = None;
            }),
            Op::TaskDrop { id } => self.with_task(*id, |t| {
                t.status = Status::Dropped;
                t.completed_at = Some(event.timestamp);
            }),
            Op::TaskDelete { id } => {
                self.tasks.remove(id);
                self.tombstones.insert(*id);
            }
            Op::TaskMove { id, d } => self.with_task(*id, |t| move_task(t, d)),

            Op::TaskDescribe { id, d } => self.with_task(*id, |t| t.description = d.body.clone()),
            Op::TaskLog { id, d } => {
                let at = event.timestamp;
                self.with_task(*id, |t| add_log_entry(t, d, at));
            }
            Op::TaskLogEdit { id, d } => self.with_task(*id, |t| edit_log_entry(t, d)),

            Op::StepAdd { id, d } => self.with_task(*id, |t| add_step(t, d)),
            Op::StepDone { id, d } => self.with_step(*id, d.step, |s| s.done = true),
            Op::StepUndone { id, d } => self.with_step(*id, d.step, |s| s.done = false),
            Op::StepText { id, d } => self.with_step(*id, d.step, |s| s.text = d.text.clone()),
            Op::StepReorder { id, d } => {
                self.with_step(*id, d.step, |s| s.order = d.order.clone());
                self.with_task(*id, sort_steps);
            }
            Op::StepRemove { id, d } => self.with_task(*id, |t| t.steps.retain(|s| s.id != d.step)),

            Op::ListAdd { id, d } => {
                let mut list = List::new(*id, d.name.clone(), d.order.clone());
                list.color = d.color.clone();
                self.lists.insert(*id, list);
            }
            Op::ListRename { id, d } => {
                if let Some(list) = self.lists.get_mut(id) {
                    list.name = d.name.clone();
                }
            }
            Op::ListArchive { id } => {
                if let Some(list) = self.lists.get_mut(id) {
                    list.archived = true;
                }
            }
            Op::ListDelete { id } => {
                self.lists.remove(id);
                self.tombstones.insert(*id);
                for task in self.tasks.values_mut() {
                    if task.list == Some(*id) {
                        task.list = None;
                    }
                }
            }
        }
    }

    fn with_task(&mut self, id: TaskId, f: impl FnOnce(&mut Task)) {
        if let Some(task) = self.tasks.get_mut(&id) {
            f(task);
        }
    }

    fn with_step(&mut self, task: TaskId, step: StepId, f: impl FnOnce(&mut Step)) {
        self.with_task(task, |t| {
            if let Some(s) = t.steps.iter_mut().find(|s| s.id == step) {
                f(s);
            }
        });
    }

    pub fn open_tasks(&self) -> impl Iterator<Item = &Task> {
        self.tasks.values().filter(|t| t.is_open())
    }

    pub fn archived_tasks(&self) -> impl Iterator<Item = &Task> {
        self.tasks.values().filter(|t| t.is_archived())
    }

    pub fn inbox(&self) -> impl Iterator<Item = &Task> {
        self.open_tasks().filter(|t| t.list.is_none())
    }

    pub fn tasks_in(&self, list: ListId) -> impl Iterator<Item = &Task> {
        self.open_tasks().filter(move |t| t.list == Some(list))
    }

    pub fn tasks_tagged(&self, tag: &Tag) -> impl Iterator<Item = &Task> {
        self.tasks.values().filter(move |t| t.tags.contains(tag))
    }

    pub fn active_lists(&self) -> impl Iterator<Item = &List> {
        self.lists.values().filter(|l| !l.archived)
    }

    /// Drives sinking a finished list in the sidebar; it never vanishes alone.
    pub fn is_settled(&self, list: ListId) -> bool {
        self.tasks_in(list).next().is_none()
    }

    /// No catalogue to administer: tags exist because a task mentions them.
    pub fn tags(&self) -> BTreeSet<&Tag> {
        self.tasks.values().flat_map(|t| &t.tags).collect()
    }
}

fn task_from(id: TaskId, d: &TaskAdd) -> Task {
    Task {
        priority: d.priority.unwrap_or_default(),
        date: d.date.clone(),
        deadline: d.deadline.clone(),
        list: d.list,
        tags: d.tags.clone(),
        reminders: d.reminders.clone(),
        ..Task::new(id, d.title.clone(), d.order.clone())
    }
}

fn patch(task: &mut Task, d: &TaskPatch) {
    if let Some(v) = &d.title {
        task.title = v.clone();
    }
    if let Some(v) = &d.date {
        task.date = v.clone();
    }
    if let Some(v) = &d.deadline {
        task.deadline = v.clone();
    }
    if let Some(v) = d.priority {
        task.priority = v;
    }
    if let Some(v) = &d.tags {
        task.tags = v.clone();
    }
    if let Some(v) = &d.reminders {
        task.reminders = v.clone();
    }
}

fn move_task(task: &mut Task, d: &TaskMove) {
    if let Some(v) = &d.list {
        task.list = *v;
    }
    if let Some(v) = &d.order {
        task.order = v.clone();
    }
}

fn add_log_entry(task: &mut Task, d: &LogAdd, at: jiff::Timestamp) {
    task.log.push(LogEntry {
        id: d.entry,
        at,
        body: d.body.clone(),
    });
}

/// Editing an entry changes what it says, never when it happened.
fn edit_log_entry(task: &mut Task, d: &LogEdit) {
    if let Some(entry) = task.log.iter_mut().find(|e| e.id == d.entry) {
        entry.body = d.body.clone();
    }
}

fn add_step(task: &mut Task, d: &StepAdd) {
    task.steps.push(Step {
        id: d.step,
        text: d.text.clone(),
        done: false,
        order: d.order.clone(),
    });
    sort_steps(task);
}

fn sort_steps(task: &mut Task) {
    task.steps.sort_by(|a, b| a.order.cmp(&b.order));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        event::{Body, DeviceId, ListAdd, Name, StepRef, StepReorder, StepText},
        model::{DateSpec, Priority},
    };

    fn at(ms: i64) -> jiff::Timestamp {
        jiff::Timestamp::from_millisecond(ms).unwrap()
    }

    fn ev(ms: i64, device: &str, op: Op) -> Event {
        Event::new(DeviceId(device.into()), at(ms), op)
    }

    fn add(id: TaskId, title: &str) -> Op {
        Op::TaskAdd {
            id,
            d: TaskAdd::new(title, "a0"),
        }
    }

    fn a_date() -> DateSpec {
        DateSpec::floating("2026-08-05T10:00:00".parse().unwrap(), "America/Santiago")
    }

    fn update(id: TaskId, d: TaskPatch) -> Op {
        Op::TaskUpdate { id, d }
    }

    #[test]
    fn replays_creation_and_completion() {
        let id = Ulid::generate();
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "ship it")),
            ev(2, "dev_a", Op::TaskDone { id }),
        ]);

        let task = &state.tasks[&id];
        assert_eq!(task.status, Status::Done);
        assert_eq!(task.completed_at, Some(at(2)));
        assert!(task.is_archived());
    }

    #[test]
    fn concurrent_edits_to_different_fields_both_survive() {
        let id = Ulid::generate();
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "ship it")),
            ev(
                2,
                "dev_a",
                update(
                    id,
                    TaskPatch {
                        date: Some(Some(a_date())),
                        ..Default::default()
                    },
                ),
            ),
            ev(
                3,
                "dev_b",
                update(
                    id,
                    TaskPatch {
                        priority: Some(Priority::P1),
                        ..Default::default()
                    },
                ),
            ),
        ]);

        let task = &state.tasks[&id];
        assert_eq!(task.date, Some(a_date()));
        assert_eq!(task.priority, Priority::P1);
    }

    #[test]
    fn an_explicit_null_clears_the_field() {
        let id = Ulid::generate();
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "ship it")),
            ev(
                2,
                "dev_a",
                update(
                    id,
                    TaskPatch {
                        date: Some(Some(a_date())),
                        ..Default::default()
                    },
                ),
            ),
            ev(
                3,
                "dev_a",
                update(
                    id,
                    TaskPatch {
                        date: Some(None),
                        ..Default::default()
                    },
                ),
            ),
        ]);

        assert_eq!(state.tasks[&id].date, None);
    }

    #[test]
    fn a_late_update_cannot_resurrect_a_deleted_task() {
        let id = Ulid::generate();
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "ship it")),
            ev(2, "dev_a", Op::TaskDelete { id }),
            ev(
                3,
                "dev_b",
                update(
                    id,
                    TaskPatch {
                        title: Some("back from the dead".into()),
                        ..Default::default()
                    },
                ),
            ),
        ]);

        assert!(state.tasks.is_empty());
    }

    #[test]
    fn replay_is_deterministic_regardless_of_read_order() {
        let id = Ulid::generate();
        let mut events = vec![
            ev(1, "dev_a", add(id, "ship it")),
            ev(
                2,
                "dev_b",
                update(
                    id,
                    TaskPatch {
                        title: Some("from b".into()),
                        ..Default::default()
                    },
                ),
            ),
            ev(
                2,
                "dev_a",
                update(
                    id,
                    TaskPatch {
                        title: Some("from a".into()),
                        ..Default::default()
                    },
                ),
            ),
        ];

        events.sort_by(|x, y| x.sort_key().cmp(&y.sort_key()));
        let forward = State::replay(&events);

        events.reverse();
        events.sort_by(|x, y| x.sort_key().cmp(&y.sort_key()));

        assert_eq!(State::replay(&events), forward);
        assert_eq!(forward.tasks[&id].title, "from b");
    }

    #[test]
    fn the_log_grows_in_order_and_keeps_its_timestamps() {
        let id = Ulid::generate();
        let (first, second) = (Ulid::generate(), Ulid::generate());
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "work")),
            ev(
                10,
                "dev_a",
                Op::TaskLog {
                    id,
                    d: LogAdd {
                        entry: first,
                        body: "first attempt failed".into(),
                    },
                },
            ),
            ev(
                20,
                "dev_b",
                Op::TaskLog {
                    id,
                    d: LogAdd {
                        entry: second,
                        body: "an index was missing".into(),
                    },
                },
            ),
        ]);

        let log = &state.tasks[&id].log;
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].at, at(10));
        assert_eq!(log[1].at, at(20));
        assert!(log[0].at < log[1].at);
    }

    #[test]
    fn editing_an_entry_changes_the_text_not_the_time() {
        let id = Ulid::generate();
        let entry = Ulid::generate();
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "work")),
            ev(
                10,
                "dev_a",
                Op::TaskLog {
                    id,
                    d: LogAdd {
                        entry,
                        body: "typo here".into(),
                    },
                },
            ),
            ev(
                99,
                "dev_a",
                Op::TaskLogEdit {
                    id,
                    d: LogEdit {
                        entry,
                        body: "fixed".into(),
                    },
                },
            ),
        ]);

        let e = state.tasks[&id].entry(entry).unwrap();
        assert_eq!(e.body, "fixed");
        assert_eq!(e.at, at(10), "the entry keeps when it happened");
    }

    #[test]
    fn steps_are_kept_in_their_own_order() {
        let id = Ulid::generate();
        let (a, b, c) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
        let mut events = vec![ev(1, "dev_a", add(id, "deploy"))];
        for (step, text, order) in [(a, "third", "a3"), (b, "first", "a1"), (c, "second", "a2")] {
            events.push(ev(
                2,
                "dev_a",
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step,
                        text: text.into(),
                        order: order.into(),
                    },
                },
            ));
        }

        let state = State::replay(&events);
        let texts: Vec<_> = state.tasks[&id].steps.iter().map(|s| &s.text).collect();
        assert_eq!(texts, ["first", "second", "third"]);
    }

    #[test]
    fn reordering_a_step_moves_it() {
        let id = Ulid::generate();
        let (a, b) = (Ulid::generate(), Ulid::generate());
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "deploy")),
            ev(
                2,
                "dev_a",
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step: a,
                        text: "first step".into(),
                        order: "a1".into(),
                    },
                },
            ),
            ev(
                3,
                "dev_a",
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step: b,
                        text: "validate".into(),
                        order: "a2".into(),
                    },
                },
            ),
            ev(
                4,
                "dev_a",
                Op::StepReorder {
                    id,
                    d: StepReorder {
                        step: b,
                        order: "a0".into(),
                    },
                },
            ),
        ]);

        assert_eq!(state.tasks[&id].steps[0].text, "validate");
    }

    #[test]
    fn steps_track_progress() {
        let id = Ulid::generate();
        let (a, b) = (Ulid::generate(), Ulid::generate());
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "deploy")),
            ev(
                2,
                "dev_a",
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step: a,
                        text: "first step".into(),
                        order: "a1".into(),
                    },
                },
            ),
            ev(
                3,
                "dev_a",
                Op::StepAdd {
                    id,
                    d: StepAdd {
                        step: b,
                        text: "validate".into(),
                        order: "a2".into(),
                    },
                },
            ),
            ev(
                4,
                "dev_a",
                Op::StepDone {
                    id,
                    d: StepRef { step: a },
                },
            ),
        ]);

        assert_eq!(state.tasks[&id].steps_done(), (1, 2));
    }

    #[test]
    fn an_op_for_an_unknown_step_is_ignored() {
        let id = Ulid::generate();
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "deploy")),
            ev(
                2,
                "dev_a",
                Op::StepText {
                    id,
                    d: StepText {
                        step: Ulid::generate(),
                        text: "ghost".into(),
                    },
                },
            ),
        ]);

        assert!(state.tasks[&id].steps.is_empty());
    }

    #[test]
    fn describing_replaces_and_clearing_removes() {
        let id = Ulid::generate();
        let described = State::replay(&[
            ev(1, "dev_a", add(id, "work")),
            ev(
                2,
                "dev_a",
                Op::TaskDescribe {
                    id,
                    d: Body {
                        body: Some("query string is lost".into()),
                    },
                },
            ),
        ]);
        assert_eq!(
            described.tasks[&id].description.as_deref(),
            Some("query string is lost")
        );

        let cleared = State::replay(&[
            ev(1, "dev_a", add(id, "work")),
            ev(
                2,
                "dev_a",
                Op::TaskDescribe {
                    id,
                    d: Body {
                        body: Some("draft".into()),
                    },
                },
            ),
            ev(
                3,
                "dev_a",
                Op::TaskDescribe {
                    id,
                    d: Body { body: None },
                },
            ),
        ]);
        assert_eq!(cleared.tasks[&id].description, None);
    }

    #[test]
    fn moving_to_the_inbox_is_not_the_same_as_only_reordering() {
        let id = Ulid::generate();
        let list = Ulid::generate();
        let base = vec![
            ev(
                1,
                "dev_a",
                Op::ListAdd {
                    id: list,
                    d: ListAdd {
                        name: "checkout rewrite".into(),
                        order: "a0".into(),
                        color: None,
                    },
                },
            ),
            ev(
                2,
                "dev_a",
                Op::TaskAdd {
                    id,
                    d: TaskAdd {
                        list: Some(list),
                        ..TaskAdd::new("ship it", "a0")
                    },
                },
            ),
        ];

        let mut reordered = base.clone();
        reordered.push(ev(
            3,
            "dev_a",
            Op::TaskMove {
                id,
                d: TaskMove {
                    order: Some("a5".into()),
                    ..Default::default()
                },
            },
        ));
        assert_eq!(State::replay(&reordered).tasks[&id].list, Some(list));

        let mut to_inbox = base;
        to_inbox.push(ev(
            3,
            "dev_a",
            Op::TaskMove {
                id,
                d: TaskMove {
                    list: Some(None),
                    ..Default::default()
                },
            },
        ));
        assert_eq!(State::replay(&to_inbox).tasks[&id].list, None);
    }

    #[test]
    fn deleting_a_list_returns_its_tasks_to_the_inbox() {
        let id = Ulid::generate();
        let list = Ulid::generate();
        let state = State::replay(&[
            ev(
                1,
                "dev_a",
                Op::ListAdd {
                    id: list,
                    d: ListAdd {
                        name: "temporal".into(),
                        order: "a0".into(),
                        color: None,
                    },
                },
            ),
            ev(
                2,
                "dev_a",
                Op::TaskAdd {
                    id,
                    d: TaskAdd {
                        list: Some(list),
                        ..TaskAdd::new("ship it", "a0")
                    },
                },
            ),
            ev(3, "dev_a", Op::ListDelete { id: list }),
        ]);

        assert!(state.lists.is_empty());
        assert_eq!(state.tasks[&id].list, None);
        assert_eq!(state.inbox().count(), 1);
    }

    #[test]
    fn a_list_with_everything_done_is_settled() {
        let id = Ulid::generate();
        let list = Ulid::generate();
        let events = vec![
            ev(
                1,
                "dev_a",
                Op::ListAdd {
                    id: list,
                    d: ListAdd {
                        name: "rediseño onboarding".into(),
                        order: "a0".into(),
                        color: None,
                    },
                },
            ),
            ev(
                2,
                "dev_a",
                Op::TaskAdd {
                    id,
                    d: TaskAdd {
                        list: Some(list),
                        ..TaskAdd::new("ship it", "a0")
                    },
                },
            ),
        ];

        assert!(!State::replay(&events).is_settled(list));

        let mut done = events;
        done.push(ev(3, "dev_a", Op::TaskDone { id }));
        assert!(State::replay(&done).is_settled(list));
    }

    #[test]
    fn renaming_a_list_keeps_its_tasks() {
        let id = Ulid::generate();
        let list = Ulid::generate();
        let state = State::replay(&[
            ev(
                1,
                "dev_a",
                Op::ListAdd {
                    id: list,
                    d: ListAdd {
                        name: "antiguo".into(),
                        order: "a0".into(),
                        color: None,
                    },
                },
            ),
            ev(
                2,
                "dev_a",
                Op::TaskAdd {
                    id,
                    d: TaskAdd {
                        list: Some(list),
                        ..TaskAdd::new("ship it", "a0")
                    },
                },
            ),
            ev(
                3,
                "dev_a",
                Op::ListRename {
                    id: list,
                    d: Name {
                        name: "checkout rewrite".into(),
                    },
                },
            ),
        ]);

        assert_eq!(state.lists[&list].name, "checkout rewrite");
        assert_eq!(state.tasks_in(list).count(), 1);
    }

    #[test]
    fn tags_are_collected_from_use_not_from_a_catalogue() {
        let (a, b) = (Ulid::generate(), Ulid::generate());
        let state = State::replay(&[
            ev(
                1,
                "dev_a",
                Op::TaskAdd {
                    id: a,
                    d: TaskAdd {
                        tags: vec![Tag::new("work").unwrap(), Tag::new("urgent").unwrap()],
                        ..TaskAdd::new("one", "a0")
                    },
                },
            ),
            ev(
                2,
                "dev_a",
                Op::TaskAdd {
                    id: b,
                    d: TaskAdd {
                        tags: vec![Tag::new("work").unwrap()],
                        ..TaskAdd::new("two", "a1")
                    },
                },
            ),
        ]);

        assert_eq!(state.tags().len(), 2);
        assert_eq!(state.tasks_tagged(&Tag::new("work").unwrap()).count(), 2);
    }

    #[test]
    fn archived_tasks_stay_out_of_the_open_views_but_remain() {
        let id = Ulid::generate();
        let state = State::replay(&[
            ev(1, "dev_a", add(id, "ship it")),
            ev(2, "dev_a", Op::TaskDone { id }),
        ]);

        assert_eq!(state.open_tasks().count(), 0);
        assert_eq!(state.archived_tasks().count(), 1);
        assert_eq!(state.tasks.len(), 1, "archived is not deleted");
    }

    #[test]
    fn dropping_is_not_completing() {
        let (a, b) = (Ulid::generate(), Ulid::generate());
        let state = State::replay(&[
            ev(1, "dev_a", add(a, "done properly")),
            ev(1, "dev_b", add(b, "never happened")),
            ev(2, "dev_a", Op::TaskDone { id: a }),
            ev(2, "dev_b", Op::TaskDrop { id: b }),
        ]);

        assert_eq!(state.tasks[&a].status, Status::Done);
        assert_eq!(state.tasks[&b].status, Status::Dropped);
    }
}
