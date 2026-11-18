use std::ops::Range;

use crate::refs::{bare, linked, next_char, past_code, tagged, wiki};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Named {
    Id(String),
    Quoted(String),
    Wiki(String),
}

// `at` is what a link would take the place of: the id, the words between the quotes, the whole `[[…]]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spot {
    pub named: Named,
    pub at: Range<usize>,
}

const QUOTED_AT_MOST: usize = 200;
const QUOTES: [(char, char); 3] = [('«', '»'), ('“', '”'), ('"', '"')];

pub fn spots(text: &str) -> Vec<Spot> {
    let mut found = Vec::new();
    let mut skipped = crate::fences::fenced_spans(text);
    if let Some(front) = crate::front::span(text) {
        skipped.insert(0, (front.start, front.end));
    }
    let bytes = text.as_bytes();
    let mut past = 0;
    let mut at = 0;
    while at < bytes.len() {
        while skipped.get(past).is_some_and(|(_, to)| *to <= at) {
            past += 1;
        }
        if let Some((from, to)) = skipped.get(past)
            && at >= *from
        {
            at = *to;
            continue;
        }
        let rest = &text[at..];
        at = match bytes[at] {
            b'`' => past_code(text, at),
            b'[' if rest.starts_with("[[") => match wiki(rest) {
                Some((end, Some(name))) => {
                    found.push(Spot {
                        named: Named::Wiki(name.to_string()),
                        at: at..at + end + 2,
                    });
                    at + end + 2
                }
                Some((end, None)) => at + end + 2,
                None => at + 2,
            },
            b'[' => linked(text, at, &mut |_| {}),
            b'<' => tagged(rest, at, &mut |_| {}),
            b'h' if rest.starts_with("http://") || rest.starts_with("https://") => {
                bare(rest, at, &mut |_| {})
            }
            _ => match quoted(text, at).or_else(|| id(text, at)) {
                Some((spot, end)) => {
                    found.push(spot);
                    end
                }
                None => at + next_char(bytes, at),
            },
        };
    }
    found
}

fn quoted(text: &str, at: usize) -> Option<(Spot, usize)> {
    let open = text[at..].chars().next()?;
    let (_, close) = QUOTES.iter().find(|(one, _)| *one == open)?;
    let from = at + open.len_utf8();
    if open == '"'
        && text[..at]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric)
    {
        return None;
    }
    let line = text[from..].find('\n').map_or(text.len(), |n| from + n);
    let to = from + text[from..line].find(*close)?;
    let said = &text[from..to];
    let inner = said.trim();
    let plain = !inner.is_empty()
        && inner.chars().count() <= QUOTED_AT_MOST
        && !said.contains(['[', ']', '`', '«', '»', '“', '”', '"'])
        && !a_task(inner)
        && !a_document(inner);
    let after = to + close.len_utf8();
    if !plain
        || (open == '"'
            && text[after..]
                .chars()
                .next()
                .is_some_and(char::is_alphanumeric))
    {
        return None;
    }
    let lead = said.len() - said.trim_start().len();
    let spot = Spot {
        named: Named::Quoted(inner.to_string()),
        at: from + lead..from + lead + inner.len(),
    };
    Some((spot, after))
}

fn id(text: &str, at: usize) -> Option<(Spot, usize)> {
    let glued = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-');
    let before = text[..at].chars().next_back();
    if before.is_some_and(|c| glued(c) || matches!(c, '/' | '.' | ':' | '#' | '@' | '\\')) {
        return None;
    }
    let len = text[at..]
        .find(|c: char| !glued(c))
        .unwrap_or(text.len() - at);
    let token = &text[at..at + len];
    let end = at + len;
    let mut after = text[end..].chars();
    let filed = matches!(after.next(), Some('.' | '/' | '\\'))
        && after.next().is_some_and(|c| c.is_ascii_alphanumeric());
    if filed || !(a_task(token) || a_document(token)) {
        return None;
    }
    let spot = Spot {
        named: Named::Id(token.to_string()),
        at: at..end,
    };
    Some((spot, end))
}

fn a_task(token: &str) -> bool {
    token.len() == 26
        && token.starts_with(|c: char| ('0'..='7').contains(&c))
        && token.chars().all(|c| {
            c.is_ascii_digit() || c.is_ascii_uppercase() && !matches!(c, 'I' | 'L' | 'O' | 'U')
        })
}

fn a_document(token: &str) -> bool {
    let Some((stem, number)) = token.rsplit_once('-') else {
        return false;
    };
    (1..=64).contains(&stem.len())
        && stem
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        && (4..=12).contains(&number.len())
        && number.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
#[path = "named_test.rs"]
mod tests;
