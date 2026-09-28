mod repeat;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Mark {
    Date,
    Repeat,
    Deadline,
    List,
    Tag,
    Priority,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Certainty {
    #[default]
    Sure,
    Assumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub from: usize,
    pub to: usize,
    pub mark: Mark,
    pub certainty: Certainty,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Offer {
    pub spans: Vec<Span>,
    pub date: DateSpec,
    pub title: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Parsed {
    pub title: String,
    pub date: Option<DateSpec>,
    pub deadline: Option<DateSpec>,
    pub priority: Option<Priority>,
    pub tags: Vec<Tag>,
    pub list: Option<String>,
    pub repeat: Option<tisty_core::model::Repeat>,
    pub spans: Vec<Span>,
    pub offers: Vec<Offer>,
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
            repeat: p.repeat,
            source: None,
        }
    }
}

pub fn parse(input: &str, now: &Zoned, locale: &str) -> Parsed {
    let v = vocab::for_locale(locale);
    let tz = now.time_zone().iana_name().unwrap_or("UTC");
    let taken = take_markers(input, v);

    let mut parsed = Parsed {
        title: String::new(),
        priority: taken.priority,
        tags: taken.tags,
        list: taken.list,
        spans: taken.spans,
        ..Default::default()
    };

    if let Some(literal) = fully_quoted(&taken.text) {
        parsed.title = literal;
    } else {
        let over = repeat::take(&taken.text, &protect_quoted(&taken.text), now, v);
        parsed.repeat = over.repeat;
        if over.repeat.is_some() {
            parsed.spans.push(Span {
                from: over.from,
                to: over.to,
                mark: Mark::Repeat,
                certainty: Certainty::Sure,
            });
        }
        let mut read = timed(&over.text, now, tz, v);
        if let Some(first) = &over.first {
            read.date = Some(match read.date.take() {
                Some(said) if said.has_time => {
                    said.moved(first.at.date().to_datetime(said.at.time()))
                }
                _ => first.clone(),
            });
        }
        if over.repeat.is_some() {
            let shift = |at: &mut usize| {
                if *at >= over.head {
                    *at += over.shift;
                }
            };
            for span in read.spans.iter_mut() {
                shift(&mut span.from);
                shift(&mut span.to);
            }
            for span in read.offers.iter_mut().flat_map(|o| o.spans.iter_mut()) {
                shift(&mut span.from);
                shift(&mut span.to);
            }
        }
        let markers = parsed.spans.clone();
        parsed.title = read.title;
        parsed.date = read.date;
        parsed.deadline = read.deadline;
        if parsed.date.is_some()
            && let Some(over) = parsed.repeat
        {
            parsed.repeat = Some(tisty_core::model::Repeat {
                from: tisty_core::model::From::Due,
                ..over
            });
        }
        parsed.spans.extend(carved(read.spans, &markers, input, v));
        ends_the_series(&mut parsed, input, v);
        parsed.offers = read
            .offers
            .into_iter()
            .map(|offer| Offer {
                spans: carved(offer.spans, &markers, input, v),
                ..offer
            })
            .collect();
    }

    parsed.spans.sort_by_key(|span| span.from);
    let at = |byte: usize| input[..byte].chars().count();
    for span in &mut parsed.spans {
        span.from = at(span.from);
        span.to = at(span.to);
    }
    for span in parsed.offers.iter_mut().flat_map(|o| o.spans.iter_mut()) {
        span.from = at(span.from);
        span.to = at(span.to);
    }
    parsed
}

pub fn priority_word(p: Priority, locale: &str) -> &'static str {
    vocab::for_locale(locale).spoken(p)
}

pub fn parse_priority(raw: &str, locale: &str) -> Option<Priority> {
    vocab::for_locale(locale)
        .priority(raw)
        .or_else(|| vocab::EN.priority(raw))
}

pub fn parse_date(input: &str, now: &Zoned, locale: &str) -> Option<DateSpec> {
    let input = input.trim();
    let tz = now.time_zone().iana_name().unwrap_or("UTC");

    if let Ok(date) = input.parse::<jiff::civil::Date>() {
        return Some(DateSpec::all_day(date, tz));
    }

    let parsed = parse(&format!("· {input}"), now, locale);
    parsed
        .date
        .or(parsed.deadline)
        .or_else(|| parsed.offers.into_iter().next().map(|offer| offer.date))
}

struct Timed {
    title: String,
    date: Option<DateSpec>,
    deadline: Option<DateSpec>,
    spans: Vec<Span>,
    offers: Vec<Offer>,
}

