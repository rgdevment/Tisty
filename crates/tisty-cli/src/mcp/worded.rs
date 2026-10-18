use tisty_core::model::{Task, TaskId};
use tisty_core::{Paths, Store};

use super::{opened, retold};

/// What an assistant changed on a task reaches its page too, when it has one, so the two agree.
/// A task with no page yet reads its words from the log, so nothing is written for it.
pub(super) fn worded(
    paths: &Paths,
    store: &mut Store,
    id: TaskId,
    edit: impl FnOnce(&str, &Task) -> Option<String>,
) {
    let Ok((state, _)) = opened(paths) else {
        return;
    };
    let owner = tisty_core::task_docs::owner(&state, id);
    let (Some(kept), Some(task)) = (
        tisty_core::task_docs::of(&state, owner),
        state.tasks.get(&id),
    ) else {
        return;
    };
    let file = kept.file.clone();
    let whole = tisty_core::docs::amend(&paths.docs(), paths.data(), &file, |was| {
        edit(was, task).filter(|now| now != was)
    });
    // The log already holds the change; a page that could not be written catches up when opened.
    if let Ok(Some(whole)) = whole {
        let _ = retold(&state, store, &file, &whole);
    }
}
