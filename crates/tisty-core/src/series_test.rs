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
        self.then(Op::TaskDone { id, filled: false })
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
fn a_routine_that_has_never_been_closed_is_not_on_the_shelf() {
    let mut state = State::default();
    let id = Ulid::generate();
    let mut add = TaskAdd::new("water the plants", "a0");
    add.date = Some(day("2026-09-01"));
    add.repeat = Some(daily(From::Due));
    state.apply(&event(Op::TaskAdd { id, d: add }));

    assert!(
        routines(&state).is_empty(),
        "the shelf showed a series with no history yet"
    );
    assert_eq!(how_many(&state), 0, "and the count disagreed with it");
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

    let mut shut = event(Op::TaskDone { id, filled: false });
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
fn a_backfilled_turn_reports_no_lateness_because_none_was_measured() {
    let id = Ulid::generate();
    let mut d = TaskAdd::new("water the balcony", "a0");
    d.date = Some(day("2026-08-18"));
    d.repeat = Some(daily(From::Due));

    let mut state = State::default();
    state.apply(&event(Op::TaskAdd { id, d }));
    state.apply(&Event::new(
        DeviceId("dev_a".into()),
        at(864_000),
        Op::TaskDone { id, filled: true },
    ));

    let told = series(&state, id).expect("a routine of one turn still is a series");
    let last = told.turns.last().expect("one turn");
    assert_eq!(
        last.late, None,
        "ten days of marking late is not ten days of doing it late"
    );
    assert!(last.filled);
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
    events.push(event(Op::TaskDone {
        id: rival,
        filled: false,
    }));

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
