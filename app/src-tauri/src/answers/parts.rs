use std::collections::BTreeMap;
use std::sync::Mutex;

use tisty_core::capture::{Draft, Filing};
use tisty_core::event::{StepRef, TaskMove};
use tisty_core::model::{Status, Task, TaskId};
use tisty_core::{Op, State};

use crate::{Answer, Refusal, Session, held};

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Whole {
    pub title: String,
    pub open: usize,
    pub closed: usize,
    /// Open but put out of sight: not shown, still let go when the whole is closed.
    pub away: usize,
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Offered {
    pub id: String,
    pub title: String,
}

/// Every task that holds parts, with what a row needs to show it, and what a part needs to name
/// the whole it belongs to even when the whole is not in the view.
pub fn wholes(state: &State) -> BTreeMap<String, Whole> {
    let mut all: BTreeMap<TaskId, Whole> = BTreeMap::new();
    for part in state.tasks.values() {
        let Some(whole) = part.part_of.and_then(|id| state.tasks.get(&id)) else {
            continue;
        };
        let counted = all.entry(whole.id).or_insert_with(|| Whole {
            title: whole.title.clone(),
            open: 0,
            closed: 0,
            away: 0,
        });
        match (part.is_open(), part.folded()) {
            (true, true) => counted.away += 1,
            (true, false) => counted.open += 1,
            (false, false) => counted.closed += 1,
            (false, true) => {}
        }
    }
    all.into_iter()
        .map(|(id, one)| (id.to_string(), one))
        .collect()
}

/// Judged here as well as at replay, so the person is told why instead of seeing nothing land.
pub fn hanging(state: &State, id: TaskId, whole: Option<TaskId>) -> Result<Op, &'static str> {
    let task = state.tasks.get(&id).ok_or("notATaskId")?;
    if let Some(whole) = whole {
        let holder = state.tasks.get(&whole).ok_or("notATaskId")?;
        if whole == id {
            return Err("partOfItself");
        }
        if !holder.is_open() || holder.folded() {
            return Err("wholeClosed");
        }
        if holder.part_of.is_some() {
            return Err("partOfAPart");
        }
        if state.holds_parts(id) {
            return Err("wholeIsNoPart");
        }
        if repeats(task) {
            return Err("partRepeats");
        }
        if repeats(holder) {
            return Err("wholeRepeats");
        }
    }
    Ok(Op::TaskMove {
        id,
        d: TaskMove {
            part_of: Some(whole),
            ..Default::default()
        },
    })
}

pub fn part_added(state: &State, whole: TaskId, title: &str) -> Result<(TaskId, Vec<Op>), Refusal> {
    let holder = state
        .tasks
        .get(&whole)
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    takes_parts(holder).map_err(Refusal::of)?;
    let draft = Draft {
        title: title.trim().to_string(),
        filing: holder.list.map(Filing::Kept),
        ..Default::default()
    };
    let plan = tisty_core::capture::plan(state, draft)?;
    Ok((plan.task, with_whole(plan.ops, plan.task, whole)))
}

/// One transaction, so undo puts the step back where it was and takes the part away.
pub fn step_turned(state: &State, id: TaskId, step: &str) -> Result<Vec<Op>, Refusal> {
    let holder = state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    let step: tisty_core::model::StepId = step.parse().map_err(|_| Refusal::of("notAStepId"))?;
    let said = holder
        .steps
        .iter()
        .find(|one| one.id == step)
        .ok_or_else(|| Refusal::of("notAStepId"))?;
    takes_parts(holder).map_err(Refusal::of)?;
    let draft = Draft {
        title: said.text.clone(),
        filing: holder.list.map(Filing::Kept),
        ..Default::default()
    };
    let plan = tisty_core::capture::plan(state, draft)?;
    let mut ops = with_whole(plan.ops, plan.task, id);
    ops.push(Op::StepRemove {
        id,
        d: StepRef { step },
    });
    Ok(ops)
}

