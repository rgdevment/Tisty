use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use tisty_core::event::Event;
use tisty_core::model::{Kept, Task, TaskId};
use tisty_core::{DeviceId, Op, Paths, State};
use ulid::Ulid;

use super::jsonrpc::speaking_through;
use super::looking::brief;

pub(super) const MOVED_SHOWN: usize = 20;
const LOOKED: &str = "looked.json";

pub(super) struct LeftOff {
    pub(super) at: jiff::Timestamp,
    pub(super) task: Option<TaskId>,
}

pub(super) fn own_events(paths: &Paths, agent: &DeviceId) -> Vec<Event> {
    tisty_core::store::segments_in(&paths.store().join(&agent.0))
        .unwrap_or_default()
        .iter()
        .filter_map(|segment| tisty_core::store::read_tail(segment, 0).ok())
        .flatten()
        .collect()
}

pub(super) fn left_off<'a>(
    paths: &Paths,
    state: &State,
    mine: impl Iterator<Item = &'a Event>,
) -> Option<LeftOff> {
    let via = speaking_through();
    let owners = tisty_core::task_docs::behind(state);
    let mut worked: Vec<(jiff::Timestamp, Option<TaskId>)> = mine
        .filter(|one| one.via == via)
        .filter_map(|one| {
            let (named, id) = what_it_touched(one)?;
            ["task.", "doc.", "folder."]
                .iter()
                .any(|kind| named.starts_with(kind))
                .then(|| (one.timestamp, touched_task(state, &owners, &named, &id)))
        })
        .collect();
    worked.sort_by_key(|(at, _)| std::cmp::Reverse(*at));
    let task = worked
        .iter()
        .filter_map(|(_, task)| state.tasks.get(task.as_ref()?))
        .find(|task| task.is_open() && !task.folded() && task.resolved.is_none())
        .map(|task| task.id);
    let at = worked.first().map(|(at, _)| *at).max(looked(paths));
    Some(LeftOff { at: at?, task })
}

pub(super) fn last_task<'a>(state: &'a State, left: Option<&LeftOff>) -> Option<&'a Task> {
    state.tasks.get(&left?.task?)
}

fn looked_by_each(paths: &Paths) -> BTreeMap<String, jiff::Timestamp> {
    std::fs::read(paths.private().join(LOOKED))
        .ok()
        .and_then(|said| serde_json::from_slice(&said).ok())
        .unwrap_or_default()
}

fn looked(paths: &Paths) -> Option<jiff::Timestamp> {
    looked_by_each(paths)
        .get(&speaking_through().unwrap_or_default())
        .copied()
}

pub(super) fn looks_now(paths: &Paths, at: jiff::Timestamp) {
    let mut all = looked_by_each(paths);
    all.insert(speaking_through().unwrap_or_default(), at);
    let private = paths.private();
    if std::fs::create_dir_all(&private).is_ok()
        && let Ok(said) = serde_json::to_vec(&all)
    {
        let _ = tisty_core::store::write_atomic(&private.join(LOOKED), &said);
    }
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

fn mover(state: &State, event: &Event, me: &DeviceId) -> Option<String> {
    if let Op::DocSaid { d, .. } = &event.op
        && let Some(hand) = d.by.as_deref()
        && !state.assistants.contains(&event.device)
        && !state.mine_to_write(hand)
    {
        return Some(hand.to_string());
    }
    whose(state, &event.device, event.via.as_deref(), me)
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
    let mut papers: BTreeMap<Ulid, Touched> = BTreeMap::new();
    // The log refuses two events at one instant, so leaving out the cursor's own skips nothing.
    for one in log.iter().filter(|one| one.timestamp > since) {
        let Some((named, id)) = what_it_touched(one) else {
            continue;
        };
        let who = mover(state, one, me);
        if let Some(task) = touched_task(state, &owners, &named, &id) {
            tasks.entry(task).or_default().mark(one, who);
        } else if named.starts_with("doc.")
            && let Ok(id) = id.parse::<Ulid>()
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
            let kept = state.docs.get(&id)?;
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

fn touched_task(
    state: &State,
    owners: &BTreeMap<String, TaskId>,
    named: &str,
    id: &str,
) -> Option<TaskId> {
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

fn what_it_touched(event: &Event) -> Option<(String, String)> {
    let said = serde_json::to_value(&event.op).ok()?;
    let named = said.get("op")?.as_str()?.to_string();
    let id = said.get("id")?.as_str()?.to_string();
    Some((named, id))
}
