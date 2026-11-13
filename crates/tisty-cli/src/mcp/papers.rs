pub(super) mod alike;
pub(super) mod compacting;
pub(super) mod editing;
pub(super) mod filing;
pub(super) mod flagging;
pub(super) mod importing;
pub(super) mod listing;
pub(super) mod outward;
pub(super) mod paging;
pub(super) mod reading;
pub(super) mod tabling;
pub(super) mod task_pages;
pub(super) mod tongues;
pub(super) mod writing;

use serde_json::{Value, json};
use tisty_core::{Paths, State, Store};

use super::Refused;
use super::asked::text;

pub(super) const FOLDERS_AT_MOST: usize = 64;

// A put-away document is the person's to bring back, and a second one on its subject would drift.
pub(super) fn put_away(state: &State, kept: &tisty_core::model::Kept, named: &str) -> Refused {
    Refused::Tool(format!(
        "{named} is put away, so nothing was written. If it should change, tell the person: {}. \
         If its subject moved on, `find` the document that covers it now and write there.",
        brought_back(state, kept)
    ))
}

pub(super) fn not_a_page_while_away(
    state: &State,
    kept: &tisty_core::model::Kept,
    named: &str,
) -> Refused {
    Refused::Tool(format!(
        "{named} is put away, and as a page it would leave the archive with no way back, so \
         nothing was moved. If it should become a page, tell the person: {}.",
        brought_back(state, kept)
    ))
}

fn brought_back(state: &State, kept: &tisty_core::model::Kept) -> String {
    if state.closed_with_its_task(kept) {
        return "it is the page of a closed task, which they reopen if the work came back".into();
    }
    match (state.held_by_another(kept), kept.archived) {
        (true, true) => format!(
            "they bring back {} and then this document, from the window",
            super::holder(state, kept)
        ),
        (true, false) => format!(
            "they bring back {}, from the window",
            super::holder(state, kept)
        ),
        (false, _) => {
            "they bring it back, or ask you to with `archive_doc` and `archived` false".into()
        }
    }
}

pub(super) fn still_arriving(state: &State, which: &str) -> Refused {
    Refused::Missing(format!(
        "{} is named in the log but its text is not on this machine yet. It may still be \
         arriving from another one.",
        super::doc_named(state, which)
    ))
}

