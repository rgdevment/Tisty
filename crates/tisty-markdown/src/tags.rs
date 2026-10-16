use std::ops::Range;

use unicode_normalization::UnicodeNormalization;

// Past this a body is carrying something else, like a stylesheet whose colours all read as tags.
pub const AT_MOST: usize = 64;

pub fn tags_in(body: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut fenced = crate::fences::fencing();
    for line in body.lines() {
        if fenced(line) {
            continue;
        }
        // Composed first, or «ñ» pasted from macOS as two code points files #diseño under «disen».
        for (_, tag) in spots(&composed(line)) {
            if seen.insert(tag.clone()) {
                found.push(tag);
            }
            if found.len() == AT_MOST {
                return found;
            }
        }
    }
    found
}

/// Every tag on one composed line, with the bytes it takes and the name it is filed under.
pub fn spots(line: &str) -> Vec<(Range<usize>, String)> {
    scan(line, false)
}

/// The same tags on a line as it was written, where an accent may still sit apart from its letter.
pub fn spots_as_written(line: &str) -> Vec<(Range<usize>, String)> {
    scan(line, true)
}

fn scan(line: &str, apart: bool) -> Vec<(Range<usize>, String)> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    // A lone backtick is prose: only one that closes marks code.
    let mut code = false;
    let mut at = 0;
    while at < line.len() {
        if bytes[at] == b'`' {
            code = !code && line[at + 1..].contains('`');
            at += 1;
            continue;
        }
        if bytes[at] != b'#' || code {
            at += 1;
            continue;
        }
        let before = line[..at].chars().next_back();
        // Anything but a separator before the hash ties it to what came first, like bg-#fff or a URL.
        if before.is_some_and(|one| one.is_alphanumeric() || "#/-_.:".contains(one)) {
            at += 1;
            continue;
        }
        let rest = &line[at + 1..];
        // `#-` and `#_` are how code and addresses are written, never how anybody labels work.
        if !rest.chars().next().is_some_and(char::is_alphanumeric) {
            at += 1;
            continue;
        }
        let word: String = rest
            .chars()
            .take_while(|one| {
                one.is_alphanumeric()
                    || *one == '-'
                    || *one == '_'
                    || (apart && unicode_normalization::char::is_combining_mark(*one))
            })
            .collect();
        if let Some(tag) = named(&word).filter(|one| worth_reading(one)) {
            found.push((at..at + 1 + word.len(), tag));
        }
        at += 1 + word.len();
    }
    found
}

pub fn named(raw: &str) -> Option<String> {
    // Accents come off, so #camion on Monday and #camión on Friday are one tag.
    let normalised: String = composed(raw)
        .trim()
        .to_lowercase()
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .map(|c| {
            if c.is_whitespace() || c == '_' {
                '-'
            } else {
                c
            }
        })
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect();

    let trimmed = normalised.trim_matches('-');
    if !trimmed.chars().any(char::is_alphanumeric) {
        return None;
    }
    Some(collapse_dashes(trimmed))
}

/// What a reader may take from writing nobody meant as a label: «#1234» is a ticket, not a tag.
pub fn worth_reading(name: &str) -> bool {
    name.chars().nth(1).is_some() && name.chars().any(char::is_alphabetic) && !coloured(name)
}

/// «#ffcc00» and «#fff» name a colour written in prose; «#bad» and «#dad» are still words.
fn coloured(name: &str) -> bool {
    let hex = matches!(name.len(), 3 | 6 | 8) && name.chars().all(|one| one.is_ascii_hexdigit());
    let first = name.chars().next();
    hex && (name.chars().any(|one| one.is_ascii_digit())
        || name.chars().all(|one| Some(one) == first))
}

pub fn composed(text: &str) -> String {
    if text.is_ascii() {
        return text.to_string();
    }
    text.nfc().collect()
}

fn collapse_dashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut previous_dash = false;
    for c in s.chars() {
        if c == '-' {
            if !previous_dash {
                out.push(c);
            }
            previous_dash = true;
        } else {
            out.push(c);
            previous_dash = false;
        }
    }
    out
}
