use std::ops::Range;

use crate::{Mark, Span};

pub fn fold(spans: &mut Vec<Span>, text: &str) {
    doubled(spans, text);
}

fn doubled(spans: &mut Vec<Span>, text: &str) {
    let walls = walls(spans, text);
    let mut at = 0;
    while at + 1 < walls.len() {
        let ((open, held), (shut, kept)) = (walls[at].clone(), walls[at + 1].clone());
        let inside = &text[open + 2..shut];
        // Both walls inside the same mark, or the pen would cut across a word in bold or a link.
        if held != kept
            || inside.is_empty()
            || inside.starts_with(char::is_whitespace)
            || inside.ends_with(char::is_whitespace)
        {
            at += 1;
            continue;
        }
        let (color, worn) = suffix(&text[shut + 2..same(spans, shut)]);
        spans.push(Span {
            mark: Mark::Pen { color },
            at: open..shut + 2 + worn,
        });
        at += 2;
    }
}

// Only a pair of `=` on its own counts: `===` and longer are something else being written.
fn walls(spans: &[Span], text: &str) -> Vec<(usize, Option<Range<usize>>)> {
    let mut found = Vec::new();
    for span in spans.iter().filter(|one| one.mark == Mark::Text) {
        let bytes = text[span.at.clone()].as_bytes();
        let mut at = 0;
        while at + 1 < bytes.len() {
            let run = bytes[at..].iter().take_while(|b| **b == b'=').count();
            if run == 2 {
                let wall = span.at.start + at;
                found.push((wall, crate::inline::holder(spans, wall)));
            }
            at += run.max(1);
        }
    }
    found
}

// A colour written past the end of the words, across a link or a bold, belongs to neither.
fn same(spans: &[Span], wall: usize) -> usize {
    spans
        .iter()
        .filter(|one| one.mark == Mark::Text && one.at.start <= wall && wall < one.at.end)
        .map(|one| one.at.end)
        .next()
        .unwrap_or(wall + 2)
}

fn suffix(after: &str) -> (Option<String>, usize) {
    let named = after
        .strip_prefix('{')
        .and_then(|rest| rest.split_once('}'))
        .map(|(name, _)| name)
        .filter(|name| crate::vocab::pen(name));
    match named {
        Some(name) if name == crate::vocab::YELLOW => (None, name.len() + 2),
        Some(name) => (Some(name.to_string()), name.len() + 2),
        None => (None, 0),
    }
}
