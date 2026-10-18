use jiff::ToSpan;

use crate::model::{Cadence, DateSpec, Repeat, Unit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ruled {
    Once,
    Kept(Repeat),
    Odd,
}

const DAYS: [&str; 7] = ["MO", "TU", "WE", "TH", "FR", "SA", "SU"];

fn weekday(start: &DateSpec) -> &'static str {
    DAYS[usize::from(
        start
            .date()
            .weekday()
            .to_monday_zero_offset()
            .unsigned_abs(),
    )]
}

// What only says again what the start already says is no rule of its own.
fn restates(key: &str, value: &str, start: &DateSpec) -> bool {
    match key {
        "WKST" => true,
        "BYDAY" => value == weekday(start),
        "BYMONTHDAY" => value.parse::<i8>().ok() == Some(start.date().day()),
        "BYMONTH" => value.parse::<i8>().ok() == Some(start.date().month()),
        _ => false,
    }
}

fn last(start: &DateSpec, unit: Unit, every: u16, count: u32) -> Option<jiff::civil::Date> {
    let steps = i64::from(every) * i64::from(count.checked_sub(1)?);
    let span = match unit {
        Unit::Day => steps.days(),
        Unit::Week => steps.weeks(),
        Unit::Month => steps.months(),
        Unit::Year => steps.years(),
    };
    start.date().checked_add(span).ok()
}

/// A repeat Tisty cannot follow is said so, rather than made into one it would follow wrongly.
pub fn ruled(rule: Option<&str>, irregular: bool, start: Option<&DateSpec>) -> Ruled {
    let Some(rule) = rule else {
        return Ruled::Once;
    };
    let Some(start) = start.filter(|_| !irregular) else {
        return Ruled::Odd;
    };
    let mut unit = None;
    let mut every = 1u16;
    let mut until = None;
    let mut count = None;
    for part in rule.split(';').filter(|one| !one.is_empty()) {
        let Some((key, value)) = part.split_once('=') else {
            return Ruled::Odd;
        };
        match key {
            "FREQ" => {
                unit = match value {
                    "DAILY" => Some(Unit::Day),
                    "WEEKLY" => Some(Unit::Week),
                    "MONTHLY" => Some(Unit::Month),
                    "YEARLY" => Some(Unit::Year),
                    _ => return Ruled::Odd,
                }
            }
            "INTERVAL" => match value.parse() {
                Ok(one) if one > 0 => every = one,
                _ => return Ruled::Odd,
            },
            "UNTIL" => {
                let day = value
                    .get(..8)
                    .and_then(|one| jiff::civil::Date::strptime("%Y%m%d", one).ok());
                match day {
                    Some(day) => until = Some(day),
                    None => return Ruled::Odd,
                }
            }
            "COUNT" => match value.parse::<u32>() {
                Ok(one) if one > 0 => count = Some(one),
                _ => return Ruled::Odd,
            },
            _ if restates(key, value, start) => {}
            _ => return Ruled::Odd,
        }
    }
    let Some(unit) = unit else {
        return Ruled::Odd;
    };
    if let Some(count) = count {
        until = until.or_else(|| last(start, unit, every, count));
    }
    Ruled::Kept(Repeat {
        until,
        ..Repeat::due(Cadence { every, unit })
    })
}
