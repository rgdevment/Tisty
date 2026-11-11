use tisty_core::model::{Kept, Task};
use tisty_core::{State, TaskId};

pub(super) fn named_document<'a>(state: &'a State, said: &str) -> Option<&'a Kept> {
    state
        .docs
        .values()
        .find(|one| one.file == said && state.doc_in_sight(one))
}

pub(super) fn named_task<'a>(state: &'a State, said: &str) -> Option<&'a Task> {
    said.parse::<TaskId>()
        .ok()
        .and_then(|id| state.tasks.get(&id))
        .filter(|one| !one.folded())
}

fn paged_task<'a>(state: &'a State, said: &str) -> Option<&'a Task> {
    tisty_core::task_docs::behind(state)
        .get(said)
        .and_then(|id| state.tasks.get(id))
        .filter(|one| !one.folded())
}

pub(super) fn not_a_task_id(state: &State, said: &str) -> String {
    if let Some(task) = paged_task(state, said) {
        return format!(
            "{said:?} is the page of the task {:?}, and a task is named by its own id: [{}].",
            task.title, task.id
        );
    }
    match named_document(state, said) {
        Some(kept) => format!(
            "{said:?} is {}, not a task: `read_doc` reads it (`outline_doc` first when it is \
             long), and `edit_doc` and `append_doc` write in it.",
            a_document(kept)
        ),
        None => {
            format!("{said:?} is not a task id. Use the `id` that `find` or `propose` gave you.")
        }
    }
}

pub(super) fn not_a_document(state: &State, which: &str) -> Option<String> {
    named_task(state, which).map(|task| {
        format!(
            "{which:?} is the task {:?}, not a document: `read` shows it and `note` writes on it.",
            task.title
        )
    })
}

pub(super) fn no_document_called(state: &State, which: &str) -> String {
    not_a_document(state, which)
        .unwrap_or_else(|| format!("no document here is called {which:?}. `docs` lists them all."))
}

pub(super) fn no_document_by_that_id(state: &State, which: &str) -> String {
    not_a_document(state, which).unwrap_or_else(|| {
        format!(
            "no document here is called {which:?}. Ids are opaque, like q7ntmzbm-0001, and a title \
             is not one — `docs` prints the id beside the title of every document."
        )
    })
}

pub(super) fn a_document(kept: &Kept) -> String {
    match kept.title.as_deref().map(str::trim) {
        Some(title) if !title.is_empty() => format!("the document {title:?}"),
        _ => "an untitled document".into(),
    }
}
