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
mod tests {
    use super::*;
    use crate::event::{Event, LogAdd, Op, TaskAdd};
    use jiff::Timestamp;

    use crate::event::DeviceId;
    use ulid::Ulid;

    fn when() -> jiff::civil::Date {
        at(4 * MONTH).to_zoned(here()).date()
    }

    fn here() -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::UTC
    }

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
        events.push(event(when, Op::TaskDone { id, filled: false }));
        id
    }

    const MONTH: i64 = 60 * 60 * 24 * 31;

    #[test]
    fn a_backfilled_turn_lands_in_the_month_it_covered() {
        let id = Ulid::generate();
        let covered = at(0).to_zoned(here()).date();
        let marked = at(3 * MONTH);

        let mut d = TaskAdd::new("take the pill", "a0");
        d.date = Some(crate::model::DateSpec::all_day(covered, "UTC"));

        let mut state = State::default();
        state.apply(&event(0, Op::TaskAdd { id, d }));
        state.apply(&Event::new(
            DeviceId("dev_a".into()),
            marked,
            Op::TaskDone { id, filled: true },
        ));

        let told = shape(&state, 6, &here(), when());
        let key = format!("{:04}-{:02}", covered.year(), covered.month());
        assert_eq!(
            told.months.iter().find(|m| m.key == key).map(|m| m.closed),
            Some(1),
            "the pill was July's, whatever day you got round to ticking it"
        );

        let stamped = marked.to_zoned(here()).date();
        let other = format!("{:04}-{:02}", stamped.year(), stamped.month());
        assert_eq!(
            told.months
                .iter()
                .find(|m| m.key == other)
                .map(|m| m.closed),
            Some(0),
            "and it cannot be counted twice"
        );
    }

    #[test]
    fn the_shape_counts_what_is_closed_and_what_of_it_says_something() {
        let mut events = Vec::new();
        closed(&mut events, 0, true);
        closed(&mut events, 10, false);

        let told = shape(&State::replay(&events), 18, &here(), when());

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

        assert_eq!(
            shape(&State::replay(&events), 18, &here(), when()).closed,
            1
        );
    }

    #[test]
    fn the_months_come_back_in_order_and_only_the_last_ones() {
        let mut events = Vec::new();
        for n in 0..5 {
            closed(&mut events, n * MONTH, false);
        }

        let told = shape(&State::replay(&events), 3, &here(), when());

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
    fn a_month_with_nothing_closed_is_a_bar_at_zero_and_not_a_month_that_vanishes() {
        let mut events = Vec::new();
        closed(&mut events, 0, false);
        closed(&mut events, 3 * MONTH, false);

        let told = shape(&State::replay(&events), 5, &here(), when());

        assert_eq!(
            told.months.len(),
            5,
            "a strip is a timeline, not a bar chart"
        );
        assert!(
            told.months.iter().any(|one| one.closed == 0),
            "the quiet months have to be there for the busy ones to mean anything"
        );
        let keys: Vec<&str> = told.months.iter().map(|one| one.key.as_str()).collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        assert_eq!(keys, sorted);
    }

    #[test]
    fn what_was_given_up_does_not_swell_what_was_closed() {
        let mut events = Vec::new();
        let id = closed(&mut events, 0, false);
        events.push(event(
            10,
            Op::TaskDrop {
                id: Ulid::generate(),
            },
        ));
        let dropped = Ulid::generate();
        events.push(event(
            20,
            Op::TaskAdd {
                id: dropped,
                d: TaskAdd::new("something else", "a0"),
            },
        ));
        events.push(event(30, Op::TaskDrop { id: dropped }));

        let told = shape(&State::replay(&events), 18, &here(), when());

        assert_eq!(told.closed, 1, "{id} is the only one that was closed");
        assert_eq!(told.dropped, 1);
        assert_eq!(
            told.months.iter().map(|one| one.closed).sum::<usize>(),
            1,
            "a decision against something is not a closing"
        );
    }

    #[test]
    fn the_beginning_is_the_earliest_closing_and_not_the_first_one_read() {
        let mut events = Vec::new();
        closed(&mut events, 2 * MONTH, false);
        closed(&mut events, 0, false);

        let told = shape(&State::replay(&events), 18, &here(), when());

        assert_eq!(told.since, Some(at(0).to_zoned(here()).date()));
    }
}
