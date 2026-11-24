use std::collections::BTreeSet;

use crate::State;
use crate::event::{DeviceId, Op};
use crate::model::{Folder, FolderId, Kept};

pub const FOLDER: &str = "AGENTS";
pub const ICON: &str = "robot";

/// Every AGENTS at the top, oldest first: two computers that made one before they synced hold two until they join.
pub fn roots(state: &State) -> Vec<FolderId> {
    let mut all: Vec<FolderId> = state
        .folders
        .values()
        .filter(|one| {
            one.parent.is_none()
                && !one.archived
                && one.icon.as_deref() == Some(ICON)
                && one.name == FOLDER
        })
        .map(|one| one.id)
        .collect();
    all.sort_unstable();
    all
}

pub fn root_of(state: &State) -> Option<FolderId> {
    roots(state).first().copied()
}

pub fn tree(state: &State, root: FolderId) -> BTreeSet<FolderId> {
    grown(state, BTreeSet::from([root]))
}

/// The computer whose own files this folder holds: the one named by the folder right under AGENTS it sits in.
pub fn computer_of(state: &State, folder: Option<FolderId>) -> Option<DeviceId> {
    let roots = roots(state);
    let mut at = folder;
    for _ in 0..=crate::model::DEEPEST {
        let one = state.folders.get(&at?)?;
        if one.parent.is_some_and(|up| roots.contains(&up)) {
            return one.computer.clone();
        }
        at = one.parent;
    }
    None
}

/// Whether a folder right under AGENTS is this computer's, whatever it is called.
pub fn of_computer(one: &Folder, root: FolderId, computer: &DeviceId) -> bool {
    one.parent == Some(root) && !one.archived && one.computer.as_ref() == Some(computer)
}

// An AGENTS not yet joined still guards what it holds.
pub fn trees(state: &State) -> BTreeSet<FolderId> {
    grown(state, roots(state).into_iter().collect())
}

fn grown(state: &State, mut all: BTreeSet<FolderId>) -> BTreeSet<FolderId> {
    loop {
        let more: Vec<FolderId> = state
            .folders
            .values()
            .filter(|one| one.parent.is_some_and(|up| all.contains(&up)) && !all.contains(&one.id))
            .map(|one| one.id)
            .collect();
        if more.is_empty() {
            return all;
        }
        all.extend(more);
    }
}

pub fn holds_folder(state: &State, folder: FolderId) -> bool {
    trees(state).contains(&folder)
}

pub fn holds(state: &State, kept: &Kept) -> bool {
    holding(state)(kept)
}

// A page sits wherever the document it belongs to sits.
pub fn holding(state: &State) -> impl Fn(&Kept) -> bool + '_ {
    let tree = trees(state);
    move |kept| {
        let up = kept.page_of.and_then(|up| state.docs.get(&up));
        [Some(kept), up]
            .into_iter()
            .flatten()
            .any(|one| one.folder.is_some_and(|at| tree.contains(&at)))
    }
}

// Off the top, renamed, re-iconed, archived, made again or deleted while it holds anything, AGENTS would stop guarding what it holds.
pub fn unhooks(state: &State, op: &Op) -> bool {
    let root = |id: &FolderId| roots(state).contains(id);
    match op {
        Op::FolderAdd { id, .. } | Op::FolderArchive { id } => root(id),
        Op::FolderMove { id, d } => root(id) && matches!(d.folder, Some(Some(_))),
        Op::FolderRename { id, d } => root(id) && crate::text::plainly(&d.name) != FOLDER,
        Op::FolderLook { id, d } => {
            root(id)
                && d.icon
                    .as_ref()
                    .is_some_and(|icon| icon.as_deref() != Some(ICON))
        }
        Op::FolderDelete { id } => root(id) && holds_anything(state, *id),
        _ => false,
    }
}

fn holds_anything(state: &State, folder: FolderId) -> bool {
    state.folders.values().any(|one| one.parent == Some(folder))
        || state.docs.values().any(|one| one.folder == Some(folder))
}

#[cfg(test)]
#[path = "rooted_test.rs"]
mod tests;
