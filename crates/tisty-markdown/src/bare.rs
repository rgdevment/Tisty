use std::ops::Range;

use crate::{Mark, Span};

pub fn fold(spans: &mut Vec<Span>, text: &str) {
    let linked: Vec<Range<usize>> = spans
        .iter()
        .filter(|one| matches!(one.mark, Mark::Link { .. } | Mark::Image { .. }))
        .map(|one| one.at.clone())
        .collect();
    let mut found = Vec::new();
    for run in runs(spans, text) {
        if linked
            .iter()
            .any(|one| one.start <= run.start && run.end <= one.end)
        {
            continue;
        }
        found.extend(addresses(text, run));
    }
    if found.is_empty() {
        return;
    }
    let mut kept = Vec::with_capacity(spans.len() + found.len() * 2);
    for span in spans.drain(..) {
        if span.mark != Mark::Text {
            kept.push(span);
            continue;
        }
        let inside = found.iter().filter_map(|one| {
            let start = one.start.max(span.at.start);
            let end = one.end.min(span.at.end);
            (start < end).then_some(start..end)
        });
        let pieces: Vec<Range<usize>> = crate::inline::outside(span.at.clone(), &found)
            .into_iter()
            .chain(inside)
            .collect();
        kept.extend(pieces.into_iter().map(|at| Span {
            mark: Mark::Text,
            at,
        }));
    }
    for at in found {
        kept.push(Span {
            mark: Mark::Link {
                to: target(&text[at.clone()]),
            },
            at,
        });
    }
    *spans = kept;
}

// Text arrives cut wherever the tokenizer paused, an escaping backslash included, so an address may span several pieces.
fn runs(spans: &[Span], text: &str) -> Vec<Range<usize>> {
    let mut texts: Vec<Range<usize>> = spans
        .iter()
        .filter(|one| one.mark == Mark::Text)
        .map(|one| one.at.clone())
        .collect();
    texts.sort_by_key(|one| one.start);
    let mut out: Vec<Range<usize>> = Vec::new();
    for one in texts {
        match out.last_mut() {
            Some(last) if last.end == one.start || &text[last.end..one.start] == "\\" => {
                last.end = one.end
            }
            _ => out.push(one),
        }
    }
    out
}

fn addresses(text: &str, run: Range<usize>) -> Vec<Range<usize>> {
    let said = &text[run.clone()];
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = said[from..].find("http").map(|one| from + one) {
        let rest = &said[at..];
        let scheme = ["https://", "http://"]
            .into_iter()
            .find(|one| rest.starts_with(one));
        let before = said[..at].chars().next_back();
        let Some(scheme) = scheme.filter(|_| before.is_none_or(|one| !one.is_alphanumeric()))
        else {
            from = at + 4;
            continue;
        };
        let long = rest
            .find(|one: char| one.is_whitespace() || one == '<')
            .unwrap_or(rest.len());
        let end = trimmed(&rest[..long]);
        if end > scheme.len() {
            out.push(run.start + at..run.start + at + end);
        }
        from = at + long.max(scheme.len());
    }
    out
}

// What closes a sentence or one of Tisty's own walls is not part of the address, and a bracket only when nothing opened it.
fn trimmed(said: &str) -> usize {
    let mut end = said.len();
    let opened = said.matches('(').count();
    let mut shut = said.matches(')').count();
    loop {
        let left = &said[..end];
        if left.ends_with('}')
            && let Some(wall) = left.rfind("=={")
        {
            end = wall;
            continue;
        }
        if ["==", "++", "::"].iter().any(|wall| left.ends_with(wall)) {
            end -= 2;
            continue;
        }
        let Some(last) = left.chars().next_back() else {
            return end;
        };
        let stray = match last {
            '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' | '*' | '_' | '~' => true,
            ')' => shut > opened,
            _ => false,
        };
        if !stray {
            return end;
        }
        if last == ')' {
            shut -= 1;
        }
        end -= last.len_utf8();
    }
}

// The address as a browser wants it: no escaping backslash, and `&amp;` back to the `&` it stands for.
fn target(said: &str) -> String {
    let mut out = String::with_capacity(said.len());
    let mut chars = said.chars().peekable();
    while let Some(one) = chars.next() {
        if one == '\\' && chars.peek().is_some_and(char::is_ascii_punctuation) {
            continue;
        }
        out.push(one);
    }
    out.replace("&amp;", "&")
}
