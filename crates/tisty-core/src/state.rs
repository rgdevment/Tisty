use std::collections::{BTreeMap, BTreeSet};

use ulid::Ulid;

use crate::{
    event::{Event, LogAdd, LogEdit, Op, StepAdd, TaskAdd, TaskMove, TaskPatch},
    model::{List, ListId, LogEntry, Status, Step, StepId, Tag, Task, TaskId},
    order,
};

/// Whether descriptions, journals and steps came along.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    #[default]
    Whole,
    Summary,
}

/// Replaying in canonical order is what yields last-write-wins per field.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    pub tasks: BTreeMap<TaskId, Task>,
    pub lists: BTreeMap<ListId, List>,
    pub(crate) fill: Fill,
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
                let mut task = task_from(*id, d);
                task.retally();
                self.tasks.insert(*id, task);
            }
            Op::TaskUpdate { id, d } => self.with_task(*id, |t| patch(t, d)),
            Op::TaskDone { id } => self.with_task(*id, |t| {
                t.status = Status::Done;
                t.completed_at = Some(event.timestamp);
            }),
            Op::TaskReopen { id } => self.with_task(*id, |t| {
                t.status = Status::Open;
                t.completed_at = None;
                // Open and hidden is a state no view reaches: it would vanish
                // from the interface, from `tisty ls` and from `tisty export`.
                t.hidden = false;
            }),
            Op::TaskHide { id } => self.with_task(*id, |t| t.hidden = true),
            Op::TaskShow { id } => self.with_task(*id, |t| t.hidden = false),
            Op::TaskDrop { id } => self.with_task(*id, |t| {
                t.status = Status::Dropped;
                t.completed_at = Some(event.timestamp);
                // What you decided not to do is not what you did: it folds away
                // on its own, and can be brought back by hand like any other.
                t.hidden = true;
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
            Op::ListUnarchive { id } => {
                if let Some(list) = self.lists.get_mut(id) {
                    list.archived = false;
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

    /// False when descriptions, journals and steps were left behind; such a state must never be cached.
    pub fn has_bodies(&self) -> bool {
        self.fill == Fill::Whole
    }

    fn with_task(&mut self, id: TaskId, f: impl FnOnce(&mut Task)) {
        // Retallying without the bodies counts empty vectors and zeroes the volume.
        let whole = self.has_bodies();
        if let Some(task) = self.tasks.get_mut(&id) {
            f(task);
            if whole {
                task.retally();
            }
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

    /// Permanent — a late event cannot resurrect what this marks.
    pub fn is_erased(&self, id: Ulid) -> bool {
        self.tombstones.contains(&id)
    }

    pub fn erased(&self) -> impl Iterator<Item = &Ulid> {
        self.tombstones.iter()
    }

    /// For restoring a projection stored elsewhere; omitting tombstones lets deletions resurrect.
    pub fn mark_erased(&mut self, id: Ulid) {
        self.tombstones.insert(id);
    }

    /// Last place in a list, for a task that arrived without neighbours —
    /// dropped on the sidebar, where there was nothing to land between.
    pub fn order_last_in(&self, list: Option<ListId>) -> String {
        let keys: Vec<&str> = self
            .open_tasks()
            .filter(|task| task.list == list)
            .map(|task| task.order.as_str())
            .collect();
        order::last_of(keys)
    }

    /// The client says which two tasks it was dropped between; the key is the
    /// core's to compute, as it is the only place that knows what an order is.
    pub fn order_between(&self, after: Option<TaskId>, before: Option<TaskId>) -> String {
        let key = |id: Option<TaskId>| {
            id.and_then(|id| self.tasks.get(&id))
                .map(|task| task.order.clone())
        };
        let (a, b) = (key(after), key(before));
        // A stale neighbour would put the task somewhere nobody pointed at.
        match (a, b) {
            (Some(a), Some(b)) if a >= b => order::after(&a),
            (a, b) => order::between(a.as_deref(), b.as_deref()),
        }
    }

    /// Backlinks: the index is derived on read, so nothing can fall out of step
    /// with the prose it came from. Matches the label as well as the address,
    /// because a ticket is a link and what people ask for is its code.
    pub fn linking_to(&self, target: &str) -> Vec<&Task> {
        let wanted = target.trim().to_lowercase();
        self.tasks
            .values()
            .filter(|task| {
                task.volume.refs > 0
                    && task.references().iter().any(|one| {
                        one.target.to_lowercase() == wanted
                            || one
                                .label
                                .as_deref()
                                .is_some_and(|l| l.to_lowercase() == wanted)
                    })
            })
            .collect()
    }

    /// Every internal reference in use, for offering back what already exists
    /// instead of a second spelling of the same thing.
    pub fn references(&self) -> Vec<String> {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for task in self.tasks.values().filter(|t| t.volume.refs > 0) {
            for one in task.references() {
                if one.kind == crate::refs::Kind::Doc {
                    seen.insert(one.target);
                }
            }
        }
        seen.into_iter().collect()
    }

    pub fn tasks_tagged(&self, tag: &Tag) -> impl Iterator<Item = &Task> {
        self.tasks.values().filter(move |t| t.tags.contains(tag))
    }

    pub fn active_lists(&self) -> impl Iterator<Item = &List> {
        self.lists.values().filter(|l| !l.archived)
    }

    pub fn ordered_open(&self) -> Vec<&Task> {
        let mut tasks: Vec<_> = self.open_tasks().collect();
        tasks.sort_by(|a, b| {
            let key = |t: &Task| {
                (
                    t.date.as_ref().map(|d| d.at),
                    t.priority,
                    t.order.clone(),
                    t.id,
                )
            };
            match (&a.date, &b.date) {
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                _ => key(a).cmp(&key(b)),
            }
        });
        tasks
    }

    pub fn matching(&self, filter: &crate::view::Filter, today: jiff::civil::Date) -> Vec<&Task> {
        use crate::view::Scope;

        let open = || {
            self.ordered_open()
                .into_iter()
                .filter(|t| filter.matches(t, today))
                .collect::<Vec<_>>()
        };
        let archived = || {
            let mut done: Vec<&Task> = self
                .archived_tasks()
                .filter(|t| filter.matches(t, today))
                .collect();
            done.sort_by_key(|t| (std::cmp::Reverse(t.completed_at), std::cmp::Reverse(t.id)));
            done
        };

        match filter.scope {
            Scope::Open => open(),
            Scope::Archived => archived(),
            Scope::Either => {
                let mut all = open();
                all.extend(archived());
                all
            }
        }
    }

    /// Sorted open-first, then archived newest-first.
    pub fn search(&self, query: &str, scope: crate::view::Scope) -> Vec<&Task> {
        use crate::view::Scope;

        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Vec::new();
        }

        let mut hits: Vec<(crate::view::Hit, &Task)> = self
            .tasks
            .values()
            .filter(|t| match scope {
                Scope::Open => t.is_open(),
                Scope::Archived => t.is_archived(),
                Scope::Either => true,
            })
            .filter_map(|t| crate::view::matches_query(t, &query).map(|hit| (hit, t)))
            .collect();

        // What is still open first, or what remains to do ends up buried under
        // what is already done. Then how it matched, then how much it carries.
        // Folded last, not gone: hiding is not deleting, so what was put away
        // stays findable — just never above what was not.
        hits.sort_by_key(|(hit, t)| {
            (
                t.hidden,
                t.is_archived(),
                *hit,
                std::cmp::Reverse(t.weight()),
                std::cmp::Reverse(t.completed_at),
                std::cmp::Reverse(t.id),
            )
        });
        hits.into_iter().map(|(_, t)| t).collect()
    }

    pub fn ordered_lists(&self) -> Vec<&List> {
        let mut lists: Vec<_> = self.active_lists().collect();
        lists.sort_by(|a, b| (&a.order, a.id).cmp(&(&b.order, b.id)));
        lists
    }

    /// Exact name wins over substring matches; accepts a leading `@` like the rest of the interface.
    pub fn find_list(&self, needle: &str) -> Vec<&List> {
        let needle = loose(needle.trim_start_matches('@'));
        let exact: Vec<&List> = self
            .lists
            .values()
            .filter(|l| loose(&l.name) == needle)
            .collect();
        if !exact.is_empty() {
            return exact;
        }
        self.lists
            .values()
            .filter(|l| loose(&l.name).contains(&needle))
            .collect()
    }

    pub fn next_task_order(&self) -> String {
        order::last_of(self.tasks.values().map(|t| t.order.as_str()))
    }

    pub fn next_list_order(&self) -> String {
        order::last_of(self.lists.values().map(|l| l.order.as_str()))
    }

    pub fn is_settled(&self, list: ListId) -> bool {
        self.tasks_in(list).next().is_none()
    }

    /// Derived from tasks in use; there is no separate tag catalogue.
    pub fn tags(&self) -> BTreeSet<&Tag> {
        self.tasks.values().flat_map(|t| &t.tags).collect()
    }
}

/// Listings print «Mi Lista» as `@mi-lista`, which has to be typeable back.
fn loose(name: &str) -> String {
    name.to_lowercase().replace([' ', '_'], "-")
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
        tz: d.tz.clone(),
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

    fn day() -> jiff::civil::Date {
        "2026-08-09".parse().unwrap()
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
                    d: LogAdd::new(first, "first attempt failed"),
                },
            ),
            ev(
                20,
                "dev_b",
                Op::TaskLog {
                    id,
                    d: LogAdd::new(second, "an index was missing"),
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
                    d: LogAdd::new(entry, "typo here"),
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

    /// §3.1: «la tarea con historia sale antes que las quince triviales».
    #[test]
    fn search_puts_the_documented_before_the_trivial() {
        let mut state = State::default();
        let light = Ulid::generate();
        let heavy = Ulid::generate();
        state.apply(&ev(1, "a", add(light, "comprar pan brasil")));
        state.apply(&ev(2, "a", add(heavy, "revisar el deploy brasil")));
        state.apply(&ev(
            3,
            "a",
            Op::TaskLog {
                id: heavy,
                d: crate::event::LogAdd::new(
                    Ulid::generate(),
                    "el gateway no propagaba la cabecera",
                ),
            },
        ));

        let found = state.search("brasil", crate::view::Scope::Either);
        assert_eq!(found[0].id, heavy);
    }

    /// Naming it beats mentioning it, whatever it weighs.
    #[test]
    fn a_title_match_outranks_a_heavier_body_match() {
        let mut state = State::default();
        let named = Ulid::generate();
        let mentioned = Ulid::generate();
        state.apply(&ev(1, "a", add(named, "redirects en brasil")));
        state.apply(&ev(2, "a", add(mentioned, "migrar el proxy")));
        for n in 0..6 {
            state.apply(&ev(
                10 + n,
                "a",
                Op::TaskLog {
                    id: mentioned,
                    d: crate::event::LogAdd::new(
                        Ulid::generate(),
                        "el despliegue de brasil volvió a fallar por el proxy",
                    ),
                },
            ));
        }

        let found = state.search("brasil", crate::view::Scope::Either);
        assert_eq!(found[0].id, named);
        assert!(state.tasks[&mentioned].weight() > state.tasks[&named].weight());
    }

    fn wrote(id: Ulid, body: &str) -> Op {
        Op::TaskDescribe {
            id,
            d: Body {
                body: Some(body.into()),
            },
        }
    }

    /// Searching a ticket finds the task that points at it, above the one that
    /// merely says the same letters.
    #[test]
    fn a_reference_named_whole_outranks_the_same_letters_in_prose() {
        let mut state = State::default();
        let points = Ulid::generate();
        let mentions = Ulid::generate();
        state.apply(&ev(1, "a", add(points, "migrar el proxy")));
        state.apply(&ev(2, "a", wrote(points, "sale de [[CUSLEG-3465]]")));
        state.apply(&ev(3, "a", add(mentions, "revisar el deploy")));
        state.apply(&ev(
            4,
            "a",
            wrote(mentions, "parecido a cusleg-3465 pero no es el mismo"),
        ));

        let found = state.search("cusleg-3465", crate::view::Scope::Either);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].id, points);
    }

    /// A ticket is written as a link, so its code is the label and the address
    /// is a URL nobody types into a search box.
    #[test]
    fn a_ticket_written_as_a_link_is_found_by_its_code() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(1, "a", add(id, "migrar el proxy")));
        state.apply(&ev(
            2,
            "a",
            wrote(
                id,
                "sale de [OPS-3465](https://jira.example/browse/OPS-3465)",
            ),
        ));

        let other = Ulid::generate();
        state.apply(&ev(3, "a", add(other, "revisar el deploy")));
        state.apply(&ev(
            4,
            "a",
            wrote(other, "algo parecido a ops-3465, sin ser eso"),
        ));

        let found = state.search("ops-3465", crate::view::Scope::Either);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].id, id, "the link lost to a passing mention");
        assert_eq!(state.linking_to("OPS-3465").len(), 1);
        assert_eq!(
            state
                .linking_to("https://jira.example/browse/OPS-3465")
                .len(),
            1,
            "the address still points at it"
        );
    }

    #[test]
    fn backlinks_come_from_the_prose_and_follow_it_when_it_changes() {
        let mut state = State::default();
        let one = Ulid::generate();
        let other = Ulid::generate();
        state.apply(&ev(1, "a", add(one, "migrar el proxy")));
        state.apply(&ev(2, "a", wrote(one, "sale de [[CUSLEG-3465]]")));
        state.apply(&ev(3, "a", add(other, "revisar el deploy")));
        state.apply(&ev(
            4,
            "a",
            Op::TaskLog {
                id: other,
                d: crate::event::LogAdd::new(Ulid::generate(), "también [[cusleg-3465]]"),
            },
        ));

        assert_eq!(
            state.linking_to("CUSLEG-3465").len(),
            2,
            "case decides nothing"
        );
        assert!(state.linking_to("CUSLEG-9999").is_empty());

        state.apply(&ev(5, "a", wrote(one, "ya no sale de ninguna parte")));
        assert_eq!(state.linking_to("CUSLEG-3465").len(), 1);
    }

    #[test]
    fn dropping_between_two_tasks_lands_between_them() {
        let mut state = State::default();
        let (one, two, moved) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
        state.apply(&ev(1, "a", add(one, "first")));
        state.apply(&ev(2, "a", add(two, "second")));
        state.apply(&ev(3, "a", add(moved, "dropped in the middle")));
        state.tasks.get_mut(&one).unwrap().order = "a0".into();
        state.tasks.get_mut(&two).unwrap().order = "a1".into();

        let key = state.order_between(Some(one), Some(two));
        assert!(key.as_str() > "a0" && key.as_str() < "a1", "{key}");
    }

    #[test]
    fn dropping_at_either_end_needs_only_one_neighbour() {
        let mut state = State::default();
        let only = Ulid::generate();
        state.apply(&ev(1, "a", add(only, "the only one")));
        state.tasks.get_mut(&only).unwrap().order = "a5".into();

        assert!(state.order_between(None, Some(only)).as_str() < "a5");
        assert!(state.order_between(Some(only), None).as_str() > "a5");
        assert!(!state.order_between(None, None).is_empty());
    }

    /// A neighbour deleted on another machine while the drag was in flight.
    #[test]
    fn a_neighbour_that_is_gone_does_not_drop_the_task_somewhere_else() {
        let mut state = State::default();
        let here = Ulid::generate();
        state.apply(&ev(1, "a", add(here, "still here")));
        state.tasks.get_mut(&here).unwrap().order = "a5".into();

        let vanished = Ulid::generate();
        assert!(state.order_between(Some(here), Some(vanished)).as_str() > "a5");
        assert!(state.order_between(Some(vanished), Some(here)).as_str() < "a5");
    }

    /// Neighbours arriving the wrong way round would panic the midpoint.
    #[test]
    fn neighbours_out_of_order_land_after_the_first_instead_of_panicking() {
        let mut state = State::default();
        let (one, two) = (Ulid::generate(), Ulid::generate());
        state.apply(&ev(1, "a", add(one, "first")));
        state.apply(&ev(2, "a", add(two, "second")));
        state.tasks.get_mut(&one).unwrap().order = "a9".into();
        state.tasks.get_mut(&two).unwrap().order = "a1".into();

        assert!(state.order_between(Some(one), Some(two)).as_str() > "a9");
    }

    /// Filing from the sidebar arrives without neighbours, and the midpoint of
    /// nothing is always the same key: every task would pile on the first one.
    #[test]
    fn a_task_filed_without_neighbours_goes_last_in_its_list() {
        let mut state = State::default();
        let list = Ulid::generate();
        let (one, two) = (Ulid::generate(), Ulid::generate());
        state.apply(&ev(1, "a", add(one, "first")));
        state.apply(&ev(2, "a", add(two, "second")));
        for (id, key) in [(one, "a0"), (two, "a5")] {
            let task = state.tasks.get_mut(&id).unwrap();
            task.order = key.into();
            task.list = Some(list);
        }

        let landed = state.order_last_in(Some(list));
        assert!(landed.as_str() > "a5", "it did not go last: {landed}");
        assert_ne!(landed, state.order_between(None, None));
    }

    #[test]
    fn the_first_task_of_an_empty_list_still_gets_a_key() {
        let state = State::default();
        assert!(!state.order_last_in(None).is_empty());
        assert!(!state.order_last_in(Some(Ulid::generate())).is_empty());
    }

    /// The archive is what you did; what you decided not to do folds away.
    #[test]
    fn a_discarded_task_folds_itself_away() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(1, "a", add(id, "no lo voy a hacer")));
        assert!(!state.tasks[&id].hidden);

        state.apply(&ev(2, "a", Op::TaskDrop { id }));
        assert!(state.tasks[&id].hidden);
        assert_eq!(state.tasks[&id].status, Status::Dropped);

        state.apply(&ev(3, "a", Op::TaskShow { id }));
        assert!(!state.tasks[&id].hidden, "it can be brought back by hand");
    }

    #[test]
    fn what_was_folded_away_is_findable_but_never_first() {
        let mut state = State::default();
        let (kept, dropped) = (Ulid::generate(), Ulid::generate());
        state.apply(&ev(1, "a", add(kept, "revisar el proxy")));
        state.apply(&ev(2, "a", add(dropped, "revisar el proxy viejo")));
        state.apply(&ev(3, "a", Op::TaskDrop { id: dropped }));

        let found = state.search("proxy", crate::view::Scope::Either);
        assert_eq!(found.len(), 2, "hiding is not deleting");
        assert_eq!(found[0].id, kept);
    }

    /// Open and hidden is a state no view reaches: neither the open ones, which
    /// ask for `hidden == false`, nor the archive, which asks for archived.
    #[test]
    fn reopening_a_hidden_task_brings_it_back_into_sight() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(1, "a", add(id, "revisar el deploy")));
        state.apply(&ev(2, "a", Op::TaskDone { id }));
        state.apply(&ev(3, "a", Op::TaskHide { id }));
        assert!(state.tasks[&id].hidden);

        state.apply(&ev(4, "a", Op::TaskReopen { id }));
        assert!(!state.tasks[&id].hidden);
        assert_eq!(
            state.matching(&crate::view::Filter::default(), day()).len(),
            1
        );
    }

    #[test]
    fn hidden_work_stays_out_of_every_view_that_did_not_ask() {
        let mut state = State::default();
        let seen = Ulid::generate();
        let away = Ulid::generate();
        for (id, title) in [(seen, "a la vista"), (away, "guardada")] {
            state.apply(&ev(1, "a", add(id, title)));
            state.apply(&ev(2, "a", Op::TaskDone { id }));
        }
        state.apply(&ev(3, "a", Op::TaskHide { id: away }));

        let archive = crate::view::Filter {
            scope: crate::view::Scope::Archived,
            ..Default::default()
        };
        let folded = crate::view::Filter {
            hidden: true,
            ..archive.clone()
        };

        assert_eq!(state.matching(&archive, day()).len(), 1);
        assert_eq!(state.matching(&folded, day()).len(), 1);
        assert_eq!(state.matching(&folded, day())[0].id, away);
        assert!(
            state
                .matching(&crate::view::Filter::default(), day())
                .is_empty()
        );
    }
}
