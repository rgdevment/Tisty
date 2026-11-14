use jiff::civil::Date;
use jiff::{Span, Timestamp, Zoned};

use crate::ics::{EVE_AT, TIMED_BEFORE};
use crate::model::Task;

pub const LASTS_MINUTES: i64 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum When {
    Days { from: Date, to: Date },
    Hours { from: Timestamp, to: Timestamp },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub title: String,
    pub when: When,
    pub minutes_before: Vec<i64>,
    pub link: String,
}

pub fn entry(task: &Task, link: &str, now: &Zoned) -> Option<Entry> {
    let on = task.date.as_ref().or(task.deadline.as_ref())?;
    let here = now.time_zone();
    let (when, start) = if on.has_time {
        let from = on.instant(here).ok()?;
        let to = from.checked_add(Span::new().minutes(LASTS_MINUTES)).ok()?;
        (When::Hours { from, to }, from)
    } else {
        let from = on.date();
        let midnight = from.to_zoned(here.clone()).ok()?.timestamp();
        let to = from.tomorrow().ok()?;
        (When::Days { from, to }, midnight)
    };
    let minutes = |earlier: Timestamp| earlier.until(start).ok().map(|gap| gap.get_seconds() / 60);
    let mut minutes_before: Vec<i64> = task
        .reminders
        .iter()
        .filter_map(|one| one.instant(here).ok())
        .filter(|at| *at > now.timestamp())
        .map(minutes)
        .collect::<Option<_>>()?;
    minutes_before.sort_unstable();
    minutes_before.dedup();
    if minutes_before.is_empty() {
        minutes_before.push(match when {
            When::Hours { .. } => TIMED_BEFORE,
            When::Days { from, .. } => {
                let eve = from.yesterday().ok()?.to_datetime(EVE_AT);
                minutes(eve.to_zoned(here.clone()).ok()?.timestamp())?
            }
        });
    }
    Some(Entry {
        id: task.id.to_string(),
        title: task.title.trim().to_string(),
        when,
        minutes_before,
        link: link.to_string(),
    })
}

pub fn google(entry: &Entry) -> String {
    let dates = match entry.when {
        When::Hours { from, to } => format!("{}/{}", utc(from), utc(to)),
        When::Days { from, to } => format!("{}/{}", day(from), day(to)),
    };
    format!(
        "https://calendar.google.com/calendar/render?action=TEMPLATE&text={}&dates={dates}&details={}",
        encoded(&entry.title),
        encoded(&entry.link),
    )
}

pub fn outlook(entry: &Entry) -> String {
    composed("outlook.live.com", entry)
}

pub fn outlook_work(entry: &Entry) -> String {
    composed("outlook.cloud.microsoft", entry)
}

fn composed(host: &str, entry: &Entry) -> String {
    let (from, to, whole_day) = match entry.when {
        When::Hours { from, to } => (iso(from), iso(to), false),
        When::Days { from, to } => (from.to_string(), to.to_string(), true),
    };
    format!(
        "https://{host}/calendar/0/deeplink/compose?path=/calendar/action/compose&rru=addevent&subject={}&startdt={}&enddt={}&allday={whole_day}&body={}",
        encoded(&entry.title),
        encoded(&from),
        encoded(&to),
        encoded(&entry.link),
    )
}

pub(crate) fn utc(at: Timestamp) -> String {
    at.strftime("%Y%m%dT%H%M%SZ").to_string()
}

pub(crate) fn day(at: Date) -> String {
    at.strftime("%Y%m%d").to_string()
}

fn iso(at: Timestamp) -> String {
    at.strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}

fn encoded(said: &str) -> String {
    let mut out = String::with_capacity(said.len() * 3);
    for one in said.bytes() {
        match one {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(char::from(one));
            }
            _ => out.push_str(&format!("%{one:02X}")),
        }
    }
    out
}

#[cfg(test)]
#[path = "calendar_test.rs"]
mod tests;
