use std::collections::{BTreeSet, HashMap};

pub(crate) const SHORTEST: usize = 6;

pub(crate) fn classes(vocabulary: &[(&str, u32)]) -> HashMap<String, String> {
    let mut by_variant: HashMap<String, Vec<usize>> = HashMap::new();
    for (at, (word, _)) in vocabulary.iter().enumerate() {
        if long_enough(word) {
            for variant in deletions(word) {
                by_variant.entry(variant).or_default().push(at);
            }
        }
    }
    let mut order: Vec<usize> = (0..vocabulary.len()).collect();
    order.sort_by(|a, b| {
        vocabulary[*b]
            .1
            .cmp(&vocabulary[*a].1)
            .then(vocabulary[*a].0.cmp(vocabulary[*b].0))
    });
    let mut taken = vec![false; vocabulary.len()];
    let mut heads = HashMap::new();
    for at in order {
        let (word, _) = vocabulary[at];
        if taken[at] || !long_enough(word) {
            continue;
        }
        taken[at] = true;
        let near: BTreeSet<usize> = deletions(word)
            .iter()
            .filter_map(|variant| by_variant.get(variant))
            .flatten()
            .copied()
            .filter(|other| *other != at && !taken[*other])
            .filter(|other| strsim::osa_distance(word, vocabulary[*other].0) <= 1)
            .collect();
        for other in near {
            taken[other] = true;
            heads.insert(vocabulary[other].0.to_string(), word.to_string());
        }
    }
    heads
}

// A digit tells two words apart on purpose: «capitulo1» is no typo of «capitulo2».
fn long_enough(word: &str) -> bool {
    word.chars().count() >= SHORTEST && !word.chars().any(|one| one.is_ascii_digit())
}

fn deletions(word: &str) -> Vec<String> {
    let letters: Vec<char> = word.chars().collect();
    let mut all = vec![word.to_string()];
    for skipped in 0..letters.len() {
        all.push(
            letters
                .iter()
                .enumerate()
                .filter(|(at, _)| *at != skipped)
                .map(|(_, one)| one)
                .collect(),
        );
    }
    all
}

#[cfg(test)]
#[path = "typos_test.rs"]
mod tests;
