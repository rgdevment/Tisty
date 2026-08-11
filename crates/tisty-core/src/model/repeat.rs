use jiff::civil::DateTime;
use serde::{Deserialize, Serialize};

use super::DateSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Day,
    Week,
    Month,
    Year,
}

/// The weekday or day of month comes from the date it is anchored to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cadence {
    pub every: u16,
    pub unit: Unit,
}

/// The bin goes out every Tuesday; the plants are watered three days after you
/// last watered them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "from", content = "each")]
pub enum Repeat {
    Due(Cadence),
    Done(Cadence),
}

impl Repeat {
    pub fn cadence(self) -> Cadence {
        match self {
            Repeat::Due(c) | Repeat::Done(c) => c,
        }
    }

    /// `None` when there is nothing to count from.
    pub fn next(
        self,
        due: Option<&DateSpec>,
        done: DateTime,
        today: DateTime,
        zone: &str,
    ) -> Option<DateSpec> {
        let step = self.cadence();
        if step.every == 0 {
            return None;
        }

        match self {
            // A month or a year counts off the calendar even when it was said
            // as an interval: rent paid on the 4th, the 13th and the 30th would
            // otherwise walk down the month and skip one entirely.
            Repeat::Done(_) if matches!(step.unit, Unit::Month | Unit::Year) && due.is_some() => {
                self.off_the_calendar(step, due?, done, today)
            }
            Repeat::Done(_) => {
                let at = step.after(done)?;
                let Some(spec) = due else {
                    // No date to inherit a shape from: a whole day, not the
                    // minute the box happened to be ticked.
                    return Some(DateSpec::all_day(at.date(), zone));
                };
                // «cada día a las 10» is at ten. Counting the interval from the
                // moment it was ticked would walk the time down the day: taken
                // at 08:04, the next one would be at 08:04 for ever after.
                let at = if spec.has_time {
                    at.date().to_datetime(spec.at.time())
                } else {
                    at
                };
                Some(spec.moved(at))
            }
            Repeat::Due(_) => self.off_the_calendar(step, due?, done, today),
        }
    }

    /// Past both today and the day it was finished: a fortnight away must not
    /// come back owing a fortnight, and finishing today's must not hand back
    /// another one for today.
    fn off_the_calendar(
        self,
        step: Cadence,
        due: &DateSpec,
        done: DateTime,
        today: DateTime,
    ) -> Option<DateSpec> {
        let last = done.date().max(today.date());
        let mut at = step.after(due.at)?;
        while at.date() <= last {
            at = step.after(at)?;
        }
        Some(due.moved(at))
    }
}

impl Cadence {
    /// Built rather than added directly: `ToSpan` panics outside its range, and
    /// a cadence read off a line of text can hold any number at all.
    fn after(self, from: DateTime) -> Option<DateTime> {
        let n = i64::from(self.every);
        let span = match self.unit {
            Unit::Day => jiff::Span::new().try_days(n),
            Unit::Week => jiff::Span::new().try_weeks(n),
            // Clamped to the last day of a shorter month, not refused.
            Unit::Month => jiff::Span::new().try_months(n),
            Unit::Year => jiff::Span::new().try_years(n),
        };
        from.checked_add(span.ok()?).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn spec(at: DateTime) -> DateSpec {
        DateSpec::floating(at, "Europe/Madrid")
    }

    const EVERY_WEEK: Repeat = Repeat::Due(Cadence {
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

    /// Rent is the archetype of a fixed date even when it is said as «every
    /// month»: paid on the 4th, then the 13th, then the 30th, a relative count
    /// walks it down the month and eventually skips one.
    #[test]
    fn a_monthly_one_stays_on_its_day_however_late_it_is_paid() {
        let monthly = Repeat::Done(Cadence {
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
        let every_three = Repeat::Done(Cadence {
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

        // Three days on from the doing, at the hour that was asked for: the
        // interval counts in days, the time of day is not up for negotiation.
        assert_eq!(next.at, date(2026, 8, 12).at(9, 0, 0, 0));
    }

    /// Taken at 08:04, tomorrow's would otherwise be at 08:04 as well, and the
    /// hour would drift a little further every day.
    #[test]
    fn a_time_of_day_stays_where_it_was_asked_for() {
        let daily = Repeat::Done(Cadence {
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
        let every_day = Repeat::Done(Cadence {
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

        // A whole day, not the minute the box happened to be ticked: a task
        // that never had a time of day must not grow one.
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
        let monthly = Repeat::Due(Cadence {
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
            let absurd = Repeat::Done(Cadence {
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
        let never = Repeat::Done(Cadence {
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
}
