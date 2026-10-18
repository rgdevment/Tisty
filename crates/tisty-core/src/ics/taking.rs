use jiff::civil::Time;
use jiff::{ToSpan, Zoned};

use super::read::{Item, Kind};
use super::rule::{Ruled, ruled};
use super::when::{When, when};
use crate::model::{DateSpec, Priority, Repeat};

pub const SOURCE: &str = "ics:";
const TIMED_BEFORE: i64 = 30;
const EVE_AT: Time = Time::constant(18, 0, 0, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Odd {
    Untitled,
    Moved,
    Repeat,
    Zone,
    Past,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Taken {
    pub title: String,
    pub date: Option<DateSpec>,
    pub deadline: Option<DateSpec>,
    pub repeat: Option<Repeat>,
    pub reminder: Option<DateSpec>,
    pub place: String,
    pub described: Option<String>,
    pub priority: Option<Priority>,
    pub source: Option<String>,
    pub done: bool,
    pub odd: Vec<Odd>,
}

impl Taken {
    /// What cannot be a task, or is already done, is shown and never taken.
    pub fn takeable(&self) -> bool {
        !self.done && !self.odd.contains(&Odd::Untitled) && !self.odd.contains(&Odd::Moved)
    }
}

// An alarm of its own wins; without one, half an hour before a time, or the evening before a day.
fn reminder(at: &DateSpec, alarm: Option<i64>, now: &Zoned) -> Option<DateSpec> {
    let here = now.time_zone();
    let start = at.instant(here).ok()?.to_zoned(here.clone());
    let then = match (alarm, at.has_time) {
        (Some(minutes), _) => start.checked_sub(minutes.minutes()).ok()?,
        (None, true) => start.checked_sub(TIMED_BEFORE.minutes()).ok()?,
        (None, false) => start
            .date()
            .yesterday()
            .ok()?
            .to_datetime(EVE_AT)
            .to_zoned(here.clone())
            .ok()?,
    };
    (then.timestamp() > now.timestamp())
        .then(|| DateSpec::floating(then.datetime(), here.iana_name().unwrap_or("UTC")))
}

fn described(item: &Item) -> Option<String> {
    let said: Vec<&str> = [item.location.as_str(), item.description.as_str()]
        .into_iter()
        .filter(|one| !one.is_empty())
        .collect();
    (!said.is_empty()).then(|| said.join("\n\n"))
}

pub fn taken(item: &Item, now: &Zoned) -> Taken {
    let here = now.time_zone();
    let start = item.start.as_ref().and_then(|one| when(one, here));
    let due = item.due.as_ref().and_then(|one| when(one, here));
    let mut odd = Vec::new();
    if item.summary.is_empty() {
        odd.push(Odd::Untitled);
    }
    if item.moved {
        odd.push(Odd::Moved);
    }
    let zoned = [&start, &due]
        .into_iter()
        .flatten()
        .any(|one: &When| one.unknown_zone.is_some());
    if zoned {
        odd.push(Odd::Zone);
    }
    let (date, deadline) = match item.kind {
        Kind::Event => (start.map(|one| one.spec), None),
        Kind::Todo => (start.map(|one| one.spec), due.map(|one| one.spec)),
    };
    let repeat = match ruled(item.rule.as_deref(), item.irregular, date.as_ref()) {
        Ruled::Once => None,
        Ruled::Kept(repeat) => Some(repeat),
        Ruled::Odd => {
            odd.push(Odd::Repeat);
            None
        }
    };
    let anchor = deadline.as_ref().or(date.as_ref());
    let past = anchor
        .and_then(|one| one.instant(here).ok())
        .is_some_and(|at| at < now.timestamp());
    if past && repeat.is_none() {
        odd.push(Odd::Past);
    }
    Taken {
        title: item.summary.clone(),
        reminder: anchor.and_then(|one| reminder(one, item.alarm, now)),
        date,
        deadline,
        repeat,
        place: item.location.clone(),
        described: described(item),
        priority: item
            .priority
            .filter(|one| (1..=4).contains(one))
            .map(|_| Priority::Do),
        source: item.uid.as_ref().map(|uid| format!("{SOURCE}{uid}")),
        done: item.done,
        odd,
    }
}
