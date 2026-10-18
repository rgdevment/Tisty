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
    // A name in another case still names this task, so no guard is passed by changing it.
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

/// A routine's words are its first turn's, but they lead to the turn still open, or else the last.
pub fn behind(state: &State) -> std::collections::BTreeMap<String, TaskId> {
    let mut now: std::collections::BTreeMap<TaskId, TaskId> = std::collections::BTreeMap::new();
    let turns = state.tasks.iter().filter(|(_, one)| one.after.is_some());
    for id in turns.map(|(id, _)| id) {
        let first = owner(state, *id);
        let held = now.entry(first).or_insert(first);
        let open = |task: &TaskId| state.tasks.get(task).is_some_and(|it| it.is_open());
        if (open(id), *id) > (open(held), *held) {
            *held = *id;
        }
    }
    state
        .docs
        .values()
        .filter_map(|kept| {
            kept.task_of
                .map(|task| (kept.file.clone(), now.get(&task).copied().unwrap_or(task)))
        })
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
    let one = state.tasks.get(&task);
    let mut said = one
        .and_then(|one| one.description.as_deref())
        .map(str::trim)
        .filter(|said| !said.is_empty());
    let mut steps = one.map(|one| crate::stages::lines(&crate::stages::ordered(one)));
    let mut out = seed(task);
    let template = template.replace("\r\n", "\n");
    let template = template.trim();
    if !template.is_empty() {
        out.push('\n');
        let mut gap = false;
        for line in template.lines() {
            if std::mem::take(&mut gap) && !line.trim().is_empty() {
                out.push('\n');
            }
            out.push_str(line);
            out.push('\n');
            let Some(hint) = hinted(line) else {
                continue;
            };
            let held = match boxed(hint) {
                true => steps.take().filter(|lines| !lines.is_empty()),
                false => said.take().map(|said| format!("{said}\n")),
            };
            if let Some(held) = held {
                out.push('\n');
                out.push_str(&held);
                gap = true;
            }
        }
    }
    if let Some(said) = said {
        out.push_str(&format!("\n{said}\n"));
    }
    match one {
        Some(one) => crate::stages::ticked(one, &out),
        None => out,
    }
}

/// The words a description holds, written in the section the template keeps for them: the first
/// one with a plain hint. A page without one takes them at its end.
pub fn described(body: &str, said: &str) -> String {
    let said = said.trim();
    let lines: Vec<&str> = body.split_inclusive('\n').collect();
    let heading = |line: &str| line.trim_start().starts_with('#');
    let slot = (1..lines.len())
        .find(|at| heading(lines[at - 1]) && hinted(lines[*at]).is_some_and(|hint| !boxed(hint)));
    let Some(hint) = slot else {
        let mut out = body.trim_end_matches(['\n', '\r']).to_string();
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(said);
        out.push('\n');
        return out;
    };
    let end = (hint + 1..lines.len())
        .find(|at| heading(lines[*at]))
        .unwrap_or(lines.len());
    let mut out: String = lines[..=hint].concat();
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push('\n');
    out.push_str(said);
    out.push('\n');
    if end < lines.len() {
        out.push('\n');
        out.push_str(&lines[end..].concat());
    }
    out
}

// The window reads a box hint the same way, so both agree on which hint asks for steps.
pub(crate) fn boxed(hint: &str) -> bool {
    hint.strip_prefix('-')
        .is_some_and(|rest| rest.trim_start().starts_with("[ ]"))
}

fn hinted(line: &str) -> Option<&str> {
    line.trim()
        .strip_prefix("<!--")?
        .strip_suffix("-->")
        .map(str::trim)
}

/// `from` is the turn asking: a routine's document is its first turn's, seeded with today's steps.
pub fn made(
    root: &Path,
    state: &State,
    task: TaskId,
    from: TaskId,
    template: &str,
    by: Option<String>,
) -> crate::Result<Op> {
    let first = opened(state, from, template);
    made_with(root, state, task, &first, by).map(|(op, _)| op)
}

/// Made with the words a person already wrote; false when the file was there before them.
pub fn made_with(
    root: &Path,
    state: &State,
    task: TaskId,
    first: &str,
    by: Option<String>,
) -> crate::Result<(Op, bool)> {
    let (id, file) = planned(state, task);
    let created = crate::docs::create_named(root, &file, first)?;
    let body = match created {
        true => first.to_string(),
        false => crate::docs::read(root, &file)?,
    };
    let op = Op::DocAdd {
        id,
        d: DocAdd {
            file,
            order: crate::order::last_of(state.docs.values().map(|one| one.order.as_str())),
            by,
            said: Some(Said::of(&body)),
            task_of: Some(task),
            ..DocAdd::default()
        },
    };
    Ok((op, created))
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

#[cfg(test)]
#[path = "task_docs_test.rs"]
mod tests;
