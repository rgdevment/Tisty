use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use tisty_core::event::Event;
use tisty_core::model::{Kept, Task, TaskId};
use tisty_core::{DeviceId, Paths, State};
use ulid::Ulid;

use super::jsonrpc::speaking_through;
use super::looking::{brief, placed};

pub(super) const MOVED_SHOWN: usize = 20;

pub(super) struct LeftOff {
    pub(super) at: jiff::Timestamp,
    pub(super) task: Option<TaskId>,
}

pub(super) fn left_off(paths: &Paths, state: &State, agent: &DeviceId) -> Option<LeftOff> {
    let owners = tisty_core::task_docs::behind(state);
    let segments = tisty_core::store::segments_in(&paths.store().join(&agent.0)).ok()?;
    let mut at = None;
    for segment in segments.iter().rev() {
        let Ok(events) = tisty_core::store::read_tail(segment, 0) else {
            continue;
        };
        for one in events.iter().rev().filter(|one| worked(one)) {
            let at = *at.get_or_insert(one.timestamp);
            let open = task_touched(state, &owners, one)
                .and_then(|id| state.tasks.get(&id))
                .filter(|task| task.is_open() && !task.folded() && task.resolved.is_none());
            if let Some(task) = open {
                return Some(LeftOff {
                    at,
                    task: Some(task.id),
                });
            }
        }
    }
    at.map(|at| LeftOff { at, task: None })
}

pub(super) fn last_named(state: &State, left: Option<&LeftOff>) -> Option<String> {
    let task = state.tasks.get(&left?.task?)?;
    Some(format!(
        "{:?} {} [{}]",
        task.title,
        placed(state, task),
        task.id
    ))
}

pub(super) fn last_shown(state: &State, left: Option<&LeftOff>) -> Option<Value> {
    let task = state.tasks.get(&left?.task?)?;
    let mut shown = json!({ "id": task.id.to_string(), "title": task.title });
    if let Some(list) = task.list.and_then(|at| state.lists.get(&at)) {
        shown["list"] = json!(list.name);
    }
    Some(shown)
}

pub(super) fn whose(
    state: &State,
    by: &DeviceId,
    via: Option<&str>,
    me: &DeviceId,
) -> Option<String> {
    if by == me && via == speaking_through().as_deref() {
        return None;
    }
    if by != me && !state.assistants.contains(by) {
        return Some("the person".to_string());
    }
    Some(via.map_or_else(
        || "an assistant".to_string(),
        tisty_core::agent::client_named,
    ))
}

#[derive(Default)]
struct Touched {
    last: Option<jiff::Timestamp>,
    by: BTreeSet<String>,
}

impl Touched {
    fn mark(&mut self, event: &Event, who: Option<String>) {
        self.last = self.last.max(Some(event.timestamp));
        self.by.extend(who);
    }
}

pub(super) struct Moved {
    pub(super) tasks: Vec<Value>,
    pub(super) tasks_all: usize,
    pub(super) docs: Vec<Value>,
    pub(super) docs_all: usize,
    pub(super) by: BTreeSet<String>,
}

pub(super) fn moved_since(
    state: &State,
    log: &[Event],
    since: jiff::Timestamp,
    me: &DeviceId,
    today: jiff::civil::Date,
) -> Moved {
    let owners = tisty_core::task_docs::behind(state);
    let mut tasks: BTreeMap<TaskId, Touched> = BTreeMap::new();
    let mut papers: BTreeMap<String, Touched> = BTreeMap::new();
    // The log refuses two events at one instant, so leaving out the cursor's own skips nothing.
    for one in log.iter().filter(|one| one.timestamp > since) {
        let who = whose(state, &one.device, one.via.as_deref(), me);
        if let Some(task) = task_touched(state, &owners, one) {
            tasks.entry(task).or_default().mark(one, who);
        } else if let Some((named, id)) = what_it_touched(one)
            && named.starts_with("doc.")
        {
            papers.entry(id).or_default().mark(one, who);
        }
    }
    let tasks: Vec<(&Task, Touched)> = tasks
        .into_iter()
        .filter_map(|(id, touched)| {
            Some((state.tasks.get(&id).filter(|one| !one.folded())?, touched))
        })
        .collect();
    let docs: Vec<(&Kept, Touched)> = papers
        .into_iter()
        .filter_map(|(id, touched)| {
            let id = id.parse::<Ulid>().ok()?;
            let kept = state.docs.values().find(|one| one.id == id)?;
            kept.task_of.is_none().then_some((kept, touched))
        })
        .collect();
    let (tasks_all, docs_all) = (tasks.len(), docs.len());
    let mut by = BTreeSet::new();
    let tasks = shown(tasks, &mut by, |task| brief(task, state, today));
    let docs = shown(
        docs,
        &mut by,
        |kept| json!({ "doc": kept.file, "title": kept.title }),
    );
    Moved {
        tasks,
        tasks_all,
        docs,
        docs_all,
        by,
    }
}

fn shown<T>(
    mut all: Vec<(T, Touched)>,
    by: &mut BTreeSet<String>,
    told: impl Fn(T) -> Value,
) -> Vec<Value> {
    all.sort_by_key(|(_, touched)| std::cmp::Reverse(touched.last));
    for (_, touched) in &all {
        by.extend(touched.by.iter().cloned());
    }
    all.into_iter()
        .take(MOVED_SHOWN)
        .map(|(one, touched)| {
            let mut one = told(one);
            if !touched.by.is_empty() {
                one["by"] = json!(touched.by);
            }
            one
        })
        .collect()
}

pub(super) fn task_touched(
    state: &State,
    owners: &BTreeMap<String, TaskId>,
    event: &Event,
) -> Option<TaskId> {
    let (named, id) = what_it_touched(event)?;
    if named.starts_with("task.") {
        return id.parse().ok();
    }
    if !named.starts_with("doc.") {
        return None;
    }
    // Writing a task's document is that task moving, never a document of its own.
    let kept = state.docs.get(&id.parse::<Ulid>().ok()?)?;
    owners.get(&kept.file).copied()
}

fn worked(event: &Event) -> bool {
    what_it_touched(event).is_some_and(|(named, _)| {
        ["task.", "doc.", "folder."]
            .iter()
            .any(|kind| named.starts_with(kind))
    })
}

fn what_it_touched(event: &Event) -> Option<(String, String)> {
    let said = serde_json::to_value(&event.op).ok()?;
    let named = said.get("op")?.as_str()?.to_string();
    let id = said.get("id")?.as_str()?.to_string();
    Some((named, id))
}
