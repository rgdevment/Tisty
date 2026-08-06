use jiff::{
    Zoned,
    civil::{Date, Time, Weekday},
};

use crate::scan::Anchor;

pub fn to_date(anchor: Anchor, now: &Zoned) -> Option<Date> {
    let today = now.date();

    let date = match anchor {
        Anchor::Today => today,
        Anchor::Tomorrow => today.tomorrow().ok()?,
        Anchor::DayAfterTomorrow => today.tomorrow().ok()?.tomorrow().ok()?,
        Anchor::Weekday(d) | Anchor::NextWeekday(d) => next_weekday(today, d)?,
        Anchor::InDays(n) => today.checked_add(jiff::Span::new().days(n)).ok()?,
        Anchor::InWeeks(n) => today.checked_add(jiff::Span::new().weeks(n)).ok()?,
        Anchor::InMonths(n) => today.checked_add(jiff::Span::new().months(n)).ok()?,
        Anchor::EndOfWeek => end_of_week(today)?,
        Anchor::Weekend => weekend(today)?,
        Anchor::EndOfMonth => today.last_of_month(),
        Anchor::OnDate(day, month, year) => explicit(today, day, month, year)?,
    };

    Some(date)
}

/// Always forward: "monday" on a Monday means the next one, never today.
fn next_weekday(today: Date, target: usize) -> Option<Date> {
    let current = weekday_index(today.weekday());
    let ahead = (target + 7 - current) % 7;
    let days = if ahead == 0 { 7 } else { ahead };
    today.checked_add(jiff::Span::new().days(days as i64)).ok()
}

fn weekday_index(w: Weekday) -> usize {
    match w {
        Weekday::Monday => 0,
        Weekday::Tuesday => 1,
        Weekday::Wednesday => 2,
        Weekday::Thursday => 3,
        Weekday::Friday => 4,
        Weekday::Saturday => 5,
        Weekday::Sunday => 6,
    }
}

fn end_of_week(today: Date) -> Option<Date> {
    let ahead = 6 - weekday_index(today.weekday());
    today.checked_add(jiff::Span::new().days(ahead as i64)).ok()
}

fn weekend(today: Date) -> Option<Date> {
    let ahead = (5 + 7 - weekday_index(today.weekday())) % 7;
    today.checked_add(jiff::Span::new().days(ahead as i64)).ok()
}

fn explicit(today: Date, day: u8, month: Option<u8>, year: Option<i16>) -> Option<Date> {
    let month = month.unwrap_or(today.month() as u8) as i8;

    // A written year is a decision, not a hint: never roll it forward.
    if let Some(year) = year {
        return Date::new(year, month, day as i8).ok();
    }

    let candidate = Date::new(today.year(), month, day as i8).ok()?;
    if candidate >= today {
        return Some(candidate);
    }
    Date::new(today.year() + 1, month, day as i8).ok()
}

/// A time already past today means tomorrow.
pub fn place_time(date: Option<Date>, time: Time, now: &Zoned) -> Option<Date> {
    match date {
        Some(d) => Some(d),
        None if time > now.time() => Some(now.date()),
        None => now.date().tomorrow().ok(),
    }
}
