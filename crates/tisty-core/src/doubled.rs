use std::collections::BTreeMap;

use crate::State;
use crate::event::{Op, TaskMove};
use crate::model::{DocId, ListId};
use crate::state::same_name;

const GUIDE_BY: &str = "Tisty";

/// What a merge of two histories leaves twice: lists of one name, and the guide each side brought.
pub fn doubled(state: &State) -> Vec<Op> {
    let mut ops = Vec::new();
    for group in lists_named_alike(state) {
        let keep = group[0];
        for gone in &group[1..] {
            for task in state.tasks.values().filter(|one| one.list == Some(*gone)) {
                ops.push(Op::TaskMove {
                    id: task.id,
                    d: TaskMove {
                        list: Some(Some(keep)),
                        order: None,
                        part_of: None,
                    },
                });
            }
            ops.push(Op::ListDelete { id: *gone });
        }
    }
    for id in guides_twice(state) {
        ops.push(Op::DocArchive { id });
    }
    ops
}

// The list that keeps the name is the one with the most in it, then the one that came first.
fn lists_named_alike(state: &State) -> Vec<Vec<ListId>> {
    let held = |id: &ListId| {
        state
            .tasks
            .values()
            .filter(|one| one.list == Some(*id))
            .count()
    };
    let mut groups: Vec<Vec<ListId>> = Vec::new();
    let mut alive: Vec<_> = state.lists.values().filter(|one| !one.archived).collect();
    alive.sort_by(|a, b| {
        held(&b.id)
            .cmp(&held(&a.id))
            .then(a.order.cmp(&b.order))
            .then(a.id.cmp(&b.id))
    });
    for one in alive {
        match groups
            .iter_mut()
            .find(|group| same_name(&state.lists[&group[0]].name, &one.name))
        {
            Some(group) => group.push(one.id),
            None => groups.push(vec![one.id]),
        }
    }
    groups.retain(|group| group.len() > 1);
    groups
}

fn guides_twice(state: &State) -> Vec<DocId> {
    let mut first: BTreeMap<String, DocId> = BTreeMap::new();
    let mut later = Vec::new();
    let mut guides: Vec<_> = state
        .docs
        .values()
        .filter(|one| one.guest && !one.archived && one.by.as_deref() == Some(GUIDE_BY))
        .collect();
    guides.sort_by(|a, b| a.made.cmp(&b.made).then(a.file.cmp(&b.file)));
    for one in guides {
        let title = crate::text::folded(one.title.as_deref().unwrap_or_default().trim());
        if first.contains_key(&title) {
            later.push(one.id);
        } else {
            first.insert(title, one.id);
        }
    }
    later
}

#[cfg(test)]
#[path = "doubled_test.rs"]
mod tests;
