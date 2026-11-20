use std::collections::BTreeMap;

use super::rooted::{roots, tree};
use crate::State;
use crate::event::{DeviceId, Filed, Op};
use crate::model::FolderId;

type Landed = BTreeMap<(FolderId, String), FolderId>;

pub fn joined(state: &State) -> Vec<Op> {
    let all = roots(state);
    let Some((&keep, others)) = all.split_first() else {
        return Vec::new();
    };
    if !placed_by_person(state, keep) {
        return Vec::new();
    }
    let mut ops = Vec::new();
    for &other in others {
        if !all_by_person(state, other) {
            crate::witness::warn(
                crate::witness::channel::AGENT,
                "a second AGENTS holds something an agent placed, so it is left as it is",
                &[("at", crate::witness::Fact::Id(other.to_string()))],
            );
            continue;
        }
        folded(state, other, keep, &mut Landed::new(), &mut ops);
        ops.push(Op::FolderDelete { id: other });
    }
    ops
}

// A move stamps the mover as whoever placed it, so what an agent placed would come out the person's.
fn all_by_person(state: &State, root: FolderId) -> bool {
    let person = |who: Option<&DeviceId>| who.is_some_and(|who| !state.assistants.contains(who));
    let held = tree(state, root);
    let folders = held.iter().all(|at| {
        state
            .folders
            .get(at)
            .is_none_or(|one| person(one.placed_by.as_ref()))
    });
    folders
        && state
            .docs
            .values()
            .filter(|one| one.folder.is_some_and(|at| held.contains(&at)))
            .all(|one| person(one.placed_by.as_ref()))
}

fn placed_by_person(state: &State, folder: FolderId) -> bool {
    state
        .folders
        .get(&folder)
        .and_then(|one| one.placed_by.as_ref())
        .is_some_and(|who| !state.assistants.contains(who))
}

fn moved_to(into: FolderId) -> Filed {
    Filed {
        folder: Some(Some(into)),
        page_of: None,
        order: None,
    }
}

fn folded(state: &State, from: FolderId, into: FolderId, landed: &mut Landed, ops: &mut Vec<Op>) {
    let mut docs: Vec<_> = state
        .docs
        .values()
        .filter(|one| one.folder == Some(from) && one.page_of.is_none() && one.task_of.is_none())
        .map(|one| one.id)
        .collect();
    docs.sort_unstable();
    ops.extend(docs.into_iter().map(|id| Op::DocMove {
        id,
        d: moved_to(into),
    }));
    let mut children: Vec<_> = state
        .folders
        .values()
        .filter(|one| one.parent == Some(from))
        .collect();
    children.sort_unstable_by_key(|one| one.id);
    for child in children {
        // An archived folder never joins a live one, or what it holds away would come back.
        let twin = match child.archived {
            true => None,
            false => twin(state, into, &child.name, landed),
        };
        match twin {
            Some(twin) => {
                folded(state, child.id, twin, landed, ops);
                ops.push(Op::FolderDelete { id: child.id });
            }
            None => {
                if !child.archived {
                    landed.insert((into, child.name.clone()), child.id);
                }
                ops.push(Op::FolderMove {
                    id: child.id,
                    d: moved_to(into),
                });
            }
        }
    }
}

fn twin(state: &State, under: FolderId, name: &str, landed: &Landed) -> Option<FolderId> {
    state
        .folders
        .values()
        .filter(|one| one.parent == Some(under) && !one.archived && one.name == name)
        .map(|one| one.id)
        .min()
        .or_else(|| landed.get(&(under, name.to_string())).copied())
}

#[cfg(test)]
#[path = "joined_test.rs"]
mod tests;