fn timed(text: &str, now: &Zoned, tz: &str, v: &vocab::Vocabulary) -> Timed {
    let protected = protect_quoted(text);
    let tokens = scan::tokenize(&protected);
    let scanned = scan::scan(&tokens, v);
    let untouched = || Timed {
        title: tidy(text),
        date: None,
        deadline: None,
        spans: Vec::new(),
        offers: Vec::new(),
    };

    let reads: Vec<(&scan::Found, Read)> = [scanned.found.as_ref(), scanned.also.as_ref()]
        .into_iter()
        .flatten()
        .filter_map(|found| resolved(text, &tokens, found, now, tz).map(|read| (found, read)))
        .collect();

    if !reads.is_empty() {
        let mut cut: Vec<(usize, usize)> =
            reads.iter().flat_map(|(f, _)| f.spans.clone()).collect();
        cut.sort_unstable();
        let title = unquote(&without_spans(text, &tokens, &cut, v));
        if title.is_empty() {
            return untouched();
        }

        let mut timed = Timed {
            title,
            ..untouched()
        };
        for (found, read) in reads {
            match found.role {
                Role::Date => timed.date = Some(read.spec),
                Role::Deadline => timed.deadline = Some(read.spec),
            }
            timed.spans.extend(read.spans);
        }
        return timed;
    }

    let Some(found) = &scanned.offer else {
        return untouched();
    };
    let Some(read) = resolved(text, &tokens, found, now, tz) else {
        return untouched();
    };
    let title = unquote(&without_spans(text, &tokens, &found.spans, v));
    if title.is_empty() {
        return untouched();
    }

    Timed {
        offers: vec![Offer {
            spans: read.spans,
            date: read.spec,
            title,
        }],
        ..untouched()
    }
}

struct Read {
    spec: DateSpec,
    spans: Vec<Span>,
}

fn resolved(
    text: &str,
    tokens: &[scan::Token],
    found: &scan::Found,
    now: &Zoned,
    tz: &str,
) -> Option<Read> {
    let date = found.anchor.and_then(|a| resolve::to_date(a, now));
    let date = match found.time {
        Some(t) => resolve::place_time(date, t, now),
        None => date,
    }?;

    let spec = match found.time {
        Some(t) => DateSpec::floating(date.to_datetime(t), tz),
        None => DateSpec::all_day(date, tz),
    };
    let mark = match found.role {
        Role::Date => Mark::Date,
        Role::Deadline => Mark::Deadline,
    };

    let spans = found
        .spans
        .iter()
        .map(|(from, to)| {
            let (from, to) = pared(text, tokens[*from].start, tokens[*to - 1].end);
            Span {
                from,
                to,
                mark,
                certainty: found.certainty,
            }
        })
        .collect();

    Some(Read { spec, spans })
}

fn carved(spans: Vec<Span>, markers: &[Span], text: &str, v: &vocab::Vocabulary) -> Vec<Span> {
    let mut out = Vec::new();

    for span in spans {
        let mut at = span.from;
        for hole in markers
            .iter()
            .filter(|m| m.from >= span.from && m.to <= span.to)
        {
            if hole.from > at {
                out.push(Span {
                    from: at,
                    to: hole.from,
                    ..span
                });
            }
            at = hole.to;
        }
        if at < span.to {
            out.push(Span {
                from: at,
                to: span.to,
                ..span
            });
        }
    }

    out.retain_mut(|span| {
        let (from, to) = pared(text, span.from, span.to);
        span.from = from;
        span.to = to;
        text[from..to]
            .split_whitespace()
            .any(|word| !droppable(&word.to_lowercase(), v))
    });
    out
}

fn droppable(word: &str, v: &vocab::Vocabulary) -> bool {
    v.article.contains(&word)
        || v.time_prep.contains(&word)
        || v.date_prep.contains(&word)
        || v.deadline_prep.contains(&word)
        || v.in_prep.contains(&word)
}

fn pared(text: &str, from: usize, to: usize) -> (usize, usize) {
    let edge = |c: char| !c.is_alphanumeric();
    let slice = &text[from..to];
    let lead = slice.len() - slice.trim_start_matches(edge).len();
    let tail = slice.len() - slice.trim_end_matches(edge).len();
    if lead + tail >= slice.len() {
        return (from, to);
    }
    (from + lead, to - tail)
}

struct Taken {
    text: String,
    tags: Vec<Tag>,
    priority: Option<Priority>,
    list: Option<String>,
    spans: Vec<Span>,
}

