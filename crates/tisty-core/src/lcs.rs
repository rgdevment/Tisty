pub const CELLS_AT_MOST: usize = 2001 * 2001;

pub struct Table {
    wide: usize,
    cells: Vec<u32>,
}

impl Table {
    pub fn at(&self, i: usize, j: usize) -> u32 {
        self.cells[i * self.wide + j]
    }
}

// One flat run of cells, so a lopsided pair costs one allocation and never one per row.
pub fn table<T: PartialEq>(a: &[T], b: &[T]) -> Option<Table> {
    let wide = b.len() + 1;
    if (a.len() + 1).saturating_mul(wide) > CELLS_AT_MOST {
        return None;
    }
    let mut kept = Table {
        wide,
        cells: vec![0; (a.len() + 1) * wide],
    };
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            let cell = match a[i] == b[j] {
                true => kept.at(i + 1, j + 1) + 1,
                false => kept.at(i + 1, j).max(kept.at(i, j + 1)),
            };
            kept.cells[i * wide + j] = cell;
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
        } else if kept.at(i + 1, j) >= kept.at(i, j + 1) {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}
