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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Mark {
    Date,
    Deadline,
    List,
    Tag,
    Priority,
}

/// Applied like any certainty, only marked as one — otherwise the same sentence would store differently depending on where it was typed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Certainty {
    #[default]
    Sure,
    Assumed,
}

/// Offsets are code points.
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
    /// What the title would become if it were taken.
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
    /// Where each reading came from in the input, so a client can point at it.
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
        let read = timed(&taken.text, now, tz, v);
        let markers = parsed.spans.clone();
        parsed.title = read.title;
        parsed.date = read.date;
        parsed.deadline = read.deadline;
        parsed.spans.extend(carved(read.spans, &markers, input, v));
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

/// `!1` or `!urgente`, the same two forms the capture accepts.
pub fn parse_priority(raw: &str, locale: &str) -> Option<Priority> {
    raw.parse::<u8>()
        .ok()
        .and_then(|n| Priority::try_from(n).ok())
        .or_else(|| vocab::for_locale(locale).priority(raw))
}

/// A flag value is a date on its own, with no title to separate it from.
pub fn parse_date(input: &str, now: &Zoned, locale: &str) -> Option<DateSpec> {
    let input = input.trim();
    let tz = now.time_zone().iana_name().unwrap_or("UTC");

    if let Ok(date) = input.parse::<jiff::civil::Date>() {
        return Some(DateSpec::all_day(date, tz));
    }

    // A flag value is explicitly a date, so the ambiguity that holds an offer back does not apply.
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
        let title = unquote(&without_spans(text, &tokens, &cut));
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
    let title = unquote(&without_spans(text, &tokens, &found.spans));
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

/// A marker inside the phrase keeps its own colour and its own words: blanking
/// it left a hole the scanner walked across, so one range covered them both.
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

    // A piece left with nothing but particles is what the swallow dragged in,
    // not a reading: «llamar a @juan mañana» would paint the «a» blue.
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

/// Punctuation around the phrase is not part of the reading: «mañana,» would otherwise draw its comma inside the highlight.
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

/// Markers are blanked instead of removed, so every offset downstream still points at the text the user typed.
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
        // Between quotes nothing is interpreted, markers included.
        let quotes = word.matches('"').count();
        let quoted = inside;
        if quotes % 2 == 1 {
            inside = !inside;
        }
        if quoted || inside {
            continue;
        }

        let mark = if let Some(raw) = word.strip_prefix('#') {
            // A bare `#42` is a written reference — «review PR #42» — not a marker.
            match Tag::new(raw) {
                Ok(tag) if raw.parse::<u64>().is_err() => {
                    taken.tags.push(tag);
                    Mark::Tag
                }
                _ => continue,
            }
        } else if let Some(raw) = word.strip_prefix('!') {
            // `!1` is what fits in a terminal; `!urgente` is what the window shows.
            match raw
                .parse::<u8>()
                .ok()
                .and_then(|n| Priority::try_from(n).ok())
                .or_else(|| v.priority(raw))
            {
                Some(p) => {
                    taken.priority = Some(p);
                    Mark::Priority
                }
                None => continue,
            }
        } else if let Some(raw) = word.strip_prefix('@') {
            // Trailing punctuation would create «juan,» next to «juan».
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

fn words(input: &str) -> Vec<(usize, &str)> {
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

/// The title with these readings removed; public so a client can preview unmarking one.
pub fn title_without(input: &str, spans: &[Span]) -> String {
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
    sewn(kept)
}

/// The temporal phrase can sit mid-sentence, so both sides of the hole are the title.
fn without_spans(text: &str, tokens: &[scan::Token], spans: &[(usize, usize)]) -> String {
    let mut kept = Vec::new();
    let mut at = 0;

    for (from, to) in spans {
        kept.push(text[at..tokens[*from].start].to_string());
        at = tokens.get(*to).map_or(text.len(), |token| token.start);
    }
    kept.push(text[at..].to_string());
    sewn(kept)
}

fn sewn(pieces: Vec<String>) -> String {
    let mut title = String::new();
    for piece in pieces {
        let tidied = tidy(&piece);
        let piece = tidied.trim_end_matches(',').trim_end();
        let piece = LOOSE_ENDS
            .iter()
            .find_map(|word| piece.strip_suffix(&format!(" {word}")))
            .unwrap_or(piece);
        if piece.is_empty() || LOOSE_ENDS.contains(&piece) {
            continue;
        }
        if !title.is_empty() {
            title.push(' ');
        }
        title.push_str(piece);
    }
    title
}

const LOOSE_ENDS: &[&str] = &[
    "y", "e", "o", "u", "ni", "and", "or", "nor", "to", "al", "a", "el", "la", "los", "las", "por",
    "for",
];

fn tidy(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> Zoned {
        "2026-08-05T09:00:00[America/Santiago]".parse().unwrap()
    }

    fn spans(input: &str, locale: &str) -> Vec<(String, Mark, Certainty)> {
        let parsed = parse(input, &now(), locale);
        let chars: Vec<char> = input.chars().collect();
        parsed
            .spans
            .iter()
            .map(|s| (chars[s.from..s.to].iter().collect(), s.mark, s.certainty))
            .collect()
    }

    fn offered(input: &str, locale: &str) -> Option<(String, String, String)> {
        let parsed = parse(input, &now(), locale);
        let chars: Vec<char> = input.chars().collect();
        parsed.offers.first().map(|offer| {
            let span = offer.spans[0];
            (
                chars[span.from..span.to].iter().collect(),
                offer.date.date().to_string(),
                offer.title.clone(),
            )
        })
    }

    #[test]
    fn a_bare_three_is_this_afternoon_and_not_tomorrow_dawn() {
        let seven: Zoned = "2026-08-05T07:00:00[America/Santiago]".parse().unwrap();
        let p = parse("tomar café a las 3", &seven, "es");
        let at = p.date.expect("a date");

        assert_eq!(at.date().to_string(), "2026-08-05");
        assert_eq!(at.at.time().to_string(), "15:00:00");
        assert_eq!(p.spans[0].certainty, Certainty::Assumed);
    }

    #[test]
    fn the_same_three_rolls_over_once_the_afternoon_is_gone() {
        let evening: Zoned = "2026-08-05T18:00:00[America/Santiago]".parse().unwrap();
        let p = parse("tomar café a las 3", &evening, "es");
        let at = p.date.expect("a date");

        assert_eq!(at.date().to_string(), "2026-08-06");
        assert_eq!(at.at.time().to_string(), "15:00:00");
    }

    #[test]
    fn a_described_noun_comes_back_as_an_offer() {
        assert_eq!(
            offered("revisar el informe del lunes", "es"),
            Some((
                "del lunes".to_string(),
                "2026-08-10".to_string(),
                "revisar el informe".to_string()
            ))
        );
    }

    #[test]
    fn an_offer_before_a_noun_leaves_its_article_behind() {
        assert_eq!(
            offered("review the monday report", "en"),
            Some((
                "monday".to_string(),
                "2026-08-10".to_string(),
                "review the report".to_string()
            ))
        );
    }

    #[test]
    fn a_word_that_means_something_else_is_not_offered() {
        for input in [
            "reunión por la mañana",
            "mañana de verano",
            "revisar lo de hace 3 días",
        ] {
            assert_eq!(offered(input, "es"), None, "{input}");
        }
    }

    #[test]
    fn nothing_is_offered_once_something_was_taken() {
        let p = parse(
            "preparar la reunión del martes para el jueves",
            &now(),
            "es",
        );
        assert!(p.date.is_some());
        assert!(p.offers.is_empty());
    }

    #[test]
    fn a_date_flag_takes_the_offer_it_would_otherwise_hold_back() {
        assert!(parse_date("del lunes", &now(), "es").is_some());
    }

    #[test]
    fn a_phrase_without_anything_temporal_keeps_its_whole_title() {
        let p = parse("actualizar las dependencias", &now(), "es");
        assert_eq!(p.title, "actualizar las dependencias");
        assert!(p.date.is_none());
        assert!(p.spans.is_empty());
    }

    #[test]
    fn markers_are_taken_out_of_the_title() {
        let p = parse("revisar el deploy #backend !1", &now(), "es");
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

    #[test]
    fn a_regional_locale_still_speaks_its_language() {
        let p = parse("comprar pan mañana", &now(), "es-CL");
        assert_eq!(p.title, "comprar pan");
        assert!(p.date.is_some());
    }

    #[test]
    fn a_decomposed_enye_is_still_a_word() {
        let p = parse("comprar pan man\u{0303}ana", &now(), "es");
        assert_eq!(p.title, "comprar pan");
        assert!(p.date.is_some());
    }

    #[test]
    fn quoted_text_is_never_interpreted() {
        let p = parse("\"reunión el lunes\"", &now(), "es");
        assert_eq!(p.title, "reunión el lunes");
        assert!(p.date.is_none());
        assert!(p.spans.is_empty());
    }

    #[test]
    fn every_span_points_at_what_it_read() {
        assert_eq!(
            spans("comprar pan mañana #casa @compras !1", "es"),
            [
                ("mañana".to_string(), Mark::Date, Certainty::Sure),
                ("#casa".to_string(), Mark::Tag, Certainty::Sure),
                ("@compras".to_string(), Mark::List, Certainty::Sure),
                ("!1".to_string(), Mark::Priority, Certainty::Sure),
            ]
        );
    }

    #[test]
    fn offsets_survive_a_marker_written_with_accents() {
        assert_eq!(
            spans("#niño revisar la sesión mañana", "es"),
            [
                ("#niño".to_string(), Mark::Tag, Certainty::Sure),
                ("mañana".to_string(), Mark::Date, Certainty::Sure),
            ]
        );
    }

    #[test]
    fn a_phrase_split_by_the_title_reports_both_halves() {
        assert_eq!(
            spans("reunión el martes en la sala 3c a las 16:00", "es"),
            [
                ("el martes".to_string(), Mark::Date, Certainty::Sure),
                ("a las 16:00".to_string(), Mark::Date, Certainty::Sure),
            ]
        );
    }

    /// A blanked marker left a hole the scanner walked across, so one date span
    /// covered the tag too: it painted over it and unmarking lost the word.
    #[test]
    fn a_marker_inside_the_phrase_keeps_its_own_span() {
        assert_eq!(
            spans("reunión el martes #trabajo a las 16:00", "es"),
            [
                ("el martes".to_string(), Mark::Date, Certainty::Sure),
                ("#trabajo".to_string(), Mark::Tag, Certainty::Sure),
                ("a las 16:00".to_string(), Mark::Date, Certainty::Sure),
            ]
        );
    }

    #[test]
    fn a_list_between_the_verb_and_the_day_is_not_swallowed() {
        assert_eq!(
            spans("llamar a @juan mañana", "es"),
            [
                ("@juan".to_string(), Mark::List, Certainty::Sure),
                ("mañana".to_string(), Mark::Date, Certainty::Sure),
            ]
        );
    }

    #[test]
    fn no_span_ever_covers_another() {
        for text in [
            "comprar pan para #casa mañana",
            "entregar el informe para @trabajo el lunes",
            "reunión el martes #trabajo a las 16:00",
            "llamar a @juan mañana !1",
        ] {
            let read = parse(text, &now(), "es");
            let mut ranges: Vec<_> = read.spans.iter().map(|s| (s.from, s.to)).collect();
            ranges.sort_unstable();
            for pair in ranges.windows(2) {
                assert!(
                    pair[0].1 <= pair[1].0,
                    "{text}: {:?} overlaps {:?}",
                    pair[0],
                    pair[1]
                );
            }
        }
    }

    #[test]
    fn a_deadline_is_marked_as_one() {
        let read = spans("entregar el informe antes del viernes", "es");
        assert_eq!(read[0].1, Mark::Deadline);
    }

    #[test]
    fn mid_sentence_without_a_signal_is_an_assumption() {
        let p = parse("llamar mañana al banco", &now(), "es");
        assert_eq!(p.title, "llamar al banco");
        assert_eq!(p.spans[0].certainty, Certainty::Assumed);
    }

    #[test]
    fn a_phrase_at_the_end_is_no_assumption() {
        let p = parse("llamar al banco mañana", &now(), "es");
        assert_eq!(p.spans[0].certainty, Certainty::Sure);
    }

    #[test]
    fn a_described_noun_is_left_alone() {
        let p = parse("revisar el informe del lunes", &now(), "es");
        assert_eq!(p.title, "revisar el informe del lunes");
        assert!(p.spans.is_empty());
    }

    /// `app/src/core.ts` mirrors these names by hand; a rename here is silent there until something stops lighting up.
    #[test]
    fn a_span_reaches_the_window_under_the_names_it_declares() {
        let span = Span {
            from: 3,
            to: 9,
            mark: Mark::Deadline,
            certainty: Certainty::Assumed,
        };
        assert_eq!(
            serde_json::to_string(&span).unwrap(),
            r#"{"from":3,"to":9,"mark":"deadline","certainty":"assumed"}"#
        );
    }

    #[test]
    fn quoting_an_accented_word_keeps_the_rest_in_place() {
        let p = parse("mandar \"café ñandú\" mañana", &now(), "es");
        assert_eq!(p.title, "mandar \"café ñandú\"");
        assert!(p.date.is_some());
    }
}
