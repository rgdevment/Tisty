use tisty_core::model::{Task, TaskId};
use tisty_core::{Paths, Store};

/// What to add to an answer when the task's page could not take the change the log already holds.
pub(super) fn worded(
    paths: &Paths,
    store: &mut Store,
    id: TaskId,
    edit: impl FnOnce(&str, &Task) -> Option<String>,
) -> &'static str {
    match tisty_core::task_docs::wrote(paths, store, id, edit) {
        Ok(()) => "",
        Err(_) => {
            " The task holds it, but its page could not be written just now, so the page does not show it yet: say so to the person rather than trying again."
        }
    }
}
