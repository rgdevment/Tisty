use std::ops::Range;
use std::path::Path;

use jiff::civil::Date;
use tisty_markdown::escaped;

use crate::State;
use crate::model::Kept;

pub struct Said<'a> {
    pub today: &'a str,
    pub one: &'a str,
    pub many: &'a str,
}

pub struct Page<'a> {
    pub root: &'a Path,
    pub state: &'a State,
    pub kept: &'a Kept,
    pub today: Date,
    pub zone: &'a jiff::tz::TimeZone,
}

struct Strip {
    today: bool,
    closed: Vec<(String, String)>,
}

/// The days a page's sections name, read once from its words, before anything of the log is asked.
pub struct Days(Vec<(Range<usize>, i8)>);

pub fn days(body: &str) -> Days {
    let doc = tisty_markdown::read(body);
    Days(
        doc.blocks()
            .iter()
            .filter(|block| block.kind == tisty_markdown::Kind::Heading(2))
            .filter_map(|block| {
                let title = doc.said(block).trim_start_matches('#');
                Some((block.at.clone(), crate::diary::day_named(title)?))
            })
            .collect(),
    )
}

fn strips(days: &Days, page: &Page) -> Vec<(Range<usize>, i8, Strip)> {
    let Some((year, month)) = crate::diary::month_of_page(page.root, page.state, page.kept) else {
        return Vec::new();
    };
    days.0
        .iter()
        .filter_map(|(at, day)| {
            let date = Date::new(year, month, *day).ok()?;
            let strip = Strip {
                today: date == page.today,
                closed: crate::diary::closed_on(page.state, date, page.zone)
                    .into_iter()
                    .map(|task| (task.id.to_string(), task.title.clone()))
                    .collect(),
            };
            (strip.today || !strip.closed.is_empty()).then(|| (at.clone(), *day, strip))
        })
        .collect()
}

// What the strips would say, so the window draws the page again only when that changes.
pub fn print(days: &Days, page: &Page) -> String {
    strips(days, page)
        .iter()
        .map(|(_, day, strip)| {
            let ids: Vec<&str> = strip.closed.iter().map(|(id, _)| id.as_str()).collect();
            format!("{day}:{}:{}", strip.today, ids.join(","))
        })
        .collect::<Vec<_>>()
        .join("|")
}

// Drawn beside the words and never into them: the strip is the log's, the text stays the person's.
pub fn dressed(html: &str, days: &Days, page: &Page, said: &Said) -> String {
    let mut out = html.to_string();
    for (at, _, strip) in strips(days, page).into_iter().rev() {
        let marker = format!("data-at=\"{}-{}\"", at.start, at.end);
        let Some(from) = out.find(&marker) else {
            continue;
        };
        let Some(close) = out[from..].find("</h2>") else {
            continue;
        };
        out.insert_str(from + close + "</h2>".len(), &drawn(&strip, said));
    }
    out
}

fn drawn(strip: &Strip, said: &Said) -> String {
    let mut out = String::from("<div class=\"diary-day\" contenteditable=\"false\">");
    if strip.today {
        out.push_str(&format!(
            "<span class=\"diary-today\">{}</span>",
            escaped(said.today)
        ));
    }
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
    out.push_str("</div>");
    out
}

#[cfg(test)]
#[path = "diary_page_test.rs"]
mod tests;
