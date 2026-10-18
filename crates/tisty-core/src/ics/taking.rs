use jiff::civil::Time;
use jiff::{Span, Zoned};

use super::read::{Alarm, Item, Kind};
use super::rule::{Ruled, ruled, step};
use super::when::{When, when};
use crate::model::{DateSpec, Priority, Repeat};

pub const SOURCE: &str = "ics:";
const TIMED_BEFORE: i64 = 30;
const EVE_AT: Time = Time::constant(18, 0, 0, 0);
const STEPS_AT_MOST: usize = 20_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Odd {
    Untitled,
    Moved,
    Cancelled,
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
    /// What cannot be a task, is already done or was called off is shown and never taken.
    pub fn takeable(&self) -> bool {
        !self.done
            && ![Odd::Untitled, Odd::Moved, Odd::Cancelled]
                .iter()
                .any(|one| self.odd.contains(one))
    }
}

// An alarm of its own wins; without one, half an hour before a time, or the evening before a day.
fn reminder(at: &DateSpec, alarm: Option<i64>, now: &Zoned) -> Option<DateSpec> {
    let here = now.time_zone();
    let start = at.instant(here).ok()?.to_zoned(here.clone());
    let back = |minutes: i64| Span::new().try_minutes(minutes).ok();
    let then = match (alarm, at.has_time) {
        (Some(minutes), _) => start.checked_sub(back(minutes)?).ok()?,
        (None, true) => start.checked_sub(back(TIMED_BEFORE)?).ok()?,
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

fn gone(at: &DateSpec, now: &Zoned) -> bool {
    match at.has_time {
        true => at
            .instant(now.time_zone())
            .is_ok_and(|one| one < now.timestamp()),
        false => at.date() < now.date(),
    }
}

// A series that began long ago starts at its next turn, never as years overdue.
fn caught_up(at: &DateSpec, repeat: Repeat, now: &Zoned) -> Option<DateSpec> {
    let mut next = at.clone();
    for _ in 0..STEPS_AT_MOST {
        if !gone(&next, now) {
            return Some(next);
        }
        next = next.moved(next.at.checked_add(step(repeat.each)?).ok()?);
    }
    None
}

// What carries no identity of its own is known by what it says and when, so opening it twice takes it once.
fn fingerprint(item: &Item) -> String {
    let said = format!(
        "{}|{}|{}",
        item.summary,
        item.start.as_ref().map_or("", |one| one.value.as_str()),
        item.due.as_ref().map_or("", |one| one.value.as_str())
    );
    let hash = said.bytes().fold(0xcbf2_9ce4_8422_2325u64, |hash, one| {
        (hash ^ u64::from(one)).wrapping_mul(0x0100_0000_01b3)
    });
    format!("~{hash:016x}")
}

// An alarm counts from the start unless it says the end; with none, the reminder serves the deadline.
fn alarm_from(
    item: &Item,
    date: Option<&DateSpec>,
    deadline: Option<&DateSpec>,
) -> Option<DateSpec> {
    match item.alarm {
        Some(Alarm {
            from_end: false, ..
        }) => date.or(deadline).cloned(),
        _ => deadline.or(date).cloned(),
    }
}

pub fn taken(item: &Item, now: &Zoned) -> Taken {
    let here = now.time_zone();
    let start = item.start.as_ref().and_then(|one| when(one, here));
    let due = item.due.as_ref().and_then(|one| when(one, here));
    let mut odd = Vec::new();
    for (wrong, said) in [
        (item.summary.is_empty(), Odd::Untitled),
        (item.moved, Odd::Moved),
        (item.cancelled, Odd::Cancelled),
    ] {
        if wrong {
            odd.push(said);
        }
    }
    let zoned = [&start, &due]
        .into_iter()
        .flatten()
        .any(|one: &When| one.unknown_zone.is_some());
    if zoned {
        odd.push(Odd::Zone);
    }
    let (mut date, mut deadline) = match item.kind {
        Kind::Event => (start.map(|one| one.spec), None),
        Kind::Todo => (start.map(|one| one.spec), due.map(|one| one.spec)),
    };
    let mut repeat = match ruled(item.rule.as_deref(), item.irregular, date.as_ref()) {
        Ruled::Once => None,
        Ruled::Kept(repeat) => Some(repeat),
        Ruled::Odd => {
            odd.push(Odd::Repeat);
            None
        }
    };
    if let Some(series) = repeat {
        let ended = series.until.is_some_and(|last| last < now.date());
        match date.as_ref().and_then(|at| caught_up(at, series, now)) {
            Some(next) if !ended => {
                let was = date.as_ref().map_or(next.at, |one| one.at);
                let moved = next.at.since(was).ok();
                deadline = deadline.and_then(|one| {
                    let at = one.at.checked_add(moved?).ok()?;
                    Some(one.moved(at))
                });
                date = Some(next);
            }
            _ => repeat = None,
        }
    }
    let anchor = deadline.as_ref().or(date.as_ref());
    if anchor.is_some_and(|one| gone(one, now)) {
        odd.push(Odd::Past);
    }
    let alarm = item.alarm.map(|one| one.minutes);
    let reminded = alarm_from(item, date.as_ref(), deadline.as_ref());
    Taken {
        title: item.summary.clone(),
        reminder: reminded.and_then(|one| reminder(&one, alarm, now)),
        date,
        deadline,
        repeat,
        place: item.location.clone(),
        described: described(item),
        priority: item
            .priority
            .filter(|one| (1..=4).contains(one))
            .map(|_| Priority::Do),
        source: Some(format!(
            "{SOURCE}{}",
            item.uid.clone().unwrap_or_else(|| fingerprint(item))
        )),
        done: item.done,
        odd,
    }
}
