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
        .filter(|root| series(state, *root).is_some_and(|told| told.turns.len() > told.open))
        .count()
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
    fn a_gap_breaks_the_streak() {
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
    fn closing_after_the_due_date_is_recorded_as_the_delay_it_was() {
        let id = Ulid::generate();
        let mut add = TaskAdd::new("take the pill", "a0");
        add.date = Some(day("2026-08-01"));
        add.repeat = Some(daily(From::Due));
        let mut born = event(Op::TaskAdd { id, d: add });
        born.timestamp = at(0);

        let mut shut = event(Op::TaskDone { id });
        shut.timestamp = "2026-08-03T09:00:00Z".parse().unwrap();

        let state = State::replay(&[born, shut]);
        let told = series(&state, id).unwrap();

        assert_eq!(
            told.turns[0].late,
            Some(2),
            "two days late, and the record says so"
        );
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
            all[0].last, chain.ids[1],
            "the third turn is still open, so the series opens on the last closed one"
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
    fn a_series_opens_on_a_turn_the_archive_can_show() {
        let chain = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .turn("2026-08-02", daily(From::Due));

        let state = State::replay(&chain.events);
        let told = series(&state, chain.ids[0]).unwrap();

        assert_eq!(told.open, 1);
        assert_eq!(
            told.last, chain.ids[0],
            "the running turn is not in the archive, so opening it would show nothing"
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
    fn changing_the_cadence_does_not_rewrite_what_came_before() {
        let monthly = Repeat {
            from: From::Due,
            each: Cadence {
                every: 1,
                unit: Unit::Month,
            },
            until: None,
        };
        let told = Chain::new()
            .turn("2026-06-01", monthly)
            .done()
            .turn("2026-07-01", monthly)
            .done()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .told();

        assert_eq!(
            told.skipped, 0,
            "each pair is measured with the cadence that ruled it, not the newest one"
        );
        assert_eq!(told.kept, 3);
        assert_eq!(told.owed, 3);
    }

    #[test]
    fn a_turn_without_a_date_leaves_the_series_unmeasurable_instead_of_clean() {
        let mut chain = Chain::new().turn("2026-08-01", daily(From::Due)).done();
        let id = Ulid::generate();
        let mut add = TaskAdd::new("take the pill", "a0");
        add.repeat = Some(daily(From::Due));
        add.after = chain.ids.last().copied();
        chain.events.push(event(Op::TaskAdd { id, d: add }));
        chain.ids.push(id);

        let told = chain.told();

        assert!(
            !told.measurable,
            "a chain with a dateless turn cannot claim it counted every date"
        );
        assert_eq!(told.skipped, 0);
    }

    #[test]
    fn reopening_an_old_turn_does_not_wipe_out_what_it_owed() {
        let chain = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .turn("2026-08-02", daily(From::Due))
            .done()
            .turn("2026-08-03", daily(From::Due));

        let mut events = chain.events.clone();
        events.push(event(Op::TaskReopen { id: chain.ids[0] }));
        let state = State::replay(&events);
        let told = series(&state, chain.ids[1]).unwrap();

        assert_eq!(told.open, 2, "the reopened one and the running one");
        assert_eq!(
            told.owed, 2,
            "only the turn still running is not owed yet; a reopened one already came due"
        );
    }

    #[test]
    fn a_turn_with_two_successors_still_finds_itself_in_its_own_series() {
        let chain = Chain::new()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .turn("2026-08-02", daily(From::Due))
            .done();

        let rival = Ulid::generate();
        let mut add = TaskAdd::new("take the pill", "a0");
        add.date = Some(day("2026-08-02"));
        add.repeat = Some(daily(From::Due));
        add.after = Some(chain.ids[0]);
        let mut events = chain.events.clone();
        events.push(event(Op::TaskAdd { id: rival, d: add }));
        events.push(event(Op::TaskDone { id: rival }));

        let state = State::replay(&events);

        for id in [chain.ids[1], rival] {
            let told = series(&state, id).unwrap();
            assert!(
                told.turns.iter().any(|turn| turn.id == id),
                "a series that leaves out the turn it was asked about is a lie"
            );
        }
    }

    #[test]
    fn a_chain_that_points_at_itself_stops_instead_of_spinning() {
        let id = Ulid::generate();
        let mut add = TaskAdd::new("take the pill", "a0");
        add.date = Some(day("2026-08-01"));
        add.repeat = Some(daily(From::Due));
        add.after = Some(id);
        let state = State::replay(&[event(Op::TaskAdd { id, d: add })]);

        let told = series(&state, id).expect("a self-cycle is still a series of one");

        assert_eq!(told.turns.len(), 1);
    }

    #[test]
    fn asking_for_a_task_that_is_not_there_gives_nothing_instead_of_panicking() {
        let state = State::replay(&[]);

        assert!(series(&state, Ulid::generate()).is_none());
        assert_eq!(how_many(&state), 0);
        assert!(routines(&state).is_empty());
    }

    #[test]
    fn a_cadence_of_zero_leaves_no_gaps_instead_of_filling_the_calendar() {
        let never = Repeat {
            from: From::Due,
            each: Cadence {
                every: 0,
                unit: Unit::Day,
            },
            until: None,
        };
        let told = Chain::new()
            .turn("2026-08-01", never)
            .done()
            .turn("2026-09-01", never)
            .done()
            .told();

        assert_eq!(
            told.skipped, 0,
            "a cadence that never advances measures nothing"
        );
        assert_eq!(told.owed, 2);
    }

    #[test]
    fn a_series_with_an_end_carries_it_so_the_card_can_say_so() {
        let until = Repeat {
            from: From::Due,
            each: Cadence {
                every: 1,
                unit: Unit::Day,
            },
            until: Some("2026-08-31".parse().unwrap()),
        };
        let told = Chain::new().turn("2026-08-01", until).done().told();

        assert_eq!(
            told.repeat.and_then(|one| one.until),
            Some("2026-08-31".parse().unwrap())
        );
    }

    #[test]
    fn a_wild_gap_is_capped_instead_of_eating_the_memory() {
        let told = Chain::new()
            .turn("2000-01-01", daily(From::Due))
            .done()
            .turn("2026-08-01", daily(From::Due))
            .done()
            .told();

        assert_eq!(
            told.skipped, GAPS_AT_MOST,
            "the walk stops at the cap rather than walking a quarter of a century"
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
