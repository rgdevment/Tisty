use std::ops::Range;

use unicode_normalization::UnicodeNormalization;

// Past this a body is carrying something else, like a stylesheet whose colours all read as tags.
pub const AT_MOST: usize = 64;

pub fn tags_in(body: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for tag in crate::front::tags(body).iter().filter_map(|one| named(one)) {
        if seen.insert(tag.clone()) {
            found.push(tag);
        }
        if found.len() == AT_MOST {
            return found;
        }
    }
    let past = crate::front::span(body).map_or(0, |at| at.end);
    let mut fenced = crate::fences::fencing();
    for line in body[past..].lines() {
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

/// Every tag on one line, with the bytes it takes and the name it is filed under.
pub fn spots(line: &str) -> Vec<(Range<usize>, String)> {
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
        // Anything but a separator before the hash ties it to what came first, like bg-#fff or a URL.
        // A mark sits on what it follows, so it ties only what that does: after an emoji it is no word.
        let before = line[..at]
            .chars()
            .rev()
            .find(|one| !unicode_normalization::char::is_combining_mark(*one));
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
        let word = word_in(rest);
        if let Some(tag) = named(word).filter(|one| worth_reading(one)) {
            found.push((at..at + 1 + word.len(), tag));
        }
        at += 1 + word.len();
    }
    found
}

// A joiner is how many scripts write inside a word, so it goes on only where the word does.
fn word_in(rest: &str) -> &str {
    let mut end = 0;
    let mut chars = rest.char_indices().peekable();
    while let Some((at, one)) = chars.next() {
        let joins = matches!(one, '\u{200C}' | '\u{200D}')
            && chars.peek().is_some_and(|(_, next)| next.is_alphanumeric());
        let holds = one.is_alphanumeric()
            || one == '-'
            || one == '_'
            || unicode_normalization::char::is_combining_mark(one);
        if !holds && !joins {
            break;
        }
        end = at + one.len_utf8();
    }
    &rest[..end]
}

// Where a mark is an accent or a vowel a reader can do without, over a letter that stands without it.
fn wears_accents(base: char) -> bool {
    base <= '\u{06FF}'
        || ('\u{0750}'..='\u{077F}').contains(&base)
        || ('\u{08A0}'..='\u{08FF}').contains(&base)
        || ('\u{1E00}'..='\u{1FFF}').contains(&base)
}

// It only asks for another drawing of what comes before it, so two tags would look the same and be two.
fn picks_a_drawing(one: char) -> bool {
    matches!(one, '\u{180B}'..='\u{180D}' | '\u{FE00}'..='\u{FE0F}' | '\u{E0100}'..='\u{E01EF}')
}

pub fn named(raw: &str) -> Option<String> {
    use unicode_normalization::char::is_combining_mark;
    // Accents come off, so #camion on Monday and #camión on Friday are one tag. Elsewhere a mark
    // is how the word is written: without its vowels, हिन्दी is another word.
    let mut kept = String::with_capacity(raw.len());
    let mut base = '-';
    for one in raw.trim().to_lowercase().nfd() {
        if is_combining_mark(one) {
            if !picks_a_drawing(one) && !wears_accents(base) {
                kept.push(one);
            }
            continue;
        }
        let one = if one.is_whitespace() || one == '_' {
            '-'
        } else {
            one
        };
        // A mark on what goes goes with it, or it would land on the letter before.
        base = if one.is_alphanumeric() || one == '-' {
            one
        } else {
            '-'
        };
        if one.is_alphanumeric() || one == '-' {
            kept.push(one);
        }
    }
    let normalised: String = kept.nfc().collect();

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

#[cfg(test)]
#[path = "tags_test.rs"]
mod tests;
