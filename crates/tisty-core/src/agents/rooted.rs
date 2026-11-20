use std::collections::BTreeSet;

use crate::State;
use crate::model::{FolderId, Kept};

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
    let mut all = BTreeSet::from([root]);
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

// An AGENTS not yet joined still guards what it holds.
pub fn holds_folder(state: &State, folder: FolderId) -> bool {
    roots(state)
        .into_iter()
        .any(|root| tree(state, root).contains(&folder))
}

// A page sits wherever the document it belongs to sits.
pub fn holds(state: &State, kept: &Kept) -> bool {
    let up = kept.page_of.and_then(|up| state.docs.get(&up));
    [Some(kept), up]
        .into_iter()
        .flatten()
        .any(|one| one.folder.is_some_and(|at| holds_folder(state, at)))
}
