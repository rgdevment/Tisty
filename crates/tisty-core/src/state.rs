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
    pub sourced: BTreeMap<String, TaskId>,
    pub dropped: BTreeSet<DeviceId>,
    pub retired: BTreeSet<String>,
    pub shed: BTreeSet<String>,
    pub forebears: BTreeSet<String>,
    pub(crate) fill: Fill,
    tombstones: BTreeSet<Ulid>,
}

fn here(d: &crate::event::DocAdd, signed: &crate::event::Signature) -> Option<String> {
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

    /// What the archive holds: the document's own mark, or the folder it sits in.
    pub fn held_away(&self, kept: &Kept) -> bool {
        kept.archived || self.held_by_another(kept)
    }

    /// What holds it from above, its own mark left out: nothing it answers for by itself, and so
    /// nothing `archive_doc` or a move can take back.
    pub fn held_by_another(&self, kept: &Kept) -> bool {
        kept.folder.is_some_and(|at| self.folder_away(at))
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

    fn named_trail(&self, at: FolderId) -> Option<Vec<String>> {
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
            && !match &event.op {
                Op::TaskLog { .. } => true,
                Op::TaskUpdate { d, .. } => self.filed_by_agents(task) || only_bells(d),
                Op::TaskResolve { .. }
                | Op::TaskDescribe { .. }
                | Op::StepAdd { .. }
                | Op::StepDone { .. } => self.attended_by_agents(task),
                _ => false,
            }
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

        match &event.op {
            Op::TaskAdd { id, d } => {
                let mut task = task_from(*id, d);
                task.created_by = Some(event.device.clone());
                task.created_via = event.via.clone();
                task.retally();
                if let Some(source) = &task.source {
                    self.sourced.insert(source.clone(), *id);
                }
                self.tasks.insert(*id, task);
            }
            Op::TaskUpdate { id, d } => {
                let person = !self.assistants.contains(&event.device);
                self.with_task(*id, |t| patch(t, d, person))
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
            Op::TaskDelete { id } => {
                if self
                    .tasks
                    .get(id)
                    .is_some_and(|task| task.read_as == Some(Reading::Story))
                {
                    crate::witness::warn(
                        crate::witness::channel::STORE,
                        "a delete reached a task kept as a story, and was let go",
                        &[
                            ("at", crate::witness::Fact::Id(id.to_string())),
                            ("by", crate::witness::Fact::Id(event.device.0.clone())),
                        ],
                    );
                    return;
                }
                self.tasks.remove(id);
                self.tombstones.insert(*id);
            }
            Op::TaskMove { id, d } => self.with_task(*id, |t| move_task(t, d)),

            Op::TaskDescribe { id, d } => self.with_task(*id, |t| t.description = d.body.clone()),
            Op::TaskLog { id, d } => {
                let at = event.timestamp;
                let by = event.device.clone();
                let via = event.via.clone();
                self.with_task(*id, |t| add_log_entry(t, d, at, &by, via));
            }
            Op::TaskLogEdit { id, d } => self.with_task(*id, |t| edit_log_entry(t, d)),
            Op::TaskResolve { id, d } => {
                let said = crate::model::Resolved {
                    at: d.at.unwrap_or(event.timestamp),
                    by: d.by.clone().unwrap_or_else(|| event.device.clone()),
                    entry: d.entry,
                    via: d.via.clone().or_else(|| event.via.clone()),
                };
                self.with_task(*id, |t| {
                    if t.is_open() {
                        t.resolved = Some(said);
                    }
                });
            }
            Op::TaskUnresolve { id } => self.with_task(*id, |t| t.resolved = None),

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
                // Naming a folder that is already here does not bring it out of the archive.
                let away = self.folders.get(id).is_some_and(|one| one.archived);
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
                        archived: away,
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
                let landed = match d.folder {
                    None => true,
                    Some(parent) => {
                        let room = parent.is_none_or(|at| self.has_room_under(at))
                            && !self.would_loop(*id, parent)
                            && self.depth(parent) + self.tallest_under(*id)
                                <= crate::model::DEEPEST;
                        if room && let Some(folder) = self.folders.get_mut(id) {
                            folder.parent = parent;
                        }
                        room
                    }
                };
                if landed
                    && let Some(order) = d.order.clone()
                    && let Some(folder) = self.folders.get_mut(id)
                {
                    folder.order = order;
                }
            }
            Op::FolderDelete { id } => {
                // What the folder held is only away because the folder said so. Losing the folder
                // would let all of it back out at once, so the mark is written down before the
                // anchor goes — here, where a delete arriving from another machine lands too.
                let away = self.folder_away(*id);
                let gone = self.named_trail(*id);
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
                        folder.archived = folder.archived || away;
                    }
                }
                for doc in self.docs.values_mut() {
                    if doc.folder == Some(*id) {
                        doc.folder = None;
                        doc.archived = doc.archived || (away && doc.page_of.is_none());
                        if doc.archived && doc.page_of.is_none() {
                            doc.folder_was = gone.clone();
                        }
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
                        title: d.said.as_ref().map(|one| one.title.clone()),
                        bytes: d.said.as_ref().and_then(|one| one.bytes),
                        wrote: Some(d.wrote.or(d.made).unwrap_or(event.timestamp)),
                        made: Some(d.made.unwrap_or(event.timestamp)),
                        made_by: Some(event.device.clone()),
                        wrote_by: Some(event.device.clone()),
                        by: d.by.clone().or_else(|| here(d, &self.signed)),
                        born_by: d.by.clone().or_else(|| here(d, &self.signed)),
                        edited_by: d.said.as_ref().and_then(|one| one.by.clone()),
                        guest: d.guest,
                        folder: match under {
                            Some(one) => one.folder,
                            None => d.folder,
                        },
                        page_of,
                        archived: page_of.is_none() && under.is_some_and(|one| one.archived),
                        locked: false,
                        tags: crate::tagging::worth_keeping(
                            &d.said
                                .as_ref()
                                .and_then(|one| one.tags.clone())
                                .unwrap_or_default(),
                        ),
                        flagged: None,
                        folder_was: None,
                    },
                );
            }
            Op::DocSaid { id, d } => {
                if let Some(kept) = self.docs.get_mut(id) {
                    kept.title = Some(d.title.clone());
                    kept.bytes = d.bytes;
                    // A note from a build that never read tags says nothing about them.
                    if let Some(tags) = &d.tags {
                        kept.tags = crate::tagging::worth_keeping(tags);
                    }
                    kept.wrote = Some(event.timestamp);
                    kept.wrote_by = Some(event.device.clone());
                    // A note with no hand on it says nothing about whose it was, which is not
                    // the same as saying nobody's: settling a body read from disk must not wipe
                    // the name the machine that wrote it put there.
                    if let Some(by) = &d.by {
                        kept.edited_by = Some(by.clone());
                    }
                }
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
                            .map(|one| one.folder);
                        // Only the cover it is walking out of: a folder above it holds it
                        // just the same once it is a document of its own.
                        let leaving = allowed.is_none()
                            && self.docs.get(id).is_some_and(|one| {
                                !one.archived
                                    && !one.folder.is_some_and(|at| self.folder_away(at))
                                    && self.held_away(one)
                            });
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
                            if let Some(folder) = under {
                                doc.folder = folder;
                                doc.archived = false;
                                doc.folder_was = None;
                                doc.flagged = None;
                                doc.locked = false;
                                doc.order = beside.unwrap_or_else(|| doc.order.clone());
                            }
                            if leaving {
                                doc.archived = true;
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
                    doc.folder_was = None;
                }
                // Pages live where their document lives, and follow it without being told.
                if let Some(under) = self.docs.get(id).filter(|one| one.page_of.is_none()) {
                    let (parent, folder) = (under.id, under.folder);
                    for one in self.docs.values_mut() {
                        if one.page_of == Some(parent) {
                            one.folder = folder;
                            one.folder_was = None;
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
            Op::DocSigned { id, d } => {
                if let Some(kept) = self.docs.get_mut(id) {
                    // Held here and not only where the event is written, so a build that read
                    // the rule differently cannot sign over somebody else's writing. What
                    // nobody signed is another matter: signing it is owning it.
                    if kept.guest && kept.by.is_some() {
                        return;
                    }
                    let said = d.trim();
                    let now = (!said.is_empty()).then(|| said.to_string());
                    if kept.born_by.is_none() {
                        kept.born_by.clone_from(&now);
                    }
                    kept.guest = false;
                    kept.by = now;
                }
            }
            Op::FolderArchive { id } => self.shelf(*id, true),
            Op::FolderUnarchive { id } => self.shelf(*id, false),
            Op::DocArchive { id } => {
                let by_person = !self.assistants.contains(&event.device);
                self.shelve(*id, true, by_person)
            }
            Op::DocLock { id } => self.bolt(*id, true),
            Op::DocUnlock { id } => self.bolt(*id, false),
            Op::DocUnarchive { id } => self.shelve(*id, false, true),
            Op::DocFlag { id, d } => {
                let said = crate::model::Flagged {
                    at: d.at.unwrap_or(event.timestamp),
                    by: d.by.clone().unwrap_or_else(|| event.device.clone()),
                    body: d.body.clone(),
                    via: d.via.clone().or_else(|| event.via.clone()),
                };
                if let Some(doc) = self.docs.get_mut(id) {
                    doc.flagged = Some(said);
                }
            }
            Op::DocUnflag { id } => {
                if let Some(doc) = self.docs.get_mut(id) {
                    doc.flagged = None;
                }
            }
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
            // Self-declared like `k`, or declared by the machine that hosts it: nobody else's word.
            Op::DeviceHost { d, of } => {
                if (event.device == *d || event.device == *of)
                    && self.assistants.contains(d)
                    && !self.assistants.contains(of)
                {
                    self.hosts.insert(d.clone(), of.clone());
                }
            }
            Op::Signed { d } => {
                let said = |one: &Option<String>| {
                    one.as_deref()
                        .map(str::trim)
                        .filter(|one| !one.is_empty())
                        .map(str::to_string)
                };
                self.signed = crate::event::Signature {
                    alias: said(&d.alias),
                    name: said(&d.name),
                    email: said(&d.email),
                };
                // Kept in the order they were signed, and never thinned: going back to a
                // name years later must not move it ahead of whoever was first.
                if let Some(one) = &self.signed.alias
                    && !self
                        .signed_before
                        .last()
                        .is_some_and(|was| same_name(was, one))
                {
                    self.signed_before.push(one.clone());
                }
            }
            Op::DeviceRemove { d } => {
                self.devices.remove(d);
                self.agents.remove(d);
                self.hosts.remove(d);
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
        if repeat.cadence().every == 0 {
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
        // Borrowed, not cloned: sort_by asks for the key on both sides of every comparison, and
        // the tally alone sorts this list thirteen times for one snapshot.
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
            .chain(self.docs.values().flat_map(|one| &one.tags))
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
            // Mine to re-sign is what a hand of mine signed, or what nobody ever signed at all.
            // Another name is another person, whether it arrived in a parcel or through a store
            // two people share.
            .filter(|one| match one.by.as_deref() {
                Some(by) => !one.guest && self.mine_to_write(by),
                None => true,
            })
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

    fn mine_to_write(&self, hand: &str) -> bool {
        alike(Some(hand), self.signed.alias.as_deref())
            || self.signed_before.iter().any(|was| same_name(was, hand))
    }

    pub fn docs_tagged(&self, tag: &Tag) -> impl Iterator<Item = &crate::model::Kept> {
        self.docs
            .values()
            .filter(move |one| !self.held_away(one) && one.tags.contains(tag))
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
