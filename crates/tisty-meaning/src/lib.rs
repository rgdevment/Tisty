mod pieces;

use pieces::Pieces;

const HEAD: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unread(&'static str);

impl std::fmt::Display for Unread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for Unread {}

// A static model: each piece has its vector, and a text means the mean of its pieces' vectors.
pub struct Model {
    pieces: Pieces,
    dims: usize,
    scales: Vec<f32>,
    table: Vec<u8>,
    values_at: usize,
}

impl Model {
    pub fn read(table: Vec<u8>, vocab: &str) -> Result<Self, Unread> {
        let number = |at: usize| -> Result<usize, Unread> {
            let bytes = table
                .get(at..at + 4)
                .ok_or(Unread("the table is shorter than its head"))?;
            usize::try_from(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
                .map_err(|_| Unread("the table names more rows than fit"))
        };
        let (rows, dims) = (number(0)?, number(4)?);
        let scales_end = rows
            .checked_mul(4)
            .and_then(|one| one.checked_add(HEAD))
            .ok_or(Unread("the table names more rows than fit"))?;
        let values_end = rows
            .checked_mul(dims)
            .and_then(|one| one.checked_add(scales_end))
            .ok_or(Unread("the table names more rows than fit"))?;
        if dims == 0 || table.len() != values_end {
            return Err(Unread("the table is not as long as its head says"));
        }
        let pieces = Pieces::read(vocab).ok_or(Unread("the vocabulary is unreadable"))?;
        if pieces.len() != rows {
            return Err(Unread("the vocabulary and the table differ in length"));
        }
        let scales = table[HEAD..scales_end]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|one| f32::from_le_bytes(*one))
            .collect();
        Ok(Self {
            pieces,
            dims,
            scales,
            table,
            values_at: scales_end,
        })
    }

    pub fn dims(&self) -> usize {
        self.dims
    }

    pub fn pieces(&self, text: &str) -> Vec<u32> {
        self.pieces.of(text)
    }

    // A text with no pieces means nothing, so its vector is all zeros and nothing is alike it.
    pub fn meaning(&self, text: &str) -> Vec<f32> {
        let mut sum = vec![0f32; self.dims];
        let mut counted = 0usize;
        for id in self.pieces.of(text) {
            let row = id as usize;
            let scale = self.scales[row];
            let from = self.values_at + row * self.dims;
            for (into, value) in sum.iter_mut().zip(&self.table[from..from + self.dims]) {
                *into += f32::from(i8::from_le_bytes([*value])) * scale;
            }
            counted += 1;
        }
        if counted > 0 {
            for one in &mut sum {
                *one /= counted as f32;
            }
        }
        let norm = sum.iter().map(|one| one * one).sum::<f32>().sqrt();
        if norm > 0.0 {
            for one in &mut sum {
                *one /= norm;
            }
        }
        sum
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
