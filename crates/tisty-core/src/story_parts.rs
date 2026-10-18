use std::collections::BTreeMap;

use super::{Chapter, Page};
use crate::State;
use crate::event::{DeviceId, Event, Op};
use crate::model::TaskId;

pub(super) type Key = (jiff::Timestamp, DeviceId, u64);

fn moves_parts(op: &Op) -> bool {
    matches!(
        op,
        Op::TaskAdd { .. }
            | Op::TaskUpdate { .. }
            | Op::TaskMove { .. }
            | Op::TaskDelete { .. }
            | Op::DeviceJoin { .. }
    )
}

/// Whether a part landed is the state's call, so the trail replays the state rather than guess it.
fn parted(events: &[Event], id: TaskId) -> Vec<(Key, Page)> {
    let mut sorted: Vec<&Event> = events.iter().filter(|one| moves_parts(&one.op)).collect();
    sorted.sort_by_key(|one| one.sort_key());
    let mut world = State::default();
    let mut out = Vec::new();
    for event in sorted {
        let titled = |world: &State, task: TaskId| {
            world
                .tasks
                .get(&task)
                .map(|one| one.title.clone())
                .unwrap_or_default()
        };
        let whole_was = world.tasks.get(&id).and_then(|one| one.part_of);
        let named_was = whole_was.map(|whole| titled(&world, whole));
        let parts_was: BTreeMap<TaskId, String> = world
            .parts_of(id)
            .map(|one| (one.id, one.title.clone()))
            .collect();
        world.apply(event);
        let Some(me) = world.tasks.get(&id) else {
            continue;
        };
        let mut told = Vec::new();
        if me.part_of != whole_was {
            if let (Some(whole), Some(title)) = (whole_was, named_was) {
                told.push(Chapter::Loosened { whole, title });
            }
            if let Some(whole) = me.part_of {
                let title = titled(&world, whole);
                told.push(Chapter::Joined { whole, title });
            }
        }
        let parts_now: BTreeMap<TaskId, String> = world
            .parts_of(id)
            .map(|one| (one.id, one.title.clone()))
            .collect();
        for (part, title) in &parts_was {
            if !parts_now.contains_key(part) && world.tasks.contains_key(part) {
                told.push(Chapter::PartLoosened {
                    part: *part,
                    title: title.clone(),
                });
            }
        }
        for (part, title) in &parts_now {
            if !parts_was.contains_key(part) {
                told.push(Chapter::PartJoined {
                    part: *part,
                    title: title.clone(),
                });
            }
        }
        for chapter in told {
            let key = (event.timestamp, event.device.clone(), event.seq);
            let page = Page {
                n: 0,
                at: event.timestamp,
                by: event.device.clone(),
                undoing: event.undo,
                chapter,
            };
            out.push((key, page));
        }
    }
    out
}

// A part told in the same event as the task's own change reads after it, as a birth before its joining.
pub(super) fn merged(keys: Vec<Key>, pages: Vec<Page>, events: &[Event], id: TaskId) -> Vec<Page> {
    let mut keyed: Vec<(Key, u8, Page)> = keys
        .into_iter()
        .zip(pages)
        .map(|(key, page)| (key, 0, page))
        .collect();
    keyed.extend(
        parted(events, id)
            .into_iter()
            .map(|(key, page)| (key, 1, page)),
    );
    keyed.sort_by(|a, b| (&a.0, a.1).cmp(&(&b.0, b.1)));
    keyed
        .into_iter()
        .enumerate()
        .map(|(n, (_, _, page))| Page { n, ..page })
        .collect()
}
