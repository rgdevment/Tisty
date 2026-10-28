use jiff::civil::Date;
use serde::Serialize;

use crate::model::Task;
use crate::state::State;

/// How far behind the time gone its parts may fall, in hundredths, and still be on track.
pub const SLACK: i64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Pace {
    OnTrack,
    Slow,
    /// Past half its time with less than half the parts it should have closed by now.
    Behind,
}

/// Time gone against parts closed, for a task that started and runs to a deadline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Paced {
    pub pace: Pace,
    /// Percent of the days between ☀ and ⚑ already gone.
    pub time: i64,
    /// Percent of its parts closed.
    pub done: i64,
}

/// Counted in whole numbers, so a case on the very edge never falls the wrong side by rounding.
pub fn pace(task: &Task, open: usize, closed: usize, today: Date) -> Option<Paced> {
    let start = task.date.as_ref()?.date();
    let due = task.deadline.as_ref()?.date();
    let total = i64::try_from(open + closed).ok()?;
    let closed = i64::try_from(closed).ok()?;
    if !task.is_open() || total == 0 || start > today {
        return None;
    }
    let span = i64::from(start.until(due).ok()?.get_days()).max(1);
    let gone = i64::from(start.until(today).ok()?.get_days()).min(span);
    let pace = if 100 * closed * span >= (100 * gone - SLACK * span) * total {
        Pace::OnTrack
    } else if 2 * gone > span && 2 * closed * span < gone * total {
        Pace::Behind
    } else {
        Pace::Slow
    };
    Some(Paced {
        pace,
        time: (100 * gone + span / 2) / span,
        done: (100 * closed + total / 2) / total,
    })
}

/// The parts counted as the window counts them: those put out of sight are left out.
pub fn paced(state: &State, task: &Task, today: Date) -> Option<Paced> {
    let parts: Vec<&Task> = state
        .parts_of(task.id)
        .filter(|one| !one.folded())
        .collect();
    let open = parts.iter().filter(|one| one.is_open()).count();
    pace(task, open, parts.len() - open, today)
}

#[cfg(test)]
#[path = "pace_test.rs"]
mod tests;
