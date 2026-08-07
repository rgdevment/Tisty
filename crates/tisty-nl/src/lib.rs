//! Natural language capture. Nothing reaches a model, so it stays reproducible.

mod resolve;
mod scan;
mod vocab;

use jiff::Zoned;
use serde::{Deserialize, Serialize};
use tisty_core::{
    DateSpec, Priority, Tag,
    capture::{Draft, Filing},
};

use scan::Role;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Parsed {
    pub title: String,
    pub date: Option<DateSpec>,
    pub deadline: Option<DateSpec>,
    pub priority: Option<Priority>,
    pub tags: Vec<Tag>,
    pub list: Option<String>,
}

impl From<Parsed> for Draft {
    fn from(p: Parsed) -> Self {
        Self {
            title: p.title,
            date: p.date,
            deadline: p.deadline,
            priority: p.priority,
            tags: p.tags,
            filing: p.list.map(Filing::Marked),
        }
    }
}

pub fn parse(input: &str, now: &Zoned, locale: &str) -> Parsed {
    let v = vocab::for_locale(locale);
    let tz = now.time_zone().iana_name().unwrap_or("UTC");

    let (text, tags, priority, list) = take_markers(input);

    if let Some(literal) = fully_quoted(&text) {
        return Parsed {
            title: literal,
            tags,
            priority,
            list,
            ..Default::default()
        };
    }

    let protected = protect_quoted(&text);
    let tokens = scan::tokenize(&protected);
    let Some(found) = scan::scan(&tokens, v) else {
        return Parsed {
            title: unquote(text.trim()),
            tags,
            priority,
            list,
            ..Default::default()
        };
    };

    let date = found.anchor.and_then(|a| resolve::to_date(a, now));
    let date = match found.time {
        Some(t) => resolve::place_time(date, t, now),
        None => date,
    };

    let Some(date) = date else {
        return Parsed {
            title: unquote(text.trim()),
            tags,
            priority,
            list,
            ..Default::default()
        };
    };

    let title = unquote(text[..tokens[found.from_token].start].trim());
    if title.is_empty() {
        return Parsed {
            title: unquote(text.trim()),
            tags,
            priority,
            list,
            ..Default::default()
        };
    }

    let spec = match found.time {
        Some(t) => DateSpec::floating(date.to_datetime(t), tz),
        None => DateSpec::all_day(date, tz),
    };

    let (date, deadline) = match found.role {
        Role::Date => (Some(spec), None),
        Role::Deadline => (None, Some(spec)),
    };

    Parsed {
        title,
        date,
        deadline,
        priority,
        tags,
        list,
    }
}

/// A flag value is a date on its own, with no title to separate it from.
pub fn parse_date(input: &str, now: &Zoned, locale: &str) -> Option<DateSpec> {
    let input = input.trim();
    let tz = now.time_zone().iana_name().unwrap_or("UTC");

    if let Ok(date) = input.parse::<jiff::civil::Date>() {
        return Some(DateSpec::all_day(date, tz));
    }

    let parsed = parse(&format!("· {input}"), now, locale);
    parsed.date.or(parsed.deadline)
}

fn take_markers(input: &str) -> (String, Vec<Tag>, Option<Priority>, Option<String>) {
    let mut tags = Vec::new();
    let mut priority = None;
    let mut list = None;
    let mut kept = Vec::new();

    for word in input.split_whitespace() {
        if let Some(raw) = word.strip_prefix('@')
            && let Ok(tag) = Tag::new(raw)
        {
            tags.push(tag);
            continue;
        }
        if let Some(digit) = word.strip_prefix('!')
            && let Ok(n) = digit.parse::<u8>()
            && let Ok(p) = Priority::try_from(n)
        {
            priority = Some(p);
            continue;
        }
        // A bare `#42` is part of the title: a list is never named by digits alone.
        if let Some(raw) = word.strip_prefix('#')
            && !raw.is_empty()
            && raw.parse::<u64>().is_err()
        {
            list = Some(raw.to_string());
            continue;
        }
        kept.push(word);
    }

    (kept.join(" "), tags, priority, list)
}

fn fully_quoted(text: &str) -> Option<String> {
    let t = text.trim();
    let inner = t.strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains('"')).then(|| inner.to_string())
}

/// Placeholders keep the same byte length, or the offsets stop matching.
fn protect_quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut inside = false;

    for c in text.chars() {
        if c == '"' {
            inside = !inside;
            out.push('"');
        } else if inside && c.is_whitespace() {
            out.push('_');
        } else if inside {
            out.push('x');
        } else {
            out.push(c);
        }
    }
    out
}

fn unquote(text: &str) -> String {
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> Zoned {
        "2026-08-05T09:00:00[America/Santiago]".parse().unwrap()
    }

    #[test]
    fn a_phrase_without_anything_temporal_keeps_its_whole_title() {
        let p = parse("actualizar las dependencias", &now(), "es");
        assert_eq!(p.title, "actualizar las dependencias");
        assert!(p.date.is_none());
    }

    #[test]
    fn markers_are_taken_out_of_the_title() {
        let p = parse("revisar el deploy @backend !1", &now(), "es");
        assert_eq!(p.title, "revisar el deploy");
        assert_eq!(p.priority, Some(Priority::P1));
        assert_eq!(p.tags.len(), 1);
    }

    #[test]
    fn a_bare_date_needs_no_title_around_it() {
        assert!(parse_date("mañana", &now(), "es").is_some());
        assert!(parse_date("2026-12-24", &now(), "es").is_some());
        assert!(parse_date("next friday", &now(), "en").is_some());
        assert!(parse_date("not a date", &now(), "en").is_none());
    }

    /// A client hands over what the system reports, not a canonical code.
    #[test]
    fn a_regional_locale_still_speaks_its_language() {
        let p = parse("comprar pan mañana", &now(), "es-CL");
        assert_eq!(p.title, "comprar pan");
        assert!(p.date.is_some());
    }

    #[test]
    fn quoted_text_is_never_interpreted() {
        let p = parse("\"reunión el lunes\"", &now(), "es");
        assert_eq!(p.title, "reunión el lunes");
        assert!(p.date.is_none());
    }
}
