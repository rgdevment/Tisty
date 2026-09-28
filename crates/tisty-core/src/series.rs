use std::collections::{HashMap, HashSet};

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::model::{Cadence, DateSpec, From, ListId, Repeat, Status, Tag, Task, TaskId};
use crate::state::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Turn {
    pub id: TaskId,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<DateSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub late: Option<i64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<jiff::civil::Date>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub told: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub filled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    /// The latest turn that is no longer open: the one the archive can actually show.
    pub last: TaskId,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list: Option<ListId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<Tag>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<Repeat>,
    pub turns: Vec<Turn>,
    pub kept: usize,
    /// Turns that came due plus the dates the cadence skipped: what was owed, not what exists.
    pub owed: usize,
    pub dropped: usize,
    pub open: usize,
    pub skipped: usize,
    pub streak: usize,
    pub longest: usize,
    pub measurable: bool,
}

/// A corrupt chain could otherwise walk the calendar forever between two turns.
const GAPS_AT_MOST: usize = 4096;

pub fn series(state: &State, id: TaskId) -> Option<Series> {
    let one = state.tasks.get(&id)?;
    if one.repeat.is_none() && one.after.is_none() {
        return None;
    }

    let chain = walked(state, id);
    let last = chain.last()?;
    let repeat = chain.iter().rev().find_map(|task| task.repeat);
    let dated = chain.iter().all(|task| task.date.is_some());
    let measurable = dated && repeat.is_some_and(|it| it.from == From::Due);

    let mut turns: Vec<Turn> = Vec::with_capacity(chain.len());
    for (at, task) in chain.iter().enumerate() {
        let then = chain[at.saturating_sub(1)].repeat;
        let gaps = match (at, then) {
            (0, _) => Vec::new(),
            (_, Some(over)) if over.from == From::Due => between(
                over.cadence(),
                chain[at - 1].date.as_ref(),
                task.date.as_ref(),
            ),
            _ => Vec::new(),
        };
        turns.push(Turn {
            id: task.id,
            status: task.status,
            due: task.date.clone(),
            closed: task.completed_at,
            late: overdue(task),
            gaps,
            told: task.weight() > 0,
            filled: task.filled,
            zone: task.closed_in.clone(),
        });
    }

    let kept = turns
        .iter()
        .filter(|turn| turn.status == Status::Done)
        .count();
    let dropped = turns
        .iter()
        .filter(|turn| turn.status == Status::Dropped)
        .count();
    let open = turns
        .iter()
        .filter(|turn| turn.status == Status::Open)
        .count();
    let skipped = turns.iter().map(|turn| turn.gaps.len()).sum();

    let running = usize::from(turns.last().is_some_and(|turn| turn.status == Status::Open));
    let owed = turns.len() - running + skipped;
    let (streak, longest) = run(&turns);

    let shown = chain
        .iter()
        .rev()
        .find(|task| task.is_archived())
        .unwrap_or(last);

    Some(Series {
        last: shown.id,
        title: last.title.clone(),
        list: last.list,
        tags: last.tags.clone(),
        repeat,
        turns,
        kept,
        owed,
        dropped,
        open,
        skipped,
        streak,
        longest,
        measurable,
    })
}

pub fn how_many(state: &State) -> usize {
    heads(state)
        .filter(|root| series(state, *root).is_some_and(|told| walked_at_all(&told)))
        .count()
}

/// A series whose every turn is still open has left no history for the archive to hold.
fn walked_at_all(told: &Series) -> bool {
    told.turns.len() > told.open
}

/// Climbing to the root per task is quadratic on a long chain; a root is spotted in one pass.
fn heads(state: &State) -> impl Iterator<Item = TaskId> + '_ {
    state
        .tasks
        .values()
        .filter(|task| task.repeat.is_some() || task.after.is_some())
        .filter(|task| {
            task.after
                .is_none_or(|before| !state.tasks.contains_key(&before))
        })
        .map(|task| task.id)
}

pub fn routines(state: &State) -> Vec<Series> {
    let mut all: Vec<Series> = heads(state)
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|root| series(state, root))
        .filter(walked_at_all)
        .collect();
    all.sort_by(|one, two| {
        two.turns
            .last()
            .map(|turn| turn.id)
            .cmp(&one.turns.last().map(|turn| turn.id))
    });
    all
}

/// Walks both ways so a forked chain still contains `from`, which walking down from the root loses.
fn walked(state: &State, from: TaskId) -> Vec<&Task> {
    let mut back: HashMap<TaskId, TaskId> = HashMap::new();
    for task in state.tasks.values() {
        if let Some(after) = task.after {
            back.entry(after)
                .and_modify(|held| *held = (*held).min(task.id))
                .or_insert(task.id);
        }
    }

    let mut before = Vec::new();
    let mut seen = HashSet::new();
    let mut climbing = state.tasks.get(&from).and_then(|task| task.after);
    while let Some(here) = climbing {
        if !seen.insert(here) {
            break;
        }
        let Some(task) = state.tasks.get(&here) else {
            break;
        };
        before.push(task);
        climbing = task.after;
    }
    before.reverse();

    let mut chain = before;
    let mut walking = Some(from);
    while let Some(here) = walking {
        if !seen.insert(here) {
            break;
        }
        let Some(task) = state.tasks.get(&here) else {
            break;
        };
        chain.push(task);
        walking = back.get(&here).copied().filter(|next| *next != here);
    }
    chain
}

fn between(
    step: Cadence,
    from: Option<&DateSpec>,
    to: Option<&DateSpec>,
) -> Vec<jiff::civil::Date> {
    let (Some(from), Some(to)) = (from, to) else {
        return Vec::new();
    };
    if step.every == 0 {
        return Vec::new();
    }
    let mut at = from.at;
    let mut gaps = Vec::new();
    while gaps.len() < GAPS_AT_MOST {
        let Some(next) = step.after(at) else { break };
        if next >= to.at {
            break;
        }
        gaps.push(next.date());
        at = next;
    }
    gaps
}

fn overdue(task: &Task) -> Option<i64> {
    if task.filled {
        return None;
    }
    let due = task.date.as_ref()?;
    let zone = jiff::tz::TimeZone::get(&due.tz).unwrap_or_else(|_| jiff::tz::TimeZone::system());
    let on = task.counted_on(&zone)?;
    Some(on.since(due.date()).ok()?.get_days() as i64)
}

fn run(turns: &[Turn]) -> (usize, usize) {
    let mut now = 0;
    let mut best = 0;
    for turn in turns {
        if !turn.gaps.is_empty() || turn.status == Status::Dropped {
            now = 0;
        }
        if turn.status == Status::Done {
            now += 1;
            best = best.max(now);
        }
    }
    (now, best)
}

#[cfg(test)]
#[path = "series_test.rs"]
mod tests;
