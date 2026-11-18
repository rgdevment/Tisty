use std::collections::{BTreeMap, BTreeSet};

use tisty_markdown::named::{Named, spots};
use tisty_markdown::refs::{DOC, TASK, links, pictures};

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
pub(super) type Shown = BTreeMap<String, String>;

// A `[[title]]` is aimed at a title, so a document renamed later is found by whoever reads.
pub(super) const TITLED: &str = "[[";

pub(super) enum Holder<'a> {
    Doc(&'a Kept),
    Task(&'a Task),
    Nothing,
}

pub(super) struct Read {
    pub(super) ties: Ties,
    pub(super) away: Vec<String>,
    pub(super) body: Option<String>,
    pub(super) shows: Shown,
}

enum Aim {
    Paper(String),
    File(String),
}

pub(super) struct Ground<'a> {
    state: &'a State,
    docs: &'a Docs,
    pages_of_books: BTreeMap<DocId, BTreeMap<&'a str, &'a str>>,
    pages_of_tasks: BTreeMap<TaskId, Vec<&'a str>>,
    tongues: BTreeMap<DocId, Vec<&'a Kept>>,
    sheet: Option<&'a str>,
}

impl<'a> Ground<'a> {
    pub(super) fn new(state: &'a State, docs: &'a Docs) -> Self {
        let mut ground = Self {
            state,
            docs,
            pages_of_books: BTreeMap::new(),
            pages_of_tasks: BTreeMap::new(),
            tongues: state.families(),
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
                    .insert(&kept.file, &kept.order);
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

    // A document's ties and card read its book, pages and languages, and its own record names none.
    pub(super) fn leaned_on(&self, kept: &Kept) -> String {
        let book = self.book_of(kept).unwrap_or_default();
        let pages: Vec<String> = self
            .pages_of_books
            .get(&kept.id)
            .into_iter()
            .flatten()
            .map(|(file, order)| format!("{file}:{order}"))
            .collect();
        let tongues: Vec<String> = self
            .tongues_of(kept)
            .iter()
            .map(|one| format!("{}:{}", one.file, one.lang.as_deref().unwrap_or_default()))
            .collect();
        format!(
            "{book}\u{1}{}\u{2}{}",
            pages.join("\u{1}"),
            tongues.join("\u{1}")
        )
    }

    pub(super) fn tongues_of(&self, kept: &Kept) -> &[&'a Kept] {
        if kept.page_of.is_some() || kept.task_of.is_some() {
            return &[];
        }
        let anchor = self.state.anchor_of(kept);
        self.tongues.get(&anchor).map_or(&[], Vec::as_slice)
    }

    pub(super) fn has_pages(&self, kept: &Kept) -> bool {
        self.pages_of_books.contains_key(&kept.id)
    }

    pub(super) fn book_of(&self, kept: &Kept) -> Option<&str> {
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

    pub(super) fn read(&self, own: &str, holder: &Holder) -> Read {
        let mut read = Read {
            ties: Ties::new(),
            away: Vec::new(),
            body: None,
            shows: Shown::new(),
        };
        match holder {
            Holder::Doc(kept) => self.of_doc(own, kept, &mut read),
            Holder::Task(task) => self.of_task(own, task, &mut read),
            Holder::Nothing => {}
        }
        read
    }

    fn of_doc(&self, own: &str, kept: &Kept, read: &mut Read) {
        let pages = self.pages_of_books.get(&kept.id);
        read.body = self.docs.read(&kept.file);
        if let Some(body) = &read.body {
            self.written(own, body, pages, (&mut read.ties, &mut read.shows), None);
        }
        if let Some(book) = self.book_of(kept) {
            read.ties.insert((book.to_string(), Tie::Structure));
        }
        tagged(&kept.tags, &mut read.ties);
    }

    fn of_task(&self, own: &str, task: &Task, read: &mut Read) {
        let Read {
            ties, away, shows, ..
        } = read;
        let said = task
            .description
            .as_deref()
            .into_iter()
            .chain(task.log.iter().map(|entry| entry.body.as_str()))
            .chain(task.steps.iter().map(|step| step.text.as_str()));
        for text in said {
            self.written(own, text, None, (ties, shows), Some(away));
        }
        for file in self.pages_of_tasks.get(&task.id).into_iter().flatten() {
            if let Some(page) = self.docs.read(file) {
                self.written(own, &page, None, (ties, shows), Some(away));
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

    fn written(
        &self,
        own: &str,
        text: &str,
        pages: Option<&BTreeMap<&str, &str>>,
        (ties, shows): (&mut Ties, &mut Shown),
        mut away: Option<&mut Vec<String>>,
    ) {
        for link in links(text) {
            match self.aimed(&link.target) {
                Some(Aim::File(at)) => {
                    ties.insert((at, Tie::Uses));
                }
                // A book's page lines place its pages: the page says so from its side.
                Some(Aim::Paper(id))
                    if id != own
                        && !(link.card
                            && pages.is_some_and(|all| all.contains_key(id.as_str()))) =>
                {
                    let tie = if link.card { Tie::Cite } else { Tie::Mention };
                    ties.insert((id, tie));
                }
                None => {
                    if let Some(away) = away.as_deref_mut()
                        && outside(&link.target)
                        && !away.contains(&link.target)
                    {
                        away.push(link.target);
                    }
                }
                _ => {}
            }
        }
        for spot in spots(text) {
            let named = match spot.named {
                Named::Id(said) => said.parse::<TaskId>().map(|id| id.to_string()).ok(),
                Named::Wiki(title) => Some(format!("{TITLED}{title}")),
                Named::Quoted(_) => None,
            };
            if let Some(named) = named.filter(|one| one != own) {
                ties.insert((named, Tie::Mention));
            }
        }
        for (target, label) in pictures(text) {
            let Some(file) =
                crate::attach::canonical(&target).filter(|one| crate::attach::pictured(one))
            else {
                continue;
            };
            let leaf = file.rsplit('/').next().unwrap_or(&file);
            if let Some(said) = crate::attach::shows(&label, leaf) {
                shows
                    .entry(file.clone())
                    .or_insert_with(|| said.to_string());
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

fn outside(target: &str) -> bool {
    target.contains("://") || target.starts_with("mailto:")
}

pub(super) fn paper_named(named: &str) -> String {
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
