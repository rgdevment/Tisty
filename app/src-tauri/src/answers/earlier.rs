use std::sync::Mutex;

use tisty_core::event::DeviceId;
use tisty_core::witness::channel;
use tisty_core::{Op, Task, history};

use super::papers::{book_of, named_anew, one_step_back, writable};
use crate::{Answer, Refusal, Session, blamed, held, signing};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Earlier {
    print: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    wrote: Option<String>,
    replaced: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    via: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    by: Option<String>,
    bytes: u64,
    here: bool,
}

pub fn listed(session: &Session, id: &str, now: jiff::Timestamp) -> Answer<Vec<history::Row>> {
    let Some(kept) = session.state.docs.values().find(|one| one.file == id) else {
        return Err(Refusal::about("noSuchDoc", id.to_string()));
    };
    let log = session.log().map_err(|e| {
        blamed(
            channel::WINDOW,
            "the log could not be read for a history",
            e,
        )
    })?;
    let here = &session.config.device_id;
    let elsewhere = |device: &DeviceId| {
        device != here
            && session.config.agent_id.as_ref() != Some(device)
            && session.state.hosts.get(device) != Some(here)
    };
    Ok(history::rows(
        session.paths.data(),
        id,
        &history::logged(&log, kept.id),
        session.config.history_days(),
        now,
        elsewhere,
    ))
}

#[tauri::command(async)]
pub fn doc_versions(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<Earlier>> {
    let rows = listed(&held(&session), &id, jiff::Timestamp::now())?;
    Ok(rows
        .into_iter()
        .map(|one| Earlier {
            print: one.print,
            wrote: one.wrote.map(|at| at.to_string()),
            replaced: one.replaced.to_string(),
            via: one.via,
            by: one.device.map(|device| device.0),
            bytes: one.bytes,
            here: one.here,
        })
        .collect())
}

pub fn read(session: &Session, id: &str, print: &str) -> Answer<String> {
    history::read(session.paths.data(), id, print).ok_or_else(|| Refusal::of("versionGone"))
}

#[tauri::command(async)]
pub fn doc_version(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    print: String,
) -> Answer<String> {
    read(&held(&session), &id, &print)
}

fn page_of_task(session: &Session, id: &str) -> Answer<Option<Task>> {
    let Some(task) = tisty_core::task_docs::behind(&session.state)
        .get(id)
        .copied()
    else {
        return Ok(None);
    };
    match session.state.tasks.get(&task) {
        Some(one) if one.is_open() => Ok(Some(one.clone())),
        _ => Err(Refusal::of("taskClosed")),
    }
}

/// A task's page comes back with its words alone: its boxes stay as the task has them, since the file's may be stale.
pub fn put_back(session: &mut Session, id: &str, was: &str) -> Answer<tisty_core::docs::Doc> {
    if tisty_core::task_docs::task_of_file(id).is_some() {
        session.reload()?;
    }
    let task = page_of_task(session, id)?;
    writable(&session.state, id)?;
    let root = session.paths.docs();
    let now = tisty_core::docs::read(&root, id)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be read", e))?;
    let was = match &task {
        Some(task) => tisty_core::stages::ticked(task, was),
        None => was.to_string(),
    };
    let print = tisty_core::attach::printed(now.as_bytes());
    let undone = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .filter(|one| {
            one.compacted
                .as_ref()
                .is_some_and(|done| done.undone_by(&now, &was))
        })
        .map(|one| one.id);
    let made = tisty_core::docs::rewrite(&root, &session.writer(), id, &was, &print).map_err(
        |e| match e {
            tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
            _ => blamed(channel::WINDOW, "a document could not be written", e),
        },
    )?;
    let whole = match made {
        tisty_core::docs::Rewrite::Moved => return Err(Refusal::about("documentMoved", id)),
        tisty_core::docs::Rewrite::Made { whole, .. } => whole,
    };
    session.mind_body(id, &tisty_core::docs::settled(&whole));
    session.corpus.forget(id);
    let title = tisty_core::docs::titled(&whole);
    let book = book_of(session, id, &title);
    let hand = signing(&session.state);
    session.retell(id, &whole, hand);
    named_anew(session, book, id, &title);
    if let Some(task) = &task {
        settled(session, task, &now, &whole)?;
    }
    // The body is back already; a mark left standing is a lesser wrong than reporting the undo as failed.
    if let Some(doc) = undone
        && let Err(e) = session.commit(Op::DocUncompacted { id: doc })
    {
        tisty_core::witness::warn(
            tisty_core::witness::channel::STORE,
            "a document went back past its compaction but the mark stayed",
            &[("why", tisty_core::witness::Fact::Why(e.to_string()))],
        );
    }
    Ok(tisty_core::docs::Doc {
        title,
        id: id.to_string(),
    })
}

fn settled(session: &mut Session, task: &Task, before: &str, whole: &str) -> Answer<()> {
    let theirs = tisty_core::stages::ticked(task, before);
    let mut ops = tisty_core::stages::reconciled(task, Some(&theirs), whole);
    let at = jiff::Timestamp::now().as_second();
    let paged = tisty_core::paged::paged(task.id, &theirs, whole);
    ops.extend(tisty_core::paged::quieted(paged, &session.written, at));
    if ops.is_empty() {
        return Ok(());
    }
    let sent = ops.clone();
    session.commit_all(ops)?;
    tisty_core::paged::told(&mut session.written, &sent, at);
    Ok(())
}

pub fn went_back(session: &mut Session, id: &str) -> Answer<tisty_core::docs::Doc> {
    writable(&session.state, id)?;
    let Some(was) = one_step_back(session, id) else {
        return Err(Refusal::of("nothingKeptBeside"));
    };
    put_back(session, id, &was)
}

#[tauri::command(async)]
pub fn doc_back(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<tisty_core::docs::Doc> {
    let mut session = held(&session);
    went_back(&mut session, &id)
}

pub fn went_back_to(session: &mut Session, id: &str, print: &str) -> Answer<tisty_core::docs::Doc> {
    let was = read(session, id, print)?;
    put_back(session, id, &was)
}

#[tauri::command(async)]
pub fn doc_back_to(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    print: String,
) -> Answer<tisty_core::docs::Doc> {
    let mut session = held(&session);
    went_back_to(&mut session, &id, &print)
}

pub fn emptied(session: &Session, id: &str) -> Answer<usize> {
    history::empty(session.paths.data(), id).map_err(|e| match e {
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        _ => blamed(
            channel::WINDOW,
            "a document's history could not be emptied",
            e,
        ),
    })
}

#[tauri::command(async)]
pub fn doc_history_empty(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<usize> {
    emptied(&held(&session), &id)
}

pub fn kept_days(session: &mut Session, days: u32) -> Answer<u32> {
    if !history::DAYS.contains(&days) {
        return Err(Refusal::of("notAHistoryLimit"));
    }
    session.keep(|config| config.history_days = Some(days))?;
    Ok(session.config.history_days())
}

#[tauri::command]
pub fn keep_history_days(session: tauri::State<'_, Mutex<Session>>, days: u32) -> Answer<u32> {
    kept_days(&mut held(&session), days)
}

#[cfg(test)]
#[path = "earlier_test.rs"]
mod tests;
