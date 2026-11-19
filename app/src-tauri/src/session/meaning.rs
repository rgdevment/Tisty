use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tisty_core::brain::Meaning;
use tisty_core::witness::{self, Fact, channel};
use tisty_meaning::Model;

// A vector made by another table never meets one made by this, so a new table needs a new name.
const VERSION: &str = "static-similarity-mrl-multilingual-v1@b68f412:256:i8";
const FOLDER: &str = "resources/brain";
const TRIES_AGAIN_AFTER: Duration = Duration::from_secs(10 * 60);

pub(super) struct Shipped {
    at: Option<PathBuf>,
    loaded: Option<Model>,
    refused_at: Option<Instant>,
}

impl Shipped {
    pub(super) fn beside_the_app() -> Self {
        Self::at(std::env::current_exe().ok().as_deref().map(beside))
    }

    pub(super) fn at(at: Option<PathBuf>) -> Self {
        Self {
            at,
            loaded: None,
            refused_at: None,
        }
    }

    fn loaded(&mut self) -> Option<&Model> {
        if self.loaded.is_none() && !self.refused() {
            match self.at.as_deref().map(read) {
                Some(Ok(model)) => self.loaded = Some(model),
                Some(Err(why)) => self.refuse(why),
                None => self.refuse("the app's folder is unknown".to_string()),
            }
        }
        self.loaded.as_ref()
    }

    fn refuse(&mut self, why: String) {
        self.refused_at = Some(Instant::now());
        witness::warn(
            channel::BRAIN,
            "the model of meanings could not be read, so only shared words make things alike",
            &[
                ("at", Fact::Path(self.at.clone().unwrap_or_default())),
                ("why", Fact::Why(why)),
            ],
        );
    }
}

impl Meaning for Shipped {
    fn version(&self) -> &str {
        VERSION
    }

    fn meaning(&mut self, text: &str) -> Option<Vec<f32>> {
        Some(self.loaded()?.meaning(text))
    }

    fn refused(&self) -> bool {
        self.refused_at
            .is_some_and(|at| at.elapsed() < TRIES_AGAIN_AFTER)
    }

    fn rest(&mut self) {
        self.loaded = None;
    }
}

#[cfg(target_os = "macos")]
fn beside(exe: &Path) -> PathBuf {
    let folder = exe.parent().unwrap_or(exe);
    folder.join("../Resources").join(FOLDER)
}

#[cfg(not(target_os = "macos"))]
fn beside(exe: &Path) -> PathBuf {
    let folder = exe.parent().unwrap_or(exe);
    folder.join(FOLDER)
}

fn read(at: &Path) -> Result<Model, String> {
    let table = std::fs::read(at.join("table.bin")).map_err(|e| e.to_string())?;
    let vocab = std::fs::read_to_string(at.join("vocab.txt")).map_err(|e| e.to_string())?;
    Model::read(table, &vocab).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "meaning_test.rs"]
mod tests;
