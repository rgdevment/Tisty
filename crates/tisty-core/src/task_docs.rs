use std::path::Path;

use ulid::Ulid;

use crate::event::{DocAdd, Op, Said};
use crate::model::{DocId, Kept, TaskId};
use crate::state::State;

pub const KEY: &str = "task";

// Turned by a fixed word so the document never answers to the permissions held over its task.
const TURNED: u128 = 0x5441_534b_5f44_4f43_5f4f;

const PREFIX: &str = "t_";

/// Named from the task and the turn, so machines agree and one made again after a delete is new.
pub fn file_at(task: TaskId, turn: u32) -> String {
    format!(
        "{PREFIX}{}-{turn:04}",
        task.to_string().to_ascii_lowercase()
    )
}

pub fn id_at(task: TaskId, turn: u32) -> DocId {
    Ulid::from_parts(
        task.timestamp_ms(),
        task.random() ^ TURNED ^ u128::from(turn),
    )
}

pub fn task_of_file(file: &str) -> Option<TaskId> {
    owned(file).map(|(task, _)| task)
}

/// The task and the id its own making would give a file, so taking it in early makes no twin.
pub fn owned(file: &str) -> Option<(TaskId, DocId)> {
    let (head, number) = file.strip_prefix(PREFIX)?.rsplit_once('-')?;
    let turn = number.parse::<u32>().ok()?;
    let task = Ulid::from_string(&head.to_ascii_uppercase()).ok()?;
    (file_at(task, turn) == file).then(|| (task, id_at(task, turn)))
}

/// The line naming the task is the file's own bookkeeping, never what a search was looking for.
pub fn bookkept(line: &str) -> bool {
    line.split_once(':').is_some_and(|(key, said)| {
        key.trim().eq_ignore_ascii_case(KEY) && Ulid::from_string(said.trim()).is_ok()
    })
}

pub fn seed(task: TaskId) -> String {
    format!("---\n{KEY}: {task}\n---\n")
}

pub fn task_in(body: &str) -> Option<TaskId> {
    tisty_markdown::front::fields(body)
        .into_iter()
        .find(|one| one.key.eq_ignore_ascii_case(KEY))
        .and_then(|one| one.values.into_iter().next())
        .and_then(|said| Ulid::from_string(said.trim()).ok())
}

pub fn of(state: &State, task: TaskId) -> Option<&Kept> {
    state.docs.values().find(|kept| kept.task_of == Some(task))
}

pub fn behind(state: &State) -> std::collections::BTreeMap<String, TaskId> {
    state
        .docs
        .values()
        .filter_map(|kept| kept.task_of.map(|task| (kept.file.clone(), task)))
        .collect()
}

// A name a deleted document held is swept from disk, and its id stays erased: both are skipped.
fn turn(state: &State, task: TaskId) -> u32 {
    let mut at = 1;
    while state.shed.contains(&file_at(task, at)) || state.is_erased(id_at(task, at)) {
        at += 1;
    }
    at
}

pub fn made(root: &Path, state: &State, task: TaskId, by: Option<String>) -> crate::Result<Op> {
    let (id, file) = match of(state, task) {
        Some(kept) => (kept.id, kept.file.clone()),
        None => {
            let at = turn(state, task);
            (id_at(task, at), file_at(task, at))
        }
    };
    let body = match crate::docs::create_named(root, &file, &seed(task))? {
        true => seed(task),
        false => crate::docs::read(root, &file)?,
    };
    Ok(Op::DocAdd {
        id,
        d: DocAdd {
            file,
            order: crate::order::last_of(state.docs.values().map(|one| one.order.as_str())),
            by,
            said: Some(Said::of(&body)),
            task_of: Some(task),
            ..DocAdd::default()
        },
    })
}

#[cfg(test)]
#[path = "task_docs_test.rs"]
mod tests;
