use std::collections::BTreeSet;
use std::path::Path;

use crate::{
    State,
    paths::Paths,
    witness::{self, Fact, channel},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Swept {
    pub papers: usize,
    pub attachments: usize,
    pub binned: usize,
}

impl Swept {
    pub fn any(&self) -> bool {
        self.papers + self.attachments + self.binned > 0
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Already {
    #[serde(default)]
    pub papers: BTreeSet<String>,
    #[serde(default)]
    pub attachments: BTreeSet<String>,
    #[serde(default)]
    pub papers_up: BTreeSet<String>,
    #[serde(default)]
    pub attachments_up: BTreeSet<String>,
    #[serde(default)]
    pub up_to: Option<String>,
}

impl Already {
    fn facing(&mut self, dest: Option<&Path>) {
        let Some(now) = dest.and_then(crate::paths::told_apart) else {
            return;
        };
        if self.up_to.as_deref() != Some(now.as_str()) {
            self.papers_up.clear();
            self.attachments_up.clear();
            self.up_to = Some(now);
        }
    }

    fn owes(&self, all: &BTreeSet<String>, dest: bool, up: bool) -> BTreeSet<String> {
        let (here, there) = match up {
            true => (&self.attachments, &self.attachments_up),
            false => (&self.papers, &self.papers_up),
        };
        let mut owed: BTreeSet<String> = all.difference(here).cloned().collect();
        if dest {
            owed.extend(all.difference(there).cloned());
        }
        owed
    }
}

pub struct Sweeping {
    paths: Paths,
    shed: BTreeSet<String>,
    retired: BTreeSet<String>,
    dest: Option<std::path::PathBuf>,
    bin: bool,
    done: Already,
    owed: bool,
}

pub struct Walked {
    job: Sweeping,
    named: Vec<String>,
}

impl Sweeping {
    pub fn of(
        paths: &Paths,
        state: &State,
        cache: Option<&crate::cache::Cache>,
        dest: Option<&Path>,
        bin: bool,
    ) -> Self {
        let mut done = cache.map(|one| one.already()).unwrap_or_default();
        done.facing(dest);
        Self {
            paths: paths.clone(),
            shed: state.shed.clone(),
            retired: state.retired.clone(),
            dest: dest.map(Path::to_path_buf),
            bin,
            owed: !done.owes(&state.retired, dest.is_some(), true).is_empty(),
            done,
        }
    }

    pub fn already(&self) -> Already {
        self.done.clone()
    }

    pub fn walk(self) -> Walked {
        let named = match self.owed {
            true => crate::docs::referenced(&self.paths.docs()),
            false => Vec::new(),
        };
        Walked { job: self, named }
    }
}

impl Walked {
    pub fn with(self, state: &State) -> (Swept, Already) {
        let Walked { job, named } = self;
        let Sweeping {
            paths,
            shed,
            retired,
            dest,
            bin,
            mut done,
            ..
        } = job;
        let dest = dest.as_deref();
        let held = || {
            let mut all: Vec<String> = state
                .tasks
                .values()
                .flat_map(|task| task.references())
                .map(|one| one.target)
                .collect();
            all.extend(named);
            all
        };
        let swept = Swept {
            papers: papers(&paths, &shed, dest, &mut done),
            attachments: attachments(&paths, &retired, dest, held, &mut done),
            binned: if bin { self::bin(&paths) } else { 0 },
        };
        (swept, done)
    }
}

pub fn all_of_it(
    paths: &Paths,
    state: &State,
    cache: Option<&crate::cache::Cache>,
    dest: Option<&Path>,
    bin: bool,
) -> Swept {
    let job = Sweeping::of(paths, state, cache, dest, bin);
    let was = job.already();
    let (swept, done) = job.walk().with(state);
    if done != was
        && let Some(cache) = cache
    {
        cache.note_already(&done);
    }
    swept
}

/// Reading the bodies is the slow half and needs nothing but the folder, so whoever holds the
/// state can ask for the books, have them read elsewhere, and hand back what came.
pub fn bodies_of(paths: &Paths, books: &[String]) -> Vec<(String, String)> {
    let root = paths.docs();
    let mut read = Vec::new();
    for file in books {
        match crate::docs::read(&root, file) {
            Ok(body) => read.push((file.clone(), body)),
            Err(e) => witness::warn(
                channel::SYNC,
                "a document that arrived could not be read to settle its pages",
                &[
                    ("file", Fact::Id(file.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            ),
        }
    }
    read
}

pub fn settling_what_came(state: &State, read: &[(String, String)]) -> Vec<crate::Op> {
    read.iter()
        .flat_map(|(file, body)| state.settling(file, body))
        .collect()
}

pub fn settling_what_arrived(paths: &Paths, state: &State, files: &[String]) -> Vec<crate::Op> {
    let read = bodies_of(paths, &state.books_among(files));
    settling_what_came(state, &read)
}

pub fn papers(
    paths: &Paths,
    shed: &BTreeSet<String>,
    dest: Option<&Path>,
    done: &mut Already,
) -> usize {
    let owed = done.owes(shed, dest.is_some(), false);
    if owed.is_empty() {
        return 0;
    }
    let reach = dest.filter(|at| at.is_dir());
    let mut gone = crate::docs::sweep(&paths.docs(), &owed);
    if let Some(dest) = reach {
        gone += crate::docs::sweep(&dest.join("docs"), &owed);
    }
    forget_the_prints(paths, &owed);
    let here = |file: &String| went(&paths.docs(), crate::docs::resolve(&paths.docs(), file));
    done.papers
        .extend(owed.iter().filter(|file| here(file)).cloned());
    if let Some(at) = reach {
        let up = |file: &String| {
            let there = at.join("docs");
            went(&there, crate::docs::resolve(&there, file))
        };
        done.papers_up
            .extend(owed.iter().filter(|file| up(file)).cloned());
    }
    if gone > 0 {
        witness::note(
            channel::SYNC,
            "a document deleted elsewhere is gone from here too",
            &[("count", Fact::Count(gone))],
        );
    }
    gone
}

pub fn attachments(
    paths: &Paths,
    retired: &BTreeSet<String>,
    dest: Option<&Path>,
    held: impl FnOnce() -> Vec<String>,
    done: &mut Already,
) -> usize {
    let owed = done.owes(retired, dest.is_some(), true);
    if owed.is_empty() {
        return 0;
    }
    let named = held();
    let held: BTreeSet<&str> = named.iter().map(String::as_str).collect();
    let reach = dest.filter(|at| at.is_dir());
    let mut gone = crate::attach::sweep(paths.data(), &owed, &held);
    if let Some(dest) = reach {
        gone += crate::attach::sweep(dest, &owed, &held);
    }
    let went_from = |root: &Path, one: &String| {
        !held.contains(one.as_str()) && went(root, crate::attach::resolve(one, root))
    };
    done.attachments.extend(
        owed.iter()
            .filter(|one| went_from(paths.data(), one))
            .cloned(),
    );
    if let Some(at) = reach {
        done.attachments_up
            .extend(owed.iter().filter(|one| went_from(at, one)).cloned());
    }
    if gone > 0 {
        witness::note(
            channel::ATTACH,
            "what was retired elsewhere is gone from here too",
            &[("count", Fact::Count(gone))],
        );
    }
    gone
}

pub fn bin(paths: &Paths) -> usize {
    let gone = crate::attach::empty_the_bin(paths.data(), jiff::Timestamp::now().as_second());
    if gone > 0 {
        witness::note(
            channel::ATTACH,
            "what waited in the bin past its time is gone",
            &[("count", Fact::Count(gone))],
        );
    }
    gone
}

fn went(root: &Path, at: crate::Result<std::path::PathBuf>) -> bool {
    root.is_dir() && !at.is_ok_and(|at| at.exists())
}

fn forget_the_prints(paths: &Paths, owed: &BTreeSet<String>) {
    let mut said = crate::docs::Carried::read(paths.data());
    let mut changed = false;
    for file in owed {
        changed |= said.of(file).is_some();
        said.forget(file);
    }
    if changed {
        let _ = said.save(paths.data());
    }
}

#[cfg(test)]
#[path = "tidy_test.rs"]
mod tests;
