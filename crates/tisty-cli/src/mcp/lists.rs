use tisty_core::State;
use tisty_core::model::{List, ListId, Task};

use super::chores::still_in_sight;

const INBOX_NAMES: [&str; 8] = [
    "inbox",
    "the inbox",
    "no list",
    "bandeja",
    "la bandeja",
    "bandeja de entrada",
    "la bandeja de entrada",
    "sin lista",
];

pub(super) fn named(state: &State) -> Vec<&List> {
    let by_name = |one: &&List, other: &&List| (&one.name, one.id).cmp(&(&other.name, other.id));
    let mut all: Vec<&List> = state
        .lists
        .values()
        .filter(|one| !one.archived)
        .filter(|one| {
            let mut alike = state.list_called(&one.name);
            alike.retain(|other| !other.archived);
            alike.into_iter().min_by(by_name).map(|first| first.id) == Some(one.id)
        })
        .collect();
    all.sort_unstable_by(by_name);
    all
}

pub(super) fn names(state: &State) -> Vec<&str> {
    named(state).iter().map(|one| one.name.as_str()).collect()
}

pub(super) fn open_in_the_inbox(state: &State) -> usize {
    open_where(state, |one| one.list.is_none())
}

pub(super) fn open_by_name(state: &State) -> Vec<(&str, usize)> {
    named(state)
        .into_iter()
        .map(|one| {
            let alike: Vec<ListId> = state
                .list_called(&one.name)
                .into_iter()
                .map(|other| other.id)
                .collect();
            let open = open_where(state, |task| {
                task.list.is_some_and(|at| alike.contains(&at))
            });
            (one.name.as_str(), open)
        })
        .collect()
}

fn open_where(state: &State, kept: impl Fn(&Task) -> bool) -> usize {
    state
        .tasks
        .values()
        .filter(|one| kept(one) && still_in_sight(one))
        .count()
}

pub(super) fn called<'a>(state: &'a State, name: &str) -> Result<Vec<&'a List>, String> {
    let all = state.find_list(name);
    if all.is_empty() {
        let names = names(state);
        return Err(match names.is_empty() {
            true => format!("there is no list called {name:?}, and there are none yet."),
            false => format!(
                "there is no list called {name:?}, and you cannot make one. These exist: {}.",
                names.join(", ")
            ),
        });
    }
    let only_in_part = state.list_called(name).is_empty();
    if all.len() > 1 && only_in_part {
        let names: Vec<&str> = all.iter().map(|one| one.name.as_str()).collect();
        return Err(format!(
            "{name:?} matches more than one list. Send the whole name: {}.",
            names.join(", ")
        ));
    }
    Ok(all)
}

pub(super) fn is_the_inbox(state: &State, name: &str) -> bool {
    let said = tisty_core::text::folded(name.trim().trim_start_matches('@').trim());
    state.list_called(name).is_empty() && INBOX_NAMES.contains(&said.as_str())
}

pub(super) fn inbox_named(state: &State) -> &'static str {
    INBOX_NAMES
        .into_iter()
        .find(|name| is_the_inbox(state, name))
        .unwrap_or(INBOX_NAMES[0])
}

#[cfg(test)]
#[path = "lists_test.rs"]
mod tests;
