use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::policy;

const FILE: &str = "reading.json";
const EVERY: Duration = Duration::from_millis(500);
// One large file on a slow link can go minutes without a word, and a read still under way must not look over.
const FRESH_FOR: u64 = 10 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Reading {
    pub done: usize,
    pub whole: usize,
    pub at: u64,
}

pub fn reading(home: &Path) -> Option<Reading> {
    let said: Reading = serde_json::from_slice(&std::fs::read(home.join(FILE)).ok()?).ok()?;
    (said.done < said.whole && policy::now().saturating_sub(said.at) <= FRESH_FOR).then_some(said)
}

pub(super) struct Noting {
    home: PathBuf,
    last: Option<Instant>,
}

impl Noting {
    pub(super) fn new(home: &Path) -> Self {
        Self {
            home: home.to_path_buf(),
            last: None,
        }
    }

    pub(super) fn heard(&mut self, far: &crate::Reached) {
        let crate::Reached::Along {
            stage: crate::Stage::Log,
            done,
            whole,
        } = *far
        else {
            return;
        };
        if whole == 0
            || self
                .last
                .is_some_and(|last| last.elapsed() < EVERY && done < whole)
        {
            return;
        }
        self.last = Some(Instant::now());
        let said = Reading {
            done,
            whole,
            at: policy::now(),
        };
        if let Ok(bytes) = serde_json::to_vec(&said) {
            let _ = tisty_core::store::write_atomic(&self.home.join(FILE), &bytes);
        }
    }
}

impl Drop for Noting {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.home.join(FILE));
    }
}

#[cfg(test)]
#[path = "reading_test.rs"]
mod tests;
