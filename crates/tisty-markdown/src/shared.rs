pub(crate) const CELLS_AT_MOST: usize = 16_000_000;

/// Where two runs say the same, in order; what both open and close with is never weighed.
/// None when what is left between is past `most` cells: too long to tell apart.
pub(crate) fn shared<T: PartialEq>(a: &[T], b: &[T], most: usize) -> Option<Vec<(usize, usize)>> {
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

    let mut pairs: Vec<(usize, usize)> = (0..lead).map(|at| (at, at)).collect();
    pairs.extend(
        between(mid_a, mid_b, most)?
            .into_iter()
            .map(|(i, j)| (lead + i, lead + j)),
    );
    pairs.extend((0..tail).map(|at| (a.len() - tail + at, b.len() - tail + at)));
    Some(pairs)
}

// A run shared by two is no longer than the shorter, so sixteen bits hold every count the ceiling lets in.
fn between<T: PartialEq>(a: &[T], b: &[T], most: usize) -> Option<Vec<(usize, usize)>> {
    if a.is_empty() || b.is_empty() {
        return Some(Vec::new());
    }
    let wide = b.len() + 1;
    let cells = (a.len() + 1).checked_mul(wide)?;
    if cells > most || a.len().min(b.len()) > usize::from(u16::MAX) {
        return None;
    }
    let mut kept = vec![0u16; cells];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            kept[i * wide + j] = match a[i] == b[j] {
                true => kept[(i + 1) * wide + j + 1] + 1,
                false => kept[(i + 1) * wide + j].max(kept[i * wide + j + 1]),
            };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
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
    Some(out)
}

#[cfg(test)]
#[path = "shared_test.rs"]
mod tests;
