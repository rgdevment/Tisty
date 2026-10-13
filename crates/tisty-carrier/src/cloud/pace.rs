use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tisty_core::witness::{self, Fact, channel};
use tisty_sync::{Reached, Stage};

use super::index::Index;

const KEPT_EVERY: Duration = Duration::from_secs(30);
const KEPT_EVERY_FILES: usize = 20;

/// How far a push got, told file by file and written down as it goes, so a round cut short keeps what landed.
pub struct Pace<'s> {
    home: Option<PathBuf>,
    saying: &'s mut dyn FnMut(Reached),
    kept_at: Instant,
    unkept: usize,
    done: usize,
    whole: usize,
}

impl<'s> Pace<'s> {
    pub fn new(home: Option<&Path>, saying: &'s mut dyn FnMut(Reached)) -> Self {
        Self {
            home: home.map(Path::to_path_buf),
            saying,
            kept_at: Instant::now(),
            unkept: 0,
            done: 0,
            whole: 0,
        }
    }

    pub fn saying(&mut self) -> &mut dyn FnMut(Reached) {
        &mut *self.saying
    }

    pub fn begun(&mut self, whole: usize) {
        self.done = 0;
        self.whole = whole;
        if whole > 0 {
            self.said();
        }
    }

    pub fn passed(&mut self) {
        self.done = (self.done + 1).min(self.whole);
        self.said();
    }

    pub fn landed(&mut self, index: &Index) {
        self.unkept += 1;
        self.passed();
        if self.unkept >= KEPT_EVERY_FILES || self.kept_at.elapsed() >= KEPT_EVERY {
            self.kept(index);
        }
    }

    pub fn kept(&mut self, index: &Index) {
        if self.unkept > 0
            && let Some(home) = &self.home
            && let Err(why) = index.save(home)
        {
            witness::warn(
                channel::CLOUD,
                "what a round had carried so far could not be written down, so it waits for the round's end",
                &[("why", Fact::Why(why.to_string()))],
            );
        }
        self.unkept = 0;
        self.kept_at = Instant::now();
    }

    fn said(&mut self) {
        if self.whole == 0 {
            return;
        }
        (self.saying)(Reached::Along {
            stage: Stage::Uploading,
            done: self.done,
            whole: self.whole,
        });
    }
}
