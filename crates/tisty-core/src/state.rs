use std::collections::{BTreeMap, BTreeSet};

use ulid::Ulid;

use crate::{
    event::{Event, LogAdd, LogEdit, Op, StepAdd, TaskAdd, TaskMove, TaskPatch},
    model::{
        DocId, Folder, FolderId, Kept, List, ListId, LogEntry, Status, Step, StepId, Tag, Task,
        TaskId,
    },
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
    pub folders: BTreeMap<FolderId, Folder>,
    pub docs: BTreeMap<DocId, Kept>,
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
            Op::FolderAdd { id, d } => {
                self.folders.insert(
                    *id,
                    Folder {
                        id: *id,
                        name: d.name.clone(),
                        order: d.order.clone(),
                        parent: d.parent.filter(|at| at != id),
                        icon: d.icon.clone().filter(|key| crate::model::icon::known(key)),
                    },
                );
            }
            Op::FolderRename { id, d } => {
                if let Some(folder) = self.folders.get_mut(id) {
                    folder.name = d.name.clone();
                }
            }
            Op::FolderLook { id, d } => {
                if let Some(folder) = self.folders.get_mut(id)
                    && let Some(icon) = &d.icon
                {
                    folder.icon = icon.clone().filter(|key| crate::model::icon::known(key));
                }
            }
            Op::FolderMove { id, d } => {
                if let Some(parent) = d.folder
                    && parent.is_none_or(|at| self.has_room_under(at))
                    && !self.would_loop(*id, parent)
                    && self.depth(parent) + self.tallest_under(*id) <= crate::model::DEEPEST
                    && let Some(folder) = self.folders.get_mut(id)
                {
                    folder.parent = parent;
                }
            }
            Op::FolderDelete { id } => {
                self.folders.remove(id);
                self.tombstones.insert(*id);
                let orphaned: Vec<FolderId> = self
                    .folders
                    .values()
                    .filter(|one| one.parent == Some(*id))
                    .map(|one| one.id)
                    .collect();
                for child in orphaned {
                    if let Some(folder) = self.folders.get_mut(&child) {
                        folder.parent = None;
                    }
                }
                for doc in self.docs.values_mut() {
                    if doc.folder == Some(*id) {
                        doc.folder = None;
                    }
                }
            }
            Op::DocAdd { id, d } => {
                self.docs.insert(
                    *id,
                    Kept {
                        id: *id,
                        file: d.file.clone(),
                        order: d.order.clone(),
                        folder: d.folder,
                        archived: false,
                    },
                );
            }
            Op::DocMove { id, d } => {
                if let Some(folder) = d.folder
                    && let Some(doc) = self.docs.get_mut(id)
                {
                    doc.folder = folder;
                }
            }
            Op::DocDelete { id } => {
                self.docs.remove(id);
                self.tombstones.insert(*id);
            }
            Op::DocArchive { id } => {
                if let Some(doc) = self.docs.get_mut(id) {
                    doc.archived = true;
                }
            }
            Op::DocUnarchive { id } => {
                if let Some(doc) = self.docs.get_mut(id) {
                    doc.archived = false;
                }
            }
            Op::ListLook { id, d } => {
                if let Some(list) = self.lists.get_mut(id) {
                    if let Some(icon) = &d.icon {
                        list.icon = icon.clone().filter(|key| crate::model::icon::known(key));
                    }
                    if let Some(color) = &d.color {
                        list.color = color.clone();
                    }
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

    /// The same batch, with any entity its own undo buried given a fresh id.
    ///
    /// A tombstone is permanent by design — that is what makes a deletion
    /// converge between machines — so `apply` drops every event about a buried
    /// id, including the `TaskAdd` that would bring it back. Replaying a redo
    /// verbatim therefore did nothing at all and reported that it had worked.
    /// The occurrence a repeat gives birth to is the case that hits people.
    pub fn afresh(&self, ops: Vec<Op>) -> Vec<Op> {
        let born: std::collections::HashMap<Ulid, Ulid> = ops
            .iter()
            .filter_map(|op| match op {
                Op::TaskAdd { id, .. }
                | Op::ListAdd { id, .. }
                | Op::FolderAdd { id, .. }
                | Op::DocAdd { id, .. }
                    if self.is_erased(*id) =>
                {
                    Some((*id, Ulid::generate()))
                }
                _ => None,
            })
            .collect();

        if born.is_empty() {
            return ops;
        }
        ops.into_iter()
            .map(|op| match born.get(&op.about_whom()) {
                Some(&fresh) => op.about(fresh),
                None => op,
            })
            .collect()
    }

    pub fn unfiled(&self) -> Vec<&Kept> {
        self.docs
            .values()
            .filter(|one| !one.archived)
            .filter(|one| one.folder.is_none_or(|at| !self.folders.contains_key(&at)))
            .collect()
    }

    pub fn inside(&self, folder: FolderId) -> Vec<&Kept> {
        self.docs
            .values()
            .filter(|one| !one.archived && one.folder == Some(folder))
            .collect()
    }

    pub fn put_away(&self) -> Vec<&Kept> {
        self.docs.values().filter(|one| one.archived).collect()
    }

    /// A reference whose target arrives later in the log is not a mistake to
    /// erase: repairing on read keeps the intent and still shows everything.
    fn adrift(&self, folder: &Folder) -> bool {
        folder
            .parent
            .is_some_and(|at| !self.folders.contains_key(&at))
    }

    pub fn under(&self, parent: Option<FolderId>) -> Vec<&Folder> {
        let mut found: Vec<&Folder> = self
            .folders
            .values()
            .filter(|one| match parent {
                None => one.parent.is_none() || self.adrift(one),
                at => one.parent == at,
            })
            .collect();
        found.sort_by(|a, b| {
            a.order
                .cmp(&b.order)
                .then(a.name.cmp(&b.name))
                .then(a.id.cmp(&b.id))
        });
        found
    }

    pub fn held_by(&self, folder: FolderId) -> usize {
        self.counting(folder, &mut std::collections::BTreeSet::new())
    }

    fn counting(&self, folder: FolderId, seen: &mut std::collections::BTreeSet<FolderId>) -> usize {
        if !seen.insert(folder) {
            return 0;
        }
        self.inside(folder).len()
            + self
                .under(Some(folder))
                .iter()
                .map(|one| self.counting(one.id, seen))
                .sum::<usize>()
    }

    fn has_room_under(&self, at: FolderId) -> bool {
        self.folders.contains_key(&at) && self.depth(Some(at)) < crate::model::DEEPEST
    }

    pub fn depth(&self, at: Option<FolderId>) -> usize {
        let mut deep = 0;
        let mut walk = at;
        while let Some(one) = walk {
            deep += 1;
            walk = self.folders.get(&one).and_then(|folder| folder.parent);
            if deep > crate::model::DEEPEST {
                break;
            }
        }
        deep
    }

    pub fn tall_under(&self, at: FolderId) -> usize {
        self.tallest_under(at)
    }

    pub fn would_swallow(&self, moving: FolderId, under: FolderId) -> bool {
        self.would_loop(moving, Some(under))
    }

    fn tallest_under(&self, at: FolderId) -> usize {
        self.tallest(at, &mut std::collections::BTreeSet::new())
    }

    fn tallest(&self, at: FolderId, seen: &mut std::collections::BTreeSet<FolderId>) -> usize {
        if !seen.insert(at) {
            return 0;
        }
        1 + self
            .under(Some(at))
            .iter()
            .map(|one| self.tallest(one.id, seen))
            .max()
            .unwrap_or(0)
    }

    fn would_loop(&self, moving: FolderId, under: Option<FolderId>) -> bool {
        let mut at = under;
        let mut seen = std::collections::BTreeSet::new();
        while let Some(one) = at {
            if one == moving || !seen.insert(one) {
                return true;
            }
            at = self.folders.get(&one).and_then(|folder| folder.parent);
        }
        false
    }

    /// For restoring a projection stored elsewhere; omitting tombstones lets deletions resurrect.
    pub fn mark_erased(&mut self, id: Ulid) {
        self.tombstones.insert(id);
    }

    /// Every task, not only the open ones: an archived key still holds its
    /// place, and reopening one must not land on a key already taken.
    fn ordered_keys(&self, list: Option<ListId>) -> Vec<&str> {
        self.tasks
            .values()
            .filter(|task| task.list == list)
            .map(|task| task.order.as_str())
            .collect()
    }

    /// Last place in a list, for a task that arrived without neighbours —
    /// dropped on the sidebar, where there was nothing to land between.
    /// Completing a repeating task emits the next one, in the same batch: undo
    /// has to take back both or the series grows a copy every time.
    pub fn completing(&self, id: TaskId, now: jiff::Zoned) -> Vec<Op> {
        let done = vec![Op::TaskDone { id }];
        let Some(task) = self.tasks.get(&id) else {
            return done;
        };
        // Completing was idempotent until it started emitting a task: a second
        // click, or the terminal and the window at once, would make two.
        if task.status != crate::model::Status::Open {
            return done;
        }
        let Some(repeat) = task.repeat else {
            return done;
        };
        let Some(next) = repeat.next(
            task.date.as_ref(),
            now.datetime(),
            now.datetime()
                .date()
                .to_datetime(jiff::civil::Time::midnight()),
            now.time_zone().iana_name().unwrap_or("UTC"),
        ) else {
            return done;
        };
        // Past the last day the series was given: this one is completed and no
        // successor follows it.
        if repeat.ended(next.at.date()) {
            return done;
        }

        // Everything that was pinned to the old date moves with it. A deadline
        // left where it was would be overdue before the task exists, and a
        // reminder left behind is the only reason to put one on a habit.
        let along = task
            .date
            .as_ref()
            .map(|was| next.at.date().since(was.at.date()));

        let mut fresh =
            crate::event::TaskAdd::new(task.title.clone(), self.order_last_in(task.list));
        fresh.deadline = shifted(task.deadline.as_ref(), along.as_ref());
        // With no old date there is nothing to measure the move against, so the
        // cadence itself moves them: a reminder on «cada 3 días» that was left
        // where it was would be in the past from the second occurrence on, and
        // stay there for ever.
        fresh.reminders = task
            .reminders
            .iter()
            .filter_map(|one| match (&task.date, repeat.cadence().after(one.at)) {
                (None, Some(at)) => Some(one.moved(at)),
                _ => shifted(Some(one), along.as_ref()),
            })
            .collect();
        fresh.date = Some(next);
        fresh.priority = Some(task.priority);
        fresh.list = task.list;
        fresh.tags = task.tags.clone();
        fresh.repeat = Some(repeat);
        fresh.after = Some(id);

        let mut ops = done;
        let born = ulid::Ulid::generate();
        ops.push(Op::TaskAdd { id: born, d: fresh });

        // The recipe is the task. A weekly «water the plants» that came back
        // without its rooms, or a monthly report without the instructions that
        // took an afternoon to write, is a different task wearing the name.
        if let Some(body) = &task.description {
            ops.push(Op::TaskDescribe {
                id: born,
                d: crate::event::Body {
                    body: Some(body.clone()),
                },
            });
        }
        // Unticked: what was done belongs to the occurrence that was closed.
        for step in &task.steps {
            ops.push(Op::StepAdd {
                id: born,
                d: crate::event::StepAdd {
                    step: ulid::Ulid::generate(),
                    text: step.text.clone(),
                    order: step.order.clone(),
                },
            });
        }
        ops
    }

    /// Reopening a completed occurrence takes its successor back with it, or
    /// the series would run twice from then on — and undoing a tick is the
    /// commonest way to say «I pressed that by mistake».
    pub fn reopening(&self, id: TaskId) -> Vec<Op> {
        let mut ops = vec![Op::TaskReopen { id }];
        if let Some(born) = self
            .tasks
            .values()
            .find(|task| task.after == Some(id) && task.status == crate::model::Status::Open)
            .filter(|born| untouched(born))
        {
            ops.push(Op::TaskDelete { id: born.id });
        }
        ops
    }

    pub fn order_last_in(&self, list: Option<ListId>) -> String {
        order::last_of(self.ordered_keys(list))
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

    /// Steps carry an order of their own, and nothing sorts before it.
    pub fn step_order_between(
        &self,
        task: TaskId,
        after: Option<StepId>,
        before: Option<StepId>,
    ) -> String {
        let key = |id: Option<StepId>| {
            id.and_then(|id| self.tasks.get(&task)?.step(id))
                .map(|step| step.order.clone())
        };
        match (key(after), key(before)) {
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
        self.searching(query, scope, usize::MAX).0
    }

    /// The best `most` of them, and how many there were in all.
    ///
    /// A client that draws every row and ships every body over an IPC boundary
    /// cannot afford «everything»: one letter against a store with years of
    /// archive matches most of it, and the cost lands on each keystroke.
    pub fn searching(
        &self,
        query: &str,
        scope: crate::view::Scope,
        most: usize,
    ) -> (Vec<&Task>, usize) {
        use crate::view::Scope;

        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return (Vec::new(), 0);
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
                t.folded(),
                t.is_archived(),
                *hit,
                std::cmp::Reverse(t.weight()),
                std::cmp::Reverse(t.completed_at),
                std::cmp::Reverse(t.id),
            )
        });
        let total = hits.len();
        (hits.into_iter().take(most).map(|(_, t)| t).collect(), total)
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

    /// Named exactly, ignoring case and accents. `find_list` falls back to
    /// substring so a half-typed `@mark` still lands, which is the opposite of
    /// what naming a new list wants: «Tra» would file into «Trabajo» instead of
    /// making the list that was asked for.
    pub fn list_called(&self, name: &str) -> Vec<&List> {
        // Trimmed first: `loose` turns a space into a hyphen, so an untrimmed
        // name would match nothing and quietly make a second list.
        let wanted = loose(name.trim().trim_start_matches('@').trim());
        self.lists
            .values()
            .filter(|one| loose(&one.name) == wanted)
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

/// Nothing was written into it since it was born.
///
/// `TaskDelete` cannot be undone, so taking the successor back is only safe
/// while it holds nothing of the person's: an entry in its journal or a ticked
/// step means work that reopening must not destroy, and running the series
/// twice is the lesser of the two.
fn untouched(born: &Task) -> bool {
    born.log.is_empty() && !born.steps.iter().any(|step| step.done)
}

/// Carried the same distance the date moved, so what was two days before it
/// stays two days before it.
fn shifted(
    spec: Option<&crate::DateSpec>,
    along: Option<&Result<jiff::Span, jiff::Error>>,
) -> Option<crate::DateSpec> {
    let spec = spec?;
    // Nothing to move it by — a repeat with no date of its own — is not a
    // reason to drop the deadline or the reminder on the floor.
    let Some(Ok(span)) = along else {
        return Some(spec.clone());
    };
    match spec.at.checked_add(*span) {
        Ok(at) => Some(spec.moved(at)),
        Err(_) => Some(spec.clone()),
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
        repeat: d.repeat,
        after: d.after,
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
    if let Some(v) = &d.repeat {
        task.repeat = *v;
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

    use crate::model::{Cadence, Repeat, Unit};

    #[test]
    fn completing_a_repeating_task_emits_the_next_one() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new("sacar la basura", "a0");
        add.date = Some(DateSpec::floating(
            jiff::civil::date(2026, 8, 4).at(9, 0, 0, 0),
            "Europe/Madrid",
        ));
        add.repeat = Some(Repeat::due(Cadence {
            every: 1,
            unit: Unit::Week,
        }));
        add.tags = vec!["casa".parse().unwrap()];
        state.apply(&ev(1, "a", Op::TaskAdd { id, d: add }));

        let now = jiff::civil::date(2026, 8, 4)
            .at(21, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();
        let ops = state.completing(id, now);

        assert_eq!(ops.len(), 2, "{ops:?}");
        let Op::TaskAdd { d, .. } = &ops[1] else {
            panic!("the next one was not emitted");
        };
        assert_eq!(d.title, "sacar la basura");
        assert_eq!(
            d.date.as_ref().unwrap().date(),
            jiff::civil::date(2026, 8, 11)
        );
        assert_eq!(d.repeat, add_repeat());
        assert_eq!(d.tags.len(), 1, "it lost what it was filed under");
    }

    fn add_repeat() -> Option<Repeat> {
        Some(Repeat::due(Cadence {
            every: 1,
            unit: Unit::Week,
        }))
    }

    /// A deadline left where it was would be overdue before the task exists,
    /// and a reminder left behind is the only reason to put one on a habit.
    #[test]
    fn what_was_pinned_to_the_old_date_moves_with_it() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new("declarar el IVA", "a0");
        let day = |d: i8| {
            DateSpec::floating(
                jiff::civil::date(2026, 8, d).at(9, 0, 0, 0),
                "Europe/Madrid",
            )
        };
        add.date = Some(day(11));
        add.deadline = Some(day(12));
        add.reminders = vec![day(10)];
        add.repeat = Some(Repeat::done(Cadence {
            every: 1,
            unit: Unit::Month,
        }));
        state.apply(&ev(1, "a", Op::TaskAdd { id, d: add }));

        let now = jiff::civil::date(2026, 8, 11)
            .at(20, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();
        let ops = state.completing(id, now);
        let Op::TaskAdd { d, .. } = &ops[1] else {
            panic!("no next one");
        };

        assert_eq!(
            d.date.as_ref().unwrap().date(),
            jiff::civil::date(2026, 9, 11)
        );
        assert_eq!(
            d.deadline.as_ref().unwrap().date(),
            jiff::civil::date(2026, 9, 12),
            "it was born already overdue"
        );
        assert_eq!(
            d.reminders.first().unwrap().date(),
            jiff::civil::date(2026, 9, 10),
            "the reminder stayed behind"
        );
    }

    #[test]
    fn completing_twice_does_not_emit_two_of_the_next_one() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new("sacar la basura", "a0");
        add.date = Some(DateSpec::floating(
            jiff::civil::date(2026, 8, 4).at(9, 0, 0, 0),
            "Europe/Madrid",
        ));
        add.repeat = Some(Repeat::due(Cadence {
            every: 1,
            unit: Unit::Week,
        }));
        state.apply(&ev(1, "a", Op::TaskAdd { id, d: add }));

        let now = jiff::civil::date(2026, 8, 4)
            .at(21, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();
        for op in state.completing(id, now.clone()) {
            state.apply(&ev(2, "a", op));
        }
        assert_eq!(
            state.completing(id, now).len(),
            1,
            "a second click made another"
        );
    }

    #[test]
    fn completing_an_ordinary_task_emits_nothing_extra() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        state.apply(&ev(
            1,
            "a",
            Op::TaskAdd {
                id,
                d: TaskAdd::new("comprar pan", "a0"),
            },
        ));

        let now = jiff::Timestamp::now().to_zoned(jiff::tz::TimeZone::UTC);
        assert_eq!(state.completing(id, now).len(), 1);
    }
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

    /// A weekly «water the plants» that came back without its rooms is a
    /// different task wearing the same name.
    #[test]
    fn the_next_occurrence_carries_the_recipe() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new("regar las plantas", "a0");
        add.date = Some(DateSpec::floating(
            jiff::civil::date(2026, 8, 4).at(9, 0, 0, 0),
            "Europe/Madrid",
        ));
        add.repeat = Some(Repeat::due(Cadence {
            every: 1,
            unit: Unit::Week,
        }));
        state.apply(&ev(1, "a", Op::TaskAdd { id, d: add }));
        state.apply(&ev(
            2,
            "a",
            Op::TaskDescribe {
                id,
                d: crate::event::Body {
                    body: Some("agua tibia, nunca del grifo".into()),
                },
            },
        ));
        for (n, text) in [(3, "la sala"), (4, "la cocina")] {
            state.apply(&ev(
                n,
                "a",
                Op::StepAdd {
                    id,
                    d: crate::event::StepAdd {
                        step: ulid::Ulid::generate(),
                        text: text.into(),
                        order: format!("a{n}"),
                    },
                },
            ));
        }

        let now = jiff::civil::date(2026, 8, 4)
            .at(21, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();
        for op in state.completing(id, now) {
            state.apply(&ev(9, "a", op));
        }

        let born = state
            .tasks
            .values()
            .find(|task| task.after == Some(id))
            .expect("no next occurrence");
        assert_eq!(
            born.description.as_deref(),
            Some("agua tibia, nunca del grifo")
        );
        assert_eq!(
            born.steps
                .iter()
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>(),
            vec!["la sala", "la cocina"]
        );
    }

    /// What was ticked belongs to the occurrence that was closed.
    #[test]
    fn the_steps_come_back_unticked() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new("regar las plantas", "a0");
        add.repeat = Some(Repeat::done(Cadence {
            every: 3,
            unit: Unit::Day,
        }));
        state.apply(&ev(1, "a", Op::TaskAdd { id, d: add }));
        let step = ulid::Ulid::generate();
        state.apply(&ev(
            2,
            "a",
            Op::StepAdd {
                id,
                d: crate::event::StepAdd {
                    step,
                    text: "la sala".into(),
                    order: "a1".into(),
                },
            },
        ));
        state.apply(&ev(
            3,
            "a",
            Op::StepDone {
                id,
                d: crate::event::StepRef { step },
            },
        ));

        let now = jiff::civil::date(2026, 8, 4)
            .at(21, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();
        for op in state.completing(id, now) {
            state.apply(&ev(9, "a", op));
        }

        let born = state
            .tasks
            .values()
            .find(|task| task.after == Some(id))
            .expect("no next occurrence");
        assert_eq!(born.steps.len(), 1);
        assert!(!born.steps[0].done, "the new one arrived already ticked");
        assert_ne!(born.steps[0].id, step, "both share one step id");
    }

    /// A task with no repeat must emit nothing but the completion.
    #[test]
    fn nothing_is_copied_when_there_is_no_repeat() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        state.apply(&ev(1, "a", add(id, "comprar pan")));
        state.apply(&ev(
            2,
            "a",
            Op::TaskDescribe {
                id,
                d: crate::event::Body {
                    body: Some("del horno de la esquina".into()),
                },
            },
        ));

        let now = jiff::civil::date(2026, 8, 4)
            .at(21, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();

        assert_eq!(state.completing(id, now).len(), 1);
    }

    /// It was copied verbatim, so from the second occurrence on it sat in the
    /// past and never fired again.
    #[test]
    fn a_reminder_on_a_dateless_repeat_moves_with_the_cadence() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new("tomar la pastilla", "a0");
        add.repeat = Some(Repeat::done(Cadence {
            every: 3,
            unit: Unit::Day,
        }));
        add.reminders = vec![DateSpec::floating(
            jiff::civil::date(2026, 8, 11).at(9, 0, 0, 0),
            "Europe/Madrid",
        )];
        state.apply(&ev(1, "a", Op::TaskAdd { id, d: add }));

        let now = jiff::civil::date(2026, 8, 11)
            .at(21, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();
        for op in state.completing(id, now) {
            state.apply(&ev(9, "a", op));
        }

        let born = state
            .tasks
            .values()
            .find(|task| task.after == Some(id))
            .expect("no next occurrence");
        assert_eq!(
            born.reminders[0].at,
            jiff::civil::date(2026, 8, 14).at(9, 0, 0, 0),
            "the reminder stayed in the past"
        );
    }

    /// The commonest way to say «I pressed that by mistake» must not cost
    /// anything, so a successor nobody has touched goes back with it.
    #[test]
    fn reopening_takes_back_a_successor_nobody_wrote_in() {
        let (mut state, id) = repeating();
        let born = successor(&state, id);

        for op in state.reopening(id) {
            state.apply(&ev(20, "a", op));
        }

        assert!(!state.tasks.contains_key(&born));
    }

    /// `TaskDelete` cannot be undone: an entry written into the successor is
    /// work, and running the series twice is the lesser of the two.
    #[test]
    fn reopening_leaves_a_successor_that_was_written_in() {
        let (mut state, id) = repeating();
        let born = successor(&state, id);
        state.apply(&ev(
            15,
            "a",
            Op::TaskLog {
                id: born,
                d: crate::event::LogAdd::new(ulid::Ulid::generate(), "media hora en la sala"),
            },
        ));

        for op in state.reopening(id) {
            state.apply(&ev(20, "a", op));
        }

        assert!(
            state.tasks.contains_key(&born),
            "the journal entry was lost"
        );
    }

    #[test]
    fn a_ticked_step_also_keeps_the_successor_alive() {
        let (mut state, id) = repeating();
        let born = successor(&state, id);
        let step = ulid::Ulid::generate();
        state.apply(&ev(
            15,
            "a",
            Op::StepAdd {
                id: born,
                d: crate::event::StepAdd {
                    step,
                    text: "la sala".into(),
                    order: "a1".into(),
                },
            },
        ));
        state.apply(&ev(
            16,
            "a",
            Op::StepDone {
                id: born,
                d: crate::event::StepRef { step },
            },
        ));

        for op in state.reopening(id) {
            state.apply(&ev(20, "a", op));
        }

        assert!(state.tasks.contains_key(&born));
    }

    fn repeating() -> (State, TaskId) {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new("regar las plantas", "a0");
        add.repeat = Some(Repeat::done(Cadence {
            every: 1,
            unit: Unit::Week,
        }));
        state.apply(&ev(1, "a", Op::TaskAdd { id, d: add }));
        let now = jiff::civil::date(2026, 8, 4)
            .at(21, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();
        for op in state.completing(id, now) {
            state.apply(&ev(9, "a", op));
        }
        (state, id)
    }

    fn successor(state: &State, of: TaskId) -> TaskId {
        state
            .tasks
            .values()
            .find(|task| task.after == Some(of))
            .expect("no successor")
            .id
    }

    /// A tombstone is permanent, so replaying the batch verbatim applied
    /// nothing at all — and said it had worked.
    #[test]
    fn a_redo_rebuilds_what_its_undo_buried() {
        let mut state = State::default();
        let gone = ulid::Ulid::generate();
        let ops = vec![
            Op::TaskAdd {
                id: gone,
                d: TaskAdd::new("regar las plantas", "a0"),
            },
            Op::TaskDescribe {
                id: gone,
                d: crate::event::Body {
                    body: Some("agua tibia".into()),
                },
            },
        ];
        state.mark_erased(gone);

        let again = state.afresh(ops);

        assert_eq!(again.len(), 2);
        let fresh = again[0].about_whom();
        assert_ne!(fresh, gone, "it reused the buried id");
        assert_eq!(again[1].about_whom(), fresh, "the batch was torn apart");
        for op in &again {
            state.apply(&ev(9, "a", op.clone()));
        }
        assert_eq!(
            state.tasks[&fresh].description.as_deref(),
            Some("agua tibia")
        );
    }

    /// Renaming what was never buried would break every other redo.
    #[test]
    fn nothing_is_renamed_when_nothing_was_buried() {
        let state = State::default();
        let id = ulid::Ulid::generate();
        let ops = vec![Op::TaskDone { id }];

        assert_eq!(state.afresh(ops.clone()), ops);
    }

    /// A client that draws every row and ships every body over IPC cannot
    /// afford «everything» on each keystroke.
    #[test]
    fn a_search_hands_back_the_top_and_says_how_many_there_were() {
        let mut state = State::default();
        for n in 0..50 {
            let id = ulid::Ulid::generate();
            state.apply(&ev(
                n,
                "a",
                Op::TaskAdd {
                    id,
                    d: TaskAdd::new(format!("informe {n}"), format!("a{n}")),
                },
            ));
        }

        let (hits, total) = state.searching("informe", crate::view::Scope::Either, 10);

        assert_eq!(hits.len(), 10);
        assert_eq!(
            total, 50,
            "the count has to be the real one, not the capped one"
        );
    }

    #[test]
    fn the_cap_never_shrinks_a_result_that_already_fits() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        state.apply(&ev(1, "a", add(id, "informe anual")));

        let (hits, total) = state.searching("informe", crate::view::Scope::Either, 200);

        assert_eq!(hits.len(), 1);
        assert_eq!(total, 1);
    }

    /// The uncapped call is what the terminal uses, and it must not change.
    #[test]
    fn the_plain_search_still_returns_everything() {
        let mut state = State::default();
        for n in 0..30 {
            let id = ulid::Ulid::generate();
            state.apply(&ev(
                n,
                "a",
                Op::TaskAdd {
                    id,
                    d: TaskAdd::new(format!("informe {n}"), format!("a{n}")),
                },
            ));
        }

        assert_eq!(
            state.search("informe", crate::view::Scope::Either).len(),
            30
        );
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

    #[test]
    fn a_step_dropped_between_two_lands_between_them() {
        let mut state = State::default();
        let task = Ulid::generate();
        state.apply(&ev(1, "a", add(task, "revisar el deploy")));

        let mut steps = Vec::new();
        for (n, text) in ["uno", "dos", "tres"].iter().enumerate() {
            let step = Ulid::generate();
            steps.push(step);
            state.apply(&ev(
                2 + n as i64,
                "a",
                Op::StepAdd {
                    id: task,
                    d: StepAdd {
                        step,
                        text: (*text).into(),
                        order: format!("a{n}"),
                    },
                },
            ));
        }

        let key = state.step_order_between(task, Some(steps[0]), Some(steps[1]));
        assert!(key.as_str() > "a0" && key.as_str() < "a1", "{key}");
        assert!(
            state
                .step_order_between(task, None, Some(steps[0]))
                .as_str()
                < "a0"
        );
        assert!(
            state
                .step_order_between(task, Some(steps[2]), None)
                .as_str()
                > "a2"
        );
    }

    #[test]
    fn a_step_that_is_gone_does_not_move_the_one_being_dragged() {
        let mut state = State::default();
        let task = Ulid::generate();
        let step = Ulid::generate();
        state.apply(&ev(1, "a", add(task, "revisar el deploy")));
        state.apply(&ev(
            2,
            "a",
            Op::StepAdd {
                id: task,
                d: StepAdd {
                    step,
                    text: "uno".into(),
                    order: "a5".into(),
                },
            },
        ));

        let vanished = Ulid::generate();
        assert!(
            state
                .step_order_between(task, Some(step), Some(vanished))
                .as_str()
                > "a5"
        );
    }

    /// Folded by the view, never by touching the data: mutating `hidden` on a
    /// drop cost the export, the whole CLI and a correct undo.
    #[test]
    fn a_discarded_task_folds_without_being_marked_hidden() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(1, "a", add(id, "no lo voy a hacer")));
        state.apply(&ev(2, "a", Op::TaskDrop { id }));

        let task = &state.tasks[&id];
        assert_eq!(task.status, Status::Dropped);
        assert!(!task.hidden, "the flag is the person's, not the status");
        assert!(task.folded(), "and it folds all the same");
    }

    #[test]
    fn the_archive_shows_what_you_did_and_the_drawer_the_rest() {
        let mut state = State::default();
        let (done, gone) = (Ulid::generate(), Ulid::generate());
        state.apply(&ev(1, "a", add(done, "lo hice")));
        state.apply(&ev(2, "a", add(gone, "no lo haré")));
        state.apply(&ev(3, "a", Op::TaskDone { id: done }));
        state.apply(&ev(4, "a", Op::TaskDrop { id: gone }));

        let archive = |folded: bool| {
            let filter = crate::view::Filter {
                scope: crate::view::Scope::Archived,
                hidden: folded,
                ..Default::default()
            };
            state.matching(&filter, day()).len()
        };
        assert_eq!(archive(false), 1);
        assert_eq!(archive(true), 1);
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

    fn holding(names: &[&str]) -> State {
        let mut state = State::default();
        for (nth, name) in names.iter().enumerate() {
            state.apply(&crate::Event::new(
                DeviceId("dev".into()),
                jiff::Timestamp::now(),
                crate::Op::ListAdd {
                    id: ulid::Ulid::generate(),
                    d: ListAdd {
                        name: (*name).to_string(),
                        order: format!("a{nth}"),
                        color: None,
                    },
                },
            ));
        }
        state
    }

    /// `find_list` matches a substring on purpose, so a half-typed `@mark`
    /// still lands. Naming a NEW list wants the opposite: «Tra» must make
    /// «Tra», not quietly file the task under «Trabajo».
    #[test]
    fn naming_a_list_does_not_settle_for_a_longer_one() {
        let state = holding(&["Trabajo"]);

        assert!(state.list_called("Tra").is_empty());
        assert_eq!(
            state.find_list("Tra").len(),
            1,
            "the fuzzy one still matches"
        );
    }

    /// Reusing the same list under another casing is right: two lists called
    /// «Trabajo» and «trabajo» would be the same list to a reader.
    #[test]
    fn naming_a_list_reuses_the_one_already_there_whatever_the_case() {
        let state = holding(&["Trabajo"]);

        assert_eq!(state.list_called("trabajo").len(), 1);
        assert_eq!(state.list_called("  TRABAJO ").len(), 1);
    }

    #[test]
    fn two_lists_of_the_same_name_are_both_returned_so_the_caller_can_refuse() {
        let state = holding(&["Casa", "casa"]);

        assert_eq!(state.list_called("casa").len(), 2);
    }

    #[test]
    fn a_series_with_a_last_day_stops_there() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = crate::event::TaskAdd::new("tomar la pastilla".to_string(), "a0".to_string());
        add.date = Some(DateSpec::floating(
            "2026-08-12T09:00:00".parse().unwrap(),
            "Europe/Madrid",
        ));
        add.repeat = Some(crate::model::Repeat {
            from: crate::model::From::Due,
            each: crate::model::Cadence {
                every: 1,
                unit: crate::model::Unit::Day,
            },
            until: Some(jiff::civil::date(2026, 8, 12)),
        });
        state.apply(&crate::Event::new(
            DeviceId("dev".into()),
            jiff::Timestamp::now(),
            crate::Op::TaskAdd { id, d: add },
        ));

        let ops = state.completing(id, "2026-08-12T10:00:00[Europe/Madrid]".parse().unwrap());

        assert_eq!(ops.len(), 1, "no successor past the last day: {ops:?}");
        assert!(matches!(ops.first(), Some(crate::Op::TaskDone { .. })));
    }

    #[test]
    fn a_series_still_running_hands_the_next_one_on() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = crate::event::TaskAdd::new("tomar la pastilla".to_string(), "a0".to_string());
        add.date = Some(DateSpec::floating(
            "2026-08-12T09:00:00".parse().unwrap(),
            "Europe/Madrid",
        ));
        add.repeat = Some(crate::model::Repeat {
            from: crate::model::From::Due,
            each: crate::model::Cadence {
                every: 1,
                unit: crate::model::Unit::Day,
            },
            until: Some(jiff::civil::date(2026, 12, 31)),
        });
        state.apply(&crate::Event::new(
            DeviceId("dev".into()),
            jiff::Timestamp::now(),
            crate::Op::TaskAdd { id, d: add },
        ));

        let ops = state.completing(id, "2026-08-12T10:00:00[Europe/Madrid]".parse().unwrap());

        assert!(ops.iter().any(|op| matches!(op, crate::Op::TaskAdd { .. })));
    }

    #[test]
    fn a_list_takes_an_icon_and_gives_it_back() {
        let mut state = State::default();
        let list = Ulid::generate();
        state.apply(&ev(
            1,
            "a",
            Op::ListAdd {
                id: list,
                d: ListAdd {
                    name: "Casa".into(),
                    order: "a0".into(),
                    color: None,
                },
            },
        ));

        state.apply(&ev(
            1,
            "a",
            Op::ListLook {
                id: list,
                d: crate::event::Look {
                    icon: Some(Some("home".into())),
                    color: Some(Some("green".into())),
                },
            },
        ));

        assert_eq!(state.lists[&list].icon.as_deref(), Some("home"));
        assert_eq!(state.lists[&list].color.as_deref(), Some("green"));
    }

    #[test]
    fn an_icon_nobody_ships_never_reaches_the_list() {
        let mut state = State::default();
        let list = Ulid::generate();
        state.apply(&ev(
            1,
            "a",
            Op::ListAdd {
                id: list,
                d: ListAdd {
                    name: "Casa".into(),
                    order: "a0".into(),
                    color: None,
                },
            },
        ));

        state.apply(&ev(
            1,
            "a",
            Op::ListLook {
                id: list,
                d: crate::event::Look {
                    icon: Some(Some("dragon".into())),
                    color: None,
                },
            },
        ));

        assert_eq!(state.lists[&list].icon, None);
    }

    #[test]
    fn changing_the_icon_leaves_the_colour_where_it_was() {
        let mut state = State::default();
        let list = Ulid::generate();
        state.apply(&ev(
            1,
            "a",
            Op::ListAdd {
                id: list,
                d: ListAdd {
                    name: "Casa".into(),
                    order: "a0".into(),
                    color: Some("green".into()),
                },
            },
        ));

        state.apply(&ev(
            1,
            "a",
            Op::ListLook {
                id: list,
                d: crate::event::Look {
                    icon: Some(Some("home".into())),
                    color: None,
                },
            },
        ));

        assert_eq!(state.lists[&list].color.as_deref(), Some("green"));
    }

    #[test]
    fn an_icon_can_be_taken_off_again() {
        let mut state = State::default();
        let list = Ulid::generate();
        state.apply(&ev(
            1,
            "a",
            Op::ListAdd {
                id: list,
                d: ListAdd {
                    name: "Casa".into(),
                    order: "a0".into(),
                    color: None,
                },
            },
        ));
        state.apply(&ev(
            1,
            "a",
            Op::ListLook {
                id: list,
                d: crate::event::Look {
                    icon: Some(Some("home".into())),
                    color: None,
                },
            },
        ));

        state.apply(&ev(
            1,
            "a",
            Op::ListLook {
                id: list,
                d: crate::event::Look {
                    icon: Some(None),
                    color: None,
                },
            },
        ));

        assert_eq!(state.lists[&list].icon, None);
    }

    fn folder(state: &mut State, name: &str, parent: Option<FolderId>) -> FolderId {
        let id = Ulid::generate();
        state.apply(&ev(
            1,
            "a",
            Op::FolderAdd {
                id,
                d: crate::event::FolderAdd {
                    name: name.into(),
                    order: "a0".into(),
                    parent,
                    icon: None,
                },
            },
        ));
        id
    }

    fn doc(state: &mut State, file: &str, folder: Option<FolderId>) -> DocId {
        let id = Ulid::generate();
        state.apply(&ev(
            1,
            "a",
            Op::DocAdd {
                id,
                d: crate::event::DocAdd {
                    file: file.into(),
                    order: "a0".into(),
                    folder,
                },
            },
        ));
        id
    }

    #[test]
    fn a_folder_hangs_where_it_was_told_to() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let inside = folder(&mut state, "corporativo", Some(work));

        assert_eq!(state.folders[&work].parent, None);
        assert_eq!(state.folders[&inside].parent, Some(work));
    }

    #[test]
    fn renaming_a_folder_moves_nothing() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let filed = doc(&mut state, "a3f1-0001", Some(work));

        state.apply(&ev(
            2,
            "a",
            Op::FolderRename {
                id: work,
                d: crate::event::Name {
                    name: "Work".into(),
                },
            },
        ));

        assert_eq!(state.folders[&work].name, "Work");
        assert_eq!(state.docs[&filed].folder, Some(work));
        assert_eq!(state.docs[&filed].file, "a3f1-0001");
    }

    #[test]
    fn a_document_with_no_folder_is_unfiled() {
        let mut state = State::default();
        let loose = doc(&mut state, "a3f1-0001", None);

        assert_eq!(state.docs[&loose].folder, None);
    }

    #[test]
    fn filing_a_document_never_touches_its_file() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let one = doc(&mut state, "a3f1-0001", None);

        state.apply(&ev(
            2,
            "a",
            Op::DocMove {
                id: one,
                d: crate::event::Filed {
                    folder: Some(Some(work)),
                },
            },
        ));

        assert_eq!(state.docs[&one].folder, Some(work));
        assert_eq!(state.docs[&one].file, "a3f1-0001");
    }

    #[test]
    fn a_document_can_be_taken_back_out_of_every_folder() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let one = doc(&mut state, "a3f1-0001", Some(work));

        state.apply(&ev(
            2,
            "a",
            Op::DocMove {
                id: one,
                d: crate::event::Filed { folder: Some(None) },
            },
        ));

        assert_eq!(state.docs[&one].folder, None);
    }

    #[test]
    fn counting_what_hangs_below_a_knot_still_ends() {
        let mut state = State::default();
        let a = folder(&mut state, "a", None);
        let b = folder(&mut state, "b", Some(a));
        state.folders.get_mut(&a).unwrap().parent = Some(b);

        assert_eq!(state.held_by(a), 0);
        assert!(state.under(None).is_empty());
    }

    #[test]
    fn a_document_hung_from_nothing_is_shown_as_unfiled() {
        let mut state = State::default();
        let gone = Ulid::generate();

        let one = doc(&mut state, "a3f1-0001", Some(gone));

        assert_eq!(state.docs[&one].folder, Some(gone), "the filing was erased");
        assert_eq!(state.unfiled().len(), 1, "nobody could reach it");
    }

    #[test]
    fn a_folder_that_was_deleted_never_comes_back() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        state.apply(&ev(2, "a", Op::FolderDelete { id: work }));

        state.apply(&ev(
            3,
            "b",
            Op::FolderAdd {
                id: work,
                d: crate::event::FolderAdd {
                    name: "trabajo".into(),
                    order: "a0".into(),
                    parent: None,
                    icon: None,
                },
            },
        ));

        assert!(!state.folders.contains_key(&work), "it rose again");
        assert!(state.is_erased(work));
    }

    #[test]
    fn a_document_that_was_deleted_never_comes_back() {
        let mut state = State::default();
        let one = doc(&mut state, "a3f1-0001", None);
        state.apply(&ev(2, "a", Op::DocDelete { id: one }));

        state.apply(&ev(
            3,
            "b",
            Op::DocAdd {
                id: one,
                d: crate::event::DocAdd {
                    file: "a3f1-0001".into(),
                    order: "a0".into(),
                    folder: None,
                },
            },
        ));

        assert!(!state.docs.contains_key(&one), "it rose again");
    }

    #[test]
    fn making_a_folder_again_after_undoing_it_takes_a_fresh_id() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        state.apply(&ev(2, "a", Op::FolderDelete { id: work }));

        let again = state.afresh(vec![Op::FolderAdd {
            id: work,
            d: crate::event::FolderAdd {
                name: "trabajo".into(),
                order: "a0".into(),
                parent: None,
                icon: None,
            },
        }]);

        assert!(matches!(again.first(), Some(Op::FolderAdd { id, .. }) if *id != work));
    }

    #[test]
    fn an_archived_document_leaves_the_tree_without_leaving_its_folder() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let one = doc(&mut state, "a3f1-0001", Some(work));

        state.apply(&ev(2, "a", Op::DocArchive { id: one }));

        assert!(state.inside(work).is_empty(), "it still crowds the tree");
        assert_eq!(state.held_by(work), 0, "it still counts");
        assert!(state.unfiled().is_empty(), "it fell out of its folder");
        assert_eq!(state.put_away().len(), 1);
        assert_eq!(
            state.docs[&one].folder,
            Some(work),
            "it forgot where it was"
        );
    }

    #[test]
    fn unarchiving_puts_it_back_where_it_was() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let one = doc(&mut state, "a3f1-0001", Some(work));
        state.apply(&ev(2, "a", Op::DocArchive { id: one }));

        state.apply(&ev(3, "a", Op::DocUnarchive { id: one }));

        assert_eq!(state.inside(work).len(), 1);
        assert!(state.put_away().is_empty());
    }

    #[test]
    fn unarchiving_into_a_folder_that_is_gone_lands_in_unfiled() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let one = doc(&mut state, "a3f1-0001", Some(work));
        state.apply(&ev(2, "a", Op::DocArchive { id: one }));
        state.apply(&ev(3, "a", Op::FolderDelete { id: work }));

        state.apply(&ev(4, "a", Op::DocUnarchive { id: one }));

        assert_eq!(state.unfiled().len(), 1, "nobody could reach it");
    }

    #[test]
    fn deleting_a_folder_leaves_its_documents_unfiled() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let inside = folder(&mut state, "corporativo", Some(work));
        let one = doc(&mut state, "a3f1-0001", Some(work));

        state.apply(&ev(2, "a", Op::FolderDelete { id: work }));

        assert!(!state.folders.contains_key(&work));
        assert_eq!(state.folders[&inside].parent, None, "a child was orphaned");
        assert_eq!(state.docs[&one].folder, None, "a document was orphaned");
        assert_eq!(state.unfiled().len(), 1);
    }

    #[test]
    fn a_folder_whose_parent_never_arrives_is_still_reachable() {
        let mut state = State::default();
        let gone = Ulid::generate();
        let orphan = folder(&mut state, "corporativo", Some(gone));
        let one = doc(&mut state, "a3f1-0001", Some(orphan));

        assert!(
            state.under(None).iter().any(|f| f.id == orphan),
            "nobody could reach it"
        );
        assert_eq!(state.docs[&one].folder, Some(orphan), "it was torn out");
        assert_eq!(state.inside(orphan).len(), 1);
    }

    #[test]
    fn a_document_filed_before_its_folder_arrives_stays_inside_it() {
        let mut state = State::default();
        let late = Ulid::generate();
        let one = doc(&mut state, "a3f1-0001", Some(late));

        assert_eq!(state.unfiled().len(), 1, "it hides until the folder lands");

        state.apply(&ev(
            2,
            "b",
            Op::FolderAdd {
                id: late,
                d: crate::event::FolderAdd {
                    name: "trabajo".into(),
                    order: "a0".into(),
                    parent: None,
                    icon: None,
                },
            },
        ));

        assert_eq!(state.docs[&one].folder, Some(late), "the filing was erased");
        assert_eq!(state.inside(late).len(), 1);
        assert!(state.unfiled().is_empty());
    }

    #[test]
    fn a_folder_born_before_its_parent_arrives_ends_up_inside_it() {
        let mut state = State::default();
        let late = Ulid::generate();
        let child = folder(&mut state, "corporativo", Some(late));

        assert!(state.under(None).iter().any(|f| f.id == child));

        state.apply(&ev(
            2,
            "b",
            Op::FolderAdd {
                id: late,
                d: crate::event::FolderAdd {
                    name: "trabajo".into(),
                    order: "a0".into(),
                    parent: None,
                    icon: None,
                },
            },
        ));

        assert_eq!(state.under(Some(late)).len(), 1, "the nesting was erased");
        assert!(!state.under(None).iter().any(|f| f.id == child));
    }

    #[test]
    fn a_folder_moved_into_one_that_is_gone_stays_where_it_was() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let inside = folder(&mut state, "corporativo", Some(work));

        state.apply(&ev(
            2,
            "a",
            Op::FolderMove {
                id: inside,
                d: crate::event::Filed {
                    folder: Some(Some(Ulid::generate())),
                },
            },
        ));

        assert_eq!(state.folders[&inside].parent, Some(work));
    }

    #[test]
    fn the_ceiling_is_measured_from_the_root() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let inside = folder(&mut state, "corporativo", Some(work));

        assert_eq!(state.depth(Some(work)), 1);
        assert_eq!(state.depth(Some(inside)), 2);
    }

    #[test]
    fn a_folder_cannot_be_moved_where_it_would_push_a_child_too_deep() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let other = folder(&mut state, "personal", None);
        let inside = folder(&mut state, "corporativo", Some(work));

        state.apply(&ev(
            2,
            "a",
            Op::FolderMove {
                id: work,
                d: crate::event::Filed {
                    folder: Some(Some(other)),
                },
            },
        ));

        assert_eq!(
            state.folders[&work].parent, None,
            "its child fell to a third level"
        );
        assert_eq!(state.folders[&inside].parent, Some(work));
    }

    #[test]
    fn a_childless_folder_can_still_be_moved_one_level_down() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let alone = folder(&mut state, "personal", None);

        state.apply(&ev(
            2,
            "a",
            Op::FolderMove {
                id: alone,
                d: crate::event::Filed {
                    folder: Some(Some(work)),
                },
            },
        ));

        assert_eq!(state.folders[&alone].parent, Some(work));
    }

    #[test]
    fn a_folder_cannot_be_moved_into_its_own_descendant() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let inside = folder(&mut state, "corporativo", Some(work));

        state.apply(&ev(
            2,
            "a",
            Op::FolderMove {
                id: work,
                d: crate::event::Filed {
                    folder: Some(Some(inside)),
                },
            },
        ));

        assert_eq!(state.folders[&work].parent, None, "it swallowed itself");
        assert_eq!(state.folders[&inside].parent, Some(work));
    }

    #[test]
    fn what_a_folder_holds_counts_what_hangs_below_it() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let inside = folder(&mut state, "corporativo", Some(work));
        doc(&mut state, "a3f1-0001", Some(work));
        doc(&mut state, "a3f1-0002", Some(inside));
        doc(&mut state, "a3f1-0003", None);

        assert_eq!(state.held_by(work), 2, "the tree shows what hangs below");
        assert_eq!(state.held_by(inside), 1);
        assert_eq!(state.unfiled().len(), 1);
    }

    #[test]
    fn a_folder_cannot_be_moved_into_itself() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);

        state.apply(&ev(
            2,
            "a",
            Op::FolderMove {
                id: work,
                d: crate::event::Filed {
                    folder: Some(Some(work)),
                },
            },
        ));

        assert_eq!(state.folders[&work].parent, None);
    }
}
