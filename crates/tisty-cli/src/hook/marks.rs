use std::io::Write;
use std::path::{Path, PathBuf};

const FOLDER: &str = "turns";
const NAME_AT_MOST: usize = 64;
const KEPT_FOR_SECONDS: u64 = 7 * 24 * 60 * 60;
const TOUCHED: &str = "task";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    Edited,
    Called,
    Wrote,
    Ended,
    Handed,
    Asked,
    Published,
    AskedPublished,
}

impl Mark {
    fn file(self) -> &'static str {
        match self {
            Mark::Edited => "edited",
            Mark::Called => "called",
            Mark::Wrote => "wrote",
            Mark::Ended => "ended",
            Mark::Handed => "handed",
            Mark::Asked => "asked",
            Mark::Published => "published",
            Mark::AskedPublished => "asked-published",
        }
    }
}

pub struct Session {
    at: PathBuf,
}

impl Session {
    pub fn of(cache: &Path, session: &str) -> Self {
        Self {
            at: turns(cache).join(named(session)),
        }
    }

    pub fn mark(&self, what: Mark, at: i128) {
        if std::fs::create_dir_all(&self.at).is_err() {
            return;
        }
        let file = self.at.join(what.file());
        let aside = self
            .at
            .join(format!("{}.{}.tmp", what.file(), std::process::id()));
        if std::fs::write(&aside, at.to_string()).is_err()
            || std::fs::rename(&aside, &file).is_err()
        {
            let _ = std::fs::remove_file(&aside);
        }
    }

    pub fn touched(&self, task: &str) {
        if task.is_empty() || std::fs::create_dir_all(&self.at).is_err() {
            return;
        }
        let file = self.at.join(TOUCHED);
        let aside = self
            .at
            .join(format!("{TOUCHED}.{}.tmp", std::process::id()));
        if std::fs::write(&aside, named(task)).is_err() || std::fs::rename(&aside, &file).is_err() {
            let _ = std::fs::remove_file(&aside);
        }
    }

    pub fn last_touched(&self) -> Option<String> {
        std::fs::read_to_string(self.at.join(TOUCHED))
            .ok()
            .map(|said| said.trim().to_string())
            .filter(|said| !said.is_empty())
    }

    pub fn marks(&self) -> Marks {
        let read = |what: Mark| {
            std::fs::read_to_string(self.at.join(what.file()))
                .ok()
                .and_then(|said| said.trim().parse().ok())
        };
        Marks {
            edited: read(Mark::Edited),
            called: read(Mark::Called),
            wrote: read(Mark::Wrote),
            ended: read(Mark::Ended),
            handed: read(Mark::Handed),
            asked: read(Mark::Asked),
            published: read(Mark::Published),
            asked_published: read(Mark::AskedPublished),
        }
    }

    pub fn claim(&self, key: &str) -> bool {
        if std::fs::create_dir_all(&self.at).is_err() {
            return false;
        }
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.claim_file(key))
            .and_then(|mut file| file.write_all(b"1"))
            .is_ok()
    }

    pub fn claimed(&self, key: &str) -> bool {
        self.claim_file(key).exists()
    }

    fn claim_file(&self, key: &str) -> PathBuf {
        self.at.join(format!("claim-{}", named(key)))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Marks {
    pub edited: Option<i128>,
    pub called: Option<i128>,
    pub wrote: Option<i128>,
    pub ended: Option<i128>,
    pub handed: Option<i128>,
    pub asked: Option<i128>,
    pub published: Option<i128>,
    pub asked_published: Option<i128>,
}

impl Marks {
    pub fn published_unasked(&self) -> bool {
        let since_turn =
            |at: Option<i128>| at.is_some_and(|at| self.ended.is_none_or(|end| at > end));
        since_turn(self.published)
            && self
                .asked_published
                .is_none_or(|asked| self.published.is_some_and(|at| at > asked))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    HandBack,
    LetGo,
}

// A hand-back the agent let pass stays quiet until it calls Tisty again, so work outside any task costs one turn.
pub fn verdict(marks: &Marks, again: bool) -> Verdict {
    let since_turn = |at: Option<i128>| at.is_some_and(|at| marks.ended.is_none_or(|end| at > end));
    let let_pass = marks
        .handed
        .is_some_and(|handed| marks.called.is_none_or(|called| called <= handed));
    let hands_back = !again
        && since_turn(marks.edited)
        && !since_turn(marks.wrote)
        && !since_turn(marks.handed)
        && !since_turn(marks.asked)
        && !let_pass;
    match hands_back {
        true => Verdict::HandBack,
        false => Verdict::LetGo,
    }
}

fn turns(cache: &Path) -> PathBuf {
    cache.join(FOLDER)
}

fn named(said: &str) -> String {
    let kept: String = said
        .chars()
        .map(
            |one| match one.is_ascii_alphanumeric() || one == '-' || one == '_' {
                true => one,
                false => '-',
            },
        )
        .take(NAME_AT_MOST)
        .collect();
    match kept.trim_matches('-').is_empty() {
        true => "unnamed".to_string(),
        false => kept,
    }
}

pub fn sweep(cache: &Path, now: std::time::SystemTime) {
    let Ok(all) = std::fs::read_dir(turns(cache)) else {
        return;
    };
    for one in all.flatten() {
        let old = one
            .metadata()
            .and_then(|it| it.modified())
            .ok()
            .and_then(|at| now.duration_since(at).ok())
            .is_some_and(|age| age.as_secs() > KEPT_FOR_SECONDS);
        if old && one.path().is_dir() {
            let _ = std::fs::remove_dir_all(one.path());
        }
    }
}

#[cfg(test)]
#[path = "marks_test.rs"]
mod tests;
