use std::path::{Path, PathBuf};

use super::Refused;

pub const KEPT_AT_MOST: usize = 20;
// The first backup is the file as it was before Tisty first wrote over it, so the cap never takes it.
const PINNED: u64 = 1;
const SHELVES: &str = "kept";
const WHERE: &str = "where.txt";
const DIGITS: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub n: u64,
    pub bytes: u64,
    pub pinned: bool,
    /// When the copy was kept, from its file; `n` alone orders copies when this is missing.
    pub at: Option<jiff::Timestamp>,
}

fn shelf(data: &Path, at: &Path) -> PathBuf {
    let named = crate::attach::printed(at.to_string_lossy().as_bytes());
    super::folder(data).join(SHELVES).join(&named[..24])
}

fn numbered(shelf: &Path) -> Vec<u64> {
    let mut all: Vec<u64> = std::fs::read_dir(shelf)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|one| one.file_name().to_str()?.parse().ok())
        .collect();
    all.sort_unstable();
    all
}

pub(super) fn keep(data: &Path, at: &Path, bytes: &[u8]) -> Result<u64, Refused> {
    let shelf = shelf(data, at);
    std::fs::create_dir_all(&shelf)?;
    let _ = crate::paths::ours_alone(&shelf);
    crate::store::write_atomic(&shelf.join(WHERE), at.to_string_lossy().as_bytes())?;
    let was = numbered(&shelf);
    // A write tried again over a locked file would push real backups out of the cap with copies.
    if let Some(&last) = was.last()
        && std::fs::read(shelf.join(format!("{last:0DIGITS$}"))).is_ok_and(|kept| kept == bytes)
    {
        return Ok(last);
    }
    let n = was.last().map_or(PINNED, |last| last + 1);
    let into = shelf.join(format!("{n:0DIGITS$}"));
    crate::store::write_atomic(&into, bytes)?;
    let _ = crate::paths::ours_alone(&into);
    let capped: Vec<u64> = numbered(&shelf)
        .into_iter()
        .filter(|one| *one != PINNED)
        .collect();
    for old in &capped[..capped.len().saturating_sub(KEPT_AT_MOST)] {
        let _ = std::fs::remove_file(shelf.join(format!("{old:0DIGITS$}")));
    }
    Ok(n)
}

pub fn kept(data: &Path, at: &Path) -> Vec<Kept> {
    let shelf = shelf(data, at);
    numbered(&shelf)
        .into_iter()
        .rev()
        .filter_map(|n| {
            let meta = std::fs::metadata(shelf.join(format!("{n:0DIGITS$}"))).ok()?;
            let at = meta
                .modified()
                .ok()
                .and_then(|when| jiff::Timestamp::try_from(when).ok());
            Some(Kept {
                n,
                bytes: meta.len(),
                pinned: n == PINNED,
                at,
            })
        })
        .collect()
}

pub fn read_kept(data: &Path, at: &Path, n: u64) -> Result<Vec<u8>, Refused> {
    std::fs::read(shelf(data, at).join(format!("{n:0DIGITS$}"))).map_err(|_| Refused::NoSuchBackup)
}

#[cfg(test)]
#[path = "kept_test.rs"]
mod tests;
