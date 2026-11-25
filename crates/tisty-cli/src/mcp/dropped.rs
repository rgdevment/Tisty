use serde_json::{Value, json};
use tisty_core::State;
use tisty_core::model::{LogEntry, Task, TaskId};

use super::looking::kept_here;
use super::masking::masked;
use super::{Refused, when};

const REASON_AT_MOST: usize = 300;
pub(super) const SHOWN_AT_MOST: usize = 3;
// The window drops a task without asking why, so a note left that day stands for the reason; an older one is only history.
const REASON_WITHIN: jiff::SignedDuration = jiff::SignedDuration::from_hours(24);

/// What the person decided not to do reaches an agent as that decision alone; what they put out of sight, not at all.
pub(super) fn decided_not_to_do(task: &Task) -> bool {
    task.is_dropped() && !task.hidden
}

pub(super) fn reason(state: &State, task: &Task) -> Option<String> {
    let agreed = task
        .resolved
        .as_ref()
        .filter(|word| word.drop)
        .and_then(|word| task.log.iter().find(|entry| entry.id == word.entry));
    let entry = agreed.or_else(|| last_word(state, task))?;
    let plain = tisty_core::text::one_line(&entry.body);
    let plain = masked(state, &plain).unwrap_or(plain);
    let plain = kept_here(&plain);
    if plain.is_empty() {
        return None;
    }
    Some(match plain.chars().count() > REASON_AT_MOST {
        true => format!(
            "{}…",
            plain.chars().take(REASON_AT_MOST).collect::<String>()
        ),
        false => plain,
    })
}

// Only the person's own words stand for why they dropped it; an agent's note that day is not their reason.
fn last_word<'a>(state: &State, task: &'a Task) -> Option<&'a LogEntry> {
    let dropped = task.completed_at?;
    task.log
        .iter()
        .filter(|entry| entry.at <= dropped && dropped.duration_since(entry.at) <= REASON_WITHIN)
        .filter(|entry| {
            entry
                .by
                .as_ref()
                .is_none_or(|by| !state.assistants.contains(by))
        })
        .max_by_key(|entry| entry.at)
}

pub(super) fn card(state: &State, task: &Task) -> Value {
    let mut card = json!({
        "id": task.id.to_string(),
        "title": task.title,
        "status": "dropped",
    });
    if let Some(at) = task.completed_at {
        card["dropped"] = json!(when(at));
    }
    if let Some(why) = reason(state, task) {
        card["reason"] = json!(why);
    }
    card
}

pub(super) fn told(card: &Value) -> String {
    let title = card["title"].as_str().unwrap_or_default();
    let on = card["dropped"]
        .as_str()
        .map(|at| format!(" on {at}"))
        .unwrap_or_default();
    let why = match card["reason"].as_str() {
        Some(why) => format!("their reason: {why}"),
        None => "they wrote no reason".to_string(),
    };
    let end = match why.ends_with(['.', '!', '?', '…']) {
        true => "",
        false => ".",
    };
    format!("{title:?} was dropped{on} — the person decided not to do it; {why}{end}")
}

/// A write on a task the person dropped is refused by saying so, since `read` and `brain` show it.
pub(super) fn refused(state: &State, id: TaskId) -> Option<Refused> {
    let task = state.tasks.get(&id).filter(|one| decided_not_to_do(one))?;
    let card = card(state, task);
    Some(Refused::Tool(format!("{} {ONLY_THIS}", told(&card))))
}

pub(super) const ONLY_THIS: &str = "Its title, day and reason are all that reaches you, and it \
takes no writing: do not propose it again unless the person asks.";

/// Matched on what the agent may see of it, title and reason, so its other words stay unread.
pub(super) fn matching<'a>(state: &'a State, query: &str) -> Vec<&'a Task> {
    let wanted = tisty_core::text::telling(query);
    if wanted.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<(usize, &Task)> = state
        .tasks
        .values()
        .filter(|task| decided_not_to_do(task))
        .filter_map(|task| {
            let said = format!("{} {}", task.title, reason(state, task).unwrap_or_default());
            let words = tisty_core::text::telling(&said);
            let met = wanted
                .iter()
                .filter(|one| words.iter().any(|word| word.starts_with(one.as_str())))
                .count();
            (met > 0 && met * 2 >= wanted.len()).then_some((met, task))
        })
        .collect();
    found.sort_by_key(|(met, task)| {
        (
            std::cmp::Reverse(*met),
            std::cmp::Reverse(task.completed_at),
        )
    });
    found.into_iter().map(|(_, task)| task).collect()
}

/// The newest dropped task whose title says what this one does, so the same work is not filed again.
pub(super) fn alike<'a>(state: &'a State, title: &str) -> Option<&'a Task> {
    let wanted = tisty_core::text::subject(title);
    if wanted.is_empty() {
        return None;
    }
    state
        .tasks
        .values()
        .filter(|task| decided_not_to_do(task))
        .filter(|task| same_work(&wanted, &tisty_core::text::subject(&task.title)))
        .max_by_key(|task| task.completed_at)
}

// Holding back a proposal costs the agent a call, so it takes most words in common, not a shared prefix like «Tisty MCP:».
fn same_work(one: &str, other: &str) -> bool {
    if one == other {
        return true;
    }
    let words = |subject: &str| -> std::collections::BTreeSet<String> {
        tisty_core::text::telling(subject).into_iter().collect()
    };
    let (a, b) = (words(one), words(other));
    let shared = a.intersection(&b).count();
    let either = a.union(&b).count();
    shared >= 2 && shared * 4 >= either * 3
}

#[cfg(test)]
#[path = "dropped_test.rs"]
mod tests;
