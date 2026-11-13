use serde_json::Value;
use tisty_core::docs::RestorePoint;
use tisty_core::model::TaskId;
use tisty_core::{Op, Paths, State, Store};

use super::super::asked::text;
use super::super::task_notes::left_as_assistants;
use super::super::{Refused, history, hitch, moved, opened, retold};
use super::importing::{as_an_assistant, broken_rule};
use super::reading::in_sight;

const WRITE_A_PAGE: [&str; 5] = [
    "write_doc",
    "append_doc",
    "edit_doc",
    "edit_table",
    "restore_doc",
];

pub(in crate::mcp) fn page_named(name: &str, args: &Value) -> Option<String> {
    let file = text(args, "doc")?;
    (WRITE_A_PAGE.contains(&name) && tisty_core::task_docs::task_of_file(&file).is_some())
        .then_some(file)
}

pub(in crate::mcp) fn page_written(name: &str, args: &Value) -> Option<String> {
    let only_shown = name == "edit_table" && text(args, "do").is_none_or(|asked| asked == "show");
    page_named(name, args).filter(|_| !only_shown)
}

struct PageWrite<'a> {
    file: &'a str,
    before: String,
    after: String,
    point: Option<RestorePoint>,
}

pub(in crate::mcp) fn on_its_page(
    paths: &Paths,
    file: &str,
    write: impl FnOnce() -> Result<Value, Refused>,
) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let Some(id) = tisty_core::task_docs::behind(&state).get(file).copied() else {
        return write();
    };
    if let Some(kept) = state.docs.values().find(|one| one.file == file) {
        in_sight(&state, kept, file)?;
    }
    if let Some(task) = state.tasks.get(&id).filter(|one| !one.is_open()) {
        return Err(history(task));
    }
    let root = paths.docs();
    let before = tisty_core::docs::read(&root, file).map_err(hitch)?;
    let point = tisty_core::docs::restore_point(paths.data(), file);
    let mut said = write()?;
    let after = tisty_core::docs::read(&root, file).map_err(hitch)?;
    if after != before {
        let page = PageWrite {
            file,
            before,
            after,
            point,
        };
        let settled = settled(paths, id, &page)?;
        told_also(&mut said, &settled);
        noted_as_assistants(paths, &page);
        let now = tisty_core::docs::read(&root, file).map_err(hitch)?;
        if now != page.after
            && let Some(print) = said.pointer_mut("/structuredContent/print")
        {
            *print = Value::String(tisty_core::attach::printed(now.as_bytes()));
        }
    }
    Ok(said)
}

// Written by the page's name, an assistant's notes are still theirs when a digest sorts them.
fn noted_as_assistants(paths: &Paths, page: &PageWrite) {
    let section = tisty_core::assistant_notes::section;
    if section(&page.before) != section(&page.after) {
        left_as_assistants(paths, page.file, &page.before, &page.after);
    }
}

fn told_also(said: &mut Value, more: &str) {
    if more.is_empty() {
        return;
    }
    for at in ["/content/0/text", "/structuredContent/said"] {
        if let Some(Value::String(text)) = said.pointer_mut(at) {
            text.push_str(more);
        }
    }
}

fn settled(paths: &Paths, id: TaskId, page: &PageWrite) -> Result<String, Refused> {
    let (state, mut store) = opened(paths)?;
    let Some(task) = state.tasks.get(&id) else {
        return Ok(String::new());
    };
    let (before, after) = (page.before.as_str(), page.after.as_str());
    let fields = tisty_markdown::front::fields;
    if fields(after) != fields(before) {
        undone(paths, &state, &mut store, page)?;
        return Err(Refused::Tool(format!(
            "the lines between `---` at the top of {:?}'s page belong to Tisty, and this write \
             changed them, so the page was put back as it was. A task's day, list, priority and \
             reminders never change from its page: `rules` «tasks» says which tool changes each, \
             and which are the person's alone. Write again keeping those lines as they are.",
            task.title
        )));
    }
    let mut ops = tisty_core::stages::reconciled(task, Some(before), after);
    if let Some(why) = broken_rule(task, &ops) {
        undone(paths, &state, &mut store, page)?;
        return Err(Refused::Tool(format!(
            "on {:?}'s page, {why}, so the page was put back as it was. Leave that box as it \
             stands and write again.",
            task.title
        )));
    }
    ops.extend(tisty_core::paged::paged(id, before, after));
    let ops: Vec<Op> = ops
        .into_iter()
        .filter_map(|op| as_an_assistant(&state, task, op))
        .collect();
    let steps = ops
        .iter()
        .filter(|op| !matches!(op, Op::TaskPaged { .. }))
        .count();
    if ops.is_empty() {
        return Ok(String::new());
    }
    let asked = ops.clone();
    let written = store
        .append_batch_unless(ops, |events| {
            let held = State::replay(events);
            held.tasks
                .get(&id)
                .is_none_or(|now| !now.is_open() || broken_rule(now, &asked).is_some())
        })
        .map_err(hitch)?;
    if written.is_none() {
        undone(paths, &state, &mut store, page)?;
        return Err(moved(task));
    }
    if steps == 0 {
        return Ok(String::new());
    }
    let (state, mut store) = opened(paths)?;
    if let Some(task) = state.tasks.get(&id) {
        let ticked = tisty_core::stages::ticked(task, after);
        put_back(paths, &state, &mut store, page.file, after, &ticked)?;
    }
    Ok(format!(
        " Its boxes went to {:?} as {steps} change(s) to its steps; a box ticked on the \
         person's task stays your word until they take it.",
        task.title
    ))
}

// A refused write leaves no trace: the page and its one step back are as they were before it.
fn undone(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    page: &PageWrite,
) -> Result<(), Refused> {
    let back = put_back(paths, state, store, page.file, &page.after, &page.before)?;
    if back && let Some(point) = &page.point {
        tisty_core::docs::set_restore_point(paths.data(), page.file, point).map_err(hitch)?;
    }
    Ok(())
}

/// Only over what this write left: a page the window saved since keeps the person's words.
fn put_back(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    file: &str,
    left: &str,
    body: &str,
) -> Result<bool, Refused> {
    if left == body {
        return Ok(true);
    }
    let whole = tisty_core::docs::rewrite_if(&paths.docs(), paths.data(), file, left, body)
        .map_err(hitch)?;
    match whole {
        Some(whole) => retold(state, store, file, &whole).map(|()| true),
        None => Ok(false),
    }
}
