use std::collections::{BTreeMap, BTreeSet};

use ulid::Ulid;

use crate::{
    event::{DeviceId, Event, LogAdd, LogEdit, Op, StepAdd, TaskAdd, TaskMove, TaskPatch},
    model::{
        DocId, Folder, FolderId, Kept, List, ListId, LogEntry, Priority, Reading, Status, Step,
        StepId, Tag, Task, TaskId,
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
    pub signed: crate::event::Signature,
    pub signed_before: Vec<String>,
    pub agents: BTreeSet<DeviceId>,
    pub assistants: BTreeSet<DeviceId>,
    pub hosts: BTreeMap<DeviceId, DeviceId>,
    pub named: BTreeMap<DeviceId, crate::applying::Named>,
    pub runs: BTreeMap<DeviceId, String>,
    pub heard: BTreeMap<DeviceId, i64>,
    pub sealed: BTreeMap<String, String>,
    pub sourced: BTreeMap<String, TaskId>,
    pub dropped: BTreeSet<DeviceId>,
    pub kept: BTreeMap<String, (String, u64)>,
    pub holders: BTreeMap<String, BTreeSet<DeviceId>>,
    pub keys: BTreeMap<DeviceId, String>,
    pub retired: BTreeSet<String>,
    pub shed: BTreeSet<String>,
    pub forebears: BTreeSet<String>,
    pub persons: crate::person::Persons,
    pub(crate) fill: Fill,
    pub(crate) tombstones: BTreeSet<Ulid>,
}

pub(crate) fn here(d: &crate::event::DocAdd, signed: &crate::event::Signature) -> Option<String> {
    match d.guest {
        true => None,
        false => signed.alias.clone(),
    }
}

pub fn same_name(one: &str, other: &str) -> bool {
    crate::text::composed(one.trim()).to_lowercase()
        == crate::text::composed(other.trim()).to_lowercase()
}

fn alike(one: Option<&str>, other: Option<&str>) -> bool {
    match (one, other) {
        (Some(one), Some(other)) => same_name(one, other),
        (None, None) => true,
        _ => false,
    }
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

    pub fn away(&self, file: &str) -> bool {
        self.docs
            .values()
            .any(|one| one.file == file && self.held_away(one))
    }

    /// A folder is away when it says so or when any folder above it does.
    pub fn folder_away(&self, at: FolderId) -> bool {
        let mut walk = Some(at);
        let mut deep = 0;
        while let Some(one) = walk {
            let Some(folder) = self.folders.get(&one) else {
                return false;
            };
            if folder.archived {
                return true;
            }
            deep += 1;
            if deep > crate::model::DEEPEST {
                return false;
            }
            walk = folder.parent;
        }
        false
    }

    /// The documents a person keeps on their own: a task's words belong to the task.
    pub fn papers(&self) -> impl Iterator<Item = &Kept> {
        self.docs.values().filter(|one| one.task_of.is_none())
    }

    /// Whether a document belongs to what a scope asks for: the archive, the rest, or both.
    pub fn doc_kept_by(&self, scope: crate::view::Scope, kept: &Kept) -> bool {
        match scope {
            crate::view::Scope::Open => !self.held_away(kept),
            crate::view::Scope::Archived => self.held_away(kept),
            crate::view::Scope::Either => true,
        }
    }

    /// What the archive holds: the document's own mark, or the folder it sits in.
    pub fn held_away(&self, kept: &Kept) -> bool {
        kept.archived || self.held_by_another(kept)
    }

    /// What holds it from above, its own mark left out: nothing it answers for by itself, and so
    /// nothing `archive_doc` or a move can take back.
    pub fn held_by_another(&self, kept: &Kept) -> bool {
        self.closed_with_its_task(kept)
            || kept.folder.is_some_and(|at| self.folder_away(at))
            || kept.page_of.is_some_and(|up| {
                self.docs.get(&up).is_some_and(|doc| {
                    doc.archived || doc.folder.is_some_and(|at| self.folder_away(at))
                })
            })
    }

    pub fn stowed(&self, id: DocId) -> bool {
        self.docs.get(&id).is_some_and(|one| self.held_away(one))
    }

    pub fn written_shut(&self, id: DocId) -> bool {
        self.shut(id) || self.stowed(id)
    }

    pub fn shut_tight(&self, file: &str) -> bool {
        self.docs
            .values()
            .any(|one| one.file == file && self.shut(one.id))
    }

    pub fn bolted(&self, file: &str) -> bool {
        self.docs
            .values()
            .any(|one| one.file == file && self.written_shut(one.id))
    }

    fn bolt(&mut self, id: DocId, shut: bool) {
        if shut && self.docs.get(&id).is_some_and(|one| one.page_of.is_some()) {
            return;
        }
        if let Some(doc) = self.docs.get_mut(&id) {
            doc.locked = shut;
        }
    }

    fn shelf(&mut self, id: FolderId, away: bool) {
        if let Some(folder) = self.folders.get_mut(&id) {
            folder.archived = away;
        }
    }

    pub(crate) fn named_trail(&self, at: FolderId) -> Option<Vec<String>> {
        let mut names = Vec::new();
        let mut up = Some(at);
        // An ancestor can be missing when a delete from another machine lands before the add
        // that named it, and a partial way down still says more than nothing.
        while let Some(folder) = up.and_then(|id| self.folders.get(&id)) {
            names.insert(0, folder.name.clone());
            up = folder.parent;
            if names.len() > crate::model::DEEPEST {
                break;
            }
        }
        (!names.is_empty()).then_some(names)
    }

    fn shelve(&mut self, id: DocId, away: bool, by_person: bool) {
        if let Some(doc) = self.docs.get_mut(&id) {
            doc.archived = away;
            if away && by_person {
                doc.flagged = None;
            }
        }
    }

    pub fn apply(&mut self, event: &Event) {
        let at = event.timestamp.as_second();
        match self.heard.get_mut(&event.device) {
            Some(was) => *was = (*was).max(at),
            None => {
                self.heard.insert(event.device.clone(), at);
            }
        }
        self.applied(event);
    }

    fn applied(&mut self, event: &Event) {
        if let Some(id) = event.entity_id().filter(|id| self.tombstones.contains(id)) {
            crate::witness::trace(
                crate::witness::channel::STORE,
                "an event arrived for something already gone, and was let go",
                &[
                    ("at", crate::witness::Fact::Id(id.to_string())),
                    ("by", crate::witness::Fact::Id(event.device.0.clone())),
                ],
            );
            return;
        }
        // The door an assistant's hand meets, judged at replay so every machine agrees: a note or
        // a bell anywhere, a patch on what it filed, a fill-in where it filed or was let in, and
        // nothing else — whatever the server that wrote it believed.
        if self.assistants.contains(&event.device)
            && let Some(id) = event.op.about_whom()
            && let Some(task) = self.tasks.get(&id)
            && !self.lets_agents(task, &event.op)
        {
            crate::witness::warn(
                crate::witness::channel::STORE,
                "an assistant wrote on a task where the door does not let it, and was let go",
                &[
                    ("at", crate::witness::Fact::Id(id.to_string())),
                    ("by", crate::witness::Fact::Id(event.device.0.clone())),
                ],
            );
            return;
        }
        if event.op.destroys() && self.assistants.contains(&event.device) {
            crate::witness::warn(
                crate::witness::channel::STORE,
                "an assistant asked to destroy something, and was refused",
                &[
                    (
                        "at",
                        crate::witness::Fact::Id(
                            event
                                .entity_id()
                                .map(|id| id.to_string())
                                .unwrap_or_default(),
                        ),
                    ),
                    ("by", crate::witness::Fact::Id(event.device.0.clone())),
                ],
            );
            return;
        }

        self.grown(event);
        match &event.op {
            Op::TaskAdd { id, d } => self.task_added(event, *id, d),
            Op::TaskUpdate { id, d } => {
                let person = !self.assistants.contains(&event.device);
                self.with_task(*id, |t| patch(t, d, person));
                if d.repeat.is_some() {
                    self.turned_routine(*id);
                }
            }
            Op::TaskDone { id, filled } => {
                let zone = event.zone.clone();
                self.with_task(*id, |t| {
                    t.status = Status::Done;
                    t.completed_at = Some(event.timestamp);
                    t.filled = *filled;
                    t.closed_in = zone;
                })
            }
            // The agent's mark stays: reopening is most often a finish taken back by
            // mistake, and taking the mark off is the person's own op, `TaskUnresolve`.
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
            // The word the person gave a task outlives a delete written elsewhere while it still
            // read as a trace: the same stamp order everywhere, so every machine keeps it. And
            // what it was written from stays known, so an assistant does not file it again.
            Op::TaskDelete { id } => self.task_deleted(event, id),
            Op::TaskMove { id, d } => {
                self.with_task(*id, |t| move_task(t, d));
                if let Some(whole) = d.part_of {
                    self.task_parted(*id, whole, event);
                }
            }

            Op::TaskDescribe { id, d } => self.with_task(*id, |t| t.description = d.body.clone()),
            Op::TaskLog { id, d } => {
                let at = event.timestamp;
                let by = event.device.clone();
                let via = event.via.clone();
                self.with_task(*id, |t| add_log_entry(t, d, at, &by, via));
            }
            Op::TaskLogEdit { id, d } => self.with_task(*id, |t| edit_log_entry(t, d)),
            Op::TaskResolve { id, d } => self.task_resolved(event, id, d),
            Op::TaskUnresolve { id } => self.with_task(*id, |t| t.resolved = None),

            Op::StepAdd { id, d } => self.with_task(*id, |t| add_step(t, d)),
            Op::StepDone { id, d } => self.step_marked(event, *id, d.step, true),
            Op::StepUndone { id, d } => self.step_marked(event, *id, d.step, false),
            Op::StepSaid { id, d } if self.tasks.get(id).is_some_and(Task::is_open) => {
                self.with_step(*id, d.step, |s| s.said = !s.done)
            }
            Op::StepSaid { .. } => {}
            Op::StepUnsaid { id, d } => self.with_step(*id, d.step, |s| s.said = false),
            Op::StepText { id, d } => self.with_step(*id, d.step, |s| s.reword(&d.text)),
            Op::StepReorder { id, d } => {
                self.with_step(*id, d.step, |s| s.order = d.order.clone());
                self.with_task(*id, sort_steps);
            }
            Op::StepRemove { id, d } => self.with_task(*id, |t| t.steps.retain(|s| s.id != d.step)),
            // What a page says is the page's own; the log keeps only that it moved, for its trail.
            Op::TaskPaged { .. } => {}

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
            Op::FolderAdd { id, d } => self.folder_added(id, d),
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
            Op::FolderMove { id, d } => self.folder_moved(id, d),
            Op::FolderDelete { id } => self.folder_deleted(id),
            Op::DocAdd { id, d } => self.doc_added(event, id, d),
            Op::DocSaid { id, d } => self.doc_said(event, id, d),
            Op::DocMove { id, d } => self.doc_moved(id, d),
            Op::DocDelete { id } => self.doc_deleted(id),
            Op::DocSigned { id, d } => self.doc_signed(id, d),
            Op::FolderArchive { id } => self.shelf(*id, true),
            Op::FolderUnarchive { id } => self.shelf(*id, false),
            Op::DocArchive { id } => {
                let by_person = !self.assistants.contains(&event.device);
                self.shelve(*id, true, by_person)
            }
            Op::DocLock { id } => self.bolt(*id, true),
            Op::DocUnlock { id } => self.bolt(*id, false),
            Op::DocUnarchive { id } => self.shelve(*id, false, true),
            Op::DocFlag { id, d } => self.doc_flagged(event, id, d),
            Op::DocUnflag { id } => self.doc_marked(id, |doc| doc.flagged = None),
            Op::DocCompacted { id, print, before } => {
                let said = crate::model::Compacted {
                    at: event.timestamp,
                    by: event.device.clone(),
                    via: event.via.clone(),
                    print: print.clone(),
                    before: before.clone(),
                };
                self.doc_marked(id, |doc| doc.compacted = Some(said));
            }
            Op::DocUncompacted { id } => self.doc_marked(id, |doc| doc.compacted = None),
            Op::DeviceJoin { d, k, p } => self.device_joined(event, d, k, p),
            Op::DeviceKey { d, p } => self.key_published(event, d, p),
            Op::DeviceRotate { d, p } => {
                crate::signing::rotated(&mut self.keys, &event.device, d, p);
            }
            Op::DeviceNamed { d, name, os } => self.device_named(event, d, name, os.as_deref()),
            Op::DeviceRelease { .. } | Op::CloudSealed { .. } => self.said_by_a_machine(event),
            // Self-declared like `k`, or declared by the machine that hosts it: nobody else's word.
            Op::DeviceHost { d, of, .. } => {
                if (event.device == *d || event.device == *of)
                    && self.assistants.contains(d)
                    && !self.assistants.contains(of)
                {
                    self.hosts.insert(d.clone(), of.clone());
                }
            }
            Op::Signed { d } => self.store_signed(d),
            Op::DeviceRemove { d } => self.device_removed(d),
            // Every key is the one name an attachment has now, whatever shelf an older event put it on.
            Op::AttachKept { d } => {
                if let Some(at) =
                    crate::attach::canonical(&d.at).filter(|_| vouches_for_its_name(d))
                {
                    self.kept
                        .insert(at.clone(), (d.sha256.to_ascii_lowercase(), d.bytes));
                    self.holders
                        .entry(at)
                        .or_default()
                        .insert(event.device.clone());
                }
            }
            Op::AttachLetGo { d } => {
                let at = crate::attach::canonical(d).unwrap_or_else(|| d.clone());
                if let Some(who) = self.holders.get_mut(&at) {
                    who.remove(&event.device);
                    if who.is_empty() {
                        self.holders.remove(&at);
                    }
                }
            }
            Op::AttachRetire { d, .. } => {
                if let Some(at) = crate::attach::canonical(d) {
                    self.retired.insert(at.clone());
                    self.kept.remove(&at);
                    self.holders.remove(&at);
                }
            }
            Op::StoresJoined { d } => self.stores_joined(d),
            Op::ShareKey { .. } | Op::ShareCertified { .. } | Op::ShareRevoked { .. } => {
                self.persons.took(event, &self.assistants, &self.devices);
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

    pub(crate) fn with_task(&mut self, id: TaskId, f: impl FnOnce(&mut Task)) {
        let whole = self.has_bodies();
        if let Some(task) = self.tasks.get_mut(&id) {
            f(task);
            if whole {
                task.retally();
            }
        }
    }

    fn lets_agents(&self, task: &Task, op: &Op) -> bool {
        match op {
            Op::TaskLog { .. } => true,
            Op::TaskUpdate { d, .. } => self.filed_by_agents(task) || only_bells(d),
            // A word on any task is only a word: the person decides, so it needs no door.
            Op::TaskResolve { d, .. } => d.word || self.attended_by_agents(task),
            Op::StepSaid { .. } | Op::StepUnsaid { .. } => true,
            Op::TaskDescribe { .. } | Op::StepAdd { .. } | Op::StepDone { .. } => {
                self.attended_by_agents(task)
            }
            Op::StepUndone { d, .. } => self.untickable_by_agents(task, d.step),
            Op::StepText { d, .. } => self.mendable_by_agents(task, d.step),
            Op::StepRemove { d, .. } => self.mendable_by_agents(task, d.step),
            Op::TaskLogEdit { d, .. } => self.attended_by_agents(task) && !d.body.trim().is_empty(),
            Op::TaskMove { d, .. } => self.agent_may_move(task, d),
            _ => false,
        }
    }

    fn untickable_by_agents(&self, task: &Task, step: StepId) -> bool {
        self.attended_by_agents(task) && task.step(step).is_some_and(|one| one.by_agent)
    }

    fn mendable_by_agents(&self, task: &Task, step: StepId) -> bool {
        self.attended_by_agents(task) && task.step(step).is_some_and(|one| !one.done)
    }

    fn step_marked(&mut self, event: &Event, task: TaskId, step: StepId, done: bool) {
        let agent = self.assistants.contains(&event.device);
        self.with_step(task, step, |s| {
            // A redundant tick never takes a person's tick over; a person's always claims it.
            s.by_agent = done && agent && (!s.done || s.by_agent);
            s.done = done;
            s.said = false;
        });
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
            .filter(|one| !self.held_away(one) && one.page_of.is_none())
            .filter(|one| one.folder.is_none_or(|at| !self.folders.contains_key(&at)))
            .collect()
    }

    /// Pages are counted with their document, not beside it. A folder shows what stands on the
    /// same side of the archive as itself: open ones in the tree, the whole of it on the shelf.
    pub fn inside(&self, folder: FolderId) -> Vec<&Kept> {
        let away = self.folder_away(folder);
        self.docs
            .values()
            .filter(|one| one.page_of.is_none() && one.folder == Some(folder))
            .filter(|one| self.held_away(one) == away)
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

    /// The pages of a document in the order it reads them. A body says nothing about the pages it
    /// does not name, so those keep their places and the named ones are dealt back out, in the
    /// order the text names them, into the places named pages already held. That is the same run
    /// `pages_told` writes to the log, so a reader never sees an order the next settling undoes.
    pub fn pages_read(&self, doc: DocId, body: &str) -> Vec<&Kept> {
        let held = self.pages_of(doc);
        let mut takes = BTreeSet::new();
        let wanted: Vec<&Kept> = crate::refs::papers(body)
            .iter()
            .filter_map(|file| held.iter().find(|one| &one.file == file).copied())
            .filter(|one| takes.insert(one.id))
            .collect();
        let mut told = wanted.into_iter();
        held.iter()
            .map(|one| match takes.contains(&one.id) {
                true => told.next().unwrap_or(one),
                false => one,
            })
            .collect()
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
        self.docs
            .values()
            .filter(|one| self.held_away(one))
            .collect()
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
        // An open folder does not count what somebody shelved inside it, and the shelf does not
        // count back out; each side adds up only its own.
        let away = self.folder_away(folder);
        self.inside(folder).len()
            + self
                .under(Some(folder))
                .iter()
                .filter(|one| self.folder_away(one.id) == away)
                .map(|one| self.counting(one.id, seen))
                .sum::<usize>()
    }

    pub(crate) fn has_room_under(&self, at: FolderId) -> bool {
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

    pub(crate) fn tallest_under(&self, at: FolderId) -> usize {
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

    pub(crate) fn would_loop(&self, moving: FolderId, under: Option<FolderId>) -> bool {
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

    /// A task an assistant may fill in: one an assistant filed, or one the person opened to
    /// them. `assistants` only ever grows, so a task the retired agent filed stays attended.
    pub fn attended_by_agents(&self, task: &Task) -> bool {
        task.open_to_agents || self.filed_by_agents(task)
    }

    pub fn filed_by_agents(&self, task: &Task) -> bool {
        task.created_by
            .as_ref()
            .is_some_and(|who| self.assistants.contains(who))
    }

    /// The tasks other turns hang from: a root whose `repeat` was taken off reads as a trace,
    /// and erasing it would cut the series it still heads.
    pub fn roots(&self) -> std::collections::BTreeSet<TaskId> {
        self.tasks.values().filter_map(|t| t.after).collect()
    }

    pub fn erasable(&self, id: TaskId) -> Result<(), crate::model::Stays> {
        let task = self.tasks.get(&id).ok_or(crate::model::Stays::Open)?;
        task.erasable()?;
        if self.roots().contains(&id) {
            return Err(crate::model::Stays::Routine);
        }
        if self.holds_parts(id) {
            return Err(crate::model::Stays::Parts);
        }
        Ok(())
    }

    /// What the trace layer shows: closed, not folded away, and read as a trace this instant.
    pub fn the_trace(&self) -> impl Iterator<Item = &Task> {
        self.tasks
            .values()
            .filter(|t| t.is_archived() && !t.folded() && t.reading() == Reading::Trace)
    }

    /// A trace pin is let go on reopening — work starts again and is judged again by what it
    /// writes — while a story pin stays: a finish taken back by mistake must not unkeep it.
    pub fn reopening(&self, id: TaskId) -> Vec<Op> {
        let mut ops = vec![Op::TaskReopen { id }];
        if self
            .tasks
            .get(&id)
            .is_some_and(|task| task.read_as == Some(Reading::Trace))
        {
            ops.push(Op::TaskUpdate {
                id,
                d: TaskPatch {
                    read_as: Some(None),
                    ..Default::default()
                },
            });
        }
        if let Some(born) = self
            .tasks
            .values()
            .find(|task| task.after == Some(id) && task.status == crate::model::Status::Open)
            .filter(|born| untouched(born))
        {
            ops.extend(crate::task_docs::erased_with(self, born.id));
        }
        ops
    }

    pub fn order_last_in(&self, list: Option<ListId>) -> String {
        order::last_of(self.ordered_keys(list))
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
        let holding = self.lists_holding_open();
        self.lists
            .values()
            .filter(move |l| !l.archived || holding.contains(&l.id))
    }

    pub fn ordered_open(&self) -> Vec<&Task> {
        // Borrowed, not cloned: one snapshot walks this list several times over.
        fn key(t: &Task) -> (Option<jiff::civil::DateTime>, Priority, &str, TaskId) {
            (
                t.date.as_ref().map(|d| d.at),
                t.priority,
                t.order.as_str(),
                t.id,
            )
        }

        let mut tasks: Vec<_> = self.open_tasks().collect();
        tasks.sort_by(|a, b| match (&a.date, &b.date) {
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            _ => key(a).cmp(&key(b)),
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
        let terms = crate::text::terms(query);
        if terms.is_empty() {
            return (Vec::new(), 0);
        }

        let mut hits: Vec<(crate::view::Hit, &Task)> = self
            .tasks
            .values()
            .filter(|t| scope.keeps(t))
            .filter_map(|t| crate::view::matches_query(t, &terms).map(|hit| (hit, t)))
            .collect();

        hits.sort_by_key(|(hit, t)| {
            (
                t.folded(),
                t.is_archived(),
                *hit,
                std::cmp::Reverse(t.heft()),
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
        self.tasks
            .values()
            .flat_map(|t| &t.tags)
            .chain(self.papers().flat_map(|one| &one.tags))
            .collect()
    }

    pub fn author_of<'a>(&'a self, kept: &'a crate::model::Kept) -> Option<&'a str> {
        kept.by.as_deref()
    }

    pub fn born_of<'a>(&'a self, kept: &'a crate::model::Kept) -> Option<&'a str> {
        kept.born_by
            .as_deref()
            .filter(|one| !alike(Some(one), kept.by.as_deref()))
    }

    pub fn mine_to_sign(&self) -> Vec<DocId> {
        let now = self.signed.alias.as_deref();
        self.docs
            .values()
            .filter(|one| !self.written_shut(one.id))
            .filter(|one| self.mine(one))
            .filter(|one| !alike(one.by.as_deref(), now))
            .map(|one| one.id)
            .collect()
    }

    /// A hand of mine reads as the alias I sign with now: the log keeps what it was, and
    /// changing an alias rewrites nothing behind it.
    pub fn editor_of<'a>(&'a self, kept: &'a crate::model::Kept) -> Option<&'a str> {
        let hand = kept.edited_by.as_deref()?;
        if alike(Some(hand), kept.by.as_deref()) {
            return None;
        }
        // Signing once under the name a guest writes with must not hide my own hand on it.
        if kept.guest || !self.mine_to_write(hand) {
            return Some(hand);
        }
        match kept.by.as_deref().is_some_and(|by| self.mine_to_write(by)) {
            true => None,
            false => Some(self.signed.alias.as_deref().unwrap_or(hand)),
        }
    }

    /// Another name is another person, whether it came in a parcel or through a shared store.
    pub fn mine(&self, kept: &crate::model::Kept) -> bool {
        match kept.by.as_deref() {
            Some(by) => !kept.guest && self.mine_to_write(by),
            None => true,
        }
    }

    pub fn mine_to_write(&self, hand: &str) -> bool {
        alike(Some(hand), self.signed.alias.as_deref())
            || self.signed_before.iter().any(|was| same_name(was, hand))
    }

    pub fn docs_tagged(&self, tag: &Tag) -> impl Iterator<Item = &crate::model::Kept> {
        self.docs.values().filter(move |one| {
            one.task_of.is_none() && !self.held_away(one) && one.tags.contains(tag)
        })
    }
}

pub(crate) fn vouches_for_its_name(said: &crate::event::Held) -> bool {
    if !crate::attach::names_an_attachment(&said.at) {
        return false;
    }
    if said.sha256.len() != 64 || !said.sha256.chars().all(|one| one.is_ascii_hexdigit()) {
        return false;
    }
    crate::attach::leaf_of(&said.at).is_some_and(|leaf| crate::attach::vouched(leaf, &said.sha256))
}

fn loose(name: &str) -> String {
    name.to_lowercase().replace([' ', '_'], "-")
}

fn untouched(born: &Task) -> bool {
    born.journal().next().is_none() && !born.steps.iter().any(|step| step.done)
}

pub(crate) fn task_from(id: TaskId, d: &TaskAdd) -> Task {
    Task {
        priority: d.priority.unwrap_or_default(),
        date: d.date.clone(),
        deadline: d.deadline.clone(),
        list: d.list,
        tags: crate::tagging::worth_keeping(&d.tags),
        reminders: d.reminders.clone(),
        repeat: d.repeat,
        after: d.after,
        source: d.source.clone(),
        ..Task::new(id, d.title.clone(), d.order.clone())
    }
}

/// A bell is the one thing an assistant adds to any open task: `remind` only ever adds one.
pub(crate) fn only_bells(d: &TaskPatch) -> bool {
    d.reminders.is_some()
        && *d
            == TaskPatch {
                reminders: d.reminders.clone(),
                ..Default::default()
            }
}

fn patch(task: &mut Task, d: &TaskPatch, person: bool) {
    if let Some(v) = &d.title {
        task.title = v.clone();
    }
    // Converting is the person's: a patch an assistant wrote lands without it. A routine is
    // never converted, and a pin to "routine" says nothing rather than clearing.
    if person
        && let Some(v) = d.read_as
        && v != Some(Reading::Routine)
    {
        task.read_as = v;
    }
    if person && let Some(v) = d.open_to_agents {
        task.open_to_agents = v;
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
        task.tags = crate::tagging::worth_keeping(v);
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

fn add_log_entry(
    task: &mut Task,
    d: &LogAdd,
    at: jiff::Timestamp,
    by: &DeviceId,
    via: Option<String>,
) {
    task.log.push(LogEntry {
        id: d.entry,
        at,
        tz: d.tz.clone(),
        body: d.body.clone(),
        by: Some(by.clone()),
        via,
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
        by_agent: false,
        said: false,
    });
    sort_steps(task);
}

fn sort_steps(task: &mut Task) {
    task.steps.sort_by(|a, b| a.order.cmp(&b.order));
}

#[cfg(test)]
#[path = "state_test.rs"]
mod tests;

#[cfg(test)]
#[path = "state_converting.rs"]
mod converting;

#[cfg(test)]
#[path = "state_opening.rs"]
mod opening;

#[cfg(test)]
#[path = "state_signing.rs"]
mod signing;

#[cfg(test)]
#[path = "state_compacting.rs"]
mod compacting;
