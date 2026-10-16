use std::ops::Range;

use crate::{Mark, Span};

pub fn fold(spans: &mut Vec<Span>, text: &str) {
    let linked: Vec<Range<usize>> = spans
        .iter()
        .filter(|one| matches!(one.mark, Mark::Link { .. } | Mark::Image { .. }))
        .map(|one| one.at.clone())
        .collect();
    let mut found = Vec::new();
    for run in runs(spans) {
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
        for piece in crate::inline::outside(span.at.clone(), &found) {
            kept.push(Span {
                mark: Mark::Text,
                at: piece,
            });
        }
    }
    for at in found {
        kept.push(Span {
            mark: Mark::Link {
                to: text[at.clone()].to_string(),
            },
            at: at.clone(),
        });
        kept.push(Span {
            mark: Mark::Text,
            at,
        });
    }
    *spans = kept;
}

// Text arrives cut in pieces wherever the tokenizer paused, so an address may span several.
fn runs(spans: &[Span]) -> Vec<Range<usize>> {
    let mut texts: Vec<Range<usize>> = spans
        .iter()
        .filter(|one| one.mark == Mark::Text)
        .map(|one| one.at.clone())
        .collect();
    texts.sort_by_key(|one| one.start);
    let mut out: Vec<Range<usize>> = Vec::new();
    for one in texts {
        match out.last_mut() {
            Some(last) if last.end == one.start => last.end = one.end,
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

// What closes a sentence is not part of the address, and a bracket only when nothing opened it.
fn trimmed(said: &str) -> usize {
    let mut end = said.len();
    loop {
        let Some(last) = said[..end].chars().next_back() else {
            return end;
        };
        let opened = said[..end].matches('(').count();
        let shut = said[..end].matches(')').count();
        let stray = match last {
            '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' | '*' | '_' | '~' => true,
            ')' => shut > opened,
            _ => false,
        };
        if !stray {
            return end;
        }
        end -= last.len_utf8();
    }
}
