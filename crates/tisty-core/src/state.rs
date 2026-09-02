use std::collections::{BTreeMap, BTreeSet};

use ulid::Ulid;

use crate::{
    event::{DeviceId, Event, LogAdd, LogEdit, Op, StepAdd, TaskAdd, TaskMove, TaskPatch},
    model::{
        DocId, Folder, FolderId, Kept, List, ListId, LogEntry, Priority, Status, Step, StepId, Tag,
        Task, TaskId,
    },
    order,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    #[default]
    Whole,
    Summary,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    pub tasks: BTreeMap<TaskId, Task>,
    pub lists: BTreeMap<ListId, List>,
    pub folders: BTreeMap<FolderId, Folder>,
    pub docs: BTreeMap<DocId, Kept>,
    pub devices: BTreeSet<DeviceId>,
    pub agents: BTreeSet<DeviceId>,
    pub assistants: BTreeSet<DeviceId>,
    pub sourced: BTreeMap<String, TaskId>,
    pub dropped: BTreeSet<DeviceId>,
    pub retired: BTreeSet<String>,
    pub shed: BTreeSet<String>,
    pub forebears: BTreeSet<String>,
    pub(crate) fill: Fill,
    tombstones: BTreeSet<Ulid>,
}

pub const WORDS_AT_MOST: usize = 64 * 1024;

pub fn short_enough(text: &str) -> crate::Result<()> {
    if text.len() > WORDS_AT_MOST {
        return Err(crate::Error::TextTooLong {
            bytes: text.len() as u64,
            limit: WORDS_AT_MOST as u64,
        });
    }
    Ok(())
}

impl State {
    pub fn replay(events: &[Event]) -> Self {
        let mut state = Self::default();
        for event in events {
            state.apply(event);
        }
        state
    }

    pub fn shut(&self, id: DocId) -> bool {
        self.docs.get(&id).is_some_and(|one| {
            one.locked
                || one
                    .page_of
                    .is_some_and(|up| self.docs.get(&up).is_some_and(|doc| doc.locked))
        })
    }

    pub fn bolted(&self, file: &str) -> bool {
        self.docs
            .values()
            .any(|one| one.file == file && self.shut(one.id))
    }

    fn bolt(&mut self, id: DocId, shut: bool) {
        if self.docs.get(&id).is_some_and(|one| one.page_of.is_some()) {
            return;
        }
        if let Some(doc) = self.docs.get_mut(&id) {
            doc.locked = shut;
        }
    }

    fn shelve(&mut self, id: DocId, away: bool) {
        if self.docs.get(&id).is_some_and(|one| one.page_of.is_some()) {
            return;
        }
        let pages: Vec<DocId> = self
            .docs
            .values()
            .filter(|one| one.page_of == Some(id))
            .map(|one| one.id)
            .collect();
        for one in pages.into_iter().chain(std::iter::once(id)) {
            if let Some(doc) = self.docs.get_mut(&one) {
                doc.archived = away;
            }
        }
    }

