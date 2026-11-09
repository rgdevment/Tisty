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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Repeated {
    pub name: String,
    pub lists: usize,
    pub tasks: usize,
}

/// The lists `doubled` would put together, named as the one that keeps the name.
pub fn repeated(state: &State) -> Vec<Repeated> {
    lists_named_alike(state)
        .into_iter()
        .map(|group| Repeated {
            name: state.lists[&group[0]].name.clone(),
            lists: group.len(),
            tasks: state
                .tasks
                .values()
                .filter(|one| one.list.is_some_and(|list| group.contains(&list)))
                .count(),
        })
        .collect()
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
    let mut first: BTreeMap<(String, Option<String>), DocId> = BTreeMap::new();
    let mut later = Vec::new();
    let mut guides: Vec<_> = state
        .docs
        .values()
        .filter(|one| {
            one.guest && !one.archived && (one.sown || one.by.as_deref() == Some(GUIDE_BY))
        })
        .collect();
    // A guide its author signed outlasts one from before, whichever came first.
    let unsigned = |one: &&crate::model::Kept| one.by.as_deref() == Some(GUIDE_BY);
    guides.sort_by(|a, b| (unsigned(a), a.made, &a.file).cmp(&(unsigned(b), b.made, &b.file)));
    for one in guides {
        let title = one.title.as_deref().unwrap_or_default();
        let title = match crate::untouched::guide_tongue(title) {
            Some(tongue) if one.page_of.is_none() => format!("the guide in {tongue}"),
            _ => crate::text::folded(title.trim()),
        };
        // A translation keeps the title it was copied with: its language, or its book's, tells it apart.
        let lang = match one.page_of.and_then(|up| state.docs.get(&up)) {
            Some(up) => up.lang.clone(),
            None => one.lang.clone(),
        };
        match first.entry((title, lang)) {
            std::collections::btree_map::Entry::Occupied(_) => later.push(one.id),
            std::collections::btree_map::Entry::Vacant(free) => {
                free.insert(one.id);
            }
        }
    }
    later
}

#[cfg(test)]
#[path = "doubled_test.rs"]
mod tests;
