use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::Value;
use tisty_core::brain::{Brain, Round};
use tisty_core::model::{Kept, TaskId};
use tisty_core::{Paths, State};

use super::Refused;
use super::asked::text;

pub(super) mod cards;
pub(super) mod project;
mod ranking;
pub(super) mod search;
mod tally;
pub(super) mod zones;

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
        "catch_up" => (
            "workspace_catch_up",
            "`workspace_catch_up` with the same `since`",
        ),
        "brain" | "find" => (
            "workspace_search",
            "`workspace_search` (for cards/links use `workspace_read` with `view: \"card\"` or `\"links\"`)",
        ),
        "docs" => (
            "workspace_search",
            "`workspace_search` with `folder` for folder docs, or `{}` for lists, folders and tags",
        ),
        "outline_doc" => (
            "workspace_read",
            "`workspace_read` with `doc` and `view: \"card\"`",
        ),
        "lists" | "tags" => (
            "workspace_search",
            "`workspace_search` with `{}` or `mode: \"taxonomy\"`, or `workspace_catch_up`",
        ),
        "read" | "read_doc" => (
            "workspace_read",
            "`workspace_read` with `task` or `doc` and optional `view`",
        ),
        "rules" => (
            "workspace_guide",
            "`workspace_guide` with `kind: \"rules\"`",
        ),
        "language" => (
            "workspace_guide",
            "`workspace_guide` with `kind: \"markdown\"`",
        ),
        "help" => ("workspace_guide", "`workspace_guide` with `kind: \"help\"`"),
        "propose" => ("task_create", "`task_create` with the same fields"),
        "describe" => ("task_update", "`task_update` with `task` and `description`"),
        "rename" => ("task_update", "`task_update` with `task` and `title`"),
        "reschedule" => (
            "task_update",
            "`task_update` with `task`, `date`, or `deadline`",
        ),
        "remind" => (
            "task_update",
            "`task_update` with `task` and `add_reminders`",
        ),
        "move" => (
            "task_update",
            "`task_update` with `task`, `list`, or `part_of`",
        ),
        "plan" => (
            "task_edit_steps",
            "`task_edit_steps` with `action: \"add\"`",
        ),
        "tick" => (
            "task_edit_steps",
            "`task_edit_steps` with `action: \"tick\"` (or `\"set_state\"`)",
        ),
        "untick" => (
            "task_edit_steps",
            "`task_edit_steps` with `action: \"untick\"`",
        ),
        "reword_step" => (
            "task_edit_steps",
            "`task_edit_steps` with `action: \"reword\"`",
        ),
        "unplan" => (
            "task_edit_steps",
            "`task_edit_steps` with `action: \"remove\"`",
        ),
        "note" => ("task_note", "`task_note` with `action: \"add\"`"),
        "reword_note" => ("task_note", "`task_note` with `action: \"reword\"`"),
        "say_done" => ("task_resolve", "`task_resolve` with `outcome: \"done\"`"),
        "say_not_doing" => (
            "task_resolve",
            "`task_resolve` with `outcome: \"not_doing\"`",
        ),
        "write_doc" => (
            "doc_create",
            "`doc_create` for a new doc, or `doc_edit` with `mode: \"rewrite\"` / `\"compact\"`",
        ),
        "edit_doc" => (
            "doc_edit",
            "`doc_edit` with `mode: \"patch\"` and `content`",
        ),
        "append_doc" => (
            "doc_edit",
            "`doc_edit` with `mode: \"append\"` (or `\"digest_task_notes\"` for task-page digest)",
        ),
        "restore_doc" => ("doc_edit", "`doc_edit` with `mode: \"restore\"`"),
        "edit_table" => (
            "doc_edit_table",
            "`workspace_read` with `view: \"table\"` to read, or `doc_edit_table` with `action` to write",
        ),
        "folder" => (
            "doc_organize",
            "`doc_organize` with `action: \"create_folder\"`",
        ),
        "file_doc" => (
            "doc_organize",
            "`doc_organize` with `action: \"file\"` and `docs`",
        ),
        "page_doc" => (
            "doc_organize",
            "`doc_organize` with `action: \"add_page\"`, `\"remove_page\"`, or `\"reorder_pages\"`",
        ),
        "tongue_doc" => (
            "doc_organize",
            "`doc_organize` with `action: \"link_translation\"` or `\"unlink_translation\"`",
        ),
        "archive_doc" => (
            "doc_annotate",
            "`doc_annotate` with `action: \"archive\"` or `\"unarchive\"` and `docs`",
        ),
        "flag_doc" => (
            "doc_annotate",
            "`doc_annotate` with `action: \"flag\"` and `docs`",
        ),
        "sum_up" => (
            "doc_annotate",
            "`doc_annotate` with `action: \"summarize\"` and `docs`",
        ),
        "attach" => ("file_transfer", "`file_transfer` with `action: \"attach\"`"),
        "import_doc" => (
            "file_transfer",
            "`file_transfer` with `action: \"import_doc\"`",
        ),
        "export_doc" => (
            "file_transfer",
            "`file_transfer` with `action: \"export_doc\"` and `path`",
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
