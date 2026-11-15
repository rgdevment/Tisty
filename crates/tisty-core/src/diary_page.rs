use jiff::civil::Date;

use crate::State;
use crate::model::Kept;

pub struct Said<'a> {
    pub today: &'a str,
    pub one: &'a str,
    pub many: &'a str,
}

struct Strip {
    today: bool,
    closed: Vec<(String, String)>,
}

// Drawn beside the words and never into them: the strip is the log's, the text stays the person's.
pub fn dressed(
    html: &str,
    body: &str,
    state: &State,
    kept: &Kept,
    today: Date,
    zone: &jiff::tz::TimeZone,
    said: &Said,
) -> String {
    let Some((year, month)) = crate::diary::month_of_page(state, kept) else {
        return html.to_string();
    };
    let doc = tisty_markdown::read(body);
    let days: Vec<(std::ops::Range<usize>, Date)> = doc
        .blocks()
        .iter()
        .filter(|block| block.kind == tisty_markdown::Kind::Heading(2))
        .filter_map(|block| {
            let title = doc.said(block).trim_start_matches('#');
            let day = crate::diary::day_named(title)?;
            Some((block.at.clone(), Date::new(year, month, day).ok()?))
        })
        .collect();
    let mut out = html.to_string();
    for (at, date) in days.into_iter().rev() {
        let strip = Strip {
            today: date == today,
            closed: crate::diary::closed_on(state, date, zone)
                .into_iter()
                .map(|task| (task.id.to_string(), task.title.clone()))
                .collect(),
        };
        if !strip.today && strip.closed.is_empty() {
            continue;
        }
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
    let mut out = String::from("<div class=\"diary-day\">");
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

fn escaped(said: &str) -> String {
    said.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
#[path = "diary_page_test.rs"]
mod tests;