/// The open tasks this one could be a part of: none of them a part, none that repeats.
pub fn offered(state: &State, id: TaskId) -> Vec<Offered> {
    let Some(task) = state.tasks.get(&id) else {
        return Vec::new();
    };
    if state.holds_parts(id) || repeats(task) {
        return Vec::new();
    }
    let mut all: Vec<&Task> = state
        .tasks
        .values()
        .filter(|one| {
            one.id != id
                && one.id != task.part_of.unwrap_or(id)
                && one.status == Status::Open
                && !one.folded()
                && one.part_of.is_none()
                && !repeats(one)
        })
        .collect();
    let wholes: std::collections::HashSet<TaskId> =
        state.tasks.values().filter_map(|one| one.part_of).collect();
    all.sort_by(|one, other| {
        wholes
            .contains(&other.id)
            .cmp(&wholes.contains(&one.id))
            .then_with(|| one.title.to_lowercase().cmp(&other.title.to_lowercase()))
    });
    all.into_iter()
        .map(|one| Offered {
            id: one.id.to_string(),
            title: one.title.clone(),
        })
        .collect()
}

fn takes_parts(holder: &Task) -> Result<(), &'static str> {
    if holder.status != Status::Open || holder.folded() {
        return Err("wholeClosed");
    }
    if holder.part_of.is_some() {
        return Err("partOfAPart");
    }
    if repeats(holder) {
        return Err("wholeRepeats");
    }
    Ok(())
}

fn with_whole(mut ops: Vec<Op>, born: TaskId, whole: TaskId) -> Vec<Op> {
    for op in &mut ops {
        if let Op::TaskAdd { id, d } = op
            && *id == born
        {
            d.part_of = Some(whole);
        }
    }
    ops
}

fn repeats(task: &Task) -> bool {
    task.repeat.is_some() || task.after.is_some()
}

fn answered(session: &Session, id: TaskId) -> Answer<Task> {
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

fn task_id(raw: &str) -> Result<TaskId, Refusal> {
    raw.parse().map_err(|_| Refusal::of("notATaskId"))
}

/// A part or a whole can be opened from a detail even when the view never listed it.
#[tauri::command]
pub fn task_of(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Task> {
    let id = task_id(&id)?;
    let session = held(&session);
    answered(&session, id)
}

#[tauri::command]
pub fn parts_of(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<Task>> {
    let id = task_id(&id)?;
    let session = held(&session);
    let mut parts: Vec<Task> = session
        .state
        .parts_of(id)
        .filter(|one| !one.folded())
        .cloned()
        .collect();
    parts.sort_by(|one, other| one.order.cmp(&other.order));
    Ok(parts)
}

#[tauri::command]
pub fn add_part(
    session: tauri::State<'_, Mutex<Session>>,
    whole: String,
    title: String,
) -> Answer<Task> {
    let whole = task_id(&whole)?;
    let mut session = held(&session);
    let (born, ops) = part_added(&session.state, whole, &title)?;
    session.commit_all(ops)?;
    answered(&session, born)
}

#[tauri::command]
pub fn hang(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    whole: Option<String>,
) -> Answer<Task> {
    let id = task_id(&id)?;
    let whole = whole.as_deref().map(task_id).transpose()?;
    let mut session = held(&session);
    let op = hanging(&session.state, id, whole).map_err(Refusal::of)?;
    session.commit(op)?;
    answered(&session, id)
}

#[tauri::command]
pub fn step_to_part(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    step: String,
) -> Answer<Task> {
    let id = task_id(&id)?;
    let mut session = held(&session);
    let ops = step_turned(&session.state, id, &step)?;
    session.commit_all(ops)?;
    answered(&session, id)
}

#[tauri::command]
pub fn wholes_offered(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Vec<Offered>> {
    let id = task_id(&id)?;
    let session = held(&session);
    Ok(offered(&session.state, id))
}

#[cfg(test)]
#[path = "parts_test.rs"]
mod tests;
