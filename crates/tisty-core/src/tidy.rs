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

pub fn papers(paths: &Paths, state: &State, dest: Option<&Path>, done: &mut Already) -> usize {
    let owed: BTreeSet<String> = state.shed.difference(&done.papers).cloned().collect();
    if owed.is_empty() {
        return 0;
    }
    let mut gone = crate::docs::sweep(&paths.docs(), &owed);
    if let Some(dest) = dest {
        gone += crate::docs::sweep(&dest.join("docs"), &owed);
    }
    forget_the_prints(paths, &owed);
    done.papers.extend(owed);
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
    let mut gone = crate::attach::sweep(paths.data(), &owed, &held);
    if let Some(dest) = dest {
        gone += crate::attach::sweep(dest, &owed, &held);
    }
    done.attachments
        .extend(owed.into_iter().filter(|one| !held.contains(one.as_str())));
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
mod tests {
    use super::*;

    fn desk() -> (tempfile::TempDir, Paths) {
        let room = tempfile::tempdir().unwrap();
        let paths = Paths::new(room.path().join("data"), room.path().join("config"));
        std::fs::create_dir_all(paths.docs()).unwrap();
        (room, paths)
    }

    fn a_paper(paths: &Paths, name: &str) {
        std::fs::write(paths.docs().join(format!("{name}.md")), b"# Algo").unwrap();
    }

    #[test]
    fn a_paper_the_log_shed_is_taken_out_once_and_not_looked_for_again() {
        let (_room, paths) = desk();
        a_paper(&paths, "dev_a-0001");
        let mut state = State::default();
        state.shed.insert("dev_a-0001".into());
        let mut done = Already::default();

        assert_eq!(papers(&paths, &state, None, &mut done), 1);
        assert!(!paths.docs().join("dev_a-0001.md").exists());
        assert!(done.papers.contains("dev_a-0001"));

        a_paper(&paths, "dev_a-0001");
        assert_eq!(
            papers(&paths, &state, None, &mut done),
            0,
            "a set that only grows is not walked again"
        );
        assert!(
            paths.docs().join("dev_a-0001.md").exists(),
            "and a file written afterwards under a shed name is left alone"
        );

        state.shed.insert("dev_a-0002".into());
        a_paper(&paths, "dev_a-0002");
        assert_eq!(
            papers(&paths, &state, None, &mut done),
            1,
            "only the new one"
        );
    }

    #[test]
    fn nothing_retired_means_nobody_reads_every_document_to_find_out() {
        let (_room, paths) = desk();
        let state = State::default();
        let mut done = Already::default();
        let asked = std::cell::Cell::new(false);

        let gone = attachments(
            &paths,
            &state,
            None,
            || {
                asked.set(true);
                Vec::new()
            },
            &mut done,
        );

        assert_eq!(gone, 0);
        assert!(
            !asked.get(),
            "reading every body to answer a question nobody asked is the whole cost"
        );
    }

    #[test]
    fn an_attachment_something_still_names_is_left_and_asked_about_again() {
        let (_room, paths) = desk();
        let at = "attachments/ab/una-a3f90001.png";
        let shelf = paths.data().join("attachments/ab");
        std::fs::create_dir_all(&shelf).unwrap();
        std::fs::write(shelf.join("una-a3f90001.png"), b"unos bytes").unwrap();

        let mut state = State::default();
        state.retired.insert(at.into());
        let mut done = Already::default();

        assert_eq!(
            attachments(&paths, &state, None, || vec![at.to_string()], &mut done),
            0
        );
        assert!(shelf.join("una-a3f90001.png").exists());
        assert!(
            done.attachments.is_empty(),
            "it was not taken out, so it is asked about again when the document goes"
        );

        assert_eq!(
            attachments(&paths, &state, None, Vec::new, &mut done),
            1,
            "and once nothing names it, it goes"
        );
        assert!(done.attachments.contains(at));
    }
}
