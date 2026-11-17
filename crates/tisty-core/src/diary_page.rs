use std::path::Path;

use jiff::civil::Date;
use tisty_markdown::escaped;

use crate::State;
use crate::diary::Words;
use crate::diary_day::DayAt;
use crate::model::Kept;

pub struct Said<'a> {
    pub one: &'a str,
    pub many: &'a str,
    pub empty: &'a str,
    pub writing: &'a str,
    pub keep: &'a str,
    pub cancel: &'a str,
}

pub struct Page<'a> {
    pub root: &'a Path,
    pub state: &'a State,
    pub kept: &'a Kept,
    pub today: Date,
    pub zone: &'a jiff::tz::TimeZone,
}

/// A month's words cut by day, read once, before anything of the log is asked.
pub struct Days {
    intro: String,
    each: Vec<(i8, String)>,
}

pub fn days(body: &str) -> Days {
    let lines: Vec<&str> = body.lines().collect();
    let titled = crate::docs::headings(body)
        .first()
        .is_some_and(|(line, level, _)| *line == 1 && *level == 1);
    let cuts = crate::diary_day::cuts(body);
    let first = cuts.first().map_or(lines.len(), |one| one.heading);
    let intro_from = usize::from(titled).min(first);
    let each = cuts
        .iter()
        .map(|one| (one.day, lines[one.heading + 1..one.end].join("\n")))
        .collect();
    Days {
        intro: lines[intro_from..first].join("\n").trim().to_string(),
        each,
    }
}

pub struct Strip {
    day: i8,
    date: Option<Date>,
    today: bool,
    closed: Vec<(String, String)>,
}

pub fn strips(days: &Days, page: &Page) -> Vec<Strip> {
    let month = crate::diary::month_of_page(page.root, page.state, page.kept);
    days.each
        .iter()
        .map(|(day, _)| {
            let date = month.and_then(|(year, month)| Date::new(year, month, *day).ok());
            Strip {
                day: *day,
                date,
                today: date == Some(page.today),
                closed: date
                    .map(|date| crate::diary::closed_on(page.state, date, page.zone))
                    .unwrap_or_default()
                    .into_iter()
                    .map(|task| (task.id.to_string(), task.title.clone()))
                    .collect(),
            }
        })
        .collect()
}

// What the days would show of the log, so the window draws the month again only when that changes.
pub fn print(strips: &[Strip]) -> String {
    strips
        .iter()
        .filter(|strip| strip.today || !strip.closed.is_empty())
        .map(|strip| {
            let ids: Vec<&str> = strip.closed.iter().map(|(id, _)| id.as_str()).collect();
            format!("{}:{}:{}", strip.day, strip.today, ids.join(","))
        })
        .collect::<Vec<_>>()
        .join("|")
}

// The month as a line of days, newest on top as written: the words are the person's, the rest the log's.
pub fn laid(
    days: &Days,
    strips: &[Strip],
    words: &Words,
    said: &Said,
    open: Option<DayAt>,
) -> String {
    let mut out = String::from("<div class=\"diary-month\">");
    if !days.intro.is_empty() {
        out.push_str(&format!(
            "<div class=\"diary-intro\">{}</div>",
            drawn(&days.intro)
        ));
    }
    let places = crate::diary_day::places(days.each.iter().map(|(day, _)| *day));
    for (((_, text), strip), at) in days.each.iter().zip(strips).zip(places) {
        let written = match open == Some(at) {
            true => boxed(text, said),
            false => drawn(text.trim()),
        };
        out.push_str(&entry(&written, at, strip, words, said));
    }
    if days.each.is_empty() {
        out.push_str(&format!(
            "<p class=\"diary-empty\">{}</p>",
            escaped(said.empty)
        ));
    }
    out.push_str("</div>");
    out
}

fn drawn(text: &str) -> String {
    tisty_markdown::html(&tisty_markdown::read(text))
}

// The textarea's first line break is the parser's to drop, so words that begin with one keep it.
fn boxed(text: &str, said: &Said) -> String {
    let (words, notes) = crate::diary_day::parted(text);
    let rows = words.lines().count().clamp(2, 20);
    format!(
        "<div class=\"diary-box editing\"><textarea class=\"diary-box-words\" rows=\"{rows}\" aria-label=\"{}\">\n{}</textarea></div><div class=\"diary-box-bar\"><button type=\"button\" data-asked=\"dayCancel\">{}</button><button type=\"button\" class=\"diary-box-keep\" data-asked=\"dayKeep\">{}</button></div>{}",
        escaped(said.writing),
        escaped(&words),
        escaped(said.cancel),
        escaped(said.keep),
        drawn(&notes.join("\n\n"))
    )
}

fn entry(written: &str, at: DayAt, strip: &Strip, words: &Words, said: &Said) -> String {
    let weekday = strip.date.map_or("", |date| {
        words.weekdays[usize::from(date.weekday().to_monday_zero_offset().unsigned_abs())]
    });
    let short = weekday.chars().take(3).collect::<String>().to_lowercase();
    let mut out = format!(
        "<section class=\"diary-entry{}\" data-day=\"{}\" data-nth=\"{}\"><div class=\"diary-date\"><span class=\"diary-number\">{}</span><span class=\"diary-weekday\">{}</span></div><div class=\"diary-words\">{written}",
        if strip.today { " diary-is-today" } else { "" },
        at.day,
        at.nth,
        strip.day,
        escaped(&short),
    );
    if !strip.closed.is_empty() {
        let summary = match strip.closed.len() {
            1 => said.one.to_string(),
            many => said.many.replace("{name}", &many.to_string()),
        };
        out.push_str(&format!(
            "<details class=\"diary-closed\"><summary>{}</summary><ul>",
            escaped(&summary)
        ));
        for (id, title) in &strip.closed {
            out.push_str(&format!(
                "<li><a href=\"tisty:task/{}\">{}</a></li>",
                escaped(id),
                escaped(title)
            ));
        }
        out.push_str("</ul></details>");
    }
    out.push_str("</div></section>");
    out
}

#[cfg(test)]
#[path = "diary_page_test.rs"]
mod tests;
