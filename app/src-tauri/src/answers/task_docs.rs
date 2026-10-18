use std::sync::Mutex;

use tisty_core::witness::channel;
use tisty_core::{Op, Task, TaskId};

use super::papers::writable;
use super::parts::{answered, task_id};
use crate::{Answer, Refusal, Session, blamed, held, signing};

#[derive(serde::Serialize)]
pub struct Opened {
    pub file: String,
    pub body: String,
    pub held: bool,
}

fn file_of(session: &Session, id: TaskId) -> Option<String> {
    tisty_core::task_docs::of(&session.state, id).map(|one| one.file.clone())
}

fn known(session: &Session, id: TaskId) -> Result<(), Refusal> {
    match session.state.tasks.contains_key(&id) {
        true => Ok(()),
        false => Err(Refusal::of("notATaskId")),
    }
}

fn open_task(session: &Session, id: TaskId) -> Answer<Task> {
    let task = answered(session, id)?;
    match task.is_open() {
        true => Ok(task),
        false => Err(Refusal::of("taskClosed")),
    }
}

#[tauri::command(async)]
pub fn task_doc_open(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Opened>> {
    let id = task_id(&id)?;
    let found = {
        let session = held(&session);
        known(&session, id)?;
        file_of(&session, id)
    };
    let Some(file) = found else {
        return Ok(None);
    };
    let body = super::papers::doc_read(session.clone(), file.clone())?;
    let session = held(&session);
    let task = answered(&session, id)?;
    Ok(Some(Opened {
        body: tisty_core::stages::ticked(&task, &body),
        held: writable(&session.state, &file).is_err() || !task.is_open(),
        file,
    }))
}

#[tauri::command(async)]
pub fn task_doc_make(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<String> {
    let id = task_id(&id)?;
    let mut session = held(&session);
    if let Some(file) = file_of(&session, id) {
        known(&session, id)?;
        return Ok(file);
    }
    open_task(&session, id)?;
    let op = tisty_core::task_docs::made(
        &session.paths.docs(),
        &session.state,
        id,
        signing(&session.state),
    )
    .map_err(|e| blamed(channel::WINDOW, "a task's document could not be made", e))?;
    let Op::DocAdd { d, .. } = &op else {
        return Err(Refusal::of("internal"));
    };
    let file = d.file.clone();
    session.commit(op)?;
    Ok(file)
}

// Each call may write events, so it answers a block left or a box ticked, never a keystroke.
#[tauri::command(async)]
pub fn task_doc_settled(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    before: Option<String>,
    now: String,
) -> Answer<Task> {
    let id = task_id(&id)?;
    let mut session = held(&session);
    let task = open_task(&session, id)?;
    let file = file_of(&session, id).ok_or_else(|| Refusal::of("noSuchDoc"))?;
    writable(&session.state, &file)?;
    if session.moved(&file) {
        return Err(Refusal::about("documentMoved", file));
    }
    let ops = tisty_core::stages::reconciled(&task, before.as_deref(), &now);
    if !ops.is_empty() {
        session.commit_all(ops)?;
    }
    answered(&session, id)
}
