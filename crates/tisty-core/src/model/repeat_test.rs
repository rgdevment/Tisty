use super::*;
use jiff::civil::date;

fn spec(at: DateTime) -> DateSpec {
    DateSpec::floating(at, "Europe/Madrid")
}

const EVERY_WEEK: Repeat = Repeat::due(Cadence {
    every: 1,
    unit: Unit::Week,
});

#[test]
fn a_weekly_task_lands_on_the_same_weekday() {
    let due = spec(date(2026, 8, 4).at(9, 0, 0, 0));
    let next = EVERY_WEEK
        .next(
            Some(&due),
            date(2026, 8, 4).at(9, 30, 0, 0),
            date(2026, 8, 4).at(0, 0, 0, 0),
            "Europe/Madrid",
        )
        .unwrap();

    assert_eq!(next.at, date(2026, 8, 11).at(9, 0, 0, 0));
}

#[test]
fn a_fixed_schedule_does_not_drift_when_you_finish_late() {
    let due = spec(date(2026, 8, 4).at(9, 0, 0, 0));
    let next = EVERY_WEEK
        .next(
            Some(&due),
            date(2026, 8, 6).at(20, 0, 0, 0),
            date(2026, 8, 6).at(0, 0, 0, 0),
            "Europe/Madrid",
        )
        .unwrap();

    assert_eq!(
        next.at,
        date(2026, 8, 11).at(9, 0, 0, 0),
        "it followed the doing"
    );
}

#[test]
fn a_long_gap_does_not_emit_a_backlog() {
    let due = spec(date(2026, 8, 4).at(9, 0, 0, 0));
    let next = EVERY_WEEK
        .next(
            Some(&due),
            date(2026, 8, 25).at(9, 0, 0, 0),
            date(2026, 8, 25).at(0, 0, 0, 0),
            "Europe/Madrid",
        )
        .unwrap();

    assert_eq!(next.at, date(2026, 9, 1).at(9, 0, 0, 0));
}

#[test]
fn a_monthly_one_stays_on_its_day_however_late_it_is_paid() {
    let monthly = Repeat::done(Cadence {
        every: 1,
        unit: Unit::Month,
    });
    let mut due = spec(date(2026, 1, 1).at(9, 0, 0, 0));

    for (paid, expected) in [
        (date(2026, 1, 4), date(2026, 2, 1)),
        (date(2026, 2, 13), date(2026, 3, 1)),
        (date(2026, 3, 30), date(2026, 4, 1)),
    ] {
        let next = monthly
            .next(
                Some(&due),
                paid.at(9, 0, 0, 0),
                paid.at(0, 0, 0, 0),
                "Europe/Madrid",
            )
            .unwrap();
        assert_eq!(next.at.date(), expected, "paid on {paid}");
        due = next;
    }
}

#[test]
fn a_relative_one_counts_from_when_it_was_done() {
    let every_three = Repeat::done(Cadence {
        every: 3,
        unit: Unit::Day,
    });
    let due = spec(date(2026, 8, 4).at(9, 0, 0, 0));
    let next = every_three
        .next(
            Some(&due),
            date(2026, 8, 9).at(18, 0, 0, 0),
            date(2026, 8, 9).at(0, 0, 0, 0),
            "Europe/Madrid",
        )
        .unwrap();

    assert_eq!(next.at, date(2026, 8, 12).at(9, 0, 0, 0));
}

#[test]
fn a_time_of_day_stays_where_it_was_asked_for() {
    let daily = Repeat::done(Cadence {
        every: 1,
        unit: Unit::Day,
    });
    let due = spec(date(2026, 8, 11).at(10, 0, 0, 0));
    let next = daily
        .next(
            Some(&due),
            date(2026, 8, 11).at(8, 4, 0, 0),
            date(2026, 8, 11).at(0, 0, 0, 0),
            "Europe/Madrid",
        )
        .unwrap();

    assert_eq!(next.at, date(2026, 8, 12).at(10, 0, 0, 0));
}

#[test]
fn a_relative_one_needs_no_date_to_start_from() {
    let every_day = Repeat::done(Cadence {
        every: 1,
        unit: Unit::Day,
    });
    let next = every_day
        .next(
            None,
            date(2026, 8, 9).at(18, 0, 0, 0),
            date(2026, 8, 9).at(0, 0, 0, 0),
            "Europe/Madrid",
        )
        .unwrap();

    assert_eq!(next.at, date(2026, 8, 10).at(0, 0, 0, 0));
    assert!(!next.has_time);
}

#[test]
fn a_fixed_one_without_a_date_repeats_nothing() {
    assert!(
        EVERY_WEEK
            .next(
                None,
                date(2026, 8, 9).at(9, 0, 0, 0),
                date(2026, 8, 9).at(0, 0, 0, 0),
                "Europe/Madrid"
            )
            .is_none()
    );
}

#[test]
fn the_last_day_of_a_month_survives_a_shorter_one() {
    let monthly = Repeat::due(Cadence {
        every: 1,
        unit: Unit::Month,
    });
    let due = spec(date(2026, 1, 31).at(9, 0, 0, 0));
    let next = monthly
        .next(
            Some(&due),
            date(2026, 1, 31).at(9, 0, 0, 0),
            date(2026, 1, 31).at(0, 0, 0, 0),
            "Europe/Madrid",
        )
        .unwrap();

    assert_eq!(next.at, date(2026, 2, 28).at(9, 0, 0, 0));
}

#[test]
fn a_cadence_beyond_what_a_calendar_holds_repeats_nothing() {
    for every in [19999, 40000, u16::MAX] {
        let absurd = Repeat::done(Cadence {
            every,
            unit: Unit::Year,
        });
        assert!(
            absurd
                .next(
                    None,
                    date(2026, 8, 9).at(9, 0, 0, 0),
                    date(2026, 8, 9).at(0, 0, 0, 0),
                    "Europe/Madrid"
                )
                .is_none(),
            "«every {every} years» should not be a repeat"
        );
    }
}

#[test]
fn a_cadence_of_zero_repeats_nothing() {
    let never = Repeat::done(Cadence {
        every: 0,
        unit: Unit::Day,
    });
    assert!(
        never
            .next(
                None,
                date(2026, 8, 9).at(9, 0, 0, 0),
                date(2026, 8, 9).at(0, 0, 0, 0),
                "Europe/Madrid"
            )
            .is_none()
    );
}

#[test]
fn the_timezone_and_the_shape_of_the_date_carry_over() {
    let due = DateSpec::fixed(date(2026, 8, 4).at(9, 0, 0, 0), "Europe/Madrid");
    let next = EVERY_WEEK
        .next(
            Some(&due),
            date(2026, 8, 4).at(9, 0, 0, 0),
            date(2026, 8, 4).at(0, 0, 0, 0),
            "Europe/Madrid",
        )
        .unwrap();

    assert_eq!(next.tz, "Europe/Madrid");
    assert!(!next.floating);
    assert!(next.has_time);
}
