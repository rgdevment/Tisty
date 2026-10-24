use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ulid::Ulid;

use crate::event::{DeviceKind, DocAdd, Event, FolderAdd, ListAdd, Look, Op, Said};
use crate::model::{DocId, FIRST, FolderId, ListId};

pub const SOWN_BY: &str = "Tisty";
pub const GUIDE_FOLDERS: [&str; 2] = ["Guide", "Guía"];

// Normalised prints of every guide text a release sowed without noting a print (up to 1.24.4).
const SHIPPED_BEFORE_PRINTS: [&str; 36] = [
    "b96e688d2658b0a2c03fa9d57d5cd5aa64fcff5b2c870f4e2d1705b35926ff3c",
    "9125340e76db2fd685b94c0ccbea99771002a455c422bd41bca48bdebf0fcaf5",
    "1fda56168f6547c9aa7fb65cbddd64c2b86e0591544a8627f302bcb30645de89",
    "251b72c8dfa135272859617278e0dc9db47aaada1ccf84047fc326b550db47cf",
    "9b8ed0c587cfb8b2c25df33a144eec071b556daa1b3c8d193474e09aeda7bbb4",
    "fac76f374dcdf6769a984383c2b9dea04f05aa01a8ebe5877ce38c907bfd82e6",
    "7da6360ec2075ace42385f3df21a7ef128b5719ad7b03f92d0404fc98c28daf0",
    "a864ce7f44d02e4268fbe9d8b0b04da451a30225d18d6bd69fb52f9f8477b4ae",
    "0aefe2557979cd3fefee8d74d654fc4c4f637b50b38d560d4c8a80858fcb25f5",
    "3343f5ed1422efdc3957b75209d92f57f876c19c0cdd50b3b2546deb3bef8ec9",
    "e7eb197014235ca0b1feafb0ee38df58ce799279a6a09561b64a4846404bfb9b",
    "c869f57b7b1dd1957c28f984ae9e7de3e9666f83291f53c4cb0b56436d22cb0d",
    "e743de142809b3da3c258f5e527cdeff9028bd944450ea34797f79cfc98faa40",
    "fc816c649adbec89f8cd51f872343a6e9017bf6c5593b25cc2fb30807540b7e9",
    "fbabc1b7ea53d0da626f32448615aad92fece4ea1b9af51be0db4a448060d94c",
    "0e03a9cbd845f0973add1a118c7dcd6b9edad857ae635d0cd572d9665b36b614",
    "f2e1ca2001abc5ff7b448b2a1060e5682a2ad273274e739c6e51514f9625336f",
    "a48eeb90b9d4d54410aa07a6977ca18610572e9cdec6fcbfc1d3b5d815bc7d4f",
    "462798a300b749e2a595e8129372ec4809e7cc4dc5364d90274797114f000911",
    "cc33ad568eaaa69f097b639bb2e8800963d872edb4d89324e9e19c5c36527010",
    "6c72768b58d96b3e686f4bd49482448452aa93216f0423e76b33663d588c377b",
    "91db46378f4ac9d60d55c36497568eac6f7a3ab9fa3aedbf4f7b022af800cebc",
    "12a1aaf6d0280ac2c52548981aafdbcec7003796e05bd2c6b0e2e91fb97ebdb0",
    "6359091e76ce25e35aabedcf094456a784434c74c997829e5104a30a65f0b72e",
    "ca8b8ba7d8127f7e47b751e2d2c793ac2e24920c6325c7d3a38733006b14d9db",
    "1babee483d24cf16fcb6cced30c99aa29de0a5b5b98077c08944cce076a85e4a",
    "bb2f5d2a28562373bc8ed21f3508b198ec146ee2379c6ebdd68c9b9c42283ab5",
    "8997fcdc034b4c8cec16291e3a42c240bbca45b5045ad3597d4ae509027805d2",
    "abd91b6d3e76ac3d4da2cf686dc5581859b69e604aa5c2e8d277f1f32aadd895",
    "89c973647dddd334ca88502ea2ff07b119dde32fc04db35d6d8e38ac98c93c03",
    "8b3811824624df7738a0c0fbfd7ca3e9d845529a65e8bef371ae79c48132a1b7",
    "18b51116d1cd220d7e8a45e7c6d90f792b3cb727c857d9ab2765a9dbcd1500b6",
    "645f711f37fd522ee49068dc94e824dd2f036350ced444561366c32e351efc77",
    "a2df77824e506cbd112d3a412a3a97c49ffddb12ba5e5f437fc6f72e7d649bd8",
    "2997a07c296464e6c2e36f89fb6ac65d63c6ab87a797c1e2461e56a69026b98e",
    "e7e2ae0a01de129efbce385ae10e852d81f1de3b36c3dd5f41f262c952d7b3e6",
];

struct Paper {
    file: String,
    title: Option<String>,
    sown_print: Option<String>,
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
            | Op::DeviceRelease { .. }
            // The seed writes these again by itself; a machine taken back is the person's word.
            | Op::ShareKey { .. }
            | Op::ShareCertified { .. }
            | Op::ShareRevoked { machine: false, .. }
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
        let sown_print = said.and_then(|one| one.print.clone());
        self.papers.insert(
            id,
            Paper {
                file: d.file.clone(),
                title: said.map(|one| one.title.clone()),
                prints: sown_print.iter().cloned().collect(),
                sown_print,
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
        let same = self.title.as_ref().is_none_or(|was| *was == now.title)
            && self
                .prints
                .iter()
                .next()
                .is_none_or(|was| now.print.as_ref() == Some(was));
        same && (self.sown_print.is_some() || shipped_before_prints(&body))
    }
}

fn shipped_before_prints(body: &str) -> bool {
    let print = crate::attach::printed(crate::docs::settled(&unlinked(body)).as_bytes());
    SHIPPED_BEFORE_PRINTS.contains(&print.as_str())
}

// A sown guide points at its pictures and pages by names only this machine gave them.
fn unlinked(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(at) = rest.find("](") {
        out.push_str(&rest[..at + 2]);
        rest = &rest[at + 2..];
        match rest.find(')') {
            Some(end) => rest = &rest[end..],
            None => break,
        }
    }
    out.push_str(rest);
    out.replace('\r', "")
}

#[cfg(test)]
#[path = "untouched_test.rs"]
mod tests;
