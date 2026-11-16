use std::path::Path;

use jiff::civil::Date;
use tisty_markdown::escaped;

use crate::State;
use crate::diary::Words;
use crate::model::Kept;

pub struct Said<'a> {
    pub one: &'a str,
    pub many: &'a str,
    pub empty: &'a str,
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

// A heading that names no day stays inside the day above it, so nothing the person wrote is lost.
pub fn days(body: &str) -> Days {
    let lines: Vec<&str> = body.lines().collect();
    let heads = crate::docs::headings(body);
    let titled = heads
        .first()
        .filter(|(line, level, _)| *line == 1 && *level == 1);
    let starts: Vec<(usize, i8)> = heads
        .iter()
        .filter(|(_, level, _)| *level == 2)
        .filter_map(|(line, _, title)| Some((*line, crate::diary::day_named(title)?)))
        .collect();
    let words = |from: usize, to: usize| lines[from.min(to)..to].join("\n").trim().to_string();
    let first = starts.first().map_or(lines.len(), |(line, _)| line - 1);
    let intro = words(titled.map_or(0, |_| 1), first);
    let each = starts
        .iter()
        .enumerate()
        .map(|(at, (line, day))| {
            let end = starts.get(at + 1).map_or(lines.len(), |(next, _)| next - 1);
            (*day, words(*line, end))
        })
        .collect();
    Days { intro, each }
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
pub fn laid(days: &Days, strips: &[Strip], words: &Words, said: &Said) -> String {
    let mut out = String::from("<div class=\"diary-month\">");
    if !days.intro.is_empty() {
        out.push_str(&format!(
            "<div class=\"diary-intro\">{}</div>",
            drawn(&days.intro)
        ));
    }
    for ((_, text), strip) in days.each.iter().zip(strips) {
        out.push_str(&entry(text, strip, words, said));
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

fn entry(text: &str, strip: &Strip, words: &Words, said: &Said) -> String {
    let weekday = strip.date.map_or("", |date| {
        words.weekdays[usize::from(date.weekday().to_monday_zero_offset().unsigned_abs())]
    });
    let short = weekday.chars().take(3).collect::<String>().to_lowercase();
    let mut out = format!(
        "<section class=\"diary-entry{}\"><div class=\"diary-date\"><span class=\"diary-number\">{}</span><span class=\"diary-weekday\">{}</span></div><div class=\"diary-words\">{}",
        if strip.today { " diary-is-today" } else { "" },
        strip.day,
        escaped(&short),
        drawn(text)
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
