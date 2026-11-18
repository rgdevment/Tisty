use std::sync::Mutex;

use tisty_core::event::StepRef;
use tisty_core::model::{Task, TaskId};
use tisty_core::{Op, State};

use super::parts::{answered, task_id};
use crate::{Answer, Refusal, Session, held};

#[tauri::command(async)]
pub fn task_step_marked(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    step: String,
    done: bool,
) -> Answer<Task> {
    let id = task_id(&id)?;
    let mut session = held(&session);
    session.reload()?;
    if let Some(op) = marking(&session.state, id, &step, done)? {
        session.commit(op)?;
    }
    answered(&session, id)
}

pub fn marking(state: &State, id: TaskId, step: &str, done: bool) -> Answer<Option<Op>> {
    let task = state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    if !task.is_open() {
        return Err(Refusal::of("taskClosed"));
    }
    let Some(found) = task.steps.iter().find(|one| one.id.to_string() == step) else {
        return Ok(None);
    };
    if found.done == done {
        return Ok(None);
    }
    let d = StepRef { step: found.id };
    Ok(Some(match done {
        true => Op::StepDone { id, d },
        false => Op::StepUndone { id, d },
    }))
}

#[cfg(test)]
#[path = "cited_tasks_test.rs"]
mod tests;
