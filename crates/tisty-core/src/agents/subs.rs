use crate::State;
use crate::event::{DeviceId, FolderAdd, Op};
use crate::model::{FOLDER_NAME_AT_MOST, FolderId};

const APART: &str = " \u{b7} ";

/// Every computer names a folder of AGENTS the same way, so two that make it before they sync make one.
pub fn sub_id(root: FolderId, name: &str) -> FolderId {
    let printed = crate::attach::printed(format!("{root}/{name}").as_bytes());
    let bits = u128::from_str_radix(&printed[..32], 16).unwrap_or_default();
    ulid::Ulid::from(bits)
}

pub fn sub_folders(state: &State, parent: FolderId, names: &[&str]) -> Vec<Op> {
    let mut taken = orders(state, parent);
    let mut ops = Vec::new();
    for name in names {
        let id = sub_id(parent, name);
        let named = state
            .folders
            .values()
            .any(|one| one.parent == Some(parent) && !one.archived && one.name == *name);
        if named || state.folders.contains_key(&id) || state.tombstones.contains(&id) {
            continue;
        }
        let order = crate::order::last_of(taken.iter().map(String::as_str));
        taken.push(order.clone());
        ops.push(Op::FolderAdd {
            id,
            d: FolderAdd {
                name: (*name).to_string(),
                order,
                parent: Some(parent),
                icon: None,
                color: None,
                computer: None,
            },
        });
    }
    ops
}

/// The folder right under AGENTS that holds this computer's own agent files, and what makes it when it is not there.
pub fn computer_folder(state: &State, root: FolderId, me: &DeviceId) -> (FolderId, Option<Op>) {
    let here = state
        .folders
        .values()
        .filter(|one| super::of_computer(one, root, me))
        .map(|one| one.id)
        .min();
    if let Some(id) = here {
        return (id, None);
    }
    let id = ulid::Ulid::generate();
    let taken = orders(state, root);
    let made = Op::FolderAdd {
        id,
        d: FolderAdd {
            name: named(&state.called(me), me),
            order: crate::order::last_of(taken.iter().map(String::as_str)),
            parent: Some(root),
            icon: None,
            color: None,
            computer: Some(me.clone()),
        },
    };
    (id, Some(made))
}

/// The folder by this name under `parent`, made in `ops` when neither the state nor they have it.
pub fn room(state: &State, ops: &mut Vec<Op>, parent: FolderId, name: &str) -> FolderId {
    let there = state
        .folders
        .values()
        .filter(|one| one.parent == Some(parent) && !one.archived && one.name == name)
        .map(|one| one.id)
        .min();
    let made = ops.iter().find_map(|op| match op {
        Op::FolderAdd { id, d } if d.parent == Some(parent) && d.name == name => Some(*id),
        _ => None,
    });
    if let Some(id) = there.or(made) {
        return id;
    }
    let derived = sub_id(parent, name);
    let used = state.folders.contains_key(&derived) || state.tombstones.contains(&derived);
    let id = if used {
        ulid::Ulid::generate()
    } else {
        derived
    };
    let mut taken = orders(state, parent);
    taken.extend(ops.iter().filter_map(|op| match op {
        Op::FolderAdd { d, .. } if d.parent == Some(parent) => Some(d.order.clone()),
        _ => None,
    }));
    ops.push(Op::FolderAdd {
        id,
        d: FolderAdd {
            name: name.to_string(),
            order: crate::order::last_of(taken.iter().map(String::as_str)),
            parent: Some(parent),
            icon: None,
            color: None,
            computer: None,
        },
    });
    id
}

// Two computers may share a name, and an earlier Tisty joins folders by theirs.
fn named(called: &str, me: &DeviceId) -> String {
    let id = me.0.strip_prefix("dev_").unwrap_or(&me.0);
    let id: String = id.chars().take(FOLDER_NAME_AT_MOST / 2).collect();
    let room = FOLDER_NAME_AT_MOST - APART.chars().count() - id.chars().count();
    let called: String = crate::text::plainly(called)
        .replace('/', "-")
        .chars()
        .take(room)
        .collect();
    format!("{}{APART}{id}", called.trim())
}

fn orders(state: &State, parent: FolderId) -> Vec<String> {
    state
        .under(Some(parent))
        .iter()
        .map(|one| one.order.clone())
        .collect()
}

#[cfg(test)]
#[path = "subs_test.rs"]
mod tests;
