use serde_json::Value;
use tisty_core::model::{Kept, Task};
use tisty_core::{State, TaskId};

pub(super) const NAMES_A_DOCUMENT: [&str; 10] = [
    "doc",
    "docs",
    "page_of",
    "parent",
    "translation_of",
    "of",
    "after",
    "before",
    "order",
    "instead",
];

pub(super) const NAMES_A_FOLDER: [&str; 2] = ["folder", "inside"];

// The door and every tool read one value: a name without the blanks around it, an address as its id.
pub(super) fn as_the_tools_read(mut args: Value) -> Value {
    for (keys, read) in [
        (&NAMES_A_DOCUMENT[..], by_its_id as fn(&str) -> String),
        (&NAMES_A_FOLDER[..], |said: &str| said.trim().to_string()),
    ] {
        for key in keys {
            match args.get_mut(*key) {
                Some(Value::Array(many)) => many.iter_mut().for_each(|one| reread(one, read)),
                Some(one) => reread(one, read),
                None => {}
            }
        }
    }
    args
}

fn reread(said: &mut Value, read: fn(&str) -> String) {
    if let Some(text) = said.as_str() {
        *said = Value::String(read(text));
    }
}

fn by_its_id(said: &str) -> String {
    let said = said.trim();
    tisty_core::address::doc_asked(said)
        .unwrap_or(said)
        .to_string()
}

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
    let said = tisty_core::address::doc_asked(said).unwrap_or(said);
    if let Some(task) = paged_task(state, said) {
        return format!(
            "{said:?} is the page of the task {:?}, and a task is named by its own id: [{}].",
            task.title, task.id
        );
    }
    match named_document(state, said) {
        Some(kept) => format!(
            "{said:?} is {}, not a task: `workspace_read` with `doc` reads it (`view: \"card\"` \
             first when it is long), and `doc_edit` writes in it.",
            a_document(kept)
        ),
        None => {
            format!(
                "{said:?} is not a task id. Use the `id` that `workspace_search` or `task_create` gave you."
            )
        }
    }
}

pub(super) fn not_a_document(state: &State, which: &str) -> Option<String> {
    named_task(state, which).map(|task| {
        let owner = tisty_core::task_docs::owner(state, task.id);
        let page = match tisty_core::task_docs::of(state, owner) {
            Some(kept) if task.is_open() && !state.shut(kept.id) => format!(
                " Its page is `{}`: the document tools read and write it by that name.",
                kept.file
            ),
            Some(kept) => format!(" Its page is `{}`, to read by that name.", kept.file),
            None => String::new(),
        };
        format!(
            "{which:?} is the task {:?}, not a document: `workspace_read` shows it, `task_note` \
             adds to its journal, and `doc_edit` with its id adds to its page (`mode: \"append\"`) \
             or folds assistant notes into one digest (`mode: \"digest_task_notes\"`).{page}",
            task.title
        )
    })
}

pub(super) fn no_document_called(state: &State, which: &str) -> String {
    not_a_document(state, which).unwrap_or_else(|| {
        format!(
            "no document here is called {which:?}. `workspace_search` with a `query` finds one by its words."
        )
    })
}

pub(super) fn no_document_by_that_id(state: &State, which: &str) -> String {
    not_a_document(state, which).unwrap_or_else(|| {
        format!(
            "no document here is called {which:?}. Ids are opaque, like q7ntmzbm-0001, and a title \
             is not one — `workspace_search` gives the id beside the title of every document it finds."
        )
    })
}

pub(super) fn a_document(kept: &Kept) -> String {
    match kept.title.as_deref().map(str::trim) {
        Some(title) if !title.is_empty() => format!("the document {title:?}"),
        _ => "an untitled document".into(),
    }
}

#[cfg(test)]
#[path = "mistaken_test.rs"]
mod tests;
