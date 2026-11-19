use std::collections::{HashMap, HashSet};

pub(crate) struct Caps {
    pub(crate) weighed: usize,
    pub(crate) closest: usize,
    pub(crate) abroad: usize,
    pub(crate) shared: usize,
    pub(crate) told: usize,
    pub(crate) common: usize,
    pub(crate) common_at_least: usize,
    pub(crate) steps: u64,
    pub(crate) chance: f64,
}

pub(crate) const CAPS: Caps = Caps {
    weighed: 48,
    closest: 3,
    abroad: 1,
    shared: 2,
    told: 4,
    common: 20,
    common_at_least: 3,
    steps: 20_000_000,
    chance: 0.001,
};

pub(crate) struct Item {
    pub(crate) lang: Option<String>,
    pub(crate) kin: Vec<String>,
    pub(crate) words: Vec<(String, String, u32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Pair {
    pub(crate) a: usize,
    pub(crate) b: usize,
    pub(crate) score: f32,
    pub(crate) shared: Vec<String>,
}

#[derive(Debug, Default)]
pub(crate) struct Outcome {
    pub(crate) pairs: Vec<Pair>,
    pub(crate) steps: u64,
    pub(crate) common_from: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tongue {
    Same,
    Other,
}

type Vector = Vec<(u32, f32)>;

struct Base<'a> {
    told: Vec<&'a str>,
    weighty: Vec<bool>,
    vectors: Vec<Vector>,
}

pub(crate) fn alike(items: &[Item], tied: &HashSet<(usize, usize)>, caps: &Caps) -> Outcome {
    let (base, common_from) = weighed(items, caps);
    let mut postings: Vec<Vec<(usize, f32)>> = vec![Vec::new(); base.told.len()];
    for (at, vector) in base.vectors.iter().enumerate() {
        for &(term, weight) in vector {
            if base.weighty[term as usize] {
                postings[term as usize].push((at, weight));
            }
        }
    }
    let apart = |a: usize, b: usize| {
        tied.contains(&(a.min(b), a.max(b)))
            || items[a].kin.iter().any(|one| items[b].kin.contains(one))
    };
    let mut steps = 0;
    let mut spread = [Spread::default(), Spread::default()];
    let mut ranked: Vec<[Vec<(usize, f32)>; 2]> = Vec::with_capacity(items.len());
    let mut score = vec![0f32; items.len()];
    let mut both = vec![0u32; items.len()];
    let mut touched = Vec::new();
    for (at, vector) in base.vectors.iter().enumerate() {
        for &(term, weight) in vector {
            for &(other, theirs) in &postings[term as usize] {
                if other == at {
                    continue;
                }
                if both[other] == 0 {
                    touched.push(other);
                }
                score[other] += weight * theirs;
                both[other] += 1;
                steps += 1;
            }
        }
        let mut near: [Vec<(usize, f32)>; 2] = [Vec::new(), Vec::new()];
        for other in touched.drain(..) {
            let (said, shared) = (score[other], both[other]);
            score[other] = 0.0;
            both[other] = 0;
            let tongue = tongue_of(&items[at], &items[other]);
            spread[tongue as usize].add(said);
            if shared as usize >= caps.shared && !apart(at, other) {
                near[tongue as usize].push((other, said));
            }
        }
        for (tongue, room) in [(Tongue::Same, caps.closest), (Tongue::Other, caps.abroad)] {
            let list = &mut near[tongue as usize];
            list.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            list.truncate(room);
        }
        ranked.push(near);
    }
    let floors = floors(items, &spread, caps);
    let mut found: Vec<(Tongue, usize, usize, f32)> = Vec::new();
    for (at, near) in ranked.iter().enumerate() {
        for tongue in [Tongue::Same, Tongue::Other] {
            for &(other, said) in &near[tongue as usize] {
                let mutual = ranked[other][tongue as usize]
                    .iter()
                    .any(|(back, _)| *back == at);
                if at < other && mutual && said >= floors[tongue as usize] {
                    found.push((tongue, at, other, said));
                }
            }
        }
    }
    // Another language scores lower by nature, so its own slot is filled before the rest.
    found.sort_by(|x, y| {
        (y.0 as u8)
            .cmp(&(x.0 as u8))
            .then(y.3.total_cmp(&x.3))
            .then((x.1, x.2).cmp(&(y.1, y.2)))
    });
    let mut held = vec![0usize; items.len()];
    let mut pairs = Vec::new();
    for (_, a, b, said) in found {
        if held[a] >= caps.closest || held[b] >= caps.closest {
            continue;
        }
        held[a] += 1;
        held[b] += 1;
        pairs.push(Pair {
            a,
            b,
            score: said,
            shared: shared_words(&base, a, b, caps.told),
        });
    }
    Outcome {
        pairs,
        steps,
        common_from,
    }
}

fn tongue_of(one: &Item, other: &Item) -> Tongue {
    match (&one.lang, &other.lang) {
        (Some(mine), Some(theirs)) if mine != theirs => Tongue::Other,
        _ => Tongue::Same,
    }
}

fn weighed<'a>(items: &'a [Item], caps: &Caps) -> (Base<'a>, usize) {
    let mut ids: HashMap<&str, u32> = HashMap::new();
    let mut told: Vec<&str> = Vec::new();
    let mut df: Vec<u32> = Vec::new();
    let mut counts: Vec<Vec<(u32, u32)>> = Vec::with_capacity(items.len());
    for item in items {
        let here: Vec<(u32, u32)> = item
            .words
            .iter()
            .map(|(key, written, count)| {
                let id = *ids.entry(key.as_str()).or_insert_with(|| {
                    told.push(written.as_str());
                    df.push(0);
                    (df.len() - 1) as u32
                });
                (id, *count)
            })
            .collect();
        for (id, _) in &here {
            df[*id as usize] += 1;
        }
        counts.push(here);
    }
    let common_from = common_from(&df, items.len(), caps);
    let weighty = df.iter().map(|one| (*one as usize) < common_from).collect();
    let n = items.len() as f32;
    let vectors = counts
        .iter()
        .map(|here| {
            let mut vector: Vector = here
                .iter()
                .map(|(id, count)| {
                    let idf = ((n + 1.0) / (df[*id as usize] as f32 + 1.0)).ln();
                    (*id, (1.0 + (*count as f32).ln()) * idf)
                })
                .filter(|(_, weight)| *weight > 0.0)
                .collect();
            vector.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            vector.truncate(caps.weighed);
            let norm = vector.iter().map(|(_, w)| w * w).sum::<f32>().sqrt();
            for one in &mut vector {
                one.1 /= norm;
            }
            vector
        })
        .collect();
    (
        Base {
            told,
            weighty,
            vectors,
        },
        common_from,
    )
}

// The budget bounds every pair the pass compares: Σ df² over the words it weighs.
fn common_from(df: &[u32], n: usize, caps: &Caps) -> usize {
    let ceiling = (n / caps.common).max(caps.common_at_least);
    let mut by_df = vec![0u64; ceiling + 1];
    for one in df {
        if let Some(slot) = by_df.get_mut(*one as usize) {
            *slot += 1;
        }
    }
    let mut spent = 0u64;
    for (at, terms) in by_df.iter().enumerate().skip(2) {
        spent += terms * (at as u64).pow(2);
        if spent > caps.steps {
            return at;
        }
    }
    ceiling + 1
}

const BINS: usize = 1024;

struct Spread {
    bins: Vec<u64>,
}

impl Default for Spread {
    fn default() -> Self {
        Self {
            bins: vec![0; BINS],
        }
    }
}

impl Spread {
    fn add(&mut self, score: f32) {
        let at = (score.clamp(0.0, 1.0) * (BINS - 1) as f32) as usize;
        self.bins[at] += 1;
    }

