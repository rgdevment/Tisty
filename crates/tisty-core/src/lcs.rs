/// Past this many cells the table costs more than the answer is worth, and none is made.
pub const CELLS_AT_MOST: usize = 4_000_000;

/// How long a run the two still share from each place on, read from the end.
pub fn table<T: PartialEq>(a: &[T], b: &[T]) -> Option<Vec<Vec<u32>>> {
    if a.len().saturating_mul(b.len()) > CELLS_AT_MOST {
        return None;
    }
    let mut kept = vec![vec![0u32; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            kept[i][j] = match a[i] == b[j] {
                true => kept[i + 1][j + 1] + 1,
                false => kept[i + 1][j].max(kept[i][j + 1]),
            };
        }
    }
    Some(kept)
}

pub fn pairs<T: PartialEq>(a: &[T], b: &[T]) -> Vec<(usize, usize)> {
    let Some(kept) = table(a, b) else {
        return Vec::new();
    };
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            out.push((i, j));
            i += 1;
            j += 1;
        } else if kept[i + 1][j] >= kept[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}
