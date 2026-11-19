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
    pub(crate) loose_share: usize,
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
    loose_share: 4,
};

pub(crate) struct Item {
    pub(crate) lang: Option<String>,
    pub(crate) kin: Vec<String>,
    pub(crate) words: Vec<(String, String, u32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Rule {
    Words,
    Equivalent { loose: bool },
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

type Vector = Vec<(u32, f32, u32)>;

struct Base<'a> {
    told: Vec<&'a str>,
    written: Vec<&'a str>,
    weighty: Vec<bool>,
    loose: Vec<bool>,
    vectors: Vec<Vector>,
}

impl Base<'_> {
    fn counts(&self, term: u32) -> bool {
        self.weighty[term as usize] || self.loose[term as usize]
    }
}

#[derive(Default)]
struct Tally {
    score: Vec<f32>,
    both: Vec<u32>,
    equal: Vec<u32>,
    touched: Vec<usize>,
}

pub(crate) fn alike(
    items: &[Item],
    tied: &HashSet<(usize, usize)>,
    caps: &Caps,
    rule: Rule,
) -> Outcome {
    let (base, common_from) = weighed(items, caps, rule);
    let mut postings: Vec<Vec<(usize, f32, u32)>> = vec![Vec::new(); base.told.len()];
    for (at, vector) in base.vectors.iter().enumerate() {
        for &(term, weight, written) in vector {
            if base.counts(term) {
                postings[term as usize].push((at, weight, written));
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
    let mut tally = Tally {
        score: vec![0.0; items.len()],
        both: vec![0; items.len()],
        equal: vec![0; items.len()],
        touched: Vec::new(),
    };
    for (at, vector) in base.vectors.iter().enumerate() {
        steps += tallied(&base, &postings, at, vector, rule, &mut tally);
        let mut near: [Vec<(usize, f32)>; 2] = [Vec::new(), Vec::new()];
        for other in std::mem::take(&mut tally.touched) {
            let (said, shared, equal) = (tally.score[other], tally.both[other], tally.equal[other]);
            tally.score[other] = 0.0;
            tally.both[other] = 0;
            tally.equal[other] = 0;
            let tongue = tongue_of(&items[at], &items[other]);
            spread[tongue as usize].add(said);
            let fits = rule == Rule::Words || equal > 0;
            if fits && shared as usize >= caps.shared && !apart(at, other) {
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
    Outcome {
        pairs: kept_pairs(&base, &ranked, &floors, caps, rule),
        steps,
        common_from,
    }
}

// A word both sides write alike already had its say in the shared-words track.
fn tallied(
    base: &Base,
    postings: &[Vec<(usize, f32, u32)>],
    at: usize,
    vector: &Vector,
    rule: Rule,
    tally: &mut Tally,
) -> u64 {
    let mut steps = 0;
    for &(term, weight, mine) in vector {
        for &(other, theirs, written) in &postings[term as usize] {
            if other == at {
                continue;
            }
            let differs = rule != Rule::Words && written != mine;
            if rule != Rule::Words && !differs && !base.weighty[term as usize] {
                continue;
            }
            if tally.both[other] == 0 {
                tally.touched.push(other);
            }
            tally.score[other] += weight * theirs;
            tally.both[other] += 1;
            tally.equal[other] += u32::from(differs);
            steps += 1;
        }
    }
    steps
}

fn kept_pairs(
    base: &Base,
    ranked: &[[Vec<(usize, f32)>; 2]],
    floors: &[f32; 2],
    caps: &Caps,
    rule: Rule,
) -> Vec<Pair> {
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
    let mut held = vec![0usize; ranked.len()];
    let mut pairs = Vec::new();
    for (_, a, b, said) in found {
        if held[a] >= caps.closest || held[b] >= caps.closest {
            continue;
        }
        let (shared, crossing) = shared_words(base, a, b, caps.told, rule);
        if rule != Rule::Words && !crossing {
            continue;
        }
        held[a] += 1;
        held[b] += 1;
        pairs.push(Pair {
            a,
            b,
            score: said,
            shared,
        });
    }
    pairs
}

fn tongue_of(one: &Item, other: &Item) -> Tongue {
    match (&one.lang, &other.lang) {
        (Some(mine), Some(theirs)) if mine != theirs => Tongue::Other,
        _ => Tongue::Same,
    }
}

fn weighed<'a>(items: &'a [Item], caps: &Caps, rule: Rule) -> (Base<'a>, usize) {
    let mut ids: HashMap<&str, u32> = HashMap::new();
    let mut told: Vec<&str> = Vec::new();
    let mut df: Vec<u32> = Vec::new();
    let mut spoken: HashMap<&str, u32> = HashMap::new();
    let mut written: Vec<&str> = Vec::new();
    let mut first_written: Vec<u32> = Vec::new();
    let mut crossed: Vec<bool> = Vec::new();
    let mut counts: Vec<Vec<(u32, u32, u32)>> = Vec::with_capacity(items.len());
    for item in items {
        let here: Vec<(u32, u32, u32)> = item
            .words
            .iter()
            .map(|(key, said, count)| {
                let id = *ids.entry(key.as_str()).or_insert_with(|| {
                    told.push(said.as_str());
                    df.push(0);
                    (df.len() - 1) as u32
                });
                let way = match rule {
                    Rule::Words => id,
                    Rule::Equivalent { .. } => *spoken.entry(said.as_str()).or_insert_with(|| {
                        written.push(said.as_str());
                        (written.len() - 1) as u32
                    }),
                };
                match first_written.get(id as usize) {
                    Some(first) => crossed[id as usize] |= *first != way,
                    None => {
                        first_written.push(way);
                        crossed.push(false);
                    }
                }
                (id, *count, way)
            })
            .collect();
        for (id, _, _) in &here {
            df[*id as usize] += 1;
        }
        counts.push(here);
    }
    let common_from = common_from(&df, items.len(), caps);
    let weighty: Vec<bool> = df.iter().map(|one| (*one as usize) < common_from).collect();
    let loose = match rule {
        Rule::Equivalent { loose: true } => loose(&df, &weighty, &crossed, items.len(), caps),
        _ => vec![false; df.len()],
    };
    let n = items.len() as f32;
    let vectors = counts
        .iter()
        .map(|here| {
            let mut vector: Vector = here
                .iter()
                .map(|(id, count, way)| {
                    let idf = ((n + 1.0) / (df[*id as usize] as f32 + 1.0)).ln();
                    (*id, (1.0 + (*count as f32).ln()) * idf, *way)
                })
                .filter(|(_, weight, _)| *weight > 0.0)
                .collect();
            vector.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
            vector.truncate(caps.weighed);
            let norm = vector.iter().map(|(_, w, _)| w * w).sum::<f32>().sqrt();
            for one in &mut vector {
                one.1 /= norm;
            }
            vector
        })
        .collect();
    (
        Base {
            told,
            written,
            weighty,
            loose,
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

// A translation often lands on a common English word, which the 5 % cut alone would silence; the budget stays one.
fn loose(df: &[u32], weighty: &[bool], crossed: &[bool], n: usize, caps: &Caps) -> Vec<bool> {
    let below = n / caps.loose_share;
    let mut wanted: Vec<usize> = (0..df.len())
        .filter(|at| crossed[*at] && !weighty[*at] && (df[*at] as usize) < below)
        .collect();
    wanted.sort_by_key(|at| (df[*at], *at));
    let mut loose = vec![false; df.len()];
    let mut spent: u64 = (0..df.len())
        .filter(|at| weighty[*at] && df[*at] >= 2)
        .map(|at| u64::from(df[at]).pow(2))
        .sum();
    for at in wanted {
        spent += u64::from(df[at]).pow(2);
        if spent > caps.steps {
            break;
        }
        loose[at] = true;
    }
    loose
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

// A track's guess that shows no word each side wrote its own way could not say why it was made.
fn shared_words(base: &Base, a: usize, b: usize, told: usize, rule: Rule) -> (Vec<String>, bool) {
    let theirs: HashMap<u32, (f32, u32)> = base.vectors[b]
        .iter()
        .map(|(term, weight, way)| (*term, (*weight, *way)))
        .collect();
    let mut both: Vec<(u32, f32, u32, u32)> = base.vectors[a]
        .iter()
        .filter(|(term, _, _)| base.counts(*term))
        .filter_map(|(term, mine, way)| {
            let (other, their_way) = theirs.get(term)?;
            let differs = *way != *their_way;
            let counted = rule == Rule::Words || differs || base.weighty[*term as usize];
            counted.then_some((*term, mine * other, *way, *their_way))
        })
        .collect();
    both.sort_by(|x, y| y.1.total_cmp(&x.1).then(x.0.cmp(&y.0)));
    both.truncate(told);
    let crossing = both.iter().any(|(_, _, mine, theirs)| mine != theirs);
    let said = both
        .iter()
        .map(|(term, _, mine, theirs)| match rule {
            Rule::Words => base.told[*term as usize].to_string(),
            Rule::Equivalent { .. } if mine == theirs => base.written[*mine as usize].to_string(),
            Rule::Equivalent { .. } => format!(
                "{}≈{}",
                base.written[*mine as usize], base.written[*theirs as usize]
            ),
        })
        .collect();
    (said, rule != Rule::Words && crossing)
}

#[cfg(test)]
#[path = "likeness_test.rs"]
mod tests;
