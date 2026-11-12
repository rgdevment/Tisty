use serde_json::Value;
use tisty_core::model::TaskId;
use tisty_core::{Op, Paths, State, Store};

use super::super::asked::text;
use super::super::{Refused, history, hitch, moved, opened, retold};
use super::importing::{as_an_assistant, broken_rule};

const WRITE_A_PAGE: [&str; 5] = [
    "write_doc",
    "append_doc",
    "edit_doc",
    "edit_table",
    "restore_doc",
];

pub(in crate::mcp) fn page_written(name: &str, args: &Value) -> Option<String> {
    let file = text(args, "doc")?;
    (WRITE_A_PAGE.contains(&name) && tisty_core::task_docs::task_of_file(&file).is_some())
        .then_some(file)
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
    if let Some(task) = state.tasks.get(&id).filter(|one| !one.is_open()) {
        return Err(history(task));
    }
    let root = paths.docs();
    let before = tisty_core::docs::read(&root, file).map_err(hitch)?;
    let mut said = write()?;
    let after = tisty_core::docs::read(&root, file).map_err(hitch)?;
    if after != before {
        let settled = settled(paths, id, file, &before, &after)?;
        if let Some(text) = said["content"][0]["text"].as_str() {
            said["content"][0]["text"] = Value::String(format!("{text}{settled}"));
        }
    }
    Ok(said)
}

fn settled(
    paths: &Paths,
    id: TaskId,
    file: &str,
    before: &str,
    after: &str,
) -> Result<String, Refused> {
    let (state, mut store) = opened(paths)?;
    let Some(task) = state.tasks.get(&id) else {
        return Ok(String::new());
    };
    let owner = tisty_core::task_docs::owner(&state, id);
    if tisty_core::task_docs::task_in(after) != Some(owner) {
        write_page(paths, &state, &mut store, file, before)?;
        return Err(Refused::Tool(format!(
            "the lines between `---` at the top of {:?}'s page name its task, and the page \
             lost them, so it was put back as it was. Write again keeping those lines as they are.",
            task.title
        )));
    }
    let shown = tisty_core::stages::ticked(task, before);
    let mut ops = tisty_core::stages::reconciled(task, Some(&shown), after);
    if let Some(why) = broken_rule(task, &ops) {
        write_page(paths, &state, &mut store, file, before)?;
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
    if !ops.is_empty() {
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
            write_page(paths, &state, &mut store, file, before)?;
            return Err(moved(task));
        }
    }
    reticked(paths, id, file, after)?;
    Ok(match steps {
        0 => String::new(),
        many => format!(
            " Its boxes went to {:?} as {many} change(s) to its steps; a box ticked on the \
             person's task stays your word until they take it.",
            task.title
        ),
    })
}

fn reticked(paths: &Paths, id: TaskId, file: &str, after: &str) -> Result<(), Refused> {
    let (state, mut store) = opened(paths)?;
    let Some(task) = state.tasks.get(&id) else {
        return Ok(());
    };
    let now = tisty_core::stages::ticked(task, after);
    if now != after {
        write_page(paths, &state, &mut store, file, &now)?;
    }
    Ok(())
}

fn write_page(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    file: &str,
    body: &str,
) -> Result<(), Refused> {
    tisty_core::docs::write(&paths.docs(), file, body).map_err(hitch)?;
    retold(state, store, file, body)
}
