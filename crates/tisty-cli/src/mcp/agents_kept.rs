use serde_json::Value;
use tisty_core::{Paths, State, agents, model::Kept};

use super::Refused;
use super::mistaken::NAMES_A_DOCUMENT;

const NAMES_A_FOLDER: [&str; 2] = ["folder", "inside"];

pub(super) fn proposed_or_left(
    paths: &Paths,
    name: &str,
    args: &Value,
) -> Result<Option<Value>, Refused> {
    if super::only_reads(name, &serde_json::json!({ "arguments": args })) {
        return Ok(None);
    }
    let state = tisty_core::cache::project(&paths.store(), paths.cache()).map_err(super::hitch)?;
    if let Some(kept) = documents_named(args).find_map(|file| kept_in_agents(&state, file)) {
        if super::proposing::takes(name, args) {
            return super::proposing::proposed(paths, &state, name, args, &kept.file).map(Some);
        }
        let title = kept.title.clone().unwrap_or_else(|| kept.file.clone());
        return Err(refused(&format!("the document {title:?}"), PROPOSE));
    }
    let into_agents = folders_named(name, args).any(|said| {
        super::folder_named(&state, &said).is_ok_and(|at| agents::holds_folder(&state, at))
    });
    if into_agents || makes_agents(name, args) {
        return Err(refused(&format!("the folder {}", agents::FOLDER), TELL));
    }
    Ok(None)
}

const PROPOSE: &str = "To change its text, propose it: `write_doc` with `doc`, `print` and the \
                       whole `body`, `edit_doc`, or `append_doc` at its end, and nothing else \
                       with them. The person approves or discards it on the document's card.";
const TELL: &str = "Tell the person what you would change and why, and they make the change \
                    from the document's card.";

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

fn kept_in_agents<'a>(state: &'a State, file: &str) -> Option<&'a Kept> {
    let kept = state.docs.values().find(|one| one.file == file)?;
    agents::holds(state, kept).then_some(kept)
}

// The folder tool names its place in `name` as a path, and the last step is the folder itself.
fn folders_named(name: &str, args: &Value) -> impl Iterator<Item = String> {
    let given = NAMES_A_FOLDER.iter().filter_map(|key| args[*key].as_str());
    let path: Vec<&str> = match name {
        "folder" => steps(args),
        _ => Vec::new(),
    };
    let under = (path.len() > 1).then(|| path[..path.len() - 1].join(" / "));
    let whole = (!path.is_empty()).then(|| path.join(" / "));
    given.map(str::to_string).chain(under).chain(whole)
}

fn steps(args: &Value) -> Vec<&str> {
    args["name"]
        .as_str()
        .unwrap_or_default()
        .split('/')
        .map(str::trim)
        .filter(|one| !one.is_empty())
        .collect()
}

fn makes_agents(name: &str, args: &Value) -> bool {
    name == "folder"
        && steps(args)
            .last()
            .is_some_and(|last| last.eq_ignore_ascii_case(agents::FOLDER))
}
