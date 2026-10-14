use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ulid::Ulid;

use crate::event::{DeviceKind, DocAdd, Event, FolderAdd, ListAdd, Look, Op, Said};
use crate::model::{DocId, FIRST, FolderId, ListId};

pub const SOWN_BY: &str = "Tisty";
pub const GUIDE_FOLDERS: [&str; 2] = ["Guide", "Guía"];

struct Paper {
    file: String,
    title: Option<String>,
    prints: BTreeSet<String>,
}

#[derive(Default)]
struct Sown {
    agents: BTreeSet<String>,
    guide_batches: BTreeSet<Ulid>,
    lists: BTreeSet<ListId>,
    folders: BTreeSet<FolderId>,
    papers: BTreeMap<DocId, Paper>,
}

/// Holds nothing the person made: every event is one a first opening writes on its own, and
/// every document on disk is a starter guide that still reads as it did when it was written.
pub fn untouched(store: &Path, docs: &Path) -> bool {
    let Ok(mut events) = crate::store::read_all(store) else {
        return false;
    };
    events.sort_by(|one, other| one.sort_key().cmp(&other.sort_key()));
    sown_only(&events).is_some_and(|sown| sown.still_on(docs))
}

fn sown_only(events: &[Event]) -> Option<Sown> {
    let mut sown = Sown {
        agents: agents_in(events),
        guide_batches: events
            .iter()
            .filter(|one| matches!(&one.op, Op::FolderAdd { d, .. } if a_guide_folder(d)))
            .filter_map(|one| one.batch)
            .collect(),
        ..Sown::default()
    };
    for event in events {
        if !sown.takes(event) {
            return None;
        }
    }
    sown.papers
        .values()
        .all(|paper| paper.prints.len() <= 1)
        .then_some(sown)
}

fn agents_in(events: &[Event]) -> BTreeSet<String> {
    events
        .iter()
        .filter_map(|one| match &one.op {
            Op::DeviceJoin {
                d,
                k: Some(DeviceKind::Agent),
                ..
            } => Some(d.0.clone()),
            _ => None,
        })
        .collect()
}

fn a_guide_folder(d: &FolderAdd) -> bool {
    GUIDE_FOLDERS.contains(&d.name.as_str())
        && d.parent.is_none()
        && d.icon.is_none()
        && d.color.is_none()
}

fn a_first_list(d: &ListAdd) -> bool {
    d.color.is_none()
        && FIRST
            .iter()
            .any(|(english, castilian, _)| d.name == *english || d.name == *castilian)
}

fn a_first_look(d: &Look) -> bool {
    let icon = d.icon.clone().flatten();
    d.color.clone().flatten().is_none()
        && FIRST
            .iter()
            .any(|(_, _, named)| icon.as_deref() == Some(*named))
}

impl Sown {
    fn takes(&mut self, event: &Event) -> bool {
        if event.undo || event.redo {
            return false;
        }
        match &event.op {
            Op::DeviceJoin { .. }
            | Op::DeviceKey { .. }
            | Op::DeviceRotate { .. }
            | Op::DeviceHost { .. }
            | Op::DeviceNamed { .. }
            | Op::Signed { .. } => true,
            _ if self.agents.contains(&event.device.0) => false,
            Op::ListAdd { id, d } => a_first_list(d) && self.lists.insert(*id),
            Op::ListLook { id, d } => self.lists.contains(id) && a_first_look(d),
            Op::FolderAdd { id, d } => a_guide_folder(d) && self.folders.insert(*id),
            Op::AttachKept { .. } => event
                .batch
                .is_some_and(|batch| self.guide_batches.contains(&batch)),
            Op::DocAdd { id, d } => self.paper_added(*id, d),
            Op::DocSaid { id, d } => self.paper_read(id, d),
            _ => false,
        }
    }

    fn paper_added(&mut self, id: DocId, d: &DocAdd) -> bool {
        let sown = d.by.as_deref() == Some(SOWN_BY)
            && d.guest
            && d.made.is_none()
            && d.wrote.is_none()
            && d.folder
                .is_some_and(|folder| self.folders.contains(&folder))
            && d.page_of.is_none_or(|up| self.papers.contains_key(&up));
        if !sown || self.papers.contains_key(&id) {
            return false;
        }
        let said = d.said.as_ref();
        self.papers.insert(
            id,
            Paper {
                file: d.file.clone(),
                title: said.map(|one| one.title.clone()),
                prints: said.and_then(|one| one.print.clone()).into_iter().collect(),
            },
        );
        true
    }

    // Read back, never written: no hand on it, and the same title the guide was sown with.
    fn paper_read(&mut self, id: &DocId, d: &Said) -> bool {
        let Some(paper) = self.papers.get_mut(id) else {
            return false;
        };
        if d.by.is_some() || paper.title.as_ref().is_some_and(|was| *was != d.title) {
            return false;
        }
        paper.prints.extend(d.print.clone());
        true
    }

    fn still_on(&self, docs: &Path) -> bool {
        let files: BTreeSet<&str> = self.papers.values().map(|one| one.file.as_str()).collect();
        crate::docs::names(docs)
            .iter()
            .all(|one| files.contains(one.as_str()))
            && self.papers.values().all(|paper| paper.reads_as_sown(docs))
    }
}

impl Paper {
    fn reads_as_sown(&self, docs: &Path) -> bool {
        let Ok(at) = crate::docs::resolve(docs, &self.file) else {
            return false;
        };
        if !at.exists() {
            return true;
        }
        let Ok(body) = crate::docs::read(docs, &self.file) else {
            return false;
        };
        let now = Said::of(&body);
        self.title.as_ref().is_none_or(|was| *was == now.title)
            && self
                .prints
                .iter()
                .next()
                .is_none_or(|was| now.print.as_ref() == Some(was))
    }
}

#[cfg(test)]
#[path = "untouched_test.rs"]
mod tests;
