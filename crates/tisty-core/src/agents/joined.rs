use std::collections::BTreeMap;

use super::rooted::{roots, tree};
use crate::State;
use crate::event::{DeviceId, Filed, Op};
use crate::model::{Folder, FolderId};

type Landed = BTreeMap<(FolderId, String), FolderId>;

pub fn joined(state: &State) -> Vec<Op> {
    let all = roots(state);
    let Some((&keep, others)) = all.split_first() else {
        return Vec::new();
    };
    if !placed_by_person(state, keep) {
        return Vec::new();
    }
    let mut ops = gathered(state, keep);
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

// What a join lifted out, or what landed in a folder a join had already taken, goes to the folder of its name that stayed.
fn gathered(state: &State, keep: FolderId) -> Vec<Op> {
    let twins = derived_twins(state, keep);
    let mut late: Vec<_> = state
        .docs
        .values()
        .filter(|one| one.page_of.is_none() && one.task_of.is_none())
        .filter(|one| {
            one.placed_by
                .as_ref()
                .is_some_and(|who| !state.assistants.contains(who))
        })
        .filter_map(|one| left_behind(state, keep, &twins, one).map(|into| (one.id, into)))
        .collect();
    late.sort_unstable();
    late.into_iter()
        .map(|(id, into)| Op::DocMove {
            id,
            d: moved_to(into),
        })
        .collect()
}

fn dangling(state: &State, at: Option<FolderId>) -> Option<FolderId> {
    at.filter(|at| !state.folders.contains_key(at) && state.tombstones.contains(at))
}

// Worked out once a pass, and only while some document points at a folder that is gone.
fn derived_twins(state: &State, keep: FolderId) -> BTreeMap<FolderId, FolderId> {
    if !state
        .docs
        .values()
        .any(|one| dangling(state, one.folder).is_some())
    {
        return BTreeMap::new();
    }
    let children: Vec<_> = state
        .folders
        .values()
        .filter(|one| one.parent == Some(keep) && !one.archived)
        .collect();
    state
        .tombstones
        .iter()
        .flat_map(|gone| {
            children
                .iter()
                .map(move |one| (super::subs::sub_id(*gone, &one.name), one.id))
        })
        .collect()
}

fn left_behind(
    state: &State,
    keep: FolderId,
    twins: &BTreeMap<FolderId, FolderId>,
    kept: &crate::model::Kept,
) -> Option<FolderId> {
    if let Some(at) = dangling(state, kept.folder) {
        return twins.get(&at).copied();
    }
    match (kept.folder, &kept.lifted_from) {
        (None, Some((root, names))) if *root != keep && state.tombstones.contains(root) => {
            let name = names.get(1);
            Some(
                name.and_then(|name| live_child(state, keep, name))
                    .unwrap_or(keep),
            )
        }
        _ => None,
    }
}

fn live_child(state: &State, parent: FolderId, name: &str) -> Option<FolderId> {
    state
        .folders
        .values()
        .filter(|one| one.parent == Some(parent) && !one.archived && one.name == name)
        .map(|one| one.id)
        .min()
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
            false => twin(state, into, child, landed),
        };
        match twin {
            Some(twin) => {
                folded(state, child.id, twin, landed, ops);
                ops.push(Op::FolderDelete { id: child.id });
            }
            None => {
                if !child.archived && child.computer.is_none() {
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

// A computer's folder joins only its own other one, whatever either is called.
fn twin(state: &State, under: FolderId, child: &Folder, landed: &Landed) -> Option<FolderId> {
    let name = &child.name;
    state
        .folders
        .values()
        .filter(|one| one.parent == Some(under) && !one.archived)
        .filter(|one| match &child.computer {
            Some(_) => one.computer == child.computer,
            None => one.computer.is_none() && one.name == *name,
        })
        .map(|one| one.id)
        .min()
        .or_else(|| match &child.computer {
            Some(_) => None,
            None => landed.get(&(under, name.clone())).copied(),
        })
}

#[cfg(test)]
#[path = "joined_test.rs"]
mod tests;
