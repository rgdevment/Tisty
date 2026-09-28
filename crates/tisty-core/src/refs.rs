#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Doc,
    Link,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Ref {
    pub kind: Kind,
    pub target: String,
    pub label: Option<String>,
}

pub const DOC: &str = "tisty:doc/";

pub fn card(file: &str, title: &str) -> String {
    let said: String = title
        .chars()
        .flat_map(|c| {
            let slash = matches!(c, '[' | ']' | '\\').then_some('\\');
            slash.into_iter().chain(std::iter::once(c))
        })
        .collect();
    format!("![{said}]({DOC}{file})")
}

pub fn papers(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (_, one) in papered(text) {
        if !found.contains(&one) {
            found.push(one);
        }
    }
    found
}

/// Every line a page is named on, read with the same walk that decides the reading order, so the
/// two can never disagree about what is code and what is a way in. A page named twice is here twice.
pub fn paper_lines(text: &str) -> Vec<(String, usize)> {
    let mut lines: Vec<usize> = vec![0];
    for (at, one) in text.char_indices() {
        if one == '\n' {
            lines.push(at + 1);
        }
    }
    let line_of = |at: usize| match lines.binary_search(&at) {
        Ok(one) => one,
        Err(one) => one - 1,
    };
    papered(text)
        .into_iter()
        .map(|(at, file)| (file, line_of(at)))
        .collect()
}

/// Where a page is read from is the card that stands for it, `![Title](tisty:doc/id)`, and only
/// that: a plain link to the same page is a mention in the middle of a sentence, and a sentence is
/// not a chapter. A page named inside a fenced block is an example of a way in, not one, so the
/// reading order steps over code too. What keeps a file alive is deliberately more generous than
/// either — see `extract`.
fn papered(text: &str) -> Vec<(usize, String)> {
    let code = crate::docs::fenced_spans(text);
    let bytes = text.as_bytes();
    marked_past(text, &code)
        .into_iter()
        .filter(|(at, _)| *at > 0 && bytes[at - 1] == b'!')
        .filter_map(|(at, one)| {
            one.target
                .strip_prefix(DOC)
                .map(|file| (at, file.to_string()))
        })
        .collect()
}

pub fn extract(text: &str) -> Vec<Ref> {
    let mut found: Vec<Ref> = Vec::new();
    for (_, one) in marked_past(text, &[]) {
        if !found
            .iter()
            .any(|held| held.target == one.target && held.kind == one.kind)
        {
            found.push(one);
        }
    }
    found
}

fn marked_past(text: &str, code: &[(usize, usize)]) -> Vec<(usize, Ref)> {
    let mut found: Vec<(usize, Ref)> = Vec::new();
    let where_at = std::cell::Cell::new(0usize);
    let mut keep = |one: Ref| found.push((where_at.get(), one));

    let mut past = 0;
    let bytes = text.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        while code.get(past).is_some_and(|(_, to)| *to <= at) {
            past += 1;
        }
        if let Some((from, to)) = code.get(past)
            && at >= *from
        {
            at = *to;
            continue;
        }
        let rest = &text[at..];
        where_at.set(at);
        at = match bytes[at] {
            b'`' => past_code(text, at),
            b'[' if rest.starts_with("[[") => named(rest, at, &mut keep),
            b'[' => linked(text, at, &mut keep),
            b'<' => tagged(rest, at, &mut keep),
            b'h' if rest.starts_with("http://") || rest.starts_with("https://") => {
                bare(rest, at, &mut keep)
            }
            _ => at + next_char(bytes, at),
        };
    }
    found
}

fn next_char(bytes: &[u8], at: usize) -> usize {
    let mut step = 1;
    while at + step < bytes.len() && bytes[at + step] & 0xC0 == 0x80 {
        step += 1;
    }
    step
}

fn past_code(text: &str, at: usize) -> usize {
    let fence = text[at..].bytes().take_while(|b| *b == b'`').count();
    let mut from = at + fence;
    while let Some(next) = text[from..].find('`') {
        let start = from + next;
        let run = text[start..].bytes().take_while(|b| *b == b'`').count();
        if run == fence {
            return start + run;
        }
        from = start + run;
    }
    at + fence
}

fn named(rest: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let Some(end) = rest.find("]]") else {
        return at + 2;
    };
    let name = rest[2..end].trim();
    if !name.is_empty() && !name.contains(['\n', '[']) {
        keep(Ref {
            kind: Kind::Doc,
            target: name.to_string(),
            label: None,
        });
    }
    at + end + 2
}

