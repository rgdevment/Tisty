use std::collections::HashMap;

use bip39::{Language, Mnemonic};
use unicode_normalization::UnicodeNormalization;
use zeroize::{Zeroize, Zeroizing};

use crate::seed::Seed;

pub const WORDS: usize = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Words {
    English,
    Spanish,
}

/// Positions count from 1, the way a person reads them off the paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Unread {
    #[error("the phrase has {0} words, and it takes {WORDS}")]
    Count(usize),
    #[error("word {0} is not one of the list")]
    Word(usize),
    #[error("every word is on the list, but one of them is wrong or out of place")]
    Checksum,
}

/// The whole seed, written as words: whoever holds them is that person.
pub fn phrase(seed: &Seed, words: Words) -> Option<Zeroizing<String>> {
    let seed = seed.to_bytes();
    let mnemonic = Mnemonic::from_entropy_in(language(words), seed.as_ref()).ok()?;
    let mut joined = Zeroizing::new(String::with_capacity(WORDS * 16));
    for (at, word) in mnemonic.words().enumerate() {
        if at > 0 {
            joined.push(' ');
        }
        joined.push_str(word);
    }
    Some(joined)
}

/// Either language, in any case, with or without its accents: people copy words by hand.
pub fn seed_from(phrase: &str) -> Result<Seed, Unread> {
    let said: Vec<Zeroizing<String>> = phrase
        .split_whitespace()
        .map(|one| Zeroizing::new(lowered(one)))
        .collect();
    if said.len() != WORDS {
        return Err(Unread::Count(said.len()));
    }
    let mut furthest = 0;
    let mut unread = Unread::Word(1);
    // Spelled as the list spells them first: twelve English words are Spanish ones without accents.
    for loose in [false, true] {
        for language in [Language::Spanish, Language::English] {
            match listed(language, &said, loose) {
                Ok(canonical) => match Mnemonic::parse_in(language, canonical.as_str()) {
                    Ok(mnemonic) => return seeded(&mnemonic),
                    Err(_) => {
                        furthest = WORDS;
                        unread = Unread::Checksum;
                    }
                },
                Err(at) if at > furthest => {
                    furthest = at;
                    unread = Unread::Word(at + 1);
                }
                Err(_) => {}
            }
        }
    }
    Err(unread)
}

fn language(words: Words) -> Language {
    match words {
        Words::English => Language::English,
        Words::Spanish => Language::Spanish,
    }
}

/// The list's own spelling of every word said, or where the first unknown one sits.
fn listed(
    language: Language,
    said: &[Zeroizing<String>],
    loose: bool,
) -> Result<Zeroizing<String>, usize> {
    let list = language.word_list();
    let bare = loose.then(|| unaccented(language));
    // Sized once, so growing never leaves a freed copy of the phrase behind.
    let mut canonical = Zeroizing::new(String::with_capacity(WORDS * 16));
    for (at, word) in said.iter().enumerate() {
        let found = match language.find_word(word) {
            Some(found) => Some(found),
            None => bare
                .as_ref()
                .and_then(|bare| bare.get(Zeroizing::new(unmarked(word)).as_str()))
                .copied(),
        };
        let found = found.ok_or(at)?;
        if at > 0 {
            canonical.push(' ');
        }
        canonical.push_str(list[usize::from(found)]);
    }
    Ok(canonical)
}

/// Where each word sits once its accents are gone; built only when the words as spelled did not do.
fn unaccented(language: Language) -> HashMap<String, u16> {
    (0u16..)
        .zip(language.word_list())
        .map(|(at, one)| (unmarked(one), at))
        .collect()
}

fn seeded(mnemonic: &Mnemonic) -> Result<Seed, Unread> {
    let (mut entropy, length) = mnemonic.to_entropy_array();
    let mut seed = Zeroizing::new([0u8; 32]);
    let whole = length == seed.len();
    if let Some(bytes) = entropy.get(..seed.len()).filter(|_| whole) {
        seed.copy_from_slice(bytes);
    }
    entropy.zeroize();
    if !whole {
        return Err(Unread::Checksum);
    }
    Ok(Seed::from_bytes(&seed))
}

/// Measured first, so a word typed from the phrase is never left behind in a buffer that grew.
fn lowered(word: &str) -> String {
    let wide = || word.nfkd().flat_map(char::to_lowercase);
    let mut out = String::with_capacity(wide().map(char::len_utf8).sum());
    out.extend(wide());
    out
}

fn unmarked(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    out.extend(
        word.chars()
            .filter(|one| !unicode_normalization::char::is_combining_mark(*one)),
    );
    out
}

#[cfg(test)]
#[path = "phrase_test.rs"]
mod tests;
