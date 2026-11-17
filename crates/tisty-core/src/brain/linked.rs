use std::collections::{BTreeMap, BTreeSet};

use tisty_markdown::refs::{DOC, TASK, links};

use super::Docs;
use crate::model::{DocId, Kept};
use crate::{State, Task, TaskId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Tie {
    Cite,
    Mention,
    Structure,
    Tag,
    Uses,
}

impl Tie {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Tie::Cite => "cite",
            Tie::Mention => "mention",
            Tie::Structure => "structure",
            Tie::Tag => "tag",
            Tie::Uses => "uses",
        }
    }
}

pub(super) type Ties = BTreeSet<(String, Tie)>;

pub(super) enum Holder<'a> {
    Doc(&'a Kept),
    Task(&'a Task),
    Nothing,
}

enum Aim {
    Paper(String),
    File(String),
}

pub(super) struct Ground<'a> {
    state: &'a State,
    docs: &'a Docs,
    pages_of_books: BTreeMap<DocId, BTreeSet<&'a str>>,
    pages_of_tasks: BTreeMap<TaskId, Vec<&'a str>>,
    sheet: Option<&'a str>,
}

impl<'a> Ground<'a> {
    pub(super) fn new(state: &'a State, docs: &'a Docs) -> Self {
        let mut ground = Self {
            state,
            docs,
            pages_of_books: BTreeMap::new(),
            pages_of_tasks: BTreeMap::new(),
            sheet: None,
        };
        for kept in state.docs.values() {
            if let Some(task) = kept.task_of {
                ground
                    .pages_of_tasks
                    .entry(task)
                    .or_default()
                    .push(&kept.file);
            }
            if let Some(book) = kept.page_of {
                ground
                    .pages_of_books
                    .entry(book)
                    .or_default()
                    .insert(&kept.file);
            }
        }
        ground.sheet = state
            .papers()
            .filter(|one| one.brainstorm)
            .min_by_key(|one| {
                (
                    state.held_away(one),
                    one.translation_of.is_some(),
                    one.file.as_str(),
                )
            })
            .map(|one| one.file.as_str());
        ground
    }

    pub(super) fn sheet(&self) -> &str {
        self.sheet.unwrap_or_default()
    }

    // A document's ties read its book and its pages too, and its own record names neither.
    pub(super) fn leaned_on(&self, kept: &Kept) -> String {
        let book = self.book_of(kept).unwrap_or_default();
        let pages: Vec<&str> = self
            .pages_of_books
            .get(&kept.id)
            .into_iter()
            .flatten()
            .copied()
            .collect();
        format!("{book}\u{1}{}", pages.join("\u{1}"))
    }

    fn book_of(&self, kept: &Kept) -> Option<&str> {
        let book = kept.page_of.and_then(|up| self.state.docs.get(&up))?;
        Some(book.file.as_str())
    }

    pub(super) fn pages_told(&self, task: &Task) -> Vec<&str> {
        self.pages_of_tasks
            .get(&task.id)
            .into_iter()
            .flatten()
            .map(|file| self.docs.told(file))
            .collect()
    }

    pub(super) fn ties(&self, own: &str, holder: &Holder) -> Ties {
        let mut ties = Ties::new();
        match holder {
            Holder::Doc(kept) => self.of_doc(own, kept, &mut ties),
            Holder::Task(task) => self.of_task(own, task, &mut ties),
            Holder::Nothing => {}
        }
        ties
    }

    fn of_doc(&self, own: &str, kept: &Kept, ties: &mut Ties) {
        let pages = self.pages_of_books.get(&kept.id);
        if let Some(body) = self.docs.read(&kept.file) {
            self.written(own, &body, pages, ties);
        }
        if let Some(book) = self.book_of(kept) {
            ties.insert((book.to_string(), Tie::Structure));
        }
        tagged(&kept.tags, ties);
    }

    fn of_task(&self, own: &str, task: &Task, ties: &mut Ties) {
        let said = task
            .description
            .as_deref()
            .into_iter()
            .chain(task.log.iter().map(|entry| entry.body.as_str()))
            .chain(task.steps.iter().map(|step| step.text.as_str()));
        for text in said {
            self.written(own, text, None, ties);
        }
        for file in self.pages_of_tasks.get(&task.id).into_iter().flatten() {
            if let Some(page) = self.docs.read(file) {
                self.written(own, &page, None, ties);
            }
        }
        if let Some(whole) = task.part_of {
            ties.insert((whole.to_string(), Tie::Structure));
        }
        if let Some(origin) = task.source.as_deref().and_then(|one| self.origin(one)) {
            ties.insert((origin, Tie::Structure));
        }
        tagged(&task.tags, ties);
    }

    fn written(&self, own: &str, text: &str, pages: Option<&BTreeSet<&str>>, ties: &mut Ties) {
        for link in links(text) {
            match self.aimed(&link.target) {
                Some(Aim::File(at)) => {
                    ties.insert((at, Tie::Uses));
                }
                // A book's page lines place its pages: the page says so from its side.
                Some(Aim::Paper(id))
                    if id != own
                        && !(link.card && pages.is_some_and(|all| all.contains(id.as_str()))) =>
                {
                    let tie = if link.card { Tie::Cite } else { Tie::Mention };
                    ties.insert((id, tie));
                }
                _ => {}
            }
        }
        if let Some(sheet) = self.sheet.filter(|one| *one != own)
            && crate::brainstorm::ideas(text) > 0
        {
            ties.insert((sheet.to_string(), Tie::Structure));
        }
    }

    // Read from the words alone: a target that has not arrived yet is the same edge once it does.
    fn aimed(&self, target: &str) -> Option<Aim> {
        if let Some(named) = target.strip_prefix(DOC) {
            return Some(Aim::Paper(paper_named(named)));
        }
        if let Some(named) = target.strip_prefix(TASK) {
            let named = before_section(named);
            return Some(Aim::Paper(
                named
                    .parse::<TaskId>()
                    .map_or_else(|_| named.to_string(), |id| id.to_string()),
            ));
        }
        crate::attach::canonical(target).map(Aim::File)
    }

    // Only an idea names its sheet as `<file>#<heading>`; other sources are messages and links.
    fn origin(&self, source: &str) -> Option<String> {
        let (named, _) = source.split_once('#')?;
        let named = named.trim();
        crate::docs::resolve(&self.docs.root, named)
            .is_ok()
            .then(|| paper_named(named))
    }
}

fn paper_named(named: &str) -> String {
    let named = before_section(named);
    if let Some(task) = crate::task_docs::task_of_file(named) {
        return task.to_string();
    }
    named
        .parse::<DocId>()
        .map_or_else(|_| named.to_string(), |id| id.to_string())
}

fn before_section(named: &str) -> &str {
    named.split('#').next().unwrap_or(named).trim()
}

fn tagged(tags: &[crate::Tag], ties: &mut Ties) {
    for tag in tags {
        ties.insert((format!("#{tag}"), Tie::Tag));
    }
}

#[cfg(test)]
#[path = "linked_test.rs"]
mod tests;
