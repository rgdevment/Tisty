use std::sync::Mutex;

use tisty_core::{Task, TaskId};

use crate::{Answer, Refusal, Session, held, signing};

#[derive(serde::Serialize)]
pub struct Opened {
    pub file: String,
    pub body: String,
}

fn task_in(session: &Session, id: &str) -> Result<Task, Refusal> {
    let id: TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command(async)]
pub fn task_doc_open(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Opened>> {
    let mut session = held(&session);
    let task = task_in(&session, &id)?;
    let Some(file) = tisty_core::task_docs::of(&session.state, task.id).map(|one| one.file.clone())
    else {
        return Ok(None);
    };
    let body = tisty_core::docs::read(&session.paths.docs(), &file)?;
    session.mind_body(&file, &body);
    Ok(Some(Opened {
        body: tisty_core::stages::ticked(&task, &body),
        file,
    }))
}

#[tauri::command(async)]
pub fn task_doc_make(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<String> {
    let mut session = held(&session);
    let task = task_in(&session, &id)?;
    if let Some(kept) = tisty_core::task_docs::of(&session.state, task.id) {
        return Ok(kept.file.clone());
    }
    let op = tisty_core::task_docs::made(
        &session.paths.docs(),
        &session.state,
        task.id,
        signing(&session.state),
    )?;
    session.commit(op)?;
    tisty_core::task_docs::of(&session.state, task.id)
        .map(|one| one.file.clone())
        .ok_or_else(|| Refusal::of("noSuchDoc"))
}

// Called when a block is left or a box is ticked, never on a keystroke: each call may write events.
#[tauri::command(async)]
pub fn task_doc_settled(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    before: String,
    now: String,
) -> Answer<Task> {
    let mut session = held(&session);
    let task = task_in(&session, &id)?;
    let ops = tisty_core::stages::reconciled(&task, Some(&before), &now);
    if !ops.is_empty() {
        session.commit_all(ops)?;
    }
    task_in(&session, &id)
}
