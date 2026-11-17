use std::sync::Mutex;

use jiff::Timestamp;
use tisty_core::event::StepRef;
use tisty_core::model::{DateSpec, Status, Task, TaskId};
use tisty_core::{Op, State};

use super::parts::{answered, task_id};
use crate::{Answer, Refusal, Session, held};

#[derive(serde::Serialize, Debug, Clone, PartialEq)]
pub struct Cited {
    pub id: String,
    pub title: String,
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<DateSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<Timestamp>,
    pub steps: Vec<CitedStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Counted>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of: Option<String>,
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
pub struct CitedStep {
    pub id: String,
    pub html: String,
    pub done: bool,
}

#[derive(serde::Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counted {
    pub closed: usize,
    pub all: usize,
}

#[tauri::command(async)]
pub fn task_cards(session: tauri::State<'_, Mutex<Session>>, ids: Vec<String>) -> Vec<Cited> {
    let session = held(&session);
    cited(&session.state, &ids)
}

// A task erased, or one that never reached this computer, is left out, and its card keeps its words.
pub fn cited(state: &State, ids: &[String]) -> Vec<Cited> {
    ids.iter()
        .filter_map(|raw| raw.parse::<TaskId>().ok())
        .filter_map(|id| state.tasks.get(&id))
        .map(|task| card_of(state, task))
        .collect()
}

fn card_of(state: &State, task: &Task) -> Cited {
    Cited {
        id: task.id.to_string(),
        title: task.title.clone(),
        status: task.status,
        list: task
            .list
            .and_then(|list| state.lists.get(&list))
            .map(|list| list.name.clone()),
        date: task.date.clone(),
        deadline: task.deadline.clone(),
        completed_at: task.completed_at,
        steps: tisty_core::stages::ordered(task)
            .into_iter()
            .map(|step| CitedStep {
                id: step.id.to_string(),
                html: drawn(&step.text),
                done: step.done,
            })
            .collect(),
        parts: counted(state, task.id),
        part_of: task.part_of.map(|whole| whole.to_string()),
    }
}

fn drawn(text: &str) -> String {
    let html = tisty_markdown::html(&tisty_markdown::read(text));
    let html = html.trim();
    let inline = html
        .strip_prefix("<p")
        .and_then(|rest| rest.split_once('>'))
        .and_then(|(_, inner)| inner.strip_suffix("</p>"))
        .filter(|inner| !inner.contains("<p"));
    inline.unwrap_or(html).to_string()
}

fn counted(state: &State, whole: TaskId) -> Option<Counted> {
    let (closed, all) = state
        .parts_of(whole)
        .filter(|part| !part.folded())
        .fold((0, 0), |(closed, all), part| {
            (closed + usize::from(!part.is_open()), all + 1)
        });
    (all > 0).then_some(Counted { closed, all })
}

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
