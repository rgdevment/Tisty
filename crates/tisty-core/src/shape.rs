use serde::{Deserialize, Serialize};

use crate::model::Status;
use crate::state::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Month {
    pub key: String,
    pub closed: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Shape {
    pub closed: usize,
    pub dropped: usize,
    pub told: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<jiff::civil::Date>,
    pub months: Vec<Month>,
}

/// The zone comes from the caller: the core must not read the machine it runs on.
pub fn shape(
    state: &State,
    most: usize,
    zone: &jiff::tz::TimeZone,
    today: jiff::civil::Date,
) -> Shape {
    let mut shape = Shape::default();
    let mut months: std::collections::BTreeMap<String, usize> = Default::default();

    for task in state.tasks.values().filter(|one| one.is_archived()) {
        if task.status == Status::Dropped {
            shape.dropped += 1;
            continue;
        }
        shape.closed += 1;
        if task.weight() > 0 {
            shape.told += 1;
        }

        let Some(on) = task.counted_on(zone) else {
            continue;
        };
        shape.since = Some(shape.since.map_or(on, |held| held.min(on)));
        *months
            .entry(format!("{:04}-{:02}", on.year(), on.month()))
            .or_default() += 1;
    }

    shape.months = strip(&months, most, today);
    shape
}

/// A month with nothing closed is a bar at zero, not a month that never happened.
fn strip(
    months: &std::collections::BTreeMap<String, usize>,
    most: usize,
    today: jiff::civil::Date,
) -> Vec<Month> {
    if months.is_empty() || most == 0 {
        return Vec::new();
    }
    let last = today.first_of_month();
    let mut at = last
        .checked_sub(jiff::Span::new().months(most as i64 - 1))
        .unwrap_or(last);
    let mut all = Vec::with_capacity(most);
    while at <= last {
        let key = format!("{:04}-{:02}", at.year(), at.month());
        let closed = months.get(&key).copied().unwrap_or(0);
        all.push(Month { key, closed });
        at = match at.checked_add(jiff::Span::new().months(1)) {
            Ok(next) => next,
            Err(_) => break,
        };
    }
    all
}

#[cfg(test)]
#[path = "shape_test.rs"]
mod tests;