pub(crate) fn shuts(rest: &str) -> Option<usize> {
    let mut escaped = false;
    let mut any = None;
    for (at, c) in rest.char_indices() {
        if escaped {
            escaped = false;
            if c == ']' && any.is_none() {
                any = Some(at);
            }
            continue;
        }
        match c {
            '\\' => escaped = true,
            ']' => return Some(at),
            '[' | '\n' => return None,
            _ => {}
        }
    }
    any
}

fn linked(text: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let rest = &text[at + 1..];
    let Some(shut) = shuts(rest) else {
        return at + 1;
    };
    let label = &rest[..shut];

    let after = at + 1 + shut + 1;
    if text.as_bytes().get(after) != Some(&b'(') {
        return at + 1;
    }
    let tail = &text[after + 1..];

    let (target, close) = match tail.trim_start().strip_prefix('<') {
        Some(_) => {
            let opened = tail.find('<').unwrap_or(0);
            let Some(shut) = tail[opened..].find('>').map(|n| opened + n) else {
                return at + 1;
            };
            match tail[shut..].find(')') {
                Some(paren) => (&tail[opened + 1..shut], shut + paren),
                None => return at + 1,
            }
        }
        None => {
            let Some(close) = tail.find(')') else {
                return at + 1;
            };
            (tail[..close].split_whitespace().next().unwrap_or(""), close)
        }
    };
    if !target.is_empty() {
        let label = label.trim();
        keep(Ref {
            kind: Kind::Link,
            target: target.to_string(),
            label: (!label.is_empty()).then(|| label.to_string()),
        });
    }
    after + 1 + close + 1
}

/// An aligned paragraph is written as html, so what it points at is in an attribute, not in
/// brackets. Missed here, a file nothing else names reads as loose and is swept away.
fn tagged(rest: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let opens = rest[1..]
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '/');
    // A tag ends on its own line. Anything else — prose, a comparison, an autolink, a comment —
    // is read on, byte by byte, or one stray angle would swallow every reference after it.
    let shut = rest.find('\n').unwrap_or(rest.len());
    let Some(end) = opens.then(|| rest[..shut].find('>')).flatten() else {
        return at + 1;
    };
    let tag = &rest[..end];
    if !tag.contains('=') {
        return at + 1;
    }
    for name in [" href=\"", " src=\""] {
        let Some(from) = tag.find(name).map(|n| n + name.len()) else {
            continue;
        };
        let Some(stop) = tag[from..].find('"').map(|n| from + n) else {
            continue;
        };
        let target = tag[from..stop].trim();
        if !target.is_empty() {
            keep(Ref {
                kind: Kind::Link,
                target: target
                    .replace("&lt;", "<")
                    .replace("&gt;", ">")
                    .replace("&amp;", "&"),
                label: None,
            });
        }
    }
    at + end + 1
}

fn bare(rest: &str, at: usize, keep: &mut impl FnMut(Ref)) -> usize {
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '<' || c == '>')
        .unwrap_or(rest.len());
    let url = unpunctuated(&rest[..end]);

    if url.len() > url.find("//").map_or(0, |n| n + 2) {
        keep(Ref {
            kind: Kind::Link,
            target: url.to_string(),
            label: None,
        });
    }
    at + end
}

fn unpunctuated(url: &str) -> &str {
    let mut end = url.len();
    while let Some(last) = url[..end].chars().next_back() {
        let prose = match last {
            '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"' | ']' | '}' => true,
            ')' => url[..end].matches(')').count() > url[..end].matches('(').count(),
            _ => false,
        };
        if !prose {
            break;
        }
        end -= last.len_utf8();
    }
    &url[..end]
}

pub fn alerted(said: &str) -> Option<&str> {
    let rest = said.strip_prefix("[!")?;
    let (kind, after) = rest.split_once(']')?;
    let known = ["note", "tip", "important", "warning", "caution"]
        .iter()
        .any(|one| kind.eq_ignore_ascii_case(one));
    let alone = after.is_empty() || after.starts_with(char::is_whitespace);
    (known && alone).then(|| after.trim_start())
}

#[cfg(test)]
#[path = "refs_lines.rs"]
mod lines;

#[cfg(test)]
#[path = "refs_test.rs"]
mod tests;