    pub fn apply(&mut self, event: &Event) {
        if event
            .entity_id()
            .is_some_and(|id| self.tombstones.contains(&id))
        {
            return;
        }
        if event.op.destroys() && self.assistants.contains(&event.device) {
            return;
        }

        match &event.op {
            Op::TaskAdd { id, d } => {
                let mut task = task_from(*id, d);
                task.created_by = Some(event.device.clone());
                task.retally();
                if let Some(source) = &task.source {
                    self.sourced.insert(source.clone(), *id);
                }
                self.tasks.insert(*id, task);
            }
            Op::TaskUpdate { id, d } => self.with_task(*id, |t| patch(t, d)),
            Op::TaskDone { id, filled } => {
                let zone = event.zone.clone();
                self.with_task(*id, |t| {
                    t.status = Status::Done;
                    t.completed_at = Some(event.timestamp);
                    t.filled = *filled;
                    t.closed_in = zone;
                })
            }
            Op::TaskReopen { id } => self.with_task(*id, |t| {
                t.status = Status::Open;
                t.filled = false;
                t.closed_in = None;
                t.completed_at = None;
                t.hidden = false;
            }),
            Op::TaskHide { id } => self.with_task(*id, |t| t.hidden = true),
            Op::TaskShow { id } => self.with_task(*id, |t| t.hidden = false),
            Op::TaskDrop { id } => self.with_task(*id, |t| {
                t.status = Status::Dropped;
                t.completed_at = Some(event.timestamp);
            }),
            Op::TaskDelete { id } => {
                if let Some(source) = self.tasks.remove(id).and_then(|task| task.source) {
                    self.sourced.remove(&source);
                }
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
                let mut list = List::new(*id, crate::text::plainly(&d.name), d.order.clone());
                list.color = d
                    .color
                    .clone()
                    .filter(|key| crate::model::hue::kept(key).is_some());
                self.lists.insert(*id, list);
            }
            Op::ListRename { id, d } => {
                if let Some(list) = self.lists.get_mut(id) {
                    list.name = crate::text::plainly(&d.name);
                }
            }
            Op::FolderAdd { id, d } => {
                self.folders.insert(
                    *id,
                    Folder {
                        id: *id,
                        name: crate::text::plainly(&d.name),
                        order: d.order.clone(),
                        parent: d.parent.filter(|at| at != id),
                        icon: d.icon.clone().filter(|key| crate::model::icon::known(key)),
                        color: d
                            .color
                            .clone()
                            .filter(|key| crate::model::hue::kept(key).is_some()),
                    },
                );
            }
            Op::FolderRename { id, d } => {
                if let Some(folder) = self.folders.get_mut(id) {
                    folder.name = crate::text::plainly(&d.name);
                }
            }
            Op::FolderLook { id, d } => {
                if let Some(folder) = self.folders.get_mut(id) {
                    if let Some(icon) = &d.icon {
                        folder.icon = icon.clone().filter(|key| crate::model::icon::known(key));
                    }
                    if let Some(color) = &d.color {
                        folder.color = color
                            .clone()
                            .filter(|key| crate::model::hue::kept(key).is_some());
                    }
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
                // A page of a page is not a thing; the deeper one is kept as a document.
                let page_of = d
                    .page_of
                    .filter(|up| self.docs.get(up).is_some_and(|one| one.page_of.is_none()));
                let under = page_of.and_then(|up| self.docs.get(&up));
                self.docs.insert(
                    *id,
                    Kept {
                        id: *id,
                        file: d.file.clone(),
                        order: d.order.clone(),
                        folder: match under {
                            Some(one) => one.folder,
                            None => d.folder,
                        },
                        page_of,
                        archived: under.is_some_and(|one| one.archived),
                        locked: false,
                    },
                );
            }
            Op::DocMove { id, d } => {
                if let Some(page_of) = d.page_of {
                    let holds_pages = self.docs.values().any(|one| one.page_of == Some(*id));
                    let allowed = page_of.filter(|up| {
                        up != id
                            && !holds_pages
                            && self.docs.get(up).is_some_and(|one| one.page_of.is_none())
                    });
                    // Refusing to hang it somewhere is not a reason to unhang it from where it is.
                    if allowed.is_some() || page_of.is_none() {
                        let under = allowed
                            .and_then(|up| self.docs.get(&up))
                            .map(|one| (one.folder, one.archived));
                        let beside = allowed.map(|up| {
                            crate::order::last_of(
                                self.docs
                                    .values()
                                    .filter(|one| one.page_of == Some(up))
                                    .map(|one| one.order.as_str()),
                            )
                        });
                        if let Some(doc) = self.docs.get_mut(id) {
                            doc.page_of = allowed;
                            if let Some((folder, archived)) = under {
                                doc.folder = folder;
                                doc.archived = archived;
                                doc.order = beside.unwrap_or_else(|| doc.order.clone());
                            }
                        }
                    }
                }
                if let Some(order) = d.order.clone()
                    && let Some(doc) = self.docs.get_mut(id)
                {
                    doc.order = order;
                }
                if let Some(folder) = d.folder
                    && let Some(doc) = self.docs.get_mut(id)
                    && doc.page_of.is_none()
                {
                    doc.folder = folder;
                }
                // Pages live where their document lives, and follow it without being told.
                if let Some(under) = self.docs.get(id).filter(|one| one.page_of.is_none()) {
                    let (parent, folder) = (under.id, under.folder);
                    for one in self.docs.values_mut() {
                        if one.page_of == Some(parent) {
                            one.folder = folder;
                        }
                    }
                }
            }
            Op::DocDelete { id } => {
                // A page is part of its document, so it goes where the document goes.
                let pages: Vec<DocId> = self
                    .docs
                    .values()
                    .filter(|one| one.page_of == Some(*id))
                    .map(|one| one.id)
                    .collect();
                for one in pages.into_iter().chain(std::iter::once(*id)) {
                    if let Some(gone) = self.docs.remove(&one) {
                        self.shed.insert(gone.file);
                    }
                    self.tombstones.insert(one);
                }
            }
            Op::DocArchive { id } => self.shelve(*id, true),
            Op::DocLock { id } => self.bolt(*id, true),
            Op::DocUnlock { id } => self.bolt(*id, false),
            Op::DocUnarchive { id } => self.shelve(*id, false),
            Op::DeviceJoin { d, k } => {
                self.dropped.remove(d);
                self.devices.insert(d.clone());
                match k.filter(|_| d == &event.device) {
                    Some(crate::event::DeviceKind::Agent) => {
                        self.agents.insert(d.clone());
                        self.assistants.insert(d.clone());
                    }
                    Some(crate::event::DeviceKind::Machine) => {
                        self.agents.remove(d);
                    }
                    None => {}
                }
            }
            Op::DeviceRemove { d } => {
                self.devices.remove(d);
                self.agents.remove(d);
                self.dropped.insert(d.clone());
            }
            Op::AttachRetire { d } => {
                if crate::attach::names_an_attachment(d) {
                    self.retired.insert(d.clone());
                }
            }
            Op::StoresJoined { d } => {
                self.forebears.insert(d.absorbed.clone());
                self.forebears.insert(d.survivor.clone());
            }
            Op::ListLook { id, d } => {
                if let Some(list) = self.lists.get_mut(id) {
                    if let Some(icon) = &d.icon {
                        list.icon = icon.clone().filter(|key| crate::model::icon::known(key));
                    }
                    if let Some(color) = &d.color {
                        list.color = color
                            .clone()
                            .filter(|key| crate::model::hue::kept(key).is_some());
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

    pub fn has_bodies(&self) -> bool {
        self.fill == Fill::Whole
    }

    fn with_task(&mut self, id: TaskId, f: impl FnOnce(&mut Task)) {
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

    pub fn is_erased(&self, id: Ulid) -> bool {
        self.tombstones.contains(&id)
    }

    pub fn erased(&self) -> impl Iterator<Item = &Ulid> {
        self.tombstones.iter()
    }

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
            .map(
                |op| match op.about_whom().and_then(|whom| born.get(&whom).copied()) {
                    Some(fresh) => op.about(fresh),
                    None => op,
                },
            )
            .collect()
    }

    pub fn unfiled(&self) -> Vec<&Kept> {
        self.docs
            .values()
            .filter(|one| !one.archived && one.page_of.is_none())
            .filter(|one| one.folder.is_none_or(|at| !self.folders.contains_key(&at)))
            .collect()
    }

    /// Pages are counted with their document, not beside it.
    pub fn inside(&self, folder: FolderId) -> Vec<&Kept> {
        self.docs
            .values()
            .filter(|one| !one.archived && one.page_of.is_none() && one.folder == Some(folder))
            .collect()
    }

    pub fn pages_of(&self, doc: DocId) -> Vec<&Kept> {
        let mut pages: Vec<&Kept> = self
            .docs
            .values()
            .filter(|one| one.page_of == Some(doc))
            .collect();
        pages.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));
        pages
    }

    pub fn books_among(&self, files: &[String]) -> Vec<String> {
        let came: BTreeSet<&str> = files.iter().map(String::as_str).collect();
        let mut held: BTreeMap<DocId, usize> = BTreeMap::new();
        for one in self.docs.values() {
            if let Some(up) = one.page_of {
                *held.entry(up).or_default() += 1;
            }
        }
        self.docs
            .values()
            .filter(|one| {
                one.page_of.is_none()
                    && came.contains(one.file.as_str())
                    && held.get(&one.id).is_some_and(|many| *many > 1)
            })
            .map(|one| one.file.clone())
            .collect()
    }

    pub fn settling(&self, file: &str, body: &str) -> Vec<Op> {
        let Some(kept) = self.docs.values().find(|one| one.file == file) else {
            return Vec::new();
        };
        self.pages_told(kept.id, body)
            .into_iter()
            .map(|(id, order)| Op::DocMove {
                id,
                d: crate::event::Filed {
                    folder: None,
                    page_of: None,
                    order: Some(order),
                },
            })
            .collect()
    }

    pub fn pages_told(&self, doc: DocId, body: &str) -> Vec<(DocId, String)> {
        let pages = self.pages_of(doc);
        if pages.len() < 2 {
            return Vec::new();
        }
        let named = crate::refs::papers(body);
        let mut takes = BTreeSet::new();
        let wanted: Vec<&Kept> = named
            .iter()
            .filter_map(|file| pages.iter().find(|one| &one.file == file).copied())
            .filter(|one| takes.insert(one.id))
            .collect();
        if wanted.len() < 2 {
            return Vec::new();
        }

        let mut told = wanted.into_iter();
        let run: Vec<&Kept> = pages
            .iter()
            .map(|one| match takes.contains(&one.id) {
                true => told.next().unwrap_or(one),
                false => one,
            })
            .collect();

        let keys: Vec<&str> = run.iter().map(|one| one.order.as_str()).collect();
        crate::order::resequenced(&keys)
            .into_iter()
            .zip(&run)
            .filter_map(|(fresh, one)| fresh.map(|key| (one.id, key)))
            .collect()
    }

    pub fn put_away(&self) -> Vec<&Kept> {
        self.docs.values().filter(|one| one.archived).collect()
    }

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

    pub fn mark_erased(&mut self, id: Ulid) {
        self.tombstones.insert(id);
    }

    fn ordered_keys(&self, list: Option<ListId>) -> Vec<&str> {
        self.tasks
            .values()
            .filter(|task| task.list == list)
            .map(|task| task.order.as_str())
            .collect()
    }

    pub fn completing(&self, id: TaskId, now: jiff::Zoned) -> Vec<Op> {
        let done = vec![Op::TaskDone { id, filled: false }];
        let Some(task) = self.tasks.get(&id) else {
            return done;
        };
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
        if repeat.ended(next.at.date()) {
            return done;
        }

        let mut ops = done;
        let order = self.order_last_in(task.list);
        ops.extend(self.turn_after(task, id, next, repeat, false, &order));
        ops
    }

    /// The turn that follows one being closed: same shape, dates carried along by how far it moved.
    /// A bare one is a record of a day that passed, so it carries no work to do and nothing to ring.
    fn turn_after(
        &self,
        task: &Task,
        after: TaskId,
        next: crate::model::DateSpec,
        repeat: crate::model::Repeat,
        bare: bool,
        order: &str,
    ) -> Vec<Op> {
        let along = task
            .date
            .as_ref()
            .map(|was| next.at.date().since(was.at.date()));

        let mut fresh = crate::event::TaskAdd::new(task.title.clone(), order.to_string());
        if !bare {
            fresh.deadline = shifted(task.deadline.as_ref(), along.as_ref());
            fresh.reminders = task
                .reminders
                .iter()
                .filter_map(|one| match (&task.date, repeat.cadence().after(one.at)) {
                    (None, Some(at)) => Some(one.moved(at)),
                    _ => shifted(Some(one), along.as_ref()),
                })
                .collect();
        }
        fresh.date = Some(next);
        fresh.priority = Some(match task.priority {
            Priority::Minor => Priority::Unset,
            kept => kept,
        });
        fresh.list = task.list;
        fresh.tags = task.tags.clone();
        fresh.repeat = Some(repeat);
        fresh.after = Some(after);

        let born = ulid::Ulid::generate();
        let mut ops = vec![Op::TaskAdd { id: born, d: fresh }];
        if bare {
            return ops;
        }

        if let Some(body) = &task.description {
            ops.push(Op::TaskDescribe {
                id: born,
                d: crate::event::Body {
                    body: Some(body.clone()),
                },
            });
        }
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

    /// The dates the cadence would skip between this turn and today, capped at what a person recalls.
    pub fn owed_since(&self, id: TaskId, today: jiff::civil::Date) -> Vec<jiff::civil::Date> {
        const RECALLS: usize = 5;
        /// Five turns of a weekly cadence reach back five weeks, which is no longer memory.
        const REACHES: i32 = 30;
        let Some(task) = self.tasks.get(&id) else {
            return Vec::new();
        };
        let (Some(due), Some(repeat)) = (task.date.as_ref(), task.repeat) else {
            return Vec::new();
        };
        if repeat.from != crate::model::From::Due || repeat.cadence().every == 0 {
            return Vec::new();
        }

        let mut at = due.at;
        let mut all = Vec::new();
        while all.len() <= RECALLS {
            let Some(next) = repeat.cadence().after(at) else {
                break;
            };
            if next.date() > today || repeat.ended(next.date()) {
                break;
            }
            all.push(next.date());
            at = next;
        }
        let dark = today
            .since(due.at.date())
            .is_ok_and(|gone| gone.get_days() > REACHES);
        if all.len() > RECALLS || dark {
            Vec::new()
        } else {
            all
        }
    }

    /// Fills the dates the person says they did, as turns already closed, so no gap is invented.
    pub fn covering(&self, id: TaskId, now: jiff::Zoned, also: &[jiff::civil::Date]) -> Vec<Op> {
        let Some(task) = self.tasks.get(&id) else {
            return vec![Op::TaskDone { id, filled: false }];
        };
        let (Some(due), Some(repeat)) = (task.date.clone(), task.repeat) else {
            return self.completing(id, now);
        };

        // A claimed date is only honoured if it was offered: anything else would write a turn the
        // cadence never had, and a mistyped year would drag the whole series into next January.
        let offered = self.owed_since(id, now.date());
        let mut wanted: Vec<jiff::civil::Date> = also
            .iter()
            .filter(|day| offered.contains(day))
            .copied()
            .collect();
        wanted.sort_unstable();
        wanted.dedup();

        let mut ops = vec![Op::TaskDone { id, filled: false }];
        let mut before = id;
        let mut last = due.clone();
        let mut order = self.order_last_in(task.list);
        for day in wanted {
            if day <= last.date() {
                continue;
            }
            let filled = due.moved(day.to_datetime(due.at.time()));
            let mut born = self.turn_after(task, before, filled.clone(), repeat, true, &order);
            order = order::after(&order);
            let Some(Op::TaskAdd { id: fresh, .. }) = born.first() else {
                break;
            };
            let fresh = *fresh;
            ops.append(&mut born);
            ops.push(Op::TaskDone {
                id: fresh,
                filled: true,
            });
            last = filled;
            before = fresh;
        }

        if before == id {
            return self.completing(id, now);
        }

        let Some(next) = repeat.next(
            Some(&last),
            now.datetime(),
            now.datetime()
                .date()
                .to_datetime(jiff::civil::Time::midnight()),
            now.time_zone().iana_name().unwrap_or("UTC"),
        ) else {
            return ops;
        };
        if repeat.ended(next.at.date()) {
            return ops;
        }

        ops.extend(self.turn_after(task, before, next, repeat, false, &order));
        ops
    }

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

    pub fn order_between(&self, after: Option<TaskId>, before: Option<TaskId>) -> String {
        let key = |id: Option<TaskId>| {
            id.and_then(|id| self.tasks.get(&id))
                .map(|task| task.order.clone())
        };
        let (a, b) = (key(after), key(before));
        match (a, b) {
            (Some(a), Some(b)) if a >= b => order::after(&a),
            (a, b) => order::between(a.as_deref(), b.as_deref()),
        }
    }

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

    pub fn search(&self, query: &str, scope: crate::view::Scope) -> Vec<&Task> {
        self.searching(query, scope, usize::MAX).0
    }

    pub fn searching(
        &self,
        query: &str,
        scope: crate::view::Scope,
        most: usize,
    ) -> (Vec<&Task>, usize) {
        use crate::view::Scope;

        let terms = crate::text::terms(query);
        if terms.is_empty() {
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
            .filter_map(|t| crate::view::matches_query(t, &terms).map(|hit| (hit, t)))
            .collect();

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

    pub fn list_called(&self, name: &str) -> Vec<&List> {
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

    pub fn tags(&self) -> BTreeSet<&Tag> {
        self.tasks.values().flat_map(|t| &t.tags).collect()
    }
}

fn loose(name: &str) -> String {
    name.to_lowercase().replace([' ', '_'], "-")
}

fn untouched(born: &Task) -> bool {
    born.log.is_empty() && !born.steps.iter().any(|step| step.done)
}

fn shifted(
    spec: Option<&crate::DateSpec>,
    along: Option<&Result<jiff::Span, jiff::Error>>,
) -> Option<crate::DateSpec> {
    let spec = spec?;
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
        source: d.source.clone(),
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

    fn seam(was: &str, now: &str) -> crate::Op {
        crate::Op::StoresJoined {
            d: crate::event::Stitch {
                absorbed: was.into(),
                survivor: now.into(),
                ours: [crate::event::DeviceId("dev_mine".into())].into(),
                theirs: [crate::event::DeviceId("dev_yours".into())].into(),
            },
        }
    }

    fn told(ops: Vec<crate::Op>) -> State {
        let when: jiff::Timestamp = "2026-08-15T00:00:00Z".parse().unwrap();
        let events: Vec<crate::event::Event> = ops
            .into_iter()
            .enumerate()
            .map(|(at, op)| crate::event::Event {
                version: crate::event::SCHEMA_VERSION,
                timestamp: when,
                device: crate::event::DeviceId("dev_mine".into()),
                batch: None,
                undo: false,
                redo: false,
                seq: at as u64,
                op,
                optional: false,
                zone: None,
            })
            .collect();
        State::replay(&events)
    }

    #[test]
    fn a_seam_leaves_both_names_of_the_history_behind_it() {
        let said = told(vec![seam("01OLD", "01NEW")]);

        assert!(said.forebears.contains("01OLD"));
        assert!(said.forebears.contains("01NEW"));
    }

    #[test]
    fn seams_pile_up_so_a_machine_left_far_behind_still_finds_itself() {
        let said = told(vec![
            seam("01FIRST", "01SECOND"),
            seam("01SECOND", "01THIRD"),
        ]);

        for one in ["01FIRST", "01SECOND", "01THIRD"] {
            assert!(said.forebears.contains(one), "{one} no quedo en el linaje");
        }
    }

    #[test]
    fn a_history_that_was_never_joined_carries_no_forebears() {
        assert!(told(Vec::new()).forebears.is_empty());
    }

    #[test]
    fn a_seam_touches_nothing_that_belongs_to_the_data() {
        let said = told(vec![seam("01OLD", "01NEW")]);

        assert!(said.tasks.is_empty());
        assert!(said.docs.is_empty());
        assert!(said.lists.is_empty());
        assert!(said.devices.is_empty());
        assert!(said.retired.is_empty());
    }

    #[test]
    fn a_description_the_size_of_a_book_is_refused_with_the_limit_named() {
        let refused = short_enough(&"y".repeat(WORDS_AT_MOST + 1));

        assert!(matches!(
            refused,
            Err(crate::Error::TextTooLong { limit, .. }) if limit == WORDS_AT_MOST as u64
        ));
    }

    #[test]
    fn a_description_right_at_the_limit_still_goes_in() {
        assert!(short_enough(&"y".repeat(WORDS_AT_MOST)).is_ok());
    }

    #[test]
    fn the_limit_counts_bytes_so_accents_never_smuggle_past_it() {
        let accented = "á".repeat(WORDS_AT_MOST);

        assert!(accented.chars().count() == WORDS_AT_MOST);
        assert!(
            short_enough(&accented).is_err(),
            "counted characters, not bytes"
        );
    }

    use crate::model::{Cadence, Repeat, Unit};

    fn a_daily_due(on: &str) -> (Vec<Event>, TaskId) {
        let id = Ulid::generate();
        let mut add = TaskAdd::new("take the pill", "a0");
        add.date = Some(DateSpec::all_day(on.parse().unwrap(), "UTC"));
        add.repeat = Some(crate::model::Repeat::due(Cadence {
            every: 1,
            unit: Unit::Day,
        }));
        (
            vec![Event::new(
                DeviceId("dev_a".into()),
                jiff::Timestamp::from_second(1_770_000_000).unwrap(),
                Op::TaskAdd { id, d: add },
            )],
            id,
        )
    }

    fn now_on(day: &str) -> jiff::Zoned {
        format!("{day}T09:00:00Z")
            .parse::<jiff::Timestamp>()
            .unwrap()
            .to_zoned(jiff::tz::TimeZone::UTC)
    }

    #[test]
    fn nothing_is_written_for_a_date_the_person_did_not_claim() {
        let (log, id) = a_daily_due("2026-08-23");

        let ops = State::replay(&log).covering(id, now_on("2026-08-26"), &[]);

        let born = ops
            .iter()
            .filter(|op| matches!(op, Op::TaskAdd { .. }))
            .count();
        assert_eq!(
            born, 1,
            "only the next turn is born; a date nobody claimed needs no row"
        );
    }

    #[test]
    fn a_stretch_too_long_to_recall_is_never_offered() {
        let (log, id) = a_daily_due("2026-07-15");
        let state = State::replay(&log);

        assert!(
            state
                .owed_since(id, "2026-08-26".parse().unwrap())
                .is_empty(),
            "forty days are not recalled one by one, so they are not asked about"
        );
        assert_eq!(state.owed_since(id, "2026-08-26".parse().unwrap()).len(), 0);
    }

    #[test]
    fn a_short_stretch_offers_one_date_per_turn_of_the_cadence() {
        let (log, id) = a_daily_due("2026-08-23");
        let state = State::replay(&log);

        let owed = state.owed_since(id, "2026-08-26".parse().unwrap());

        assert_eq!(
            owed,
            vec![
                "2026-08-24".parse::<jiff::civil::Date>().unwrap(),
                "2026-08-25".parse().unwrap(),
                "2026-08-26".parse().unwrap(),
            ]
        );
    }

    #[test]
    fn five_turns_of_a_slow_cadence_still_reach_too_far_back_to_be_offered() {
        let id = Ulid::generate();
        let mut add = TaskAdd::new("water the plants", "a0");
        add.date = Some(DateSpec::all_day("2026-07-07".parse().unwrap(), "UTC"));
        add.repeat = Some(crate::model::Repeat::due(Cadence {
            every: 1,
            unit: Unit::Week,
        }));
        let state = State::replay(&[Event::new(
            DeviceId("dev_a".into()),
            jiff::Timestamp::from_second(1_770_000_000).unwrap(),
            Op::TaskAdd { id, d: add },
        )]);

        let owed = state.owed_since(id, "2026-08-11".parse().unwrap());

        assert!(
            owed.is_empty(),
            "five weeks is not memory, however few the turns: {owed:?}"
        );
    }

    #[test]
    fn a_date_that_was_never_offered_is_refused_rather_than_written() {
        let (log, id) = a_daily_due("2026-08-23");
        let state = State::replay(&log);

        let ops = state.covering(
            id,
            now_on("2026-08-25"),
            &["2027-01-15".parse().unwrap(), "2026-08-30".parse().unwrap()],
        );

        assert_eq!(
            ops.iter()
                .filter(|op| matches!(op, Op::TaskAdd { .. }))
                .count(),
            1,
            "a mistyped year wrote turns the cadence never had"
        );
    }

    #[test]
    fn a_filled_turn_carries_no_reminder_of_its_own() {
        let id = Ulid::generate();
        let mut add = TaskAdd::new("take the pill", "a0");
        add.date = Some(DateSpec::all_day("2026-08-23".parse().unwrap(), "UTC"));
        add.repeat = Some(crate::model::Repeat::due(Cadence {
            every: 1,
            unit: Unit::Day,
        }));
        add.reminders = vec![DateSpec::floating(
            jiff::civil::date(2026, 8, 23).at(9, 0, 0, 0),
            "UTC",
        )];
        let log = vec![Event::new(
            DeviceId("dev_a".into()),
            jiff::Timestamp::from_second(1_770_000_000).unwrap(),
            Op::TaskAdd { id, d: add },
        )];
        let state = State::replay(&log);

        let ops = state.covering(id, now_on("2026-08-25"), &["2026-08-24".parse().unwrap()]);

        let added: Vec<_> = ops
            .iter()
            .filter_map(|op| match op {
                Op::TaskAdd { d, .. } => Some(d),
                _ => None,
            })
            .collect();
        assert!(
            added.first().is_some_and(|d| d.reminders.is_empty()),
            "the filled turn would ring for a day already gone"
        );
        assert!(
            added.last().is_some_and(|d| !d.reminders.is_empty()),
            "the live turn lost the reminder the routine had"
        );
    }

    #[test]
    fn a_series_that_already_ended_is_offered_no_dates_to_fill() {
        let id = Ulid::generate();
        let mut add = TaskAdd::new("water the plants", "a0");
        add.date = Some(DateSpec::all_day("2026-08-20".parse().unwrap(), "UTC"));
        add.repeat = Some(crate::model::Repeat {
            from: crate::model::From::Due,
            each: Cadence {
                every: 1,
                unit: Unit::Day,
            },
            until: Some("2026-08-21".parse().unwrap()),
        });
        let state = State::replay(&[Event::new(
            DeviceId("dev_a".into()),
            jiff::Timestamp::from_second(1_770_000_000).unwrap(),
            Op::TaskAdd { id, d: add },
        )]);

        let owed = state.owed_since(id, "2026-08-25".parse().unwrap());

        assert_eq!(
            owed,
            vec!["2026-08-21".parse::<jiff::civil::Date>().unwrap()],
            "it offered dates past the end of the series: {owed:?}"
        );
    }

    #[test]
    fn a_claimed_date_becomes_a_turn_that_was_already_closed() {
        let (mut log, id) = a_daily_due("2026-08-23");

        let state = State::replay(&log);
        let owed = state.owed_since(id, "2026-08-25".parse().unwrap());
        let ops = state.covering(id, now_on("2026-08-25"), &owed);
        for (at, op) in ops.into_iter().enumerate() {
            let mut one = Event::new(
                DeviceId("dev_a".into()),
                jiff::Timestamp::from_second(1_770_100_000).unwrap(),
                op,
            );
            one.seq = at as u64;
            log.push(one);
        }
        let after = State::replay(&log);

        let told = crate::series::series(&after, id).unwrap();
        assert_eq!(told.kept, 3, "the claimed days count as done, not as gaps");
        assert_eq!(
            told.skipped, 0,
            "and the cadence leaves no hole behind them"
        );
        assert!(
            !told.turns[1].told && !told.turns[2].told,
            "a filled turn carries no substance of its own"
        );
    }

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

    #[test]
    fn the_next_turn_of_a_repeat_is_not_born_written_off() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new("revisar el buzón", "a0");
        add.date = Some(DateSpec::floating(
            jiff::civil::date(2026, 8, 4).at(9, 0, 0, 0),
            "Europe/Madrid",
        ));
        add.repeat = Some(Repeat::due(Cadence {
            every: 1,
            unit: Unit::Week,
        }));
        add.priority = Some(Priority::Minor);
        state.apply(&ev(1, "a", Op::TaskAdd { id, d: add }));

        let now = jiff::civil::date(2026, 8, 4)
            .at(21, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();
        let ops = state.completing(id, now);

        let Op::TaskAdd { d, .. } = &ops[1] else {
            panic!("the next one was not emitted");
        };
        assert_eq!(d.priority, Some(Priority::Unset));
    }

    #[test]
    fn a_repeat_carries_the_quadrant_it_was_placed_in() {
        let mut state = State::default();
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new("pagar el alquiler", "a0");
        add.date = Some(DateSpec::floating(
            jiff::civil::date(2026, 8, 4).at(9, 0, 0, 0),
            "Europe/Madrid",
        ));
        add.repeat = Some(Repeat::due(Cadence {
            every: 1,
            unit: Unit::Month,
        }));
        add.priority = Some(Priority::Do);
        state.apply(&ev(1, "a", Op::TaskAdd { id, d: add }));

        let now = jiff::civil::date(2026, 8, 4)
            .at(21, 0, 0, 0)
            .to_zoned(jiff::tz::TimeZone::UTC)
            .unwrap();
        let ops = state.completing(id, now);

        let Op::TaskAdd { d, .. } = &ops[1] else {
            panic!("the next one was not emitted");
        };
        assert_eq!(d.priority, Some(Priority::Do));
    }

    fn add_repeat() -> Option<Repeat> {
        Some(Repeat::due(Cadence {
            every: 1,
            unit: Unit::Week,
        }))
    }

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

    #[test]
    fn reopening_takes_back_a_successor_nobody_wrote_in() {
        let (mut state, id) = repeating();
        let born = successor(&state, id);

        for op in state.reopening(id) {
            state.apply(&ev(20, "a", op));
        }

        assert!(!state.tasks.contains_key(&born));
    }

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
        let fresh = again[0].about_whom().expect("a task op knows whose it is");
        assert_ne!(fresh, gone, "it reused the buried id");
        assert_eq!(
            again[1].about_whom(),
            Some(fresh),
            "the batch was torn apart"
        );
        for op in &again {
            state.apply(&ev(9, "a", op.clone()));
        }
        assert_eq!(
            state.tasks[&fresh].description.as_deref(),
            Some("agua tibia")
        );
    }

    #[test]
    fn nothing_is_renamed_when_nothing_was_buried() {
        let state = State::default();
        let id = ulid::Ulid::generate();
        let ops = vec![Op::TaskDone { id, filled: false }];

        assert_eq!(state.afresh(ops.clone()), ops);
    }

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

    #[test]
    fn a_task_remembers_which_writer_filed_it() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(
            1,
            "dev_agent",
            Op::TaskAdd {
                id,
                d: crate::event::TaskAdd::new("buy pink card stock", "a0"),
            },
        ));

        assert_eq!(
            state.tasks[&id].created_by,
            Some(DeviceId("dev_agent".into())),
            "the writer is in every event and must survive projection"
        );
    }

    #[test]
    fn what_a_task_was_written_from_finds_it_again() {
        let mut state = State::default();
        let id = Ulid::generate();
        let mut d = crate::event::TaskAdd::new("buy pink card stock", "a0");
        d.source = Some("wa:msg-991".into());
        state.apply(&ev(1, "dev_agent", Op::TaskAdd { id, d }));

        assert_eq!(state.sourced.get("wa:msg-991"), Some(&id));
        assert_eq!(state.sourced.get("wa:msg-992"), None);
    }

    #[test]
    fn a_join_says_whether_it_is_a_machine_or_an_agent() {
        let mut state = State::default();
        let agent = DeviceId("dev_agent".into());
        let laptop = DeviceId("dev_laptop".into());
        state.apply(&ev(
            1,
            "dev_laptop",
            Op::DeviceJoin {
                d: laptop.clone(),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ));
        state.apply(&ev(
            2,
            "dev_agent",
            Op::DeviceJoin {
                d: agent.clone(),
                k: Some(crate::event::DeviceKind::Agent),
            },
        ));

        assert!(state.devices.contains(&laptop) && state.devices.contains(&agent));
        assert!(state.agents.contains(&agent));
        assert!(!state.agents.contains(&laptop));

        state.apply(&ev(3, "dev_laptop", Op::DeviceRemove { d: agent.clone() }));
        assert!(
            !state.agents.contains(&agent),
            "removing takes the badge with it"
        );
        assert!(
            state.assistants.contains(&agent),
            "who assisted is remembered past the badge"
        );
    }

    fn joined(state: &mut State, who: &str, k: crate::event::DeviceKind) {
        state.apply(&ev(
            1,
            who,
            Op::DeviceJoin {
                d: DeviceId(who.into()),
                k: Some(k),
            },
        ));
    }

    fn a_doc(state: &mut State, who: &str) -> DocId {
        let id = Ulid::generate();
        state.apply(&ev(
            2,
            who,
            Op::DocAdd {
                id,
                d: crate::event::DocAdd {
                    file: "a note.md".into(),
                    order: order::first(),
                    folder: None,
                    page_of: None,
                },
            },
        ));
        id
    }

    #[test]
    fn a_deletion_written_by_an_assistant_is_not_honoured() {
        let mut state = State::default();
        joined(&mut state, "dev_laptop", crate::event::DeviceKind::Machine);
        joined(&mut state, "dev_agent", crate::event::DeviceKind::Agent);
        let doc = a_doc(&mut state, "dev_laptop");
        let list = Ulid::generate();
        state.apply(&ev(
            3,
            "dev_laptop",
            Op::ListAdd {
                id: list,
                d: crate::event::ListAdd {
                    name: "Home".into(),
                    order: order::first(),
                    color: None,
                },
            },
        ));

        state.apply(&ev(4, "dev_agent", Op::DocDelete { id: doc }));
        state.apply(&ev(5, "dev_agent", Op::ListDelete { id: list }));

        assert!(state.docs.contains_key(&doc));
        assert!(state.lists.contains_key(&list));
        assert!(state.shed.is_empty(), "nor does it shed the file");

        state.apply(&ev(6, "dev_laptop", Op::DocDelete { id: doc }));
        assert!(!state.docs.contains_key(&doc), "the person still deletes");
    }

    #[test]
    fn taking_the_badge_off_an_assistant_does_not_hand_it_a_deletion() {
        let mut state = State::default();
        joined(&mut state, "dev_laptop", crate::event::DeviceKind::Machine);
        joined(&mut state, "dev_agent", crate::event::DeviceKind::Agent);
        let doc = a_doc(&mut state, "dev_laptop");

        state.apply(&ev(
            3,
            "dev_laptop",
            Op::DeviceRemove {
                d: DeviceId("dev_agent".into()),
            },
        ));
        state.apply(&ev(4, "dev_agent", Op::DocDelete { id: doc }));

        assert!(state.docs.contains_key(&doc));
    }

    #[test]
    fn an_assistant_cannot_take_a_machine_off_the_list() {
        let mut state = State::default();
        joined(&mut state, "dev_laptop", crate::event::DeviceKind::Machine);
        joined(&mut state, "dev_agent", crate::event::DeviceKind::Agent);

        state.apply(&ev(
            3,
            "dev_agent",
            Op::DeviceRemove {
                d: DeviceId("dev_laptop".into()),
            },
        ));

        assert!(state.devices.contains(&DeviceId("dev_laptop".into())));
    }

    #[test]
    fn a_turn_closed_by_the_backfill_says_so() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(
            1,
            "mac0",
            Op::TaskAdd {
                id,
                d: crate::event::TaskAdd::new("take the pill", "a0"),
            },
        ));
        state.apply(&ev(2, "mac0", Op::TaskDone { id, filled: true }));

        assert!(
            state.tasks[&id].filled,
            "statistics read completed_at, and a backfilled turn did not close at that hour"
        );
    }

    #[test]
    fn deleting_a_task_takes_what_it_was_written_from_with_it() {
        let mut state = State::default();
        let id = Ulid::generate();
        let mut d = crate::event::TaskAdd::new("buy pink card stock", "a0");
        d.source = Some("wa:msg-991".into());
        state.apply(&ev(1, "dev_agent", Op::TaskAdd { id, d }));
        state.apply(&ev(2, "dev_agent", Op::TaskDelete { id }));

        assert_eq!(
            state.sourced.get("wa:msg-991"),
            None,
            "what someone deleted has to be capturable again"
        );
    }

    #[test]
    fn two_tasks_claiming_one_origin_leave_the_last_one_reachable() {
        let mut state = State::default();
        let (first, second) = (Ulid::generate(), Ulid::generate());
        for (id, title) in [(first, "first read of it"), (second, "second read of it")] {
            let mut d = crate::event::TaskAdd::new(title, "a0");
            d.source = Some("wa:msg-991".into());
            state.apply(&ev(1, "dev_agent", Op::TaskAdd { id, d }));
        }

        assert_eq!(
            state.sourced.get("wa:msg-991"),
            Some(&second),
            "the last writer wins, and replay order is the same on every machine"
        );
        assert_eq!(state.tasks.len(), 2, "neither task is thrown away for it");
    }

    #[test]
    fn coming_back_as_a_machine_takes_the_agent_badge_off() {
        let mut state = State::default();
        let who = DeviceId("dev_x".into());
        state.apply(&ev(
            1,
            "dev_x",
            Op::DeviceJoin {
                d: who.clone(),
                k: Some(crate::event::DeviceKind::Agent),
            },
        ));
        state.apply(&ev(
            2,
            "dev_x",
            Op::DeviceJoin {
                d: who.clone(),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ));

        assert!(!state.agents.contains(&who));
    }

    #[test]
    fn reopening_a_backfilled_turn_forgets_that_it_was_backfilled() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(
            1,
            "mac0",
            Op::TaskAdd {
                id,
                d: crate::event::TaskAdd::new("take the pill", "a0"),
            },
        ));
        state.apply(&ev(2, "mac0", Op::TaskDone { id, filled: true }));
        state.apply(&ev(3, "mac0", Op::TaskReopen { id }));

        assert!(
            !state.tasks[&id].filled,
            "an open turn was closed by nobody"
        );
    }

    #[test]
    fn undoing_a_reopen_puts_a_backfilled_turn_back_as_it_was() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(
            1,
            "mac0",
            Op::TaskAdd {
                id,
                d: crate::event::TaskAdd::new("take the pill", "a0"),
            },
        ));
        state.apply(&ev(2, "mac0", Op::TaskDone { id, filled: true }));

        let before = state.clone();
        let reopen = ev(3, "mac0", Op::TaskReopen { id });
        state.apply(&reopen);
        let back = crate::undo::inverse(&reopen, &before).expect("a reopen can be undone");
        for (n, op) in back.into_iter().enumerate() {
            state.apply(&ev(4 + n as i64, "mac0", op));
        }

        assert!(
            state.tasks[&id].filled,
            "undo has to be an identity, and a lost flag revives a delay nobody had"
        );
    }

    #[test]
    fn a_machine_that_joined_is_allowed_to_write() {
        let mut state = State::default();

        state.apply(&ev(
            1,
            "mac0",
            Op::DeviceJoin {
                d: DeviceId("mac0".into()),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ));

        assert!(state.devices.contains(&DeviceId("mac0".into())));
    }

    #[test]
    fn a_machine_that_was_removed_is_no_longer_allowed() {
        let mut state = State::default();
        let who = DeviceId("win1".into());

        state.apply(&ev(
            1,
            "mac0",
            Op::DeviceJoin {
                d: who.clone(),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ));
        state.apply(&ev(2, "mac0", Op::DeviceRemove { d: who.clone() }));

        assert!(!state.devices.contains(&who));
    }

    #[test]
    fn a_machine_that_comes_back_is_allowed_again_because_the_later_word_wins() {
        let mut state = State::default();
        let who = DeviceId("win1".into());

        state.apply(&ev(
            1,
            "mac0",
            Op::DeviceJoin {
                d: who.clone(),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ));
        state.apply(&ev(2, "mac0", Op::DeviceRemove { d: who.clone() }));
        state.apply(&ev(
            3,
            "win1",
            Op::DeviceJoin {
                d: who.clone(),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ));

        assert!(state.devices.contains(&who));
    }

    #[test]
    fn removing_one_machine_says_nothing_about_the_others() {
        let mut state = State::default();
        let one = DeviceId("mac0".into());
        let other = DeviceId("win1".into());

        state.apply(&ev(
            1,
            "mac0",
            Op::DeviceJoin {
                d: one.clone(),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ));
        state.apply(&ev(
            2,
            "mac0",
            Op::DeviceJoin {
                d: other.clone(),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ));
        state.apply(&ev(3, "mac0", Op::DeviceRemove { d: other }));

        assert_eq!(state.devices.len(), 1);
        assert!(state.devices.contains(&one));
    }

    #[test]
    fn a_word_about_a_machine_is_never_buried_by_a_tombstone() {
        let mut state = State::default();
        let one = Ulid::generate();
        state.apply(&ev(1, "mac0", add(one, "una tarea")));
        state.apply(&ev(2, "mac0", Op::TaskDelete { id: one }));

        state.apply(&ev(
            3,
            "mac0",
            Op::DeviceJoin {
                d: DeviceId("win1".into()),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ));

        assert!(state.devices.contains(&DeviceId("win1".into())));
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
            ev(2, "dev_a", Op::TaskDone { id, filled: false }),
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
                        priority: Some(Priority::Do),
                        ..Default::default()
                    },
                ),
            ),
        ]);

        let task = &state.tasks[&id];
        assert_eq!(task.date, Some(a_date()));
        assert_eq!(task.priority, Priority::Do);
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
        done.push(ev(3, "dev_a", Op::TaskDone { id, filled: false }));
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
            ev(2, "dev_a", Op::TaskDone { id, filled: false }),
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
            ev(
                2,
                "dev_a",
                Op::TaskDone {
                    id: a,
                    filled: false,
                },
            ),
            ev(2, "dev_b", Op::TaskDrop { id: b }),
        ]);

        assert_eq!(state.tasks[&a].status, Status::Done);
        assert_eq!(state.tasks[&b].status, Status::Dropped);
    }

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
        state.apply(&ev(
            3,
            "a",
            Op::TaskDone {
                id: done,
                filled: false,
            },
        ));
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

    #[test]
    fn reopening_a_hidden_task_brings_it_back_into_sight() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(1, "a", add(id, "revisar el deploy")));
        state.apply(&ev(2, "a", Op::TaskDone { id, filled: false }));
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
            state.apply(&ev(2, "a", Op::TaskDone { id, filled: false }));
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
                    color: None,
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
                    page_of: None,
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
                    page_of: None,
                    order: None,
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
                d: crate::event::Filed {
                    folder: Some(None),
                    page_of: None,
                    order: None,
                },
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
                    color: None,
                },
            },
        ));

        assert!(!state.folders.contains_key(&work), "it rose again");
        assert!(state.is_erased(work));
    }

    #[test]
    fn a_deleted_document_leaves_the_name_of_its_file_behind_so_it_can_be_swept() {
        let mut state = State::default();
        let one = ulid::Ulid::generate();
        state.apply(&ev(
            1,
            "a",
            Op::DocAdd {
                id: one,
                d: crate::event::DocAdd {
                    file: "a-0001".into(),
                    order: "a0".into(),
                    folder: None,
                    page_of: None,
                },
            },
        ));

        state.apply(&ev(2, "a", Op::DocDelete { id: one }));

        assert!(
            state.shed.contains("a-0001"),
            "sin el nombre, el fichero se queda huerfano en las otras maquinas"
        );
        assert!(!state.docs.contains_key(&one));
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
                    page_of: None,
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
                color: None,
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
    fn a_name_that_arrives_from_another_machine_is_cleaned_too() {
        let mut state = State::default();
        let id = Ulid::generate();
        state.apply(&ev(
            1,
            "b",
            Op::FolderAdd {
                id,
                d: crate::event::FolderAdd {
                    name: "trabajo\u{202e}odajabart".into(),
                    order: "a0".into(),
                    parent: None,
                    icon: None,
                    color: None,
                },
            },
        ));

        assert!(
            !state.folders[&id].name.contains('\u{202e}'),
            "{:?}",
            state.folders[&id].name
        );
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
                    color: None,
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
                    color: None,
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
                    page_of: None,
                    order: None,
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
        let one = folder(&mut state, "uno", None);
        let two = folder(&mut state, "dos", Some(one));
        let three = folder(&mut state, "tres", Some(two));
        let work = folder(&mut state, "trabajo", None);
        let inside = folder(&mut state, "corporativo", Some(work));

        state.apply(&ev(
            2,
            "a",
            Op::FolderMove {
                id: work,
                d: crate::event::Filed {
                    folder: Some(Some(three)),
                    page_of: None,
                    order: None,
                },
            },
        ));

        assert_eq!(
            state.folders[&work].parent, None,
            "its child fell past the deepest level"
        );
        assert_eq!(state.folders[&inside].parent, Some(work));
    }

    #[test]
    fn a_branch_may_be_moved_until_its_deepest_child_lands_on_the_fourth_level() {
        let mut state = State::default();
        let one = folder(&mut state, "uno", None);
        let two = folder(&mut state, "dos", Some(one));
        let moved = folder(&mut state, "movida", None);
        let held = folder(&mut state, "dentro", Some(moved));

        state.apply(&ev(
            2,
            "a",
            Op::FolderMove {
                id: moved,
                d: crate::event::Filed {
                    folder: Some(Some(two)),
                    page_of: None,
                    order: None,
                },
            },
        ));

        assert_eq!(state.folders[&moved].parent, Some(two));
        assert_eq!(state.depth(Some(held)), 4);
    }

    #[test]
    fn a_branch_whose_deepest_child_would_land_on_the_fifth_is_turned_away() {
        let mut state = State::default();
        let one = folder(&mut state, "uno", None);
        let two = folder(&mut state, "dos", Some(one));
        let three = folder(&mut state, "tres", Some(two));
        let moved = folder(&mut state, "movida", None);
        let held = folder(&mut state, "dentro", Some(moved));

        state.apply(&ev(
            2,
            "a",
            Op::FolderMove {
                id: moved,
                d: crate::event::Filed {
                    folder: Some(Some(three)),
                    page_of: None,
                    order: None,
                },
            },
        ));

        assert_eq!(
            state.folders[&moved].parent, None,
            "a fifth level slipped in"
        );
        assert_eq!(state.depth(Some(held)), 2);
    }

    #[test]
    fn a_lone_folder_still_fits_on_the_fourth_level_but_not_below_it() {
        let mut state = State::default();
        let mut at = None;
        for name in ["uno", "dos", "tres"] {
            at = Some(folder(&mut state, name, at));
        }
        let third = at.expect("three levels");
        let loose = folder(&mut state, "suelta", None);

        state.apply(&ev(
            2,
            "a",
            Op::FolderMove {
                id: loose,
                d: crate::event::Filed {
                    folder: Some(Some(third)),
                    page_of: None,
                    order: None,
                },
            },
        ));
        assert_eq!(state.folders[&loose].parent, Some(third));

        let deeper = folder(&mut state, "mas honda", None);
        state.apply(&ev(
            3,
            "a",
            Op::FolderMove {
                id: deeper,
                d: crate::event::Filed {
                    folder: Some(Some(loose)),
                    page_of: None,
                    order: None,
                },
            },
        ));

        assert_eq!(
            state.folders[&deeper].parent, None,
            "a fifth level slipped in"
        );
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
                    page_of: None,
                    order: None,
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
                    page_of: None,
                    order: None,
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
                    page_of: None,
                    order: None,
                },
            },
        ));

        assert_eq!(state.folders[&work].parent, None);
    }

    fn page(state: &mut State, file: &str, page_of: DocId) -> DocId {
        let id = Ulid::generate();
        state.apply(&ev(
            1,
            "a",
            Op::DocAdd {
                id,
                d: crate::event::DocAdd {
                    file: file.into(),
                    order: "a0".into(),
                    folder: None,
                    page_of: Some(page_of),
                },
            },
        ));
        id
    }

    fn moved(state: &mut State, id: DocId, d: crate::event::Filed) {
        state.apply(&ev(2, "a", Op::DocMove { id, d }));
    }

    #[test]
    fn a_page_is_born_where_its_document_lives() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let minutes = doc(&mut state, "a3f1-0001", Some(work));
        let march = page(&mut state, "a3f1-0002", minutes);

        assert_eq!(state.docs[&march].page_of, Some(minutes));
        assert_eq!(state.docs[&march].folder, Some(work));
    }

    #[test]
    fn a_page_of_a_page_is_kept_as_a_document() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        let march = page(&mut state, "a3f1-0002", minutes);
        let deeper = page(&mut state, "a3f1-0003", march);

        assert_eq!(
            state.docs[&deeper].page_of, None,
            "only a document holds pages"
        );
    }

    #[test]
    fn a_document_made_a_page_of_a_page_stays_a_document() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        let march = page(&mut state, "a3f1-0002", minutes);
        let loose = doc(&mut state, "a3f1-0003", None);

        moved(
            &mut state,
            loose,
            crate::event::Filed {
                folder: None,
                page_of: Some(Some(march)),
                order: None,
            },
        );

        assert_eq!(state.docs[&loose].page_of, None);
    }

    #[test]
    fn a_document_cannot_be_made_a_page_of_itself() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);

        moved(
            &mut state,
            minutes,
            crate::event::Filed {
                folder: None,
                page_of: Some(Some(minutes)),
                order: None,
            },
        );

        assert_eq!(state.docs[&minutes].page_of, None);
    }

    #[test]
    fn pages_follow_the_folder_their_document_moves_to() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let home = folder(&mut state, "casa", None);
        let minutes = doc(&mut state, "a3f1-0001", Some(work));
        let march = page(&mut state, "a3f1-0002", minutes);

        moved(
            &mut state,
            minutes,
            crate::event::Filed {
                folder: Some(Some(home)),
                page_of: None,
                order: None,
            },
        );

        assert_eq!(state.docs[&march].folder, Some(home));
    }

    #[test]
    fn a_page_cannot_be_filed_away_from_its_document() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let home = folder(&mut state, "casa", None);
        let minutes = doc(&mut state, "a3f1-0001", Some(work));
        let march = page(&mut state, "a3f1-0002", minutes);

        moved(
            &mut state,
            march,
            crate::event::Filed {
                folder: Some(Some(home)),
                page_of: None,
                order: None,
            },
        );

        assert_eq!(state.docs[&march].folder, Some(work));
    }

    #[test]
    fn a_page_that_becomes_a_document_stays_where_it_was_and_may_then_move() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let home = folder(&mut state, "casa", None);
        let minutes = doc(&mut state, "a3f1-0001", Some(work));
        let march = page(&mut state, "a3f1-0002", minutes);

        moved(
            &mut state,
            march,
            crate::event::Filed {
                folder: None,
                page_of: Some(None),
                order: None,
            },
        );
        assert_eq!(state.docs[&march].page_of, None);
        assert_eq!(state.docs[&march].folder, Some(work));

        moved(
            &mut state,
            march,
            crate::event::Filed {
                folder: Some(Some(home)),
                page_of: None,
                order: None,
            },
        );
        assert_eq!(state.docs[&march].folder, Some(home));
    }

    #[test]
    fn a_page_moved_to_another_document_lands_in_its_folder() {
        let mut state = State::default();
        let work = folder(&mut state, "trabajo", None);
        let home = folder(&mut state, "casa", None);
        let minutes = doc(&mut state, "a3f1-0001", Some(work));
        let diary = doc(&mut state, "a3f1-0002", Some(home));
        let march = page(&mut state, "a3f1-0003", minutes);

        moved(
            &mut state,
            march,
            crate::event::Filed {
                folder: None,
                page_of: Some(Some(diary)),
                order: None,
            },
        );

        assert_eq!(state.docs[&march].page_of, Some(diary));
        assert_eq!(state.docs[&march].folder, Some(home));
    }

    #[test]
    fn deleting_a_document_takes_its_pages() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        let march = page(&mut state, "a3f1-0002", minutes);
        let april = page(&mut state, "a3f1-0003", minutes);

        state.apply(&ev(3, "a", Op::DocDelete { id: minutes }));

        assert!(state.docs.is_empty(), "a page is part of its document");
        assert!(state.shed.contains("a3f1-0002"));
        assert!(state.shed.contains("a3f1-0003"));
        assert!(state.tombstones.contains(&march));
        assert!(state.tombstones.contains(&april));
    }

    #[test]
    fn deleting_a_page_leaves_its_document_alone() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        let march = page(&mut state, "a3f1-0002", minutes);

        state.apply(&ev(3, "a", Op::DocDelete { id: march }));

        assert!(state.docs.contains_key(&minutes));
        assert!(!state.docs.contains_key(&march));
    }

    #[test]
    fn archiving_a_document_puts_its_pages_away_and_brings_them_back() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        let march = page(&mut state, "a3f1-0002", minutes);

        state.apply(&ev(3, "a", Op::DocArchive { id: minutes }));
        assert!(state.docs[&march].archived);

        state.apply(&ev(4, "a", Op::DocUnarchive { id: minutes }));
        assert!(!state.docs[&march].archived);
    }

    #[test]
    fn a_document_that_holds_pages_cannot_become_one() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        page(&mut state, "a3f1-0002", minutes);
        let diary = doc(&mut state, "a3f1-0003", None);

        moved(
            &mut state,
            minutes,
            crate::event::Filed {
                folder: None,
                page_of: Some(Some(diary)),
                order: None,
            },
        );

        assert_eq!(state.docs[&minutes].page_of, None);
    }

    #[test]
    fn a_page_refused_a_new_document_stays_with_the_one_it_had() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        let march = page(&mut state, "a3f1-0002", minutes);

        moved(
            &mut state,
            march,
            crate::event::Filed {
                folder: None,
                page_of: Some(Some(Ulid::generate())),
                order: None,
            },
        );

        assert_eq!(
            state.docs[&march].page_of,
            Some(minutes),
            "a refusal cannot be a way to unhang a page"
        );
    }

    #[test]
    fn a_page_hung_under_a_document_that_is_away_is_away_too() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        let loose = doc(&mut state, "a3f1-0002", None);
        state.apply(&ev(2, "a", Op::DocArchive { id: minutes }));

        moved(
            &mut state,
            loose,
            crate::event::Filed {
                folder: None,
                page_of: Some(Some(minutes)),
                order: None,
            },
        );

        assert!(state.docs[&loose].archived);
    }

    #[test]
    fn a_page_hung_under_a_document_lands_after_the_pages_already_there() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        let march = page(&mut state, "a3f1-0002", minutes);
        let loose = doc(&mut state, "a3f1-0003", None);

        moved(
            &mut state,
            loose,
            crate::event::Filed {
                folder: None,
                page_of: Some(Some(minutes)),
                order: None,
            },
        );

        assert_eq!(
            state
                .pages_of(minutes)
                .iter()
                .map(|one| one.id)
                .collect::<Vec<_>>(),
            vec![march, loose],
            "it was hung last and reads last"
        );
    }

    #[test]
    fn a_page_is_not_put_away_on_its_own() {
        let mut state = State::default();
        let minutes = doc(&mut state, "a3f1-0001", None);
        let march = page(&mut state, "a3f1-0002", minutes);

        state.apply(&ev(3, "a", Op::DocArchive { id: march }));

        assert!(
            !state.docs[&march].archived,
            "it goes away with its document"
        );
        assert!(!state.docs[&minutes].archived);
    }

    #[test]
    fn a_device_only_says_what_kind_it_is_itself() {
        let mut state = State::default();
        joined(&mut state, "dev_laptop", crate::event::DeviceKind::Machine);
        joined(&mut state, "dev_agent", crate::event::DeviceKind::Agent);
        let doc = a_doc(&mut state, "dev_laptop");

        state.apply(&ev(
            4,
            "dev_agent",
            Op::DeviceJoin {
                d: DeviceId("dev_laptop".into()),
                k: Some(crate::event::DeviceKind::Agent),
            },
        ));
        assert!(
            !state.assistants.contains(&DeviceId("dev_laptop".into())),
            "no device hands another one a badge"
        );

        state.apply(&ev(5, "dev_laptop", Op::DocDelete { id: doc }));
        assert!(!state.docs.contains_key(&doc), "the person still deletes");
    }

    #[test]
    fn an_assistant_cannot_retire_an_attachment() {
        let mut state = State::default();
        joined(&mut state, "dev_agent", crate::event::DeviceKind::Agent);
        let at = "attachments/ab/a-12345678.png".to_string();

        state.apply(&ev(3, "dev_agent", Op::AttachRetire { d: at.clone() }));
        assert!(state.retired.is_empty());

        state.apply(&ev(4, "dev_laptop", Op::AttachRetire { d: at }));
        assert_eq!(state.retired.len(), 1);
    }
}

