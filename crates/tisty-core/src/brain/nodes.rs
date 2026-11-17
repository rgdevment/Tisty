use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::Docs;
use super::linked::{Ground, Holder};
use crate::model::Kept;
use crate::{State, Status, Task};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Kind {
    Doc,
    Page,
    Task,
    Tag,
    File,
}

impl Kind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Kind::Doc => "doc",
            Kind::Page => "page",
            Kind::Task => "task",
            Kind::Tag => "tag",
            Kind::File => "file",
        }
    }
}

pub(super) struct Node<'a> {
    pub(super) kind: Kind,
    pub(super) title: String,
    pub(super) group: Option<String>,
    pub(super) standing: Option<&'static str>,
    pub(super) uid: Option<String>,
    pub(super) stamp: String,
    pub(super) holder: Holder<'a>,
}

pub(super) fn nodes<'a>(
    state: &'a State,
    docs: &Docs,
    ground: &Ground,
) -> BTreeMap<String, Node<'a>> {
    let mut all = BTreeMap::new();
    for kept in state.papers() {
        let group = group_of_doc(state, kept);
        let standing = if state.held_away(kept) {
            "archived"
        } else {
            "open"
        };
        let told = serde_json::to_string(kept).unwrap_or_default();
        all.insert(
            kept.file.clone(),
            Node {
                kind: match kept.page_of {
                    Some(_) => Kind::Page,
                    None => Kind::Doc,
                },
                title: kept.title.clone().unwrap_or_else(|| kept.file.clone()),
                stamp: stamped(&[
                    &told,
                    docs.told(&kept.file),
                    group.as_deref().unwrap_or_default(),
                    standing,
                    ground.sheet(),
                    &ground.leaned_on(kept),
                ]),
                group,
                standing: Some(standing),
                uid: Some(kept.id.to_string()),
                holder: Holder::Doc(kept),
            },
        );
    }
    for task in state.tasks.values() {
        let group = task.list.map(|list| list.to_string());
        let standing = standing_of_task(task);
        let told = serde_json::to_string(task).unwrap_or_default();
        let mut parts = vec![
            told.as_str(),
            group.as_deref().unwrap_or_default(),
            standing,
            ground.sheet(),
        ];
        parts.extend(ground.pages_told(task));
        all.insert(
            task.id.to_string(),
            Node {
                kind: Kind::Task,
                title: task.title.clone(),
                stamp: stamped(&parts),
                group,
                standing: Some(standing),
                uid: None,
                holder: Holder::Task(task),
            },
        );
    }
    for tag in state.tags() {
        all.insert(
            format!("#{tag}"),
            Node {
                kind: Kind::Tag,
                title: tag.as_str().to_string(),
                group: None,
                standing: None,
                uid: None,
                stamp: String::new(),
                holder: Holder::Nothing,
            },
        );
    }
    for (at, (sha256, _)) in &state.kept {
        all.insert(
            at.clone(),
            Node {
                kind: Kind::File,
                title: at.rsplit('/').next().unwrap_or(at).to_string(),
                group: None,
                standing: None,
                uid: None,
                stamp: sha256.clone(),
                holder: Holder::Nothing,
            },
        );
    }
    all
}

// A page lives in its book's folder whatever its own record says.
fn group_of_doc(state: &State, kept: &Kept) -> Option<String> {
    let book = kept.page_of.and_then(|up| state.docs.get(&up));
    book.map_or(kept.folder, |book| book.folder.or(kept.folder))
        .map(|folder| folder.to_string())
}

fn standing_of_task(task: &Task) -> &'static str {
    match (task.status, &task.resolved) {
        (Status::Done, _) => "done",
        (Status::Dropped, _) => "dropped",
        (Status::Open, Some(said)) if said.drop => "said_dropped",
        (Status::Open, Some(_)) => "said_done",
        (Status::Open, None) => "open",
    }
}

fn stamped(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    crate::signing::hexed(&hasher.finalize())
}
