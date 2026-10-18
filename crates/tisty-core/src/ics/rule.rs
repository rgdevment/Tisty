use jiff::civil::Date;

use crate::model::{Cadence, Repeat, Unit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ruled {
    Once,
    Kept(Repeat),
    Odd,
}

const DAYS: [&str; 7] = ["MO", "TU", "WE", "TH", "FR", "SA", "SU"];

fn weekday(own: Date) -> &'static str {
    DAYS[usize::from(own.weekday().to_monday_zero_offset().unsigned_abs())]
}

// A part says again what the start says only at the rhythm that keeps it, and only on the event's own calendar.
fn restates(key: &str, value: &str, own: Date, unit: Unit) -> bool {
    match (key, unit) {
        ("WKST", _) => true,
        ("BYDAY", Unit::Week) => value == weekday(own),
        ("BYMONTHDAY", Unit::Month | Unit::Year) => value.parse::<i8>().ok() == Some(own.day()),
        ("BYMONTH", Unit::Year) => value.parse::<i8>().ok() == Some(own.month()),
        _ => false,
    }
}

/// So many turns of the cadence at once, refused rather than wrapped when a file asks past what time holds.
pub fn span(unit: Unit, steps: i64) -> Option<jiff::Span> {
    let none = jiff::Span::new();
    match unit {
        Unit::Day => none.try_days(steps),
        Unit::Week => none.try_weeks(steps),
        Unit::Month => none.try_months(steps),
        Unit::Year => none.try_years(steps),
    }
    .ok()
}

fn last(own: Date, each: Cadence, count: u32) -> Option<Date> {
    let steps = i64::from(each.every).checked_mul(i64::from(count.checked_sub(1)?))?;
    own.checked_add(span(each.unit, steps)?).ok()
}

fn unit_of(said: &str) -> Option<Unit> {
    match said {
        "DAILY" => Some(Unit::Day),
        "WEEKLY" => Some(Unit::Week),
        "MONTHLY" => Some(Unit::Month),
        "YEARLY" => Some(Unit::Year),
        _ => None,
    }
}

/// A repeat Tisty cannot follow is said so, rather than made into one it would follow wrongly.
pub fn ruled(rule: Option<&str>, irregular: bool, own: Option<Date>) -> Ruled {
    let Some(rule) = rule else {
        return Ruled::Once;
    };
    let Some(own) = own.filter(|_| !irregular) else {
        return Ruled::Odd;
    };
    let mut parts = Vec::new();
    for part in rule.split(';').filter(|one| !one.is_empty()) {
        let Some(pair) = part.split_once('=') else {
            return Ruled::Odd;
        };
        parts.push(pair);
    }
    let Some(unit) = parts
        .iter()
        .find(|(key, _)| *key == "FREQ")
        .and_then(|(_, said)| unit_of(said))
    else {
        return Ruled::Odd;
    };
    let mut each = Cadence { every: 1, unit };
    let mut until = None;
    let mut count = None;
    for (key, value) in parts {
        match key {
            "FREQ" => {}
            "INTERVAL" => match value.parse() {
                Ok(one) if one > 0 => each.every = one,
                _ => return Ruled::Odd,
            },
            "UNTIL" => match value
                .get(..8)
                .and_then(|one| Date::strptime("%Y%m%d", one).ok())
            {
                Some(day) => until = Some(day),
                None => return Ruled::Odd,
            },
            "COUNT" => match value.parse::<u32>() {
                Ok(one) if one > 0 => count = Some(one),
                _ => return Ruled::Odd,
            },
            _ if restates(key, value, own, unit) => {}
            _ => return Ruled::Odd,
        }
    }
    if let Some(count) = count {
        until = until.or_else(|| last(own, each, count));
    }
    Ruled::Kept(Repeat {
        until,
        ..Repeat::due(each)
    })
}