pub(super) fn reachable_or(
    paths: &Paths,
    said: &str,
    asked: &std::path::Path,
    doing: &str,
) -> Result<std::path::PathBuf, Refused> {
    tisty_core::agent::may_reach(asked, paths).map_err(|_| {
        Refused::Tool(format!(
            "{said:?} is not somewhere an assistant may {doing}. Those are the person's {} \
             folders, and the temporary folder.",
            tisty_core::agent::reachable()
                .iter()
                .skip(1)
                .filter_map(|one| one
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned()))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    })
}

/// Nothing indexes which document points at which, so the only way to say is to look. Worth the
/// reading: a link left hanging says nothing about being broken.
pub(super) fn pointed_at(paths: &Paths, state: &State, which: &str) -> Vec<String> {
    let named: Vec<String> = state
        .docs
        .values()
        .filter(|one| one.file != which && state.doc_in_sight(one))
        .map(|one| one.file.clone())
        .collect();
    // Only a body with a link or a picture in it can be pointing anywhere, and the cards say
    // which those are without reading one byte of the rest.
    let held = tisty_core::cache::Cache::open(paths.cache()).ok().flatten();
    let cards = tisty_core::docs::cards_of(&paths.docs(), held.as_ref(), &named);
    let mut found: Vec<String> = named
        .into_iter()
        .filter(|one| {
            cards
                .get(one)
                .is_none_or(|card| card.links > 0 || card.pictures > 0)
        })
        .filter(|one| {
            // Pointing at a document is any reference to it, card or mention alike: this answers
            // what would be left hanging, not what reads it as a chapter.
            tisty_core::docs::read(&paths.docs(), one)
                .map(|body| {
                    tisty_core::refs::extract(&body)
                        .iter()
                        .filter_map(|one| one.target.strip_prefix(tisty_core::refs::DOC))
                        .any(|at| at == which)
                })
                .unwrap_or(false)
        })
        .collect();
    found.sort();
    found
}

pub(super) fn many_docs(args: &Value, what: &str) -> Result<(Vec<String>, bool), Refused> {
    match args.get("doc") {
        Some(Value::Array(many)) => {
            if many.is_empty() {
                return Err(Refused::Tool(format!(
                    "`doc` came as an empty list, so there is nothing to {what}. Send one name, or several."
                )));
            }
            let mut named = Vec::new();
            for one in many {
                match one.as_str().map(str::trim).filter(|said| !said.is_empty()) {
                    Some(said) => named.push(said.to_string()),
                    None => {
                        return Err(Refused::Tool(format!(
                            "{one} is not a document name, and a list is taken whole or not at all, so nothing moved. `docs` lists the names."
                        )));
                    }
                }
            }
            let mut once: Vec<String> = Vec::with_capacity(named.len());
            for one in named {
                if !once.contains(&one) {
                    once.push(one);
                }
            }
            Ok((once, true))
        }
        _ => match text(args, "doc") {
            Some(one) => Ok((vec![one], false)),
            None => Err(Refused::Tool(format!(
                "to {what} a document, name it in `doc`."
            ))),
        },
    }
}

pub(super) fn said_docs(many: &[String], listed: bool) -> Value {
    match listed {
        true => json!(many),
        false => json!(many.first().cloned().unwrap_or_default()),
    }
}

// A page that took a new heading shows it where its document lists it, and the agent is told.
pub(super) fn named_anew(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    page: &str,
    whole: &str,
) -> String {
    let title = tisty_core::docs::titled(whole);
    let Some(up) = tisty_core::docs::book_to_relabel(state, page, &title) else {
        return String::new();
    };
    let Ok(Some(now)) = tisty_core::docs::relabel_page(&paths.docs(), &up.file, page, &title)
    else {
        return String::new();
    };
    let unsettled = match super::retold(state, store, &up.file, &now) {
        Ok(()) => "",
        Err(_) => " That document's record settles the next time it is written or opened.",
    };
    format!(
        " Its line in {:?} now shows the new title, so a print you hold of that one is old.{unsettled}",
        tisty_core::docs::titled(&now)
    )
}

const WRITES_DOCUMENTS: [&str; 10] = [
    "write_doc",
    "append_doc",
    "edit_doc",
    "edit_table",
    "restore_doc",
    "archive_doc",
    "flag_doc",
    "file_doc",
    "page_doc",
    "attach",
];

// A task's page is written like a document, but it ends, moves and holds files as its task does.
pub(super) fn not_a_task_s_words(name: &str, args: &Value) -> Result<(), Refused> {
    if !WRITES_DOCUMENTS.contains(&name) || task_pages::page_named(name, args).is_some() {
        return Ok(());
    }
    let named = ["doc", "page_of"]
        .iter()
        .filter_map(|key| args.get(*key))
        .flat_map(|one| match one {
            Value::Array(all) => all.iter().filter_map(Value::as_str).collect::<Vec<_>>(),
            other => other.as_str().into_iter().collect(),
        })
        .map(str::trim)
        .find(|file| tisty_core::task_docs::task_of_file(file).is_some());
    match named {
        Some(file) => Err(Refused::Tool(format!(
            "{file} is a task's page: write it with `edit_doc`, `edit_table`, `write_doc` or `append_doc` by its own name. It is no document to put away, mark, file or hang as a page: the task ends with `say_done` or `say_not_doing`, moves with `move`, and keeps files with `attach` and the task's id."
        ))),
        None => Ok(()),
    }
}
