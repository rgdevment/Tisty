use std::ops::Range;

use unicode_normalization::UnicodeNormalization;

/// A hash pinned to a letter or a digit. `# Heading` carries a space and stays a heading, a colour
/// or a fragment sits behind something that is not a separator, and a fenced block is skipped
/// whole — the same fencing the title reader steps over.
/// Past this a body is not being tagged, it is carrying something else — a stylesheet pasted
/// outside a fence reads every colour as a tag. The log is append-only: a line of thousands of
/// them is written once and kept forever, on every machine.
pub const AT_MOST: usize = 64;

pub fn tags_in(body: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut fenced = crate::fences::fencing();
    for line in body.lines() {
        if fenced(line) {
            continue;
        }
        // Composed first: text pasted from macOS carries «ñ» as two code points, and a reader
        // that stops at the first mark would file #diseño under «disen».
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

/// Every tag on one line with the bytes it takes, `#` included, and the name it is filed under.
pub fn spots(line: &str) -> Vec<(Range<usize>, String)> {
    let mut found = Vec::new();
    let bytes = line.as_bytes();
    // Only a backtick that closes marks code. One on its own is prose — a stray accent in a
    // sentence must not swallow every tag written after it.
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
        // Anything but a separator before the hash means it belongs to what came first: a colour
        // in `bg-#fff`, the fragment of an address, a word someone hyphenated.
        if before.is_some_and(|one| one.is_alphanumeric() || "#/-_.:".contains(one)) {
            at += 1;
            continue;
        }
        let rest = &line[at + 1..];
        // A letter or a digit against the hash, or it is not a tag: `#-` and `#_` are how code
        // and web addresses are written, not how anybody labels their own work.
        if !rest.chars().next().is_some_and(char::is_alphanumeric) {
            at += 1;
            continue;
        }
        let word: String = rest
            .chars()
            .take_while(|one| one.is_alphanumeric() || *one == '-' || *one == '_')
            .collect();
        if let Some(tag) = named(&word).filter(|one| worth_reading(one)) {
            found.push((at..at + 1 + word.len(), tag));
        }
        at += 1 + word.len();
    }
    found
}

pub fn named(raw: &str) -> Option<String> {
    // Accents come off: somebody writes #camion on Monday and #camión on Friday, and one tag
    // is what they meant both times.
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
    name.chars().nth(1).is_some() && name.chars().any(char::is_alphabetic)
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
