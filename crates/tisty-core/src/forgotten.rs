use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::State;
use crate::lately::{self, Used};
use crate::model::{DocId, Kept};

const DAY: u64 = 60 * 60 * 24;
pub const FORGOTTEN_AFTER: u64 = 60 * DAY;
pub const OPENED_LATELY: u64 = 7 * DAY;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Forgotten {
    pub doc: DocId,
    pub last: u64,
    pub last_opened_here: bool,
}

#[derive(Default)]
struct Seen {
    opened: Option<u64>,
    wrote: Option<u64>,
}

impl Seen {
    fn last(&self) -> Option<u64> {
        self.opened.max(self.wrote)
    }

    fn opened_within(&self, now: u64, span: u64) -> bool {
        self.opened.is_some_and(|at| now.saturating_sub(at) < span)
    }
}

pub fn opened(at: &Path, file: &str) {
    lately::used_as_named_on(at, file, lately::now());
}

pub fn opened_here(at: &Path) -> Used {
    lately::every_as_named(at)
}

pub fn forgotten(root: &Path, state: &State, opened: &Used, now: u64) -> Vec<Forgotten> {
    let mut found: Vec<Forgotten> = seen(root, state, opened)
        .into_iter()
        .filter_map(|(doc, seen)| {
            let last = seen.last()?;
            (now.saturating_sub(last) >= FORGOTTEN_AFTER).then(|| Forgotten {
                doc,
                last,
                last_opened_here: seen.opened == Some(last),
            })
        })
        .collect();
    found.sort_by_key(|one| (one.last, one.doc));
    found
}

pub fn at_random(root: &Path, state: &State, opened: &Used, now: u64, roll: u32) -> Option<DocId> {
    let all = seen(root, state, opened);
    let rested: Vec<DocId> = all
        .iter()
        .filter(|(_, seen)| !seen.opened_within(now, OPENED_LATELY))
        .map(|(doc, _)| *doc)
        .collect();
    let pool: Vec<DocId> = match rested.is_empty() {
        true => all.into_keys().collect(),
        false => rested,
    };
    let at = usize::try_from(roll).unwrap_or_default() % pool.len().max(1);
    pool.get(at).copied()
}

// A page is read as part of its document, and every language of a document as that document.
fn seen(root: &Path, state: &State, opened: &Used) -> BTreeMap<DocId, Seen> {
    let on_disk = crate::docs::names(root);
    let diaries = crate::diary::books(root, state);
    let mut seen: BTreeMap<DocId, Seen> = BTreeMap::new();
    for kept in state.papers() {
        let whole = kept
            .page_of
            .and_then(|up| state.docs.get(&up))
            .unwrap_or(kept);
        let counts = whole.page_of.is_none()
            && !state.held_away(whole)
            && whole.flagged.is_none()
            && !whole.brainstorm
            && !whole.sown
            && !crate::diary::within(&diaries, whole)
            && on_disk.contains(&whole.file);
        if !counts {
            continue;
        }
        let one = seen.entry(state.anchor_of(whole)).or_default();
        one.opened = one.opened.max(opened.get(&kept.file).copied());
        one.wrote = one.wrote.max(written(kept));
    }
    seen
}

fn written(kept: &Kept) -> Option<u64> {
    kept.wrote
        .or(kept.made)
        .and_then(|at| u64::try_from(at.as_second()).ok())
}

#[cfg(test)]
#[path = "forgotten_test.rs"]
mod tests;
