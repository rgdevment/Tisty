use super::lines::{Line, line, unescaped, unfolded};

pub const BYTES_AT_MOST: usize = 5 * 1024 * 1024;
pub const ITEMS_AT_MOST: usize = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Event,
    Todo,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub value: String,
    pub tz: Option<String>,
    pub date_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub kind: Kind,
    pub uid: Option<String>,
    pub summary: String,
    pub start: Option<Stamp>,
    pub due: Option<Stamp>,
    pub location: String,
    pub description: String,
    pub rule: Option<String>,
    pub irregular: bool,
    pub moved: bool,
    pub alarm: Option<i64>,
    pub priority: Option<u8>,
    pub done: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calendar {
    pub method: Option<String>,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Unread {
    #[error("this is not a calendar file")]
    NotACalendar,
    #[error("the calendar file is too big")]
    TooBig,
    #[error("the calendar file holds too many entries")]
    TooMany,
}

fn stamp(line: &Line) -> Stamp {
    let value = line.value.trim().to_string();
    let date_only = line
        .param("VALUE")
        .is_some_and(|one| one.eq_ignore_ascii_case("DATE"))
        || (value.len() == 8 && value.bytes().all(|one| one.is_ascii_digit()));
    Stamp {
        tz: line.param("TZID").map(str::to_string),
        value,
        date_only,
    }
}

fn blank(kind: Kind) -> Item {
    Item {
        kind,
        uid: None,
        summary: String::new(),
        start: None,
        due: None,
        location: String::new(),
        description: String::new(),
        rule: None,
        irregular: false,
        moved: false,
        alarm: None,
        priority: None,
        done: false,
    }
}

fn took(item: &mut Item, line: &Line) {
    match line.name.as_str() {
        "UID" => item.uid = Some(line.value.trim().to_string()).filter(|one| !one.is_empty()),
        "SUMMARY" => item.summary = unescaped(&line.value).trim().to_string(),
        "LOCATION" => item.location = unescaped(&line.value).trim().to_string(),
        "DESCRIPTION" => item.description = unescaped(&line.value).trim().to_string(),
        "DTSTART" => item.start = Some(stamp(line)),
        "DUE" => item.due = Some(stamp(line)),
        "RRULE" => item.rule = Some(line.value.trim().to_ascii_uppercase()),
        "EXDATE" | "RDATE" => item.irregular = true,
        "RECURRENCE-ID" => item.moved = true,
        "PRIORITY" => item.priority = line.value.trim().parse().ok(),
        "STATUS" => item.done |= line.value.trim().eq_ignore_ascii_case("COMPLETED"),
        "COMPLETED" => item.done = true,
        _ => {}
    }
}

/// Only an alarm counted back from the start is kept; one set at a clock time or after it is not a reminder.
pub fn before(trigger: &Line) -> Option<i64> {
    if trigger
        .param("VALUE")
        .is_some_and(|one| one.eq_ignore_ascii_case("DATE-TIME"))
        || trigger
            .param("RELATED")
            .is_some_and(|one| one.eq_ignore_ascii_case("END"))
    {
        return None;
    }
    let said = trigger.value.trim();
    let (back, span) = match said.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, said.strip_prefix('+').unwrap_or(said)),
    };
    match (back, seconds_in(span)?) {
        (true, seconds) => Some(seconds / 60),
        (false, 0) => Some(0),
        _ => None,
    }
}

fn seconds_in(span: &str) -> Option<i64> {
    let span = span.strip_prefix('P')?;
    let (days, time) = span.split_once('T').unwrap_or((span, ""));
    let mut seconds = 0i64;
    for (part, units) in [
        (days, [('W', 7 * 86_400), ('D', 86_400)].as_slice()),
        (time, [('H', 3600), ('M', 60), ('S', 1)].as_slice()),
    ] {
        let mut number = String::new();
        for one in part.chars() {
            match units.iter().find(|(unit, _)| *unit == one) {
                Some((_, size)) => {
                    seconds += number.parse::<i64>().ok()? * size;
                    number.clear();
                }
                None if one.is_ascii_digit() => number.push(one),
                None => return None,
            }
        }
    }
    Some(seconds)
}

pub fn read(text: &str) -> Result<Calendar, Unread> {
    if text.len() > BYTES_AT_MOST {
        return Err(Unread::TooBig);
    }
    let lines: Vec<Line> = unfolded(text.trim_start_matches('\u{feff}'))
        .iter()
        .filter_map(|raw| line(raw))
        .collect();
    let calendar = lines.first().is_some_and(|one| {
        one.name == "BEGIN" && one.value.trim().eq_ignore_ascii_case("VCALENDAR")
    });
    if !calendar {
        return Err(Unread::NotACalendar);
    }
    let mut method = None;
    let mut items = Vec::new();
    let mut open: Option<Item> = None;
    let mut alarmed = false;
    for one in &lines {
        let block = one.value.trim().to_ascii_uppercase();
        match (one.name.as_str(), block.as_str()) {
            ("BEGIN", "VEVENT") => open = Some(blank(Kind::Event)),
            ("BEGIN", "VTODO") => open = Some(blank(Kind::Todo)),
            ("BEGIN", "VALARM") => alarmed = true,
            ("END", "VALARM") => alarmed = false,
            ("END", "VEVENT" | "VTODO") => {
                items.extend(open.take());
                if items.len() > ITEMS_AT_MOST {
                    return Err(Unread::TooMany);
                }
            }
            ("METHOD", _) if open.is_none() => method = Some(block),
            ("TRIGGER", _) if alarmed => {
                if let Some(item) = open.as_mut()
                    && item.alarm.is_none()
                {
                    item.alarm = before(one);
                }
            }
            _ if alarmed => {}
            _ => {
                if let Some(item) = open.as_mut() {
                    took(item, one);
                }
            }
        }
    }
    Ok(Calendar { method, items })
}
