use tisty_core::model::{Task, TaskId};
use tisty_core::{Paths, Store};

// The log already holds the change; a page that could not be written catches up when opened.
pub(super) fn worded(
    paths: &Paths,
    store: &mut Store,
    id: TaskId,
    edit: impl FnOnce(&str, &Task) -> Option<String>,
) {
    let _ = tisty_core::task_docs::wrote(paths, store, id, edit);
}
