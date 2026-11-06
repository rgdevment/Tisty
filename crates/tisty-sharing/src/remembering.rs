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

pub fn forgotten(files: &[String]) -> Vec<Op> {
    files
        .iter()
        .map(|file| Op::SharingForgotten { file: file.clone() })
        .collect()
}
