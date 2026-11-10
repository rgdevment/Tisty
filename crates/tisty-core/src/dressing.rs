use std::collections::BTreeSet;

use ulid::Ulid;

use crate::event::{Event, Look, Op};
use crate::model::FIRST;
use crate::state::State;
use crate::untouched::{GUIDE_FOLDERS, GUIDE_HUE, GUIDE_ICON};

pub const DIAGRAM_FOLDERS: [&str; 2] = ["Diagrams", "Diagramas"];
pub const DIAGRAMS_ICON: &str = "shapes";
pub const DIAGRAMS_HUE: &str = "purple";

fn worn(icon: &str, hue: &str) -> Look {
    Look {
        icon: Some(Some(icon.to_string())),
        color: Some(Some(hue.to_string())),
    }
}

fn looked_at(events: &[Event]) -> BTreeSet<Ulid> {
    events
        .iter()
        .filter_map(|event| match &event.op {
            Op::ListLook { id, d } if d.color.is_some() => Some(*id),
            Op::FolderLook { id, .. } => Some(*id),
            _ => None,
        })
        .collect()
}

pub fn dressed(state: &State, events: &[Event]) -> Vec<Op> {
    let looked = looked_at(events);
    let lists = state
        .lists
        .values()
        .filter(|one| one.color.is_none() && !looked.contains(&one.id))
        .filter_map(|one| {
            FIRST
                .iter()
                .find(|(english, castilian, icon, _)| {
                    (one.name == *english || one.name == *castilian)
                        && one.icon.as_deref() == Some(*icon)
                })
                .map(|(_, _, icon, hue)| Op::ListLook {
                    id: one.id,
                    d: worn(icon, hue),
                })
        });
    let folders = state
        .folders
        .values()
        .filter(|one| {
            one.parent.is_none()
                && one.icon.is_none()
                && one.color.is_none()
                && !looked.contains(&one.id)
        })
        .filter_map(|one| {
            let name = one.name.as_str();
            let (icon, hue) = if GUIDE_FOLDERS.contains(&name) {
                (GUIDE_ICON, GUIDE_HUE)
            } else if DIAGRAM_FOLDERS.contains(&name) {
                (DIAGRAMS_ICON, DIAGRAMS_HUE)
            } else {
                return None;
            };
            Some(Op::FolderLook {
                id: one.id,
                d: worn(icon, hue),
            })
        });
    lists.chain(folders).collect()
}

#[cfg(test)]
#[path = "dressing_test.rs"]
mod tests;
