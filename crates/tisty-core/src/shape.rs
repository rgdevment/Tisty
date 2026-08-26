use jiff::Timestamp;
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
    pub since: Option<Timestamp>,
    pub months: Vec<Month>,
}

pub fn shape(state: &State, most: usize) -> Shape {
    let mut shape = Shape::default();
    let mut months: std::collections::BTreeMap<String, usize> = Default::default();

    for task in state.tasks.values().filter(|one| one.is_archived()) {
        shape.closed += 1;
        if task.status == Status::Dropped {
            shape.dropped += 1;
        }
        if task.weight() > 0 {
            shape.told += 1;
        }

        let Some(at) = task.completed_at else {
            continue;
        };
        shape.since = Some(shape.since.map_or(at, |held| held.min(at)));
        let on = at.to_zoned(jiff::tz::TimeZone::system()).date();
        *months
            .entry(format!("{:04}-{:02}", on.year(), on.month()))
            .or_default() += 1;
    }

    let all: Vec<Month> = months
        .into_iter()
        .map(|(key, closed)| Month { key, closed })
        .collect();
    shape.months = all.into_iter().rev().take(most).rev().collect();
    shape
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, LogAdd, Op, TaskAdd};

    use crate::event::DeviceId;
    use ulid::Ulid;

    fn at(seconds: i64) -> Timestamp {
        Timestamp::from_second(1_770_000_000 + seconds).unwrap()
    }

    fn event(seconds: i64, op: Op) -> Event {
        Event::new(DeviceId("dev_a".into()), at(seconds), op)
    }

    fn closed(events: &mut Vec<Event>, when: i64, told: bool) -> Ulid {
        let id = Ulid::generate();
        events.push(event(
            when,
            Op::TaskAdd {
                id,
                d: TaskAdd::new("something", "a0"),
            },
        ));
        if told {
            events.push(event(
                when,
                Op::TaskLog {
                    id,
                    d: LogAdd::new(Ulid::generate(), "the certificate took nine days"),
                },
            ));
        }
        events.push(event(when, Op::TaskDone { id }));
        id
    }

    const MONTH: i64 = 60 * 60 * 24 * 31;

    #[test]
    fn the_shape_counts_what_is_closed_and_what_of_it_says_something() {
        let mut events = Vec::new();
        closed(&mut events, 0, true);
        closed(&mut events, 10, false);

        let told = shape(&State::replay(&events), 18);

        assert_eq!(told.closed, 2);
        assert_eq!(told.told, 1, "only one of them left anything written");
    }

    #[test]
    fn an_open_task_is_not_part_of_the_shape_of_the_archive() {
        let mut events = Vec::new();
        closed(&mut events, 0, false);
        events.push(event(
            10,
            Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new("still going", "a0"),
            },
        ));

        assert_eq!(shape(&State::replay(&events), 18).closed, 1);
    }

    #[test]
    fn the_months_come_back_in_order_and_only_the_last_ones() {
        let mut events = Vec::new();
        for n in 0..5 {
            closed(&mut events, n * MONTH, false);
        }

        let told = shape(&State::replay(&events), 3);

        assert_eq!(
            told.months.len(),
            3,
            "a strip has a width, and it is the recent end"
        );
        let keys: Vec<&str> = told.months.iter().map(|one| one.key.as_str()).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(
            keys, sorted,
            "a timeline that runs backwards is a bug, not a style"
        );
    }

    #[test]
    fn the_beginning_is_the_earliest_closing_and_not_the_first_one_read() {
        let mut events = Vec::new();
        closed(&mut events, 2 * MONTH, false);
        closed(&mut events, 0, false);

        let told = shape(&State::replay(&events), 18);

        assert_eq!(told.since, Some(at(0)));
    }
}
