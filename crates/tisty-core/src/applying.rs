use crate::{
    State,
    event::{DeviceId, Event, Filed, Said, Signature},
    model::{DocId, Folder, FolderId, Kept, Reading, TaskId},
    state::{here, same_name},
};

impl State {
    pub(crate) fn folder_added(&mut self, id: &FolderId, d: &crate::event::FolderAdd) {
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

    pub(crate) fn task_resolved(&mut self, event: &Event, id: &TaskId, d: &crate::event::Resolve) {
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

    pub(crate) fn device_removed(&mut self, d: &DeviceId) {
        self.devices.remove(d);
        self.agents.remove(d);
        self.hosts.remove(d);
        self.dropped.insert(d.clone());
    }

    pub(crate) fn store_signed(&mut self, d: &Signature) {
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

    pub(crate) fn device_joined(
        &mut self,
        event: &Event,
        d: &DeviceId,
        k: &Option<crate::event::DeviceKind>,
    ) {
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

    pub(crate) fn folder_deleted(&mut self, id: &FolderId) {
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

    pub(crate) fn folder_moved(&mut self, id: &FolderId, d: &Filed) {
        let landed = match d.folder {
            None => true,
            Some(parent) => {
                let room = parent.is_none_or(|at| self.has_room_under(at))
                    && !self.would_loop(*id, parent)
                    && self.depth(parent) + self.tallest_under(*id) <= crate::model::DEEPEST;
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

    pub(crate) fn task_deleted(&mut self, event: &Event, id: &TaskId) {
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

    pub(crate) fn doc_signed(&mut self, id: &DocId, d: &str) {
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

    pub(crate) fn doc_deleted(&mut self, id: &DocId) {
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

    pub(crate) fn doc_said(&mut self, event: &Event, id: &DocId, d: &Said) {
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

    pub(crate) fn doc_added(&mut self, event: &Event, id: &DocId, d: &crate::event::DocAdd) {
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

    pub(crate) fn doc_moved(&mut self, id: &DocId, d: &Filed) {
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
}
