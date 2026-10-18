use jiff::civil::{Date, DateTime, Time};
use jiff::tz::TimeZone;

use super::read::Stamp;
use crate::model::DateSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct When {
    pub spec: DateSpec,
    pub unknown_zone: Option<String>,
}

fn number(said: &str, from: usize, len: usize) -> Option<i16> {
    said.get(from..from + len)?.parse().ok()
}

fn civil(value: &str) -> Option<DateTime> {
    let bare = value.trim().trim_end_matches(['Z', 'z']);
    let (day, time) = bare.split_once(['T', 't']).unwrap_or((bare, ""));
    let date = Date::new(
        number(day, 0, 4)?,
        i8::try_from(number(day, 4, 2)?).ok()?,
        i8::try_from(number(day, 6, 2)?).ok()?,
    )
    .ok()?;
    if time.is_empty() {
        return Some(date.to_datetime(Time::midnight()));
    }
    let hour = i8::try_from(number(time, 0, 2)?).ok()?;
    let minute = i8::try_from(number(time, 2, 2)?).ok()?;
    let second = number(time, 4, 2)
        .and_then(|one| i8::try_from(one).ok())
        .unwrap_or(0);
    Some(date.to_datetime(Time::new(hour, minute, second.min(59), 0).ok()?))
}

/// A time from elsewhere is read here, at the hour it falls on this clock, and stays tied to that moment.
pub fn when(stamp: &Stamp, here: &TimeZone) -> Option<When> {
    let name = here.iana_name().unwrap_or("UTC").to_string();
    let at = civil(&stamp.value)?;
    if stamp.date_only {
        return Some(When {
            spec: DateSpec::all_day(at.date(), name),
            unknown_zone: None,
        });
    }
    let utc = stamp.value.trim().ends_with(['Z', 'z']);
    let zone = match (&stamp.tz, utc) {
        (_, true) => Some(Ok(TimeZone::UTC)),
        (Some(tz), false) => Some(TimeZone::get(tz).map_err(|_| tz.clone())),
        (None, false) => None,
    };
    let fixed = |zone: TimeZone| -> Option<When> {
        let local = at.to_zoned(zone).ok()?.with_time_zone(here.clone());
        Some(When {
            spec: DateSpec::fixed(local.datetime(), name.clone()),
            unknown_zone: None,
        })
    };
    match zone {
        Some(Ok(zone)) => fixed(zone),
        Some(Err(unknown)) => Some(When {
            spec: DateSpec::floating(at, name),
            unknown_zone: Some(unknown),
        }),
        None => Some(When {
            spec: DateSpec::floating(at, name),
            unknown_zone: None,
        }),
    }
}
