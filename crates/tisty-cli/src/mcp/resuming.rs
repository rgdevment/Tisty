use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};
use tisty_core::event::Event;
use tisty_core::model::{Task, TaskId};
use tisty_core::{DeviceId, Op, Paths, State};
use ulid::Ulid;

use super::jsonrpc::speaking_through;
use super::looking::brief;

pub(super) const MOVED_SHOWN: usize = 20;
const LOOKED: &str = "looked";
/// A retry, a subagent or a second window asking within this long hears the same news again.
const SAID_AGAIN_WITHIN: i64 = 60;

pub(super) type Seen = BTreeMap<String, (jiff::Timestamp, u64)>;

#[derive(serde::Serialize, serde::Deserialize)]
struct Looked {
    seen: Seen,
    #[serde(default)]
    at: Option<jiff::Timestamp>,
    #[serde(default)]
    was: Option<Seen>,
    #[serde(default)]
    was_at: Option<jiff::Timestamp>,
    #[serde(default)]
    whole: bool,
}

impl Looked {
    fn saying_again(&self, now: jiff::Timestamp) -> bool {
        self.whole
            && self.was.is_some()
            && self
                .at
                .is_some_and(|at| now.as_second() - at.as_second() < SAID_AGAIN_WITHIN)
    }
}

pub(super) enum Since {
    At(jiff::Timestamp),
    Seen(Seen),
}

impl Since {
    fn passed(&self, event: &Event) -> bool {
        match self {
            Since::At(at) => event.timestamp > *at,
            Since::Seen(seen) => seen
                .get(&event.device.0)
                .is_none_or(|was| (event.timestamp, event.seq) > *was),
        }
    }

    pub(super) fn latest(&self) -> Option<jiff::Timestamp> {
        match self {
            Since::At(at) => Some(*at),
            Since::Seen(seen) => seen.values().map(|(at, _)| *at).max(),
        }
    }
}

pub(super) struct LeftOff {
    pub(super) seen: Option<Seen>,
    pub(super) looked_at: Option<jiff::Timestamp>,
    pub(super) at: Option<jiff::Timestamp>,
    pub(super) task: Option<TaskId>,
    pub(super) task_at: Option<jiff::Timestamp>,
    pub(super) ended: Option<TaskId>,
}

pub(super) fn hand(via: Option<&str>) -> Option<String> {
    via.map(|raw| {
        tisty_core::agent::client_id(raw).map_or_else(|| raw.trim().to_lowercase(), str::to_string)
    })
}

fn my_hand() -> Option<String> {
    hand(speaking_through().as_deref())
}

pub(super) fn own_events_newest_first(
    paths: &Paths,
    agent: &DeviceId,
) -> impl Iterator<Item = Event> {
    tisty_core::store::segments_in(&paths.store().join(&agent.0))
        .unwrap_or_default()
        .into_iter()
        .rev()
        .flat_map(|segment| {
            tisty_core::store::read_tail(&segment, 0)
                .unwrap_or_default()
                .into_iter()
                .rev()
        })
}

pub(super) fn left_off<E: std::borrow::Borrow<Event>>(
    paths: &Paths,
    state: &State,
    mine_newest_first: impl Iterator<Item = E>,
) -> LeftOff {
    let me = my_hand();
    let owners = tisty_core::task_docs::behind(state);
    let mut at = None;
    let mut task = None;
    let mut task_at = None;
    let mut ended = None;
    for one in mine_newest_first {
        let one = one.borrow();
        if hand(one.via.as_deref()) != me {
            continue;
        }
        let Some((named, id)) = what_it_touched(one) else {
            continue;
        };
        if !["task.", "doc.", "folder."]
            .iter()
            .any(|kind| named.starts_with(kind))
        {
            continue;
        }
        at.get_or_insert(one.timestamp);
        let Some(touched) = touched_task(state, &owners, &named, &id) else {
            continue;
        };
        match state.tasks.get(&touched) {
            Some(kept) if kept.is_open() && !kept.folded() && kept.resolved.is_none() => {
                task = Some(touched);
                task_at = Some(one.timestamp);
            }
            Some(_) => ended = Some(touched),
            None => {}
        }
        break;
    }
    let looked = looked(paths);
    let now = jiff::Timestamp::now();
    let (seen, looked_at) = match looked {
        Some(one) if one.saying_again(now) => (one.was, one.was_at),
        Some(one) => (Some(one.seen), one.at),
        None => (None, None),
    };
    LeftOff {
        seen,
        looked_at,
        at,
        task,
        task_at,
        ended,
    }
}

pub(super) fn last_task<'a>(state: &'a State, left: &LeftOff) -> Option<&'a Task> {
    state.tasks.get(&left.task?)
}

pub(super) fn ended_task<'a>(state: &'a State, left: &LeftOff) -> Option<&'a Task> {
    state.tasks.get(&left.ended?)
}

fn looked_at(paths: &Paths) -> std::path::PathBuf {
    let named: String = my_hand()
        .unwrap_or_else(|| "unnamed".to_string())
        .chars()
        .map(
            |one| match one.is_ascii_alphanumeric() || one == '-' || one == '_' {
                true => one,
                false => '-',
            },
        )
        .take(64)
        .collect();
    paths.private().join(LOOKED).join(format!("{named}.json"))
}

fn looked(paths: &Paths) -> Option<Looked> {
    let raw = std::fs::read(looked_at(paths)).ok()?;
    serde_json::from_slice::<Looked>(&raw).ok().or_else(|| {
        serde_json::from_slice::<Seen>(&raw)
            .ok()
            .map(|seen| Looked {
                seen,
                at: None,
                was: None,
                was_at: None,
                whole: false,
            })
    })
}

