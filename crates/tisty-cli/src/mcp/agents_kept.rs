use serde_json::Value;
use tisty_core::{Paths, State, agents, model::FolderId, model::Kept};

use super::Refused;
use super::asked::text;
use super::mistaken::{NAMES_A_DOCUMENT, NAMES_A_FOLDER};

pub(super) fn proposed_or_left(
    paths: &Paths,
    name: &str,
    args: &Value,
) -> Result<Option<Value>, Refused> {
    if super::only_reads(name, &serde_json::json!({ "arguments": args })) {
        return Ok(None);
    }
    let state = tisty_core::cache::project(&paths.store(), paths.cache()).map_err(super::hitch)?;
    let mut named = documents_named(args).filter_map(|said| kept_in_agents(&state, said));
    if let Some((kept, exactly)) = named.next() {
        if exactly && super::proposing::takes(name, args) {
            return super::proposing::proposed(paths, &state, name, args, &kept.file).map(Some);
        }
        return Err(document_kept(kept));
    }
    let into_agents = places_named(name, args).any(|said| place_in_agents(&state, &said))
        || named_alone(name, args).is_some_and(|said| meant_in_agents(&state, &said));
    if into_agents || makes_agents(name, args) {
        return Err(folder_kept());
    }
    Ok(None)
}

pub(super) fn left_out(state: &State, kept: &Kept) -> Result<(), Refused> {
    match agents::holds(state, kept) {
        true => Err(document_kept(kept)),
        false => Ok(()),
    }
}

pub(super) fn folder_left_out(
    state: &State,
    at: Option<FolderId>,
    named: &str,
) -> Result<(), Refused> {
    let inside = at.is_some_and(|at| agents::holds_folder(state, at));
    match inside || loosely(named) == loosely(agents::FOLDER) {
        true => Err(folder_kept()),
        false => Ok(()),
    }
}

const PROPOSE: &str = "To change its text, propose it with `doc_edit`: `mode: \"rewrite\"` with \
                       `print` and full `content`, `mode: \"patch\"`, or `mode: \"append\"` at \
                       its end, and nothing else with them. The person approves or discards it \
                       on the document's card.";
const TELL: &str = "Tell the person what you would change and why, and they make the change \
                    from the document's card.";

fn document_kept(kept: &Kept) -> Refused {
    let title = kept.title.clone().unwrap_or_else(|| kept.file.clone());
    refused(&format!("the document {title:?}"), PROPOSE)
}

fn folder_kept() -> Refused {
    refused(&format!("the folder {}", agents::FOLDER), TELL)
}

fn refused(what: &str, then: &str) -> Refused {
    Refused::Tool(format!(
        "{what} holds an assistant's own files in AGENTS, which only the person changes, in Tisty: \
         nothing an assistant moves, files, marks or sums up there reaches them. {then}"
    ))
}

fn documents_named(args: &Value) -> impl Iterator<Item = &str> {
    NAMES_A_DOCUMENT
        .iter()
        .filter_map(|key| args.get(*key))
        .flat_map(|said| match said {
            Value::Array(many) => many.iter().collect(),
            one => vec![one],
        })
        .filter_map(Value::as_str)
}

// Only the exact name may become a proposal; a loose likeness is refused all the same.
fn kept_in_agents<'a>(state: &'a State, said: &str) -> Option<(&'a Kept, bool)> {
    let meant = meant_document(said);
    state
        .docs
        .values()
        .filter_map(|one| {
            let exactly = one.file == said;
            (exactly || loosely(&one.file) == meant).then_some((one, exactly))
        })
        .find(|(one, _)| agents::holds(state, one))
}

fn meant_document(said: &str) -> String {
    let plain = loosely(said);
    match tisty_core::address::doc_asked(&plain) {
        Some(id) => id.to_string(),
        None => plain,
    }
}

fn meant_in_agents(state: &State, said: &str) -> bool {
    let meant = loosely(said);
    agents::trees(state).into_iter().any(|at| {
        state.folders.get(&at).is_some_and(|one| {
            [
                one.id.to_string(),
                one.name.clone(),
                state.trail(at).join("/"),
            ]
            .iter()
            .any(|name| loosely(name) == meant)
        })
    })
}

// The tools resolve a place with `folder_named` on this same value; only where it fails is a likeness enough.
fn place_in_agents(state: &State, said: &str) -> bool {
    match super::folder_named(state, said) {
        Ok(at) => agents::holds_folder(state, at),
        Err(_) => meant_in_agents(state, said),
    }
}

fn creating_folder(name: &str, args: &Value) -> bool {
    name == "doc_organize" && text(args, "action").as_deref() == Some("create_folder")
}

fn places_named(name: &str, args: &Value) -> impl Iterator<Item = String> {
    let given = NAMES_A_FOLDER.iter().filter_map(|key| text(args, key));
    let under = match creating_folder(name, args) {
        true => super::papers::filing::folder_asked(args)
            .ok()
            .and_then(|asked| asked.under),
        false => None,
    };
    given.chain(under)
}

// `create_folder` looks for a name sent alone at every depth, so that name must not be one in AGENTS.
fn named_alone(name: &str, args: &Value) -> Option<String> {
    let asked = creating_folder(name, args)
        .then(|| super::papers::filing::folder_asked(args).ok())
        .flatten()?;
    (asked.under.is_none() && text(args, "folder").is_none()).then_some(asked.name)
}

fn makes_agents(name: &str, args: &Value) -> bool {
    creating_folder(name, args)
        && super::papers::filing::folder_asked(args)
            .is_ok_and(|asked| loosely(&asked.name) == loosely(agents::FOLDER))
}

// Two names that read alike once blanks, controls, invisible marks, case and accents are gone.
fn loosely(said: &str) -> String {
    let kept: String = said
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control() && !invisible(*c))
        .collect();
    tisty_core::text::folded(&kept)
}

fn invisible(c: char) -> bool {
    matches!(
        c,
        '\u{00ad}'
            | '\u{034f}'
            | '\u{061c}'
            | '\u{180e}'
            | '\u{200b}'..='\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{206f}'
            | '\u{fe00}'..='\u{fe0f}'
            | '\u{feff}'
            | '\u{fff9}'..='\u{fffb}'
    )
}

#[cfg(test)]
#[path = "agents_kept_test.rs"]
mod tests;
