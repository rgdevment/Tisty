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
                    true => {
                        tisty_core::task_docs::template(&task, session.config.locale.as_deref())
                    }
                    false => "",
                };
                return Ok(Some(Opened {
                    file: String::new(),
                    body: tisty_core::task_docs::opened(&session.state, id, template),
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
    body: Option<String>,
) -> Answer<String> {
    let id = task_id(&id)?;
    let mut session = held(&session);
    // Another process may have written since, and a document made twice is two of them.
    session.reload()?;
    if let Some(file) = file_of(&session, id) {
        known(&session, id)?;
        // Someone else made the page while these words were typed over its template.
        if body.is_some() {
            return Err(Refusal::about("documentMoved", file));
        }
        return Ok(file);
    }
    let task = open_task(&session, id)?;
    let written = body.is_some();
    let first = body.unwrap_or_else(|| {
        let template = tisty_core::task_docs::template(&task, session.config.locale.as_deref());
        tisty_core::task_docs::opened(&session.state, id, template)
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
    renamed: Option<Vec<(String, String)>>,
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
    let renamed = renamed.unwrap_or_default();
    let mut ops = tisty_core::stages::reconciled_with(&task, before.as_deref(), &now, &renamed);
    let at = jiff::Timestamp::now().as_second();
    if let Some(before) = &before {
        let paged = tisty_core::paged::paged(task.id, before, &now);
        ops.extend(tisty_core::paged::quieted(paged, &session.written, at));
    }
    if ops.is_empty() {
        return Ok(None);
    }
    let sent = ops.clone();
    session.commit_all(ops)?;
    tisty_core::paged::told(&mut session.written, &sent, at);
    answered(&session, id).map(Some)
}

// A task's page coming back hands the task its properties and words; anything else is a document.
#[tauri::command(async)]
pub fn task_page_import(
    session: tauri::State<'_, Mutex<Session>>,
    from: String,
) -> Answer<Option<String>> {
    let from = std::path::Path::new(&from);
    let markdown = from
        .extension()
        .is_some_and(|one| one.eq_ignore_ascii_case("md") || one.eq_ignore_ascii_case("markdown"));
    if !markdown {
        return Ok(None);
    }
    let (raw, _) = super::carrying::read_any(from)?;
    let zone = jiff::tz::TimeZone::system();
    let tz = zone.iana_name().unwrap_or("UTC").to_string();
    let paths = {
        let mut session = held(&session);
        session.reload()?;
        let Some(id) = tisty_core::task_front::read(&raw, &tz).task else {
            return Ok(None);
        };
        if !session.state.tasks.contains_key(&id) {
            return Ok(None);
        }
        // Refused before the files beside it are brought in, so a refusal leaves none behind.
        open_task(&session, id)?;
        if let Some(file) = file_of(&session, id) {
            writable(&session.state, &file)?;
        }
        session.paths.clone()
    };
    let own: Vec<std::path::PathBuf> = [paths.data(), paths.config(), paths.cache()]
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .collect();
    let mut reach = |at: &std::path::Path| {
        let at = at
            .canonicalize()
            .map_err(|_| "no file is there".to_string())?;
        match own.iter().any(|one| at.starts_with(one)) {
            true => Err("it is one of Tisty's own files".to_string()),
            false => Ok(at),
        }
    };
    let (body, brought) = tisty_core::beside::brought(
        &paths,
        from,
        &raw,
        &mut tisty_core::beside::Way {
            reach: &mut reach,
            outside: "",
            notes: false,
        },
    );
    let mut session = held(&session);
    session.reload()?;
    let file = tisty_core::task_front::read(&body, &tz)
        .task
        .and_then(|id| file_of(&session, id).map(|file| (id, file)));
    let was = file
        .as_ref()
        .and_then(|(_, file)| tisty_core::docs::read(&session.paths.docs(), file).ok());
    let Some(taken) = tisty_core::task_front::taken_in(&session.state, &body, was.as_deref(), &tz)
    else {
        return Ok(None);
    };
    let id = taken.task;
    open_task(&session, id)?;
    let mut ops = brought.told;
    ops.extend(taken.ops);
    let mut over = file.map(|(_, file)| file);
    if let Some(file) = &over {
        writable(&session.state, file)?;
    } else {
        let (made, created) = tisty_core::task_docs::made_with(
            &session.paths.docs(),
            &session.state,
            tisty_core::task_docs::owner(&session.state, id),
            &taken.body,
            signing(&session.state),
        )
        .map_err(|e| blamed(channel::WINDOW, "a task's document could not be made", e))?;
        if let (false, Op::DocAdd { d, .. }) = (created, &made) {
            over = Some(d.file.clone());
        }
        ops.push(made);
    }
    if !ops.is_empty() {
        session.commit_all(ops)?;
    }
    if let Some(file) = over {
        let root = session.paths.docs();
        tisty_core::docs::write(&root, &file, &taken.body)
            .map_err(|e| blamed(channel::WINDOW, "a task's page could not be written", e))?;
        session.mind_body(&file, &tisty_core::docs::settled(&taken.body));
        session.corpus.forget(&file);
        let hand = signing(&session.state);
        session.retell(&file, &taken.body, hand);
    }
    Ok(Some(id.to_string()))
}

fn settled_after(session: &mut Session, id: TaskId) {
    if let Some(file) = file_of(session, id) {
        session.corpus.forget(&file);
    }
    let _ = session.reload();
}

pub(crate) fn closing_words(
    session: &mut Session,
    id: TaskId,
    summary: Option<&str>,
) -> tisty_core::round::Closing {
    let open = &mut *session;
    let closing =
        tisty_core::round::closing_words(&open.paths, &mut open.store, &open.state, id, summary);
    if summary.is_some_and(|one| !one.trim().is_empty()) {
        settled_after(session, id);
    }
    closing
}

pub(crate) fn round_cleared(session: &mut Session, id: TaskId, said: &str) {
    let open = &mut *session;
    tisty_core::round::round_cleared(&open.paths, &mut open.store, &open.state, id, said);
    settled_after(session, id);
}
