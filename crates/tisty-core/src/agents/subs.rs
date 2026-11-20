use crate::State;
use crate::event::{FolderAdd, Op};
use crate::model::FolderId;

/// Every computer names a folder of AGENTS the same way, so two that make it before they sync make one.
pub fn sub_id(root: FolderId, name: &str) -> FolderId {
    let printed = crate::attach::printed(format!("{root}/{name}").as_bytes());
    let bits = u128::from_str_radix(&printed[..32], 16).unwrap_or_default();
    ulid::Ulid::from(bits)
}

pub fn sub_folders(state: &State, root: FolderId, names: &[&str]) -> Vec<Op> {
    let mut taken: Vec<String> = state
        .under(Some(root))
        .iter()
        .map(|one| one.order.clone())
        .collect();
    let mut ops = Vec::new();
    for name in names {
        let id = sub_id(root, name);
        let named = state
            .folders
            .values()
            .any(|one| one.parent == Some(root) && !one.archived && one.name == *name);
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
                parent: Some(root),
                icon: None,
                color: None,
            },
        });
    }
    ops
}

#[cfg(test)]
#[path = "subs_test.rs"]
mod tests;
