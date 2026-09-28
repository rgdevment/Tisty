pub(super) mod editing;
pub(super) mod filing;
pub(super) mod importing;
pub(super) mod paging;
pub(super) mod reading;
pub(super) mod writing;

use serde_json::{Value, json};
use tisty_core::{Paths, State};

use super::Refused;
use super::asked::text;

pub(super) const FOLDERS_AT_MOST: usize = 64;

pub(super) fn reachable_or(
    paths: &Paths,
    said: &str,
    asked: &std::path::Path,
    doing: &str,
) -> Result<std::path::PathBuf, Refused> {
    tisty_core::agent::may_reach(asked, paths).map_err(|_| {
        Refused::Tool(format!(
            "{said:?} is not somewhere an assistant may {doing}. Those are: {}.",
            tisty_core::agent::reachable()
                .iter()
                .map(|one| one.display().to_string())
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
        .filter(|one| one.file != which)
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
