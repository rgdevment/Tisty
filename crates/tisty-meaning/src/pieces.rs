use std::collections::HashMap;

use regex_syntax::hir::{Class, HirKind};
use unicode_normalization::UnicodeNormalization;

const UNKNOWN: &str = "[UNK]";
const GOES_ON: &str = "##";
const LONGEST_WORD: usize = 100;
const SPECIAL: [&str; 5] = ["[PAD]", "[UNK]", "[CLS]", "[SEP]", "[MASK]"];

// The tables the model was trained with: BERT's normalizer, its word splitting and WordPiece.
pub(crate) struct Pieces {
    ids: HashMap<String, u32>,
    unknown: u32,
    control: Ranges,
    nonspacing: Ranges,
    punctuation: Ranges,
}

struct Ranges(Vec<(char, char)>);

impl Ranges {
    fn of(pattern: &str) -> Option<Self> {
        let hir = regex_syntax::Parser::new().parse(pattern).ok()?;
        let HirKind::Class(Class::Unicode(class)) = hir.kind() else {
            return None;
        };
        Some(Self(
            class
                .ranges()
                .iter()
                .map(|range| (range.start(), range.end()))
                .collect(),
        ))
    }

    fn holds(&self, one: char) -> bool {
        self.0
            .binary_search_by(|(start, end)| {
                if *end < one {
                    std::cmp::Ordering::Less
                } else if *start > one {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Equal
                }
            })
            .is_ok()
    }
}

impl Pieces {
    pub(crate) fn read(vocab: &str) -> Option<Self> {
        let ids: HashMap<String, u32> = vocab
            .strip_suffix('\n')
            .unwrap_or(vocab)
            .split('\n')
            .enumerate()
            .map(|(id, piece)| Some((piece.to_string(), u32::try_from(id).ok()?)))
            .collect::<Option<_>>()?;
        Some(Self {
            unknown: *ids.get(UNKNOWN)?,
            ids,
            control: Ranges::of(r"[\p{Cc}\p{Cf}\p{Co}]")?,
            nonspacing: Ranges::of(r"\p{Mn}")?,
            punctuation: Ranges::of(r"\p{P}")?,
        })
    }

    pub(crate) fn len(&self) -> usize {
        self.ids.len()
    }

    // A special piece written in the text is taken as it stands, before anything is normalized.
    pub(crate) fn of(&self, text: &str) -> Vec<u32> {
        let mut ids = Vec::new();
        let mut rest = text;
        while !rest.is_empty() {
            let special = SPECIAL
                .iter()
                .filter_map(|one| Some((rest.find(one)?, *one)))
                .min_by_key(|(at, _)| *at);
            let (said, after) = match special {
                Some((at, one)) => (&rest[..at], Some((one, &rest[at + one.len()..]))),
                None => (rest, None),
            };
            let normal = self.normalized(said);
            for word in self.words(&normal) {
                self.pieced(word, &mut ids);
            }
            let Some((one, after)) = after else {
                break;
            };
            ids.extend(self.ids.get(one));
            rest = after;
        }
        ids
    }

    fn normalized(&self, text: &str) -> String {
        let mut clean = String::with_capacity(text.len());
        for one in text.chars() {
            if one == '\0' || one == '\u{fffd}' || self.is_control(one) {
                continue;
            }
            if is_whitespace(one) {
                clean.push(' ');
            } else if is_chinese(one) {
                clean.extend([' ', one, ' ']);
            } else {
                clean.push(one);
            }
        }
        clean
            .nfd()
            .filter(|one| !self.nonspacing.holds(*one))
            .flat_map(char::to_lowercase)
            .collect()
    }

    fn is_control(&self, one: char) -> bool {
        !matches!(one, '\t' | '\n' | '\r') && self.control.holds(one)
    }

    fn words<'a>(&'a self, text: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        text.split(char::is_whitespace)
            .filter(|word| !word.is_empty())
            .flat_map(move |word| {
                let mut parts = Vec::new();
                let mut from = 0;
                for (at, one) in word.char_indices() {
                    if one.is_ascii_punctuation() || self.punctuation.holds(one) {
                        if from < at {
                            parts.push(&word[from..at]);
                        }
                        parts.push(&word[at..at + one.len_utf8()]);
                        from = at + one.len_utf8();
                    }
                }
                if from < word.len() {
                    parts.push(&word[from..]);
                }
                parts
            })
    }

    // Longest piece first; a word that cannot be pieced whole is one unknown, never a part of one.
    fn pieced(&self, word: &str, ids: &mut Vec<u32>) {
        if word.chars().count() > LONGEST_WORD {
            ids.push(self.unknown);
            return;
        }
        let first = ids.len();
        let mut start = 0;
        let mut sought = String::new();
        while start < word.len() {
            let mut end = word.len();
            let found = loop {
                if start >= end {
                    break None;
                }
                sought.clear();
                if start > 0 {
                    sought.push_str(GOES_ON);
                }
                sought.push_str(&word[start..end]);
                if let Some(id) = self.ids.get(&sought) {
                    break Some(*id);
                }
                end -= word[start..end].chars().last().map_or(1, char::len_utf8);
            };
            let Some(id) = found else {
                ids.truncate(first);
                ids.push(self.unknown);
                return;
            };
            ids.push(id);
            start = end;
        }
    }
}

fn is_whitespace(one: char) -> bool {
    matches!(one, '\t' | '\n' | '\r') || one.is_whitespace()
}

fn is_chinese(one: char) -> bool {
    matches!(
        u32::from(one),
        0x4E00..=0x9FFF
            | 0x3400..=0x4DBF
            | 0x20000..=0x2A6DF
            | 0x2A700..=0x2B73F
            | 0x2B740..=0x2B81F
            | 0x2B920..=0x2CEAF
            | 0xF900..=0xFAFF
            | 0x2F800..=0x2FA1F
    )
}

#[cfg(test)]
#[path = "pieces_test.rs"]
mod tests;
