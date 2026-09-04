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

    fn cached(room: &tempfile::TempDir) -> crate::cache::Cache {
        crate::cache::Cache::open(&room.path().join("cache"))
            .unwrap()
            .expect("a cache opens")
    }

    #[test]
    fn what_was_taken_out_is_remembered_across_a_reading() {
        let (room, paths) = desk();
        let cache = cached(&room);
        a_paper(&paths, "dev_a-0001");
        let mut state = State::default();
        state.shed.insert("dev_a-0001".into());

        let swept = all_of_it(&paths, &state, Some(&cache), None, true);
        assert_eq!(swept.papers, 1);
        assert!(swept.any());

        a_paper(&paths, "dev_a-0001");
        let again = all_of_it(&paths, &state, Some(&cache), None, false);
        assert_eq!(again.papers, 0, "the mark outlived the call");
        assert!(!again.any());
        assert!(paths.docs().join("dev_a-0001.md").exists());
    }

    #[test]
    fn without_a_cache_it_still_sweeps_and_simply_forgets() {
        let (_room, paths) = desk();
        a_paper(&paths, "dev_a-0001");
        let mut state = State::default();
        state.shed.insert("dev_a-0001".into());

        assert_eq!(all_of_it(&paths, &state, None, None, false).papers, 1);
        a_paper(&paths, "dev_a-0001");
        assert_eq!(
            all_of_it(&paths, &state, None, None, false).papers,
            1,
            "nothing remembers, so it looks again"
        );
    }

    #[test]
    fn a_body_that_arrived_saying_another_order_is_settled_in_one_batch() {
        let (_room, paths) = desk();
        let book = ulid::Ulid::generate();
        let mut state = State::default();
        let mut kept = |id, file: &str, order: &str, up| {
            state.docs.insert(
                id,
                crate::model::Kept {
                    title: None,
                    bytes: None,
                    wrote: None,
                    tags: Vec::new(),
                    id,
                    file: file.into(),
                    order: order.into(),
                    folder: None,
                    page_of: up,
                    archived: false,
                    locked: false,
                },
            );
        };
        kept(book, "dev_a-0001", "V", None);
        let one = ulid::Ulid::generate();
        let two = ulid::Ulid::generate();
        kept(one, "dev_a-0002", "V", Some(book));
        kept(two, "dev_a-0003", "W", Some(book));

        std::fs::write(
            paths.docs().join("dev_a-0001.md"),
            "# Libro

![dos](tisty:doc/dev_a-0003)

![uno](tisty:doc/dev_a-0002)
",
        )
        .unwrap();

        let told = settling_what_arrived(&paths, &state, &["dev_a-0001".to_string()]);
        assert_eq!(told.len(), 1, "only the one that has to move");

        assert!(
            settling_what_arrived(&paths, &state, &["dev_a-0002".to_string()]).is_empty(),
            "a page arriving moves nothing: the order lives in the book"
        );
        assert!(
            settling_what_arrived(&paths, &state, &[]).is_empty(),
            "and nothing arriving reads nothing"
        );
    }

    #[test]
    fn a_file_that_would_not_go_is_looked_for_again_next_time() {
        let (_room, paths) = desk();
        let at = paths.docs().join("dev_a-0001.md");
        std::fs::create_dir_all(&at).unwrap();
        let mut state = State::default();
        state.shed.insert("dev_a-0001".into());
        let mut done = Already::default();

        assert_eq!(
            papers(&paths, &state, None, &mut done),
            0,
            "it would not go"
        );
        assert!(
            done.papers.is_empty(),
            "so it is not written off: the next opening has to try again"
        );

        std::fs::remove_dir(&at).unwrap();
        std::fs::write(&at, b"# Algo").unwrap();
        assert_eq!(
            papers(&paths, &state, None, &mut done),
            1,
            "and then it goes"
        );
        assert!(done.papers.contains("dev_a-0001"));
    }

    #[test]
    fn a_shared_folder_that_is_not_there_is_never_mistaken_for_a_tidy_one() {
        let (room, paths) = desk();
        let away = room.path().join("nowhere");
        let mut state = State::default();
        state.shed.insert("dev_a-0001".into());
        let mut done = Already::default();

        papers(&paths, &state, Some(&away), &mut done);
        assert!(
            done.papers.is_empty(),
            "an unmounted drive looks exactly like an empty one, so nothing is written off"
        );

        std::fs::create_dir_all(away.join("docs")).unwrap();
        papers(&paths, &state, Some(&away), &mut done);
        assert!(
            done.papers.contains("dev_a-0001"),
            "once it is there, it counts"
        );
    }
}
