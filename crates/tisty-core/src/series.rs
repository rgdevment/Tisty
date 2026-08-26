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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
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
    let measurable = repeat.is_some_and(|it| it.from == From::Due);

    let mut turns: Vec<Turn> = Vec::with_capacity(chain.len());
    for (at, task) in chain.iter().enumerate() {
        let gaps = match (measurable, at) {
            (true, 0) => Vec::new(),
            (true, _) => between(
                repeat?.cadence(),
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

    let owed = turns.len() - open + skipped;
    let (streak, longest) = run(&turns);

    Some(Series {
        last: last.id,
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
    heads(state).count()
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
        .collect();
    all.sort_by(|one, two| {
        two.turns
            .last()
            .map(|turn| turn.id)
            .cmp(&one.turns.last().map(|turn| turn.id))
    });
    all
}

fn first(state: &State, from: TaskId) -> TaskId {
    let mut seen = HashSet::new();
    let mut here = from;
    while let Some(task) = state.tasks.get(&here) {
        let Some(before) = task.after else { break };
        if !seen.insert(before) || !state.tasks.contains_key(&before) {
            break;
        }
        here = before;
    }
    here
}

fn walked(state: &State, from: TaskId) -> Vec<&Task> {
    let mut back: HashMap<TaskId, TaskId> = HashMap::new();
    for task in state.tasks.values() {
        if let Some(after) = task.after {
            back.insert(after, task.id);
        }
    }

    let mut chain = Vec::new();
    let mut walking = Some(first(state, from));
    let mut seen = HashSet::new();
    while let Some(here) = walking {
        if !seen.insert(here) {
            break;
        }
        let Some(task) = state.tasks.get(&here) else {
            break;
        };
        chain.push(task);
        walking = back.get(&here).copied();
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
    let due = task.date.as_ref()?;
    let closed = task.completed_at?;
    let zone = jiff::tz::TimeZone::get(&due.tz).unwrap_or_else(|_| jiff::tz::TimeZone::system());
    let on = closed.to_zoned(zone).date();
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
mod tests {
    use super::*;
    use crate::event::DeviceId;
    use crate::event::{Event, Op, TaskAdd};
    use crate::model::Unit;
    use ulid::Ulid;

    fn at(seconds: i64) -> Timestamp {
        Timestamp::from_second(1_770_000_000 + seconds).unwrap()
    }

    fn event(op: Op) -> Event {
        Event::new(DeviceId("dev_a".into()), at(0), op)
    }

    fn daily(from: From) -> Repeat {
        Repeat {
            from,
            each: Cadence {
                every: 1,
                unit: Unit::Day,
            },
            until: None,
        }
    }

    fn day(text: &str) -> DateSpec {
        DateSpec::all_day(text.parse().unwrap(), "UTC")
    }

    struct Chain {
        events: Vec<Event>,
        ids: Vec<TaskId>,
    }

    impl Chain {
        fn new() -> Self {
            Self {
                events: Vec::new(),
                ids: Vec::new(),
            }
        }

        fn turn(mut self, on: &str, repeat: Repeat) -> Self {
            let id = Ulid::generate();
            let mut add = TaskAdd::new("take the pill", "a0");
            add.date = Some(day(on));
            add.repeat = Some(repeat);
            add.after = self.ids.last().copied();
            self.events.push(event(Op::TaskAdd { id, d: add }));
            self.ids.push(id);
            self
        }

        fn done(self) -> Self {
            let id = *self.ids.last().unwrap();
            self.then(Op::TaskDone { id })
        }

        fn given_up(self) -> Self {
            let id = *self.ids.last().unwrap();
            self.then(Op::TaskDrop { id })
        }

        fn then(mut self, op: Op) -> Self {
            self.events.push(event(op));
            self
        }

        fn told(&self) -> Series {
            let state = State::replay(&self.events);
            series(&state, *self.ids.last().unwrap()).expect("a chain is a series")
        }
    }

    #[test]
    fn a_series_gathers_the_whole_chain_from_any_turn_of_it() {
        let chain = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .turn("2026-08-02", daily(From::Due))
            .done()
            .turn("2026-08-03", daily(From::Due));

        let state = State::replay(&chain.events);
        let from_the_middle = series(&state, chain.ids[1]).unwrap();

        assert_eq!(from_the_middle.turns.len(), 3);
        assert_eq!(from_the_middle.turns[0].id, chain.ids[0]);
    }

    #[test]
    fn a_skipped_date_is_a_gap_the_series_can_point_at() {
        let told = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .turn("2026-08-04", daily(From::Due))
            .done()
            .told();

        assert_eq!(told.skipped, 2);
        assert_eq!(
            told.turns[1].gaps,
            vec![
                "2026-08-02".parse::<jiff::civil::Date>().unwrap(),
                "2026-08-03".parse().unwrap()
            ],
            "a gap the archive can name is worth more than a count"
        );
    }

    #[test]
    fn a_cadence_measured_from_the_closing_has_no_gaps_to_show() {
        let told = Chain::new()
            .turn("2026-08-01", daily(From::Done))
            .done()
            .turn("2026-08-04", daily(From::Done))
            .done()
            .told();

        assert!(!told.measurable, "nothing was skipped: the chain moved on");
        assert_eq!(told.skipped, 0);
    }

    #[test]
    fn giving_one_up_is_not_the_same_as_forgetting_it() {
        let told = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .given_up()
            .turn("2026-08-02", daily(From::Due))
            .done()
            .told();

        assert_eq!(told.dropped, 1);
        assert_eq!(told.skipped, 0, "a date decided against was never missed");
    }

    #[test]
    fn a_gap_breaks_the_streak_and_being_late_does_not() {
        let told = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .turn("2026-08-02", daily(From::Due))
            .done()
            .turn("2026-08-05", daily(From::Due))
            .done()
            .turn("2026-08-06", daily(From::Due))
            .done()
            .told();

        assert_eq!(told.streak, 2, "the gap before the third turn cut it");
        assert_eq!(told.longest, 2);
        assert_eq!(told.kept, 4);
    }

    #[test]
    fn two_series_that_share_a_title_never_mix() {
        let mine = Chain::new().turn("2026-08-01", daily(From::Due)).done();
        let mut both = mine.events.clone();
        let theirs = Chain::new().turn("2026-08-01", daily(From::Due)).done();
        both.extend(theirs.events.clone());

        let state = State::replay(&both);
        let told = series(&state, mine.ids[0]).unwrap();

        assert_eq!(
            told.turns.len(),
            1,
            "the chain says who belongs, never the words on the front"
        );
    }

    #[test]
    fn every_chain_is_gathered_once_and_never_per_turn() {
        let chain = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .turn("2026-08-02", daily(From::Due))
            .done()
            .turn("2026-08-03", daily(From::Due));

        let mut events = chain.events.clone();
        let alone = Ulid::generate();
        events.push(event(Op::TaskAdd {
            id: alone,
            d: TaskAdd::new("buy bread", "a0"),
        }));

        let all = routines(&State::replay(&events));

        assert_eq!(all.len(), 1, "three turns are one routine, not three");
        assert_eq!(all[0].turns.len(), 3);
        assert_eq!(
            all[0].last, chain.ids[2],
            "a series opens on its latest turn"
        );
    }

    #[test]
    fn a_chain_is_counted_without_climbing_it_once_per_turn() {
        let mut chain = Chain::new().turn("2026-08-01", daily(From::Due)).done();
        for at in 2..=28 {
            chain = chain
                .turn(&format!("2026-08-{at:02}"), daily(From::Due))
                .done();
        }

        let state = State::replay(&chain.events);

        assert_eq!(how_many(&state), 1);
        assert_eq!(
            series(&state, chain.ids[0]).unwrap().turns.len(),
            28,
            "counting must not cost what walking costs"
        );
    }

    #[test]
    fn what_was_owed_counts_the_dates_that_went_by_and_not_only_the_turns() {
        let told = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .turn("2026-08-04", daily(From::Due))
            .done()
            .told();

        assert_eq!(told.kept, 2);
        assert_eq!(told.skipped, 2);
        assert_eq!(
            told.owed, 4,
            "two kept and two missed is four occasions, never two out of two"
        );
    }

    #[test]
    fn a_turn_still_open_is_not_counted_as_one_that_was_missed() {
        let told = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .turn("2026-08-02", daily(From::Due))
            .done()
            .turn("2026-08-03", daily(From::Due))
            .told();

        assert_eq!(told.turns.len(), 3);
        assert_eq!(told.kept, 2);
        assert_eq!(
            told.open, 1,
            "an endless routine always has one turn still running"
        );
        assert_eq!(
            told.turns.len() - told.open,
            2,
            "what has come due is what can be judged"
        );
    }

    #[test]
    fn a_task_that_repeats_nothing_is_not_a_series() {
        let id = Ulid::generate();
        let state = State::replay(&[event(Op::TaskAdd {
            id,
            d: TaskAdd::new("buy bread", "a0"),
        })]);

        assert!(series(&state, id).is_none());
    }
}
