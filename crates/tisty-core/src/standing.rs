use jiff::civil::Date;
use serde::Serialize;

use crate::model::Task;

/// How many days before its deadline a task starts to count as due soon.
pub const DUE_SOON_DAYS: i32 = 3;

/// Where a task stands on a given day, read from ☀ `date` and ⚑ `deadline` together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Standing {
    Undated,
    /// Its day, or the start it was given, has not come yet; or its deadline is still far.
    Ahead,
    Today,
    /// The day it was meant for passed and it has no deadline: late, never overdue.
    Late,
    /// Started and still well before its deadline.
    Underway,
    DueSoon,
    Overdue,
}

/// With a deadline, ☀ is the day the work starts; alone, it is the day it is meant for.
pub fn standing(task: &Task, today: Date) -> Standing {
    let on = task.date.as_ref().map(|d| d.date());
    let Some(due) = task.deadline.as_ref().map(|d| d.date()) else {
        return match on {
            None => Standing::Undated,
            Some(day) if day < today => Standing::Late,
            Some(day) if day == today => Standing::Today,
            Some(_) => Standing::Ahead,
        };
    };
    if due < today {
        return Standing::Overdue;
    }
    if today
        .until(due)
        .is_ok_and(|left| left.get_days() <= DUE_SOON_DAYS)
    {
        return Standing::DueSoon;
    }
    match on {
        Some(start) if start <= today => Standing::Underway,
        _ => Standing::Ahead,
    }
}

#[cfg(test)]
#[path = "standing_test.rs"]
mod tests;
