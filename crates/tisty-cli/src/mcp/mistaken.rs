use tisty_core::model::Kept;
use tisty_core::{State, TaskId};

pub(super) fn not_a_task_id(state: &State, said: &str) -> String {
    let doc = state
        .docs
        .values()
        .find(|one| one.file == said && state.doc_in_sight(one));
    match doc {
        Some(kept) => format!(
            "{said:?} is {}, not a task: `read_doc` it, or `outline_doc` first when it is long.",
            a_document(kept)
        ),
        None => {
            format!("{said:?} is not a task id. Use the `id` that `find` or `propose` gave you.")
        }
    }
}

pub(super) fn no_document_called(state: &State, which: &str) -> String {
    let task = which
        .parse::<TaskId>()
        .ok()
        .and_then(|id| state.tasks.get(&id))
        .filter(|one| !one.folded());
    match task {
        Some(task) => format!(
            "{which:?} is the task {:?}, not a document: `read` it.",
            task.title
        ),
        None => format!("no document here is called {which:?}. `docs` lists them all."),
    }
}

fn a_document(kept: &Kept) -> String {
    match kept.title.as_deref().map(str::trim) {
        Some(title) if !title.is_empty() => format!("the document {title:?}"),
        _ => "an untitled document".into(),
    }
}
