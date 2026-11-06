use std::collections::BTreeMap;

use tisty_core::event::Remembered;
use tisty_core::{Op, State};

pub fn remembered(state: &State) -> &BTreeMap<String, Remembered> {
    &state.sharing
}

// Called once the envelopes have reached where they go, and never before: only then can the next version count on them.
pub fn sent(went: &[(String, Remembered)]) -> Vec<Op> {
    went.iter()
        .map(|(file, d)| Op::SharingKept {
            file: file.clone(),
            d: d.clone(),
        })
        .collect()
}

// Only what its owner stopped stays stopped: whoever edits through a link may be handed it again.
pub fn forgotten(gone: &[(String, Remembered)]) -> Vec<Op> {
    gone.iter()
        .map(|(file, was)| Op::SharingForgotten {
            file: file.clone(),
            stopped: was.admin_token.as_ref().map(|_| was.id.clone()),
        })
        .collect()
}
