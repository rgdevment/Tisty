use jiff::civil::{Date, DateTime, Time};
use jiff::tz::TimeZone;

use super::read::Stamp;
use crate::model::DateSpec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct When {
    pub spec: DateSpec,
    pub unknown_zone: Option<String>,
}

// Outlook and Exchange name zones the Windows way; these are the ones invitations most often carry.
const WINDOWS: &[(&str, &str)] = &[
    ("UTC", "UTC"),
    ("GMT Standard Time", "Europe/London"),
    ("Greenwich Standard Time", "Atlantic/Reykjavik"),
    ("W. Europe Standard Time", "Europe/Berlin"),
    ("Romance Standard Time", "Europe/Paris"),
    ("Central Europe Standard Time", "Europe/Budapest"),
    ("Central European Standard Time", "Europe/Warsaw"),
    ("E. Europe Standard Time", "Europe/Chisinau"),
    ("GTB Standard Time", "Europe/Bucharest"),
    ("FLE Standard Time", "Europe/Kiev"),
    ("Russian Standard Time", "Europe/Moscow"),
    ("Turkey Standard Time", "Europe/Istanbul"),
    ("Israel Standard Time", "Asia/Jerusalem"),
    ("South Africa Standard Time", "Africa/Johannesburg"),
    ("Arabian Standard Time", "Asia/Dubai"),
    ("India Standard Time", "Asia/Kolkata"),
    ("China Standard Time", "Asia/Shanghai"),
    ("Singapore Standard Time", "Asia/Singapore"),
    ("Tokyo Standard Time", "Asia/Tokyo"),
    ("Korea Standard Time", "Asia/Seoul"),
    ("AUS Eastern Standard Time", "Australia/Sydney"),
    ("New Zealand Standard Time", "Pacific/Auckland"),
    ("Eastern Standard Time", "America/New_York"),
    ("Central Standard Time", "America/Chicago"),
    ("Mountain Standard Time", "America/Denver"),
    ("US Mountain Standard Time", "America/Phoenix"),
    ("Pacific Standard Time", "America/Los_Angeles"),
    ("Alaskan Standard Time", "America/Anchorage"),
    ("Hawaiian Standard Time", "Pacific/Honolulu"),
    ("Atlantic Standard Time", "America/Halifax"),
    ("Central Standard Time (Mexico)", "America/Mexico_City"),
    ("Central America Standard Time", "America/Guatemala"),
    ("SA Pacific Standard Time", "America/Bogota"),
    ("Venezuela Standard Time", "America/Caracas"),
    ("SA Western Standard Time", "America/La_Paz"),
    ("Pacific SA Standard Time", "America/Santiago"),
    ("Argentina Standard Time", "America/Buenos_Aires"),
    ("Montevideo Standard Time", "America/Montevideo"),
    ("Paraguay Standard Time", "America/Asuncion"),
    ("E. South America Standard Time", "America/Sao_Paulo"),
    ("SA Eastern Standard Time", "America/Cayenne"),
];

pub(super) fn own_zone(stamp: &Stamp) -> Option<TimeZone> {
    match stamp.value.trim().ends_with(['Z', 'z']) {
        true => Some(TimeZone::UTC),
        false => stamp.tz.as_deref().and_then(zone_named),
    }
}

fn zone_named(tz: &str) -> Option<TimeZone> {
    let tz = tz.trim();
    TimeZone::get(tz).ok().or_else(|| {
        WINDOWS
            .iter()
            .find(|(windows, _)| windows.eq_ignore_ascii_case(tz))
            .and_then(|(_, iana)| TimeZone::get(iana).ok())
    })
}

fn number(said: &str, from: usize, len: usize) -> Option<i16> {
    said.get(from..from + len)?.parse().ok()
}

pub(super) fn civil(value: &str) -> Option<DateTime> {
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
        (Some(tz), false) => Some(zone_named(tz).ok_or_else(|| tz.clone())),
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
