use std::io::Read;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

const WIDTH: usize = 20;
const WIDTH_IN_BITS: usize = WIDTH * 8;
const SHIFT: usize = 11;

/// OneDrive's own fingerprint of a file, the only hash it keeps for every file in a personal drive.
#[derive(Debug, Clone)]
pub struct QuickXor {
    cells: [u8; WIDTH],
    at: usize,
    length: u64,
}

impl Default for QuickXor {
    fn default() -> Self {
        Self {
            cells: [0; WIDTH],
            at: 0,
            length: 0,
        }
    }
}

impl QuickXor {
    pub fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            let (cell, bit) = (self.at / 8, self.at % 8);
            self.cells[cell] ^= byte << bit;
            if bit > 0 {
                self.cells[(cell + 1) % WIDTH] ^= byte >> (8 - bit);
            }
            self.at = (self.at + SHIFT) % WIDTH_IN_BITS;
        }
        self.length = self.length.wrapping_add(bytes.len() as u64);
    }

    pub fn finish(&self) -> [u8; WIDTH] {
        let mut said = self.cells;
        for (one, length) in said[WIDTH - 8..].iter_mut().zip(self.length.to_le_bytes()) {
            *one ^= length;
        }
        said
    }

    pub fn printed(&self) -> String {
        STANDARD.encode(self.finish())
    }
}

pub fn of_file(local: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(local)?;
    let mut hash = QuickXor::default();
    let mut block = vec![0u8; 256 * 1024];
    loop {
        let read = file.read(&mut block)?;
        if read == 0 {
            return Ok(hash.printed());
        }
        hash.update(&block[..read]);
    }
}

#[cfg(test)]
#[path = "quickxor_test.rs"]
mod tests;
