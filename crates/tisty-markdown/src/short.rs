use std::ops::Range;

use crate::vocab;
use crate::{Mark, Span};

/// `::name hue::` is an icon and `++words++` is underlined, written the way the person types them.
pub fn fold(spans: &mut Vec<Span>, text: &str) {
    let mut found = Vec::new();
    for span in spans.iter().filter(|one| one.mark == Mark::Text) {
        iconed(text, span.at.clone(), &mut found);
    }
    found.extend(underlined(spans, text));
    spans.extend(found);
}

fn alone(at: Option<char>) -> bool {
    at.is_none_or(|c| !c.is_alphanumeric())
}

fn iconed(text: &str, within: Range<usize>, found: &mut Vec<Span>) {
    let said = &text[within.clone()];
    let mut from = 0;
    while let Some(open) = said[from..].find("::").map(|at| from + at) {
        let rest = &said[open + 2..];
        let Some(close) = rest.find("::") else {
            break;
        };
        let inner = &rest[..close];
        let (name, hue) = match inner.split_once(' ') {
            Some((name, hue)) => (name, Some(hue)),
            None => (inner, None),
        };
        let end = open + 2 + close + 2;
        let fits = vocab::icon(name)
            && hue.is_none_or(vocab::hue)
            && alone(text[..within.start + open].chars().next_back())
            && alone(text[within.start + end..].chars().next());
        if !fits {
            from = open + 1;
            continue;
        }
        found.push(Span {
            mark: Mark::Icon {
                name: name.to_string(),
                hue: hue.map(str::to_string),
            },
            at: within.start + open..within.start + end,
        });
        from = end;
    }
}

// A wall touching a letter, a digit or another `+` is somebody writing C++ or i++, not underlining.
fn underlined(spans: &[Span], text: &str) -> Vec<Span> {
    let mut walls = Vec::new();
    for span in spans.iter().filter(|one| one.mark == Mark::Text) {
        let bytes = text[span.at.clone()].as_bytes();
        let mut at = 0;
        while at + 1 < bytes.len() {
            let run = bytes[at..].iter().take_while(|b| **b == b'+').count();
            if run == 2 {
                walls.push(span.at.start + at);
            }
            at += run.max(1);
        }
    }
    let lone = |c: Option<char>| c.is_none_or(|c| !(c.is_alphanumeric() || c == '+'));
    let mut found = Vec::new();
    let mut at = 0;
    while at + 1 < walls.len() {
        let (open, shut) = (walls[at], walls[at + 1]);
        let inside = &text[open + 2..shut];
        let fits = !inside.is_empty()
            && !inside.starts_with(char::is_whitespace)
            && !inside.ends_with(char::is_whitespace)
            && lone(text[..open].chars().next_back())
            && lone(text[shut + 2..].chars().next());
        if fits {
            found.push(Span {
                mark: Mark::Underline,
                at: open..shut + 2,
            });
            at += 2;
        } else {
            at += 1;
        }
    }
    found
}
