use std::sync::Mutex;

use tisty_core::{Op, Task, TaskId};

use super::parts::{answered, task_id};
use crate::{Answer, Refusal, Session, held, signing};

#[derive(serde::Serialize)]
pub struct Opened {
    pub file: String,
    pub body: String,
}

fn file_of(session: &Session, id: TaskId) -> Option<String> {
    tisty_core::task_docs::of(&session.state, id).map(|one| one.file.clone())
}

#[tauri::command(async)]
pub fn task_doc_open(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Opened>> {
    let id = task_id(&id)?;
    let found = {
        let session = held(&session);
        answered(&session, id)?;
        file_of(&session, id)
    };
    let Some(file) = found else {
        return Ok(None);
    };
    let body = super::papers::doc_read(session.clone(), file.clone())?;
    let task = answered(&held(&session), id)?;
    Ok(Some(Opened {
        body: tisty_core::stages::ticked(&task, &body),
        file,
    }))
}

#[tauri::command(async)]
pub fn task_doc_make(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<String> {
    let id = task_id(&id)?;
    let mut session = held(&session);
    answered(&session, id)?;
    if let Some(file) = file_of(&session, id) {
        return Ok(file);
    }
    let op = tisty_core::task_docs::made(
        &session.paths.docs(),
        &session.state,
        id,
        signing(&session.state),
    )?;
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
    let task = answered(&session, id)?;
    let file = file_of(&session, id).ok_or_else(|| Refusal::of("noSuchDoc"))?;
    if session.state.bolted(&file) {
        return Err(Refusal::of(match session.state.away(&file) {
            true => "documentAway",
            false => "documentLocked",
        }));
    }
    let ops = tisty_core::stages::reconciled(&task, before.as_deref(), &now);
    if !ops.is_empty() {
        session.commit_all(ops)?;
    }
    answered(&session, id)
}
