use bip39::{Language, Mnemonic};
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroizing;

use crate::identity::Root;

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

/// The root's whole secret, written as words: whoever holds them is that person.
pub fn phrase(root: &Root, words: Words) -> Zeroizing<String> {
    let seed = root.to_bytes();
    let joined = Mnemonic::from_entropy_in(language(words), seed.as_ref())
        .map(|mnemonic| mnemonic.words().collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    Zeroizing::new(joined)
}

/// Either language, in any case, with or without its accents: people copy words by hand.
pub fn root_from(phrase: &str) -> Result<Root, Unread> {
    let said: Vec<String> = phrase.split_whitespace().map(lowered).collect();
    if said.len() != WORDS {
        return Err(Unread::Count(said.len()));
    }
    let mut furthest = 0;
    let mut unread = Unread::Word(1);
    for language in [Language::Spanish, Language::English] {
        match listed(language, &said) {
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
    Err(unread)
}

fn language(words: Words) -> Language {
    match words {
        Words::English => Language::English,
        Words::Spanish => Language::Spanish,
    }
}

/// The list's own spelling of every word said, or where the first unknown one sits.
fn listed(language: Language, said: &[String]) -> Result<Zeroizing<String>, usize> {
    let list = language.word_list();
    let exact: Vec<String> = list.iter().map(|one| lowered(one)).collect();
    let bare: Vec<String> = exact.iter().map(|one| unmarked(one)).collect();
    let mut canonical = Zeroizing::new(String::new());
    for (at, word) in said.iter().enumerate() {
        let found = exact
            .iter()
            .position(|one| one == word)
            .or_else(|| bare.iter().position(|one| *one == unmarked(word)))
            .ok_or(at)?;
        if at > 0 {
            canonical.push(' ');
        }
        canonical.push_str(list[found]);
    }
    Ok(canonical)
}

fn seeded(mnemonic: &Mnemonic) -> Result<Root, Unread> {
    let (entropy, length) = mnemonic.to_entropy_array();
    let entropy = Zeroizing::new(entropy);
    let seed: Zeroizing<[u8; 32]> = Zeroizing::new(
        entropy
            .get(..length)
            .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
            .ok_or(Unread::Count(mnemonic.word_count()))?,
    );
    Ok(Root::from_bytes(&seed))
}

fn lowered(word: &str) -> String {
    word.nfkd().flat_map(char::to_lowercase).collect()
}

fn unmarked(word: &str) -> String {
    word.chars()
        .filter(|one| !unicode_normalization::char::is_combining_mark(*one))
        .collect()
}

#[cfg(test)]
#[path = "phrase_test.rs"]
mod tests;