#[cfg(test)]
mod compacting {
    use crate::event::{DeviceId, Event, Op};
    use crate::state::State;
    use ulid::Ulid;

    fn ev(who: &str, ms: i64, op: Op) -> Event {
        Event::new(
            DeviceId(who.into()),
            jiff::Timestamp::from_millisecond(ms).unwrap(),
            op,
        )
    }

    fn added(
        events: &mut Vec<Event>,
        ms: i64,
        page_of: Option<crate::model::DocId>,
    ) -> crate::model::DocId {
        let id = Ulid::generate();
        let told = State::replay(events);
        let order = crate::order::last_of(
            told.docs
                .values()
                .filter(|one| one.page_of == page_of)
                .map(|one| one.order.as_str()),
        );
        events.push(ev(
            "dev_a",
            ms,
            Op::DocAdd {
                id,
                d: crate::event::DocAdd {
                    file: format!("dev_a-{ms:04}"),
                    order,
                    folder: None,
                    page_of,
                },
            },
        ));
        id
    }

    fn rebased(events: &mut Vec<Event>, who: &str, ms: i64, run: &[crate::model::DocId]) {
        let keys: Vec<&str> = run.iter().map(|_| "").collect();
        for (n, (id, order)) in run.iter().zip(crate::order::resequenced(&keys)).enumerate() {
            events.push(ev(
                who,
                ms + n as i64,
                Op::DocMove {
                    id: *id,
                    d: crate::event::Filed {
                        folder: None,
                        page_of: None,
                        order: order.or_else(|| Some(crate::order::first())),
                    },
                },
            ));
        }
    }

    #[test]
    fn two_machines_rebasing_one_run_at_once_still_read_it_the_same_way() {
        let mut events = Vec::new();
        let up = added(&mut events, 1, None);
        let run: Vec<crate::model::DocId> =
            (2..6).map(|ms| added(&mut events, ms, Some(up))).collect();

        let mut theirs: Vec<crate::model::DocId> = run.clone();
        theirs.swap(1, 2);
        rebased(&mut events, "dev_a", 10, &run);
        rebased(&mut events, "dev_b", 11, &theirs);

        let mut sorted = events.clone();
        sorted.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        let here: Vec<crate::model::DocId> = State::replay(&sorted)
            .pages_of(up)
            .iter()
            .map(|one| one.id)
            .collect();

        let mut shuffled = events;
        shuffled.reverse();
        shuffled.sort_by(|a, b| a.sort_key().cmp(&b.sort_key()));
        let there: Vec<crate::model::DocId> = State::replay(&shuffled)
            .pages_of(up)
            .iter()
            .map(|one| one.id)
            .collect();

        assert_eq!(here, there, "a tie is broken the same way on both machines");
        assert_eq!(here.len(), run.len(), "and nothing falls out of the run");
    }
}
