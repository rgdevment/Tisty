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
}

pub fn all_of_it(
    paths: &Paths,
    state: &State,
    cache: Option<&crate::cache::Cache>,
    dest: Option<&Path>,
    bin: bool,
) -> Swept {
    let mut done = cache.map(|one| one.already()).unwrap_or_default();
    let was = done.clone();

    let held = || {
        let mut named: Vec<String> = state
            .tasks
            .values()
            .flat_map(|task| task.references())
            .map(|one| one.target)
            .collect();
        named.extend(crate::docs::referenced(&paths.docs()));
        named
    };
    let swept = Swept {
        papers: papers(paths, state, dest, &mut done),
        attachments: attachments(paths, state, dest, held, &mut done),
        binned: if bin { self::bin(paths) } else { 0 },
    };
    if done != was
        && let Some(cache) = cache
    {
        cache.note_already(&done);
    }
    swept
}

pub fn settling_what_arrived(paths: &Paths, state: &State, files: &[String]) -> Vec<crate::Op> {
    let root = paths.docs();
    let mut told = Vec::new();
    for file in state.books_among(files) {
        match crate::docs::read(&root, &file) {
            Ok(body) => told.extend(state.settling(&file, &body)),
            Err(e) => witness::warn(
                channel::SYNC,
                "a document that arrived could not be read to settle its pages",
                &[("file", Fact::Id(file)), ("why", Fact::Why(e.to_string()))],
            ),
        }
    }
    told
}

pub fn papers(paths: &Paths, state: &State, dest: Option<&Path>, done: &mut Already) -> usize {
    let owed: BTreeSet<String> = state.shed.difference(&done.papers).cloned().collect();
    if owed.is_empty() {
        return 0;
    }
    let reach = dest.filter(|at| at.is_dir());
    let mut gone = crate::docs::sweep(&paths.docs(), &owed);
    if let Some(dest) = reach {
        gone += crate::docs::sweep(&dest.join("docs"), &owed);
    }
    forget_the_prints(paths, &owed);
    done.papers.extend(owed.into_iter().filter(|file| {
        let here = |root: &Path| went(root, crate::docs::resolve(root, file));
        here(&paths.docs()) && dest.is_none_or(|_| reach.is_some_and(|at| here(&at.join("docs"))))
    }));
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
    state: &State,
    dest: Option<&Path>,
    held: impl FnOnce() -> Vec<String>,
    done: &mut Already,
) -> usize {
    let owed: BTreeSet<String> = state
        .retired
        .difference(&done.attachments)
        .cloned()
        .collect();
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
    done.attachments.extend(owed.into_iter().filter(|one| {
        let here = |root: &Path| went(root, crate::attach::resolve(one, root));
        !held.contains(one.as_str())
            && here(paths.data())
            && dest.is_none_or(|_| reach.is_some_and(here))
    }));
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
