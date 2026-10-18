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
    let owner = tisty_core::task_docs::owner(&session.state, id);
    tisty_core::task_docs::of(&session.state, owner).map(|one| one.file.clone())
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
    template: Option<String>,
) -> Answer<Option<Opened>> {
    let id = task_id(&id)?;
    let file = {
        let session = held(&session);
        let task = answered(&session, id)?;
        match file_of(&session, id) {
            Some(file) => file,
            // A task without a document reads as the one it would have, made only once it is written.
            None => {
                let template = match task.is_open() {
                    true => template.unwrap_or_default(),
                    false => String::new(),
                };
                return Ok(Some(Opened {
                    file: String::new(),
                    body: tisty_core::task_docs::opened(&session.state, id, &template),
                    held: !task.is_open(),
                }));
            }
        }
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
pub fn task_doc_make(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    template: Option<String>,
    body: Option<String>,
) -> Answer<String> {
    let id = task_id(&id)?;
    let mut session = held(&session);
    // Another process may have written since, and a document made twice is two of them.
    session.reload()?;
    if let Some(file) = file_of(&session, id) {
        known(&session, id)?;
        return Ok(file);
    }
    open_task(&session, id)?;
    let written = body.is_some();
    let first = body.unwrap_or_else(|| {
        tisty_core::task_docs::opened(&session.state, id, &template.unwrap_or_default())
    });
    let (made, created) = tisty_core::task_docs::made_with(
        &session.paths.docs(),
        &session.state,
        tisty_core::task_docs::owner(&session.state, id),
        &first,
        signing(&session.state),
    )
    .map_err(|e| blamed(channel::WINDOW, "a task's document could not be made", e))?;
    let Op::DocAdd { d, .. } = &made else {
        return Err(Refusal::of("internal"));
    };
    let file = d.file.clone();
    session.commit_all(vec![made])?;
    // Words that reached the disk first are newer than what was typed over the page they replace.
    if written && !created {
        return Err(Refusal::about("documentMoved", file));
    }
    Ok(file)
}

// Each call may write events, so it answers a block left or a box ticked, never a keystroke.
#[tauri::command(async)]
pub fn task_doc_settled(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    before: Option<String>,
    now: String,
) -> Answer<Option<Task>> {
    let id = task_id(&id)?;
    let mut session = held(&session);
    // Steps an assistant added from another process count before the text is weighed against them.
    session.reload()?;
    let task = answered(&session, id)?;
    // A block left by the very click that closed the task has nothing left to settle into.
    if !task.is_open() {
        return Ok(None);
    }
    let file = file_of(&session, id).ok_or_else(|| Refusal::of("noSuchDoc"))?;
    writable(&session.state, &file)?;
    if session.moved(&file) {
        return Err(Refusal::about("documentMoved", file));
    }
    let ops = tisty_core::stages::reconciled(&task, before.as_deref(), &now);
    if ops.is_empty() {
        return Ok(None);
    }
    session.commit_all(ops)?;
    answered(&session, id).map(Some)
}