pub(super) fn looks_now(paths: &Paths, seen: &Seen, whole: bool) {
    let now = jiff::Timestamp::now();
    let next = match looked(paths) {
        Some(was) if was.saying_again(now) => Looked {
            seen: seen.clone(),
            at: was.at,
            was: was.was,
            was_at: was.was_at,
            whole,
        },
        Some(was) => Looked {
            seen: seen.clone(),
            at: Some(now),
            was: Some(was.seen),
            was_at: was.at,
            whole,
        },
        None => Looked {
            seen: seen.clone(),
            at: Some(now),
            was: None,
            was_at: None,
            whole,
        },
    };
    let at = looked_at(paths);
    if let Some(folder) = at.parent()
        && std::fs::create_dir_all(folder).is_ok()
        && let Ok(said) = serde_json::to_vec(&next)
    {
        let _ = tisty_core::store::write_atomic(&at, &said);
    }
}

pub(super) fn seen_whole(log: &[Event]) -> Seen {
    let mut seen = Seen::new();
    for one in log {
        bump(&mut seen, one);
    }
    seen
}

fn bump(seen: &mut Seen, event: &Event) {
    let now = (event.timestamp, event.seq);
    seen.entry(event.device.0.clone())
        .and_modify(|was| *was = (*was).max(now))
        .or_insert(now);
}

pub(super) fn whose(
    state: &State,
    by: &DeviceId,
    via: Option<&str>,
    me: &DeviceId,
) -> Option<String> {
    if by == me && hand(via) == my_hand() {
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Moving {
    Task(TaskId),
    Paper(Ulid),
}

impl Moving {
    fn is_task(self) -> bool {
        matches!(self, Moving::Task(_))
    }
}

fn moving(state: &State, owners: &BTreeMap<String, TaskId>, event: &Event) -> Option<Moving> {
    let (named, id) = what_it_touched(event)?;
    if let Some(task) = touched_task(state, owners, &named, &id) {
        return state
            .tasks
            .get(&task)
            .filter(|one| !one.folded())
            .map(|_| Moving::Task(task));
    }
    if !named.starts_with("doc.") {
        return None;
    }
    let id = id.parse::<Ulid>().ok()?;
    let kept = state.docs.get(&id)?;
    kept.task_of.is_none().then_some(Moving::Paper(id))
}

pub(super) struct Asked<'a> {
    pub(super) since: &'a Since,
    pub(super) mine_too: bool,
    pub(super) seen: Seen,
}

pub(super) struct Moved {
    pub(super) tasks: Vec<Value>,
    pub(super) tasks_all: usize,
    pub(super) docs: Vec<Value>,
    pub(super) docs_all: usize,
    pub(super) by: BTreeSet<String>,
    pub(super) seen: Seen,
    pub(super) resume: Option<jiff::Timestamp>,
}

impl Moved {
    pub(super) fn cut(&self) -> bool {
        self.tasks_all > self.tasks.len() || self.docs_all > self.docs.len()
    }
}

pub(super) fn moved_since(
    state: &State,
    log: &[Event],
    asked: Asked,
    me: &DeviceId,
    today: jiff::civil::Date,
) -> Moved {
    let owners = tisty_core::task_docs::behind(state);
    let mut shown: Vec<(Moving, BTreeSet<String>)> = Vec::new();
    let mut later: BTreeSet<Moving> = BTreeSet::new();
    let mut seen = asked.seen;
    let mut resume = None;
    for one in log {
        let touched = match asked.since.passed(one) {
            true => {
                let who = mover(state, one, me);
                (asked.mine_too || who.is_some())
                    .then(|| moving(state, &owners, one).map(|item| (item, who)))
                    .flatten()
            }
            false => None,
        };
        let at = touched
            .as_ref()
            .and_then(|(item, _)| shown.iter().position(|(was, _)| was == item));
        if !later.is_empty() {
            later.extend(touched.map(|(item, _)| item).filter(|_| at.is_none()));
            continue;
        }
        match (touched, at) {
            (Some((_, who)), Some(at)) => shown[at].1.extend(who),
            (Some((item, who)), None) => {
                let alike = shown
                    .iter()
                    .filter(|(was, _)| was.is_task() == item.is_task())
                    .count();
                if alike >= MOVED_SHOWN {
                    later.insert(item);
                    // Events stamped alike are told apart by seq, which a cursor cannot carry.
                    resume = one
                        .timestamp
                        .checked_sub(jiff::SignedDuration::from_nanos(1))
                        .ok();
                    continue;
                }
                shown.push((item, who.into_iter().collect()));
            }
            (None, _) => {}
        }
        bump(&mut seen, one);
    }
    let all = |task: bool| {
        shown
            .iter()
            .map(|(one, _)| one)
            .chain(&later)
            .filter(|one| one.is_task() == task)
            .count()
    };
    let (tasks_all, docs_all) = (all(true), all(false));
    let mut by = BTreeSet::new();
    let mut tasks = Vec::new();
    let mut docs = Vec::new();
    for (item, who) in shown {
        let told = match item {
            Moving::Task(id) => state.tasks.get(&id).map(|task| brief(task, state, today)),
            Moving::Paper(id) => state
                .docs
                .get(&id)
                .map(|kept| json!({ "doc": kept.file, "title": kept.title })),
        };
        let Some(mut told) = told else {
            continue;
        };
        if !who.is_empty() {
            told["by"] = json!(who);
            by.extend(who);
        }
        match item.is_task() {
            true => tasks.push(told),
            false => docs.push(told),
        }
    }
    Moved {
        tasks,
        tasks_all,
        docs,
        docs_all,
        by,
        seen,
        resume,
    }
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

#[cfg(test)]
#[path = "resuming_test.rs"]
mod tests;
