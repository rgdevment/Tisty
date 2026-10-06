use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Session, blamed, held};

/// Run after the round that brought the other side in: before it, nothing is twice yet.
#[tauri::command]
pub fn tidy_merged(session: tauri::State<'_, Mutex<Session>>) -> Answer<usize> {
    let mut session = held(&session);
    let ops = tisty_core::doubled::doubled(&session.state);
    let many = ops.len();
    if many > 0 {
        session.commit_all(ops).map_err(|e| {
            blamed(
                channel::STORE,
                "what the merge left twice was not tidied",
                e,
            )
        })?;
        witness::note(
            channel::STORE,
            "what a merge left twice was put together",
            &[("ops", Fact::Count(many))],
        );
    }
    Ok(many)
}
