use jiff::civil::{Date, DateTime};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cadence {
    pub every: u16,
    pub unit: Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum From {
    Due,
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repeat {
    pub from: From,
    pub each: Cadence,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<jiff::civil::Date>,
}

impl Repeat {
    pub const fn due(each: Cadence) -> Self {
        Self {
            from: From::Due,
            each,
            until: None,
        }
    }

    pub const fn done(each: Cadence) -> Self {
        Self {
            from: From::Done,
            each,
            until: None,
        }
    }

    pub fn cadence(self) -> Cadence {
        self.each
    }

    pub fn ended(self, at: jiff::civil::Date) -> bool {
        self.until.is_some_and(|last| at > last)
    }

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

        match self.from {
            From::Done if matches!(step.unit, Unit::Month | Unit::Year) && due.is_some() => {
                self.off_the_calendar(step, due?, done, today)
            }
            From::Done => {
                let at = step.after(done)?;
                let Some(spec) = due else {
                    return Some(DateSpec::all_day(at.date(), zone));
                };
                let at = if spec.has_time {
                    at.date().to_datetime(spec.at.time())
                } else {
                    at
                };
                Some(spec.moved(at))
            }
            From::Due => self.off_the_calendar(step, due?, done, today),
        }
    }

    fn off_the_calendar(
        self,
        step: Cadence,
        due: &DateSpec,
        done: DateTime,
        today: DateTime,
    ) -> Option<DateSpec> {
        let last = done.date().max(today.date());
        Some(due.moved(step.beyond(due.at, last)?))
    }
}

impl Cadence {
    pub fn beyond(self, anchor: DateTime, past: Date) -> Option<DateTime> {
        let mut at = self.after(anchor)?;
        while at.date() <= past {
            at = self.after(at)?;
        }
        Some(at)
    }

    pub fn after(self, from: DateTime) -> Option<DateTime> {
        let n = i64::from(self.every);
        let span = match self.unit {
            Unit::Day => jiff::Span::new().try_days(n),
            Unit::Week => jiff::Span::new().try_weeks(n),
            Unit::Month => jiff::Span::new().try_months(n),
            Unit::Year => jiff::Span::new().try_years(n),
        };
        from.checked_add(span.ok()?).ok()
    }
}

#[cfg(test)]
#[path = "repeat_test.rs"]
mod tests;

#[cfg(test)]
#[path = "repeat_shape.rs"]
mod shape;
