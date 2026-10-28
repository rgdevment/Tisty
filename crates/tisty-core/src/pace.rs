use jiff::civil::Date;
use serde::Serialize;

use crate::model::Task;

/// Ten points of slack, so a part closed a day late does not turn the bar amber.
pub const SLACK: i64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Pace {
    OnTrack,
    Slow,
    Behind,
}

/// In percent, so the window draws the bar without knowing how many days or parts there are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Paced {
    pub pace: Pace,
    pub time: i64,
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
        // Rounded down, so a part still open never reads as all of them closed.
        done: 100 * closed / total,
    })
}

#[cfg(test)]
#[path = "pace_test.rs"]
mod tests;
