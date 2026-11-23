use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::Value;
use tisty_core::brain::{Brain, Round};
use tisty_core::model::{Kept, TaskId};
use tisty_core::{Paths, State};

use super::Refused;
use super::asked::text;

mod cards;
pub(super) mod project;
mod ranking;
pub(super) mod search;
mod tally;
mod zones;

const SIFTS: [&str; 8] = [
    "tag",
    "list",
    "by_agent",
    "said_done",
    "part_of",
    "from_source",
    "from",
    "to",
];

pub(super) fn brain(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let doc = text(args, "doc");
    let task = text(args, "task");
    if doc.is_some() && task.is_some() {
        return Err(Refused::Tool(
            "`brain` looks at one thing at a time: a `doc` or a `task`, not both.".into(),
        ));
    }
    for (key, needs) in [("links", "a `doc` or a `task`"), ("section", "a `doc`")] {
        let named = match key {
            "section" => doc.is_some(),
            _ => doc.is_some() || task.is_some(),
        };
        if args.get(key).is_some() && !named {
            return Err(Refused::Tool(format!(
                "`{key}` is asked of one thing: send {needs} with it."
            )));
        }
    }
    let query = text(args, "query");
    if text(args, "source").is_some() || doc.is_some() && query.is_some() {
        return search::found(paths, args);
    }
    if let Some(doc) = doc {
        return cards::of_doc(paths, &doc, args);
    }
    if let Some(task) = task {
        return cards::of_task(paths, &task, args);
    }
    if project::asked(args) {
        return project::view(paths, args);
    }
    if query.is_some() || SIFTS.iter().any(|key| args.get(*key).is_some()) {
        return search::found(paths, args);
    }
    if text(args, "folder").is_some() || args.get("scope").is_some() {
        return zones::documents(paths, args);
    }
    zones::all(paths)
}

pub(super) fn moved(name: &str) -> Option<Refused> {
    let (into, way) = match name {
        "find" => (
            "brain",
            "`brain` with the same `query` and fields, or with `doc` and `query` for the lines of one document",
        ),
        "docs" => (
            "brain",
            "`brain` with `folder` for a folder's documents, or with nothing for the lists, folders and tags",
        ),
        "outline_doc" => (
            "brain",
            "`brain` with `doc`: its card holds the outline, the `print` and who points at it",
        ),
        "lists" | "tags" => ("brain", "`brain` with nothing, or `catch_up`"),
        "read_doc" => (
            "read",
            "`read` with the same `doc` and fields: a section, lines, `chars` or `as`",
        ),
        _ => return None,
    };
    Some(Refused::Tool(format!(
        "`{name}` moved into `{into}`: call {way}. Nothing was done."
    )))
}

pub(super) fn base(paths: &Paths) -> Option<Brain> {
    tisty_core::brain::kept_up(paths, Round::Looking)?;
    Brain::looked_at(paths.cache())
}

pub(super) struct Gate<'a> {
    state: &'a State,
    papers: HashMap<&'a str, &'a Kept>,
    root: PathBuf,
}

impl<'a> Gate<'a> {
    pub(super) fn new(paths: &Paths, state: &'a State) -> Self {
        Self {
            state,
            papers: state
                .docs
                .values()
                .map(|one| (one.file.as_str(), one))
                .collect(),
            root: paths.docs(),
        }
    }

    pub(super) fn sees(&self, id: &str) -> bool {
        if let Ok(ulid) = id.parse::<TaskId>() {
            if let Some(task) = self.state.tasks.get(&ulid) {
                return !task.folded();
            }
            return self
                .state
                .docs
                .get(&ulid)
                .is_some_and(|one| self.state.doc_in_sight(one));
        }
        match self.papers.get(id) {
            Some(kept) => self.state.doc_in_sight(kept),
            // A document the state does not hold is out of sight, as the shut diary is; a chat's name is no document.
            None => tisty_core::docs::resolve(&self.root, id).is_err(),
        }
    }
}

#[cfg(test)]
#[path = "brain_test.rs"]
mod tests;
