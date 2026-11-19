use std::collections::{HashMap, HashSet};

use super::dictionary::Dictionary;
use super::likeness::{Caps, Item, Outcome, Pair, Rule};
use super::spoken::ENGLISH;
use super::typos;

pub(crate) const WORDS: &str = "words";
pub(crate) const TYPO: &str = "typo";
pub(crate) const DICTIONARY: &str = "dictionary";
const SPOKEN: &str = "spoken:";
pub(crate) const SHOWN: &str = "method IN ('words', 'typo', 'dictionary')";

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Found {
    pub(crate) a: usize,
    pub(crate) b: usize,
    pub(crate) score: f32,
    pub(crate) shared: Vec<String>,
    pub(crate) method: &'static str,
}

#[derive(Debug, Default)]
pub(crate) struct Tracked {
    pub(crate) found: Vec<Found>,
    pub(crate) steps: u64,
    pub(crate) common_from: usize,
    pub(crate) typos: usize,
    pub(crate) translated: usize,
}

struct Room {
    held: Vec<usize>,
    taken: HashSet<(usize, usize)>,
}

// Shared words first, as they always were; a typo, then a translation, only fill the room they left.
pub(crate) fn alike(
    items: &[Item],
    guesses: &[Option<String>],
    tied: &HashSet<(usize, usize)>,
    caps: &Caps,
    dictionary: Option<&Dictionary>,
) -> Tracked {
    let words = super::likeness::alike(items, tied, caps, Rule::Words);
    let typos = super::likeness::alike(
        &retyped(items),
        tied,
        caps,
        Rule::Equivalent { loose: false },
    );
    let translated = dictionary.map(|dictionary| {
        super::likeness::alike(
            &translated(items, guesses, dictionary),
            tied,
            caps,
            Rule::Equivalent { loose: true },
        )
    });
    let mut tracked = Tracked {
        steps: words.steps + typos.steps,
        common_from: words.common_from,
        ..Tracked::default()
    };
    let mut room = Room {
        held: vec![0; items.len()],
        taken: HashSet::new(),
    };
    filled(&mut tracked, &mut room, words, WORDS, caps);
    tracked.typos = filled(&mut tracked, &mut room, typos, TYPO, caps);
    if let Some(translated) = translated {
        tracked.steps += translated.steps;
        tracked.translated = filled(&mut tracked, &mut room, translated, DICTIONARY, caps);
    }
    tracked
}

fn filled(
    tracked: &mut Tracked,
    room: &mut Room,
    outcome: Outcome,
    method: &'static str,
    caps: &Caps,
) -> usize {
    let mut pairs: Vec<Pair> = outcome.pairs;
    pairs.sort_by(|x, y| {
        y.score
            .total_cmp(&x.score)
            .then((x.a, x.b).cmp(&(y.a, y.b)))
    });
    let before = tracked.found.len();
    for pair in pairs {
        if room.taken.contains(&(pair.a, pair.b))
            || room.held[pair.a] >= caps.closest
            || room.held[pair.b] >= caps.closest
        {
            continue;
        }
        room.taken.insert((pair.a, pair.b));
        room.held[pair.a] += 1;
        room.held[pair.b] += 1;
        tracked.found.push(Found {
            a: pair.a,
            b: pair.b,
            score: pair.score,
            shared: pair.shared,
            method,
        });
    }
    tracked.found.len() - before
}

fn retyped(items: &[Item]) -> Vec<Item> {
    let mut df: HashMap<&str, u32> = HashMap::new();
    for item in items {
        for (key, _, _) in &item.words {
            *df.entry(key.as_str()).or_default() += 1;
        }
    }
    let mut vocabulary: Vec<(&str, u32)> = df.into_iter().collect();
    vocabulary.sort_unstable();
    let heads = typos::classes(&vocabulary);
    items
        .iter()
        .map(|item| keyed_again(item, |key| heads.get(key).map(String::as_str)))
        .collect()
}

// Within one language the pivot only joins synonyms, so two of the same language are kin here.
fn translated(items: &[Item], guesses: &[Option<String>], dictionary: &Dictionary) -> Vec<Item> {
    items
        .iter()
        .zip(guesses)
        .map(|(item, guess)| {
            let Some(guess) = guess else {
                return Item {
                    lang: item.lang.clone(),
                    kin: item.kin.clone(),
                    words: Vec::new(),
                };
            };
            let mut again = match guess.as_str() {
                ENGLISH => keyed_again(item, |_| None),
                _ => keyed_again(item, |key| dictionary.english_of(key)),
            };
            again.kin.push(format!("{SPOKEN}{guess}"));
            again
        })
        .collect()
}

fn keyed_again<'a>(item: &Item, key_of: impl Fn(&str) -> Option<&'a str>) -> Item {
    let mut at: HashMap<String, usize> = HashMap::new();
    let mut words: Vec<(String, String, u32)> = Vec::with_capacity(item.words.len());
    for (key, written, count) in &item.words {
        let key = key_of(key).map_or_else(|| key.clone(), str::to_string);
        match at.get(&key) {
            Some(&n) => words[n].2 += count,
            None => {
                at.insert(key.clone(), words.len());
                words.push((key, written.clone(), *count));
            }
        }
    }
    Item {
        lang: item.lang.clone(),
        kin: item.kin.clone(),
        words,
    }
}

#[cfg(test)]
#[path = "tracks_test.rs"]
mod tests;
