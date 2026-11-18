use std::collections::HashMap;
use std::hash::Hash;

pub(crate) const CELLS_AT_MOST: usize = 16_000_000;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Shared {
    pub pairs: Vec<(usize, usize)>,
    pub cells: usize,
}

/// Where two runs say the same, in order; what both open and close with is never weighed.
/// None when what is left between is past `most` cells: too long to tell apart.
pub(crate) fn shared<T: Hash + Eq>(a: &[T], b: &[T], most: usize) -> Option<Shared> {
    let lead = a.iter().zip(b).take_while(|(one, two)| one == two).count();
    let (rest_a, rest_b) = (&a[lead..], &b[lead..]);
    let tail = rest_a
        .iter()
        .rev()
        .zip(rest_b.iter().rev())
        .take_while(|(one, two)| one == two)
        .count();
    let (mid_a, mid_b) = (
        &rest_a[..rest_a.len() - tail],
        &rest_b[..rest_b.len() - tail],
    );

    let (between, cells) = between(mid_a, mid_b, most)?;
    let mut pairs: Vec<(usize, usize)> = (0..lead).map(|at| (at, at)).collect();
    pairs.extend(between.into_iter().map(|(i, j)| (lead + i, lead + j)));
    pairs.extend((0..tail).map(|at| (a.len() - tail + at, b.len() - tail + at)));
    Some(Shared { pairs, cells })
}

type Pairs = Vec<(usize, usize)>;

// A run shared by two is no longer than the shorter, so sixteen bits hold every count the ceiling lets in.
fn between<T: Hash + Eq>(a: &[T], b: &[T], most: usize) -> Option<(Pairs, usize)> {
    if a.is_empty() || b.is_empty() {
        return Some((Pairs::new(), 0));
    }
    let wide = b.len() + 1;
    let cells = (a.len() + 1).checked_mul(wide)?;
    if cells > most || a.len().min(b.len()) > usize::from(u16::MAX) {
        return None;
    }
    let (a, b) = numbered(a, b);
    let mut kept = vec![0u16; cells];
    for i in (0..a.len()).rev() {
        let (upper, below) = kept.split_at_mut((i + 1) * wide);
        let (row, below) = (&mut upper[i * wide..], &below[..wide]);
        let mut right = 0;
        for j in (0..b.len()).rev() {
            right = match a[i] == b[j] {
                true => below[j + 1] + 1,
                false => below[j].max(right),
            };
            row[j] = right;
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Pairs::new());
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            out.push((i, j));
            i += 1;
            j += 1;
        } else if kept[(i + 1) * wide + j] >= kept[i * wide + j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    Some((out, cells))
}

// Each piece is told from the rest once, so the table weighs numbers and never words.
fn numbered<T: Hash + Eq>(a: &[T], b: &[T]) -> (Vec<u32>, Vec<u32>) {
    let mut known: HashMap<&T, u32> = HashMap::new();
    let mut number = |one| {
        let next = known.len() as u32;
        *known.entry(one).or_insert(next)
    };
    (
        a.iter().map(&mut number).collect(),
        b.iter().map(&mut number).collect(),
    )
}

#[cfg(test)]
#[path = "shared_test.rs"]
mod tests;
