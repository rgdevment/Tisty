use serde_json::Value;
use tisty_core::{Paths, State, agents};

use super::Refused;
use super::mistaken::NAMES_A_DOCUMENT;

const NAMES_A_FOLDER: [&str; 2] = ["folder", "inside"];

pub(super) fn left_to_the_person(paths: &Paths, name: &str, args: &Value) -> Result<(), Refused> {
    if super::only_reads(name, &serde_json::json!({ "arguments": args })) {
        return Ok(());
    }
    let state = tisty_core::cache::project(&paths.store(), paths.cache()).map_err(super::hitch)?;
    if let Some(title) = documents_named(args).find_map(|file| kept_in_agents(&state, file)) {
        return Err(refused(&format!("the document {title:?}")));
    }
    let into_agents = folders_named(name, args).any(|said| {
        super::folder_named(&state, &said).is_ok_and(|at| agents::holds_folder(&state, at))
    });
    if into_agents || makes_agents(name, args) {
        return Err(refused(&format!("the folder {}", agents::FOLDER)));
    }
    Ok(())
}

fn refused(what: &str) -> Refused {
    Refused::Tool(format!(
        "{what} holds an assistant's own files in AGENTS, which only the person changes, in Tisty: \
         nothing an assistant writes, moves or marks there reaches them. Tell the person what \
         you would change and why, and they make the change from the document's card."
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

fn kept_in_agents(state: &State, file: &str) -> Option<String> {
    let kept = state.docs.values().find(|one| one.file == file)?;
    agents::holds(state, kept).then(|| kept.title.clone().unwrap_or_else(|| file.to_string()))
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