fn take_markers(input: &str, v: &vocab::Vocabulary) -> Taken {
    let mut taken = Taken {
        text: String::with_capacity(input.len()),
        tags: Vec::new(),
        priority: None,
        list: None,
        spans: Vec::new(),
    };

    let mut inside = false;
    let mut at = 0;

    for (start, word) in words(input) {
        let quotes = word.matches('"').count();
        let quoted = inside;
        if quotes % 2 == 1 {
            inside = !inside;
        }
        if quoted || inside {
            continue;
        }

        let mark = if let Some(raw) = word.strip_prefix('#') {
            match Tag::new(raw) {
                Ok(tag) if tag.worth_reading() => {
                    taken.tags.push(tag);
                    Mark::Tag
                }
                _ => continue,
            }
        } else if let Some(raw) = word.strip_prefix('!') {
            match v.priority(raw).or_else(|| vocab::EN.priority(raw)) {
                Some(p) => {
                    taken.priority = Some(p);
                    Mark::Priority
                }
                None => continue,
            }
        } else if let Some(raw) = word.strip_prefix('@') {
            let raw = raw.trim_end_matches(|c: char| !c.is_alphanumeric());
            if raw.is_empty() || raw.parse::<u64>().is_ok() {
                continue;
            }
            taken.list = Some(raw.to_string());
            Mark::List
        } else {
            continue;
        };

        taken.text.push_str(&input[at..start]);
        taken.text.extend(std::iter::repeat_n(' ', word.len()));
        at = start + word.len();
        taken.spans.push(Span {
            from: start,
            to: at,
            mark,
            certainty: Certainty::Sure,
        });
    }

    taken.text.push_str(&input[at..]);
    taken
}

pub(crate) fn words(input: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut start = None;

    for (i, c) in input.char_indices() {
        if c.is_whitespace() {
            if let Some(s) = start.take() {
                out.push((s, &input[s..i]));
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        out.push((s, &input[s..]));
    }
    out
}

pub fn title_without(input: &str, spans: &[Span], locale: &str) -> String {
    let letters: Vec<char> = input.chars().collect();
    let mut ordered: Vec<&Span> = spans.iter().collect();
    ordered.sort_by_key(|span| span.from);

    let mut kept = Vec::new();
    let mut at = 0;
    for span in ordered {
        if span.from < at || span.to > letters.len() || span.from > span.to {
            continue;
        }
        kept.push(letters[at..span.from].iter().collect());
        at = span.to;
    }
    kept.push(letters[at..].iter().collect());
    sewn(kept, vocab::for_locale(locale))
}

fn without_spans(
    text: &str,
    tokens: &[scan::Token],
    spans: &[(usize, usize)],
    v: &vocab::Vocabulary,
) -> String {
    let mut kept = Vec::new();
    let mut at = 0;

    for (from, to) in spans {
        kept.push(text[at..tokens[*from].start].to_string());
        at = tokens.get(*to).map_or(text.len(), |token| token.start);
    }
    kept.push(text[at..].to_string());
    sewn(kept, v)
}

fn sewn(pieces: Vec<String>, v: &vocab::Vocabulary) -> String {
    let mut title = String::new();
    for piece in pieces {
        let tidied = tidy(&piece);
        let piece = tidied.trim_end_matches(',').trim_end();
        let piece = v
            .loose_ends
            .iter()
            .find_map(|word| piece.strip_suffix(&format!(" {word}")))
            .unwrap_or(piece);
        if piece.is_empty() || v.loose_ends.contains(&piece) {
            continue;
        }
        if !title.is_empty() {
            title.push(' ');
        }
        title.push_str(piece);
    }
    title
}

fn tidy(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn fully_quoted(text: &str) -> Option<String> {
    let t = text.trim();
    let inner = t.strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains('"')).then(|| inner.to_string())
}

fn protect_quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut inside = false;

    for c in text.chars() {
        if c == '"' {
            inside = !inside;
            out.push('"');
        } else if inside {
            let fill = if c.is_whitespace() { '_' } else { 'x' };
            out.extend(std::iter::repeat_n(fill, c.len_utf8()));
        } else {
            out.push(c);
        }
    }
    out
}

fn unquote(text: &str) -> String {
    text.trim().to_string()
}

fn ends_the_series(parsed: &mut Parsed, input: &str, v: &vocab::Vocabulary) {
    let (Some(over), Some(last)) = (parsed.repeat, parsed.deadline.as_ref()) else {
        return;
    };
    let Some(at) = parsed
        .spans
        .iter()
        .position(|one| one.mark == Mark::Deadline)
    else {
        return;
    };
    let said = input[parsed.spans[at].from..parsed.spans[at].to].to_lowercase();
    if !said
        .split_whitespace()
        .next()
        .is_some_and(|word| v.ends_prep.contains(&word))
    {
        return;
    }
    parsed.repeat = Some(tisty_core::model::Repeat {
        until: Some(last.at.date()),
        ..over
    });
    parsed.deadline = None;
    parsed.spans[at].mark = Mark::Repeat;
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
