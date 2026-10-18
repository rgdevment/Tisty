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
    let (head, number) = file.get(PREFIX.len()..)?.rsplit_once('-')?;
    let turn = number.parse::<u32>().ok().filter(|turn| *turn > 0)?;
    let task = Ulid::from_string(&head.to_ascii_uppercase()).ok()?;
    // A disk or a sync that keeps names without their case may hand the name back in either.
    file_at(task, turn)
        .eq_ignore_ascii_case(file)
        .then(|| (task, id_at(task, turn)))
}

/// The front matter's line naming the task is bookkeeping, never what a search was looking for.
pub(crate) fn bookkept(line: &str) -> bool {
    line.split_once(':')
        .is_some_and(|(key, _)| key.trim().eq_ignore_ascii_case(KEY))
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

/// A task leaving the log for good takes its document along, so its words do not outlive it.
pub fn erased_with(state: &State, task: TaskId) -> Vec<Op> {
    let mut ops: Vec<Op> = of(state, task)
        .map(|one| Op::DocDelete { id: one.id })
        .into_iter()
        .collect();
    ops.push(Op::TaskDelete { id: task });
    ops
}

pub fn owner(state: &State, task: TaskId) -> TaskId {
    let mut at = task;
    for _ in 0..state.tasks.len() {
        match state.tasks.get(&at).and_then(|one| one.after) {
            Some(before) if before != task && state.tasks.contains_key(&before) => at = before,
            _ => break,
        }
    }
    at
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

pub fn first_words(state: &State, task: TaskId) -> String {
    opened(state, task, "")
}

pub fn opened(state: &State, task: TaskId, template: &str) -> String {
    let mut out = seed(task);
    let template = template.trim_end();
    if !template.is_empty() {
        out.push_str(&format!("\n{template}\n"));
    }
    let Some(one) = state.tasks.get(&task) else {
        return out;
    };
    if let Some(said) = one
        .description
        .as_deref()
        .map(str::trim)
        .filter(|said| !said.is_empty())
    {
        out.push_str(&format!("\n{said}\n"));
    }
    // A step the template already lists is that stage, not a second line for it.
    let given: Vec<String> = crate::stages::stages(&out)
        .iter()
        .map(|one| crate::stages::title(&one.text))
        .collect();
    let steps: Vec<_> = crate::stages::ordered(one)
        .into_iter()
        .filter(|step| !given.contains(&crate::stages::title(&step.text)))
        .collect();
    if !steps.is_empty() {
        out.push('\n');
        out.push_str(&crate::stages::lines(&steps));
    }
    out
}

/// `from` is the turn asking: a routine's document is its first turn's, seeded with today's steps.
pub fn made(
    root: &Path,
    state: &State,
    task: TaskId,
    from: TaskId,
    by: Option<String>,
) -> crate::Result<Op> {
    let (id, file) = planned(state, task);
    let first = match fresh(root, state, task) {
        true => opened(state, from, &template_words(root, state, from)),
        false => String::new(),
    };
    let body = match crate::docs::create_named(root, &file, &first)? {
        true => first,
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

fn planned(state: &State, task: TaskId) -> (DocId, String) {
    match of(state, task) {
        Some(kept) => (kept.id, kept.file.clone()),
        None => {
            let at = turn(state, task);
            (id_at(task, at), file_at(task, at))
        }
    }
}

pub fn fresh(root: &Path, state: &State, task: TaskId) -> bool {
    crate::docs::resolve(root, &planned(state, task).1).is_ok_and(|at| !at.exists())
}

fn template_words(root: &Path, state: &State, task: TaskId) -> String {
    let Some(one) = state.tasks.get(&task) else {
        return String::new();
    };
    let all = crate::templates::found(root, state);
    crate::templates::picked(&all, state, one)
        .and_then(|template| crate::docs::read(root, &template.file).ok())
        .map(|text| crate::templates::body(&text))
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "task_docs_test.rs"]
mod tests;
