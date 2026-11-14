use jiff::Timestamp;

use crate::calendar::{Entry, When, day, utc};

const LINE_OCTETS: usize = 75;

pub fn written(entry: &Entry, stamp: Timestamp) -> String {
    let (from, to) = match entry.when {
        When::Hours { from, to } => (format!(":{}", utc(from)), format!(":{}", utc(to))),
        When::Days { from, to } => (
            format!(";VALUE=DATE:{}", day(from)),
            format!(";VALUE=DATE:{}", day(to)),
        ),
    };
    let mut lines = vec![
        "BEGIN:VCALENDAR".to_string(),
        "VERSION:2.0".to_string(),
        "PRODID:-//rgdevment//Tisty//EN".to_string(),
        "CALSCALE:GREGORIAN".to_string(),
        "BEGIN:VEVENT".to_string(),
        format!("UID:{}@tisty.app", entry.id),
        format!("DTSTAMP:{}", utc(stamp)),
        format!("DTSTART{from}"),
        format!("DTEND{to}"),
        format!("SUMMARY:{}", escaped(&entry.title)),
        format!("DESCRIPTION:{}", escaped(&entry.link)),
        format!("URL:{}", entry.link),
    ];
    for minutes in &entry.minutes_before {
        let trigger = match *minutes {
            0.. => format!("-PT{minutes}M"),
            _ => format!("PT{}M", minutes.unsigned_abs()),
        };
        lines.extend([
            "BEGIN:VALARM".to_string(),
            "ACTION:DISPLAY".to_string(),
            format!("DESCRIPTION:{}", escaped(&entry.title)),
            format!("TRIGGER:{trigger}"),
            "END:VALARM".to_string(),
        ]);
    }
    lines.extend(["END:VEVENT".to_string(), "END:VCALENDAR".to_string()]);
    lines.iter().map(|one| folded(one)).collect()
}

fn escaped(said: &str) -> String {
    let mut out = String::with_capacity(said.len());
    for one in said.chars() {
        match one {
            '\\' | ';' | ',' => {
                out.push('\\');
                out.push(one);
            }
            '\n' => out.push_str("\\n"),
            '\r' => {}
            _ => out.push(one),
        }
    }
    out
}

fn folded(line: &str) -> String {
    let mut out = String::with_capacity(line.len() + line.len() / LINE_OCTETS * 3 + 2);
    let mut room = LINE_OCTETS;
    for one in line.chars() {
        let size = one.len_utf8();
        if size > room {
            out.push_str("\r\n ");
            room = LINE_OCTETS - 1;
        }
        out.push(one);
        room -= size;
    }
    out.push_str("\r\n");
    out
}
