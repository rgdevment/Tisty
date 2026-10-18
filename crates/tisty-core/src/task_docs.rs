use std::path::Path;

use ulid::Ulid;

use crate::event::{DocAdd, Op, Said};
use crate::model::{DocId, Kept, TaskId};
use crate::state::State;

/// The one line a task's document always carries, so the file knows its task wherever it goes.
pub const KEY: &str = "task";

// The random half is turned by a fixed word: the document never shares its task's id, nor the
// permissions an assistant holds over that task.
const TURNED: u128 = 0x5441_534b_5f44_4f43_5f4f;

/// Every machine names a task's document the same way, so two that make it at once make one.
pub fn file_of(task: TaskId) -> String {
    format!("t_{}-0001", task.to_string().to_ascii_lowercase())
}

pub fn id_of(task: TaskId) -> DocId {
    Ulid::from_parts(task.timestamp_ms(), task.random() ^ TURNED)
}

pub fn seed(task: TaskId) -> String {
    format!("---\n{KEY}: {task}\n---\n")
}

/// The task a document says it belongs to, read from its properties.
pub fn task_in(body: &str) -> Option<TaskId> {
    tisty_markdown::front::fields(body)
        .into_iter()
        .find(|one| one.key.eq_ignore_ascii_case(KEY))
        .and_then(|one| one.values.into_iter().next())
        .and_then(|said| Ulid::from_string(said.trim()).ok())
}

pub fn of(state: &State, task: TaskId) -> Option<&Kept> {
    state
        .docs
        .get(&id_of(task))
        .filter(|kept| kept.task_of == Some(task))
}

/// Writes the seed only where nothing is written yet, and hands back what the log has to learn.
pub fn made(root: &Path, state: &State, task: TaskId, by: Option<String>) -> crate::Result<Op> {
    let file = file_of(task);
    let body = match crate::docs::create_named(root, &file, &seed(task))? {
        true => seed(task),
        false => crate::docs::read(root, &file)?,
    };
    Ok(Op::DocAdd {
        id: id_of(task),
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