    fn floor_for(&self, kept: u64) -> f32 {
        let mut above = 0;
        for at in (0..BINS).rev() {
            above += self.bins[at];
            if above >= kept {
                return at as f32 / (BINS - 1) as f32;
            }
        }
        0.0
    }
}

// Chance is what one pair in a thousand reaches in this base, never stricter than room for each node's closest.
fn floors(items: &[Item], spread: &[Spread; 2], caps: &Caps) -> [f32; 2] {
    let n = items.len() as u64;
    let mut speakers: HashMap<&str, u64> = HashMap::new();
    for item in items {
        if let Some(lang) = &item.lang {
            *speakers.entry(lang.as_str()).or_default() += 1;
        }
    }
    let declared: u64 = speakers.values().sum();
    let abroad_pairs: u64 = speakers.values().map(|m| m * (declared - m)).sum();
    let abroad_nodes: u64 = speakers.values().filter(|m| **m < declared).sum();
    let same_pairs = n * n.saturating_sub(1) - abroad_pairs;
    let kept = |pairs: u64, room: u64| ((pairs as f64 * caps.chance).ceil() as u64).max(room);
    [
        spread[Tongue::Same as usize].floor_for(kept(same_pairs, caps.closest as u64 * n)),
        spread[Tongue::Other as usize]
            .floor_for(kept(abroad_pairs, caps.abroad as u64 * abroad_nodes)),
    ]
}

fn shared_words(base: &Base, a: usize, b: usize, told: usize) -> Vec<String> {
    let theirs: HashMap<u32, f32> = base.vectors[b].iter().copied().collect();
    let mut both: Vec<(u32, f32)> = base.vectors[a]
        .iter()
        .filter(|(term, _)| base.weighty[*term as usize])
        .filter_map(|(term, mine)| Some((*term, mine * theirs.get(term)?)))
        .collect();
    both.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
    both.iter()
        .take(told)
        .map(|(term, _)| base.told[*term as usize].to_string())
        .collect()
}

#[cfg(test)]
#[path = "likeness_test.rs"]
mod tests;
