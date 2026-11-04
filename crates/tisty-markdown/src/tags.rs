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
        let before = line[..at].chars().next_back();
        // Anything but a separator before the hash ties it to what came first, like bg-#fff or a URL.
        if before.is_some_and(|one| {
            one.is_alphanumeric()
                || unicode_normalization::char::is_combining_mark(one)
                || "#/-_.:".contains(one)
        }) {
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
                    || unicode_normalization::char::is_combining_mark(*one)
            })
            .collect();
        if let Some(tag) = named(&word).filter(|one| worth_reading(one)) {
            found.push((at..at + 1 + word.len(), tag));
        }
        at += 1 + word.len();
    }
    found
}

// The alphabets that write an accent over a letter that stands without it.
fn wears_accents(base: char) -> bool {
    base <= '\u{052F}' || ('\u{1E00}'..='\u{1FFF}').contains(&base)
}

pub fn named(raw: &str) -> Option<String> {
    use unicode_normalization::char::is_combining_mark;
    // Accents come off, so #camion on Monday and #camión on Friday are one tag. Elsewhere a mark
    // is how the word is written: without its vowels, हिन्दी is another word.
    let mut base = '-';
    let normalised: String = composed(raw)
        .trim()
        .to_lowercase()
        .nfd()
        .filter(|c| {
            if !is_combining_mark(*c) {
                base = *c;
            }
            !is_combining_mark(*c) || !wears_accents(base)
        })
        .map(|c| {
            if c.is_whitespace() || c == '_' {
                '-'
            } else {
                c
            }
        })
        .filter(|c| c.is_alphanumeric() || is_combining_mark(*c) || *c == '-')
        .nfc()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_come_off_the_alphabets_that_have_them() {
        for (said, tag) in [
            ("Camión", "camion"),
            ("camio\u{301}n", "camion"),
            ("Ελλάδα", "ελλαδα"),
            ("Йод", "иод"),
            ("Việt", "viet"),
        ] {
            assert_eq!(named(said).as_deref(), Some(tag), "{said}");
        }
    }

    #[test]
    fn elsewhere_a_mark_is_how_the_word_is_written() {
        for said in ["हिन्दी", "தமிழ்", "ไทย", "한글", "がくせい", "العربية"]
        {
            assert_eq!(named(said).as_deref(), Some(said), "{said}");
        }
        let apart: String = "한글".nfd().collect();
        assert_eq!(named(&apart).as_deref(), Some("한글"));
    }

    #[test]
    fn a_tag_read_again_is_the_same_tag() {
        for said in ["हिन्दी", "한글", "Camión_Rojo", "தமிழ் நாடு"]
        {
            let once = named(said).unwrap();
            assert_eq!(named(&once).as_deref(), Some(once.as_str()));
        }
    }

    #[test]
    fn a_sentence_reads_the_whole_word_and_nothing_glued_to_a_mark() {
        let line = "hoy #हिन्दी y #தமிழ் fin क्#no";
        let tags: Vec<String> = spots(line).into_iter().map(|(_, tag)| tag).collect();
        assert_eq!(tags, ["हिन्दी", "தமிழ்"]);
        let (at, _) = &spots(line)[0];
        assert_eq!(&line[at.clone()], "#हिन्दी");
    }
}
