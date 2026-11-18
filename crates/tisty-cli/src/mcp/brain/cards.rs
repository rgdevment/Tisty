use serde_json::{Map, Value, json};
use tisty_core::brain::{Brain, Card};
use tisty_core::model::TaskId;
use tisty_core::{Paths, State};

use super::super::asked::{how_many, text};
use super::super::jsonrpc::mirrored;
use super::super::looking::{brief, gone};
use super::super::mistaken::{no_document_called, not_a_task_id};
use super::super::papers::reading::{in_sight, outlined, walked};
use super::super::{Refused, opened};
use super::project::{day, last_moved, stage, state_line};
use super::{Gate, base};

const NEAR_SHOWN: usize = 8;
const LINKS_BY_DEFAULT: u64 = 50;
const LINKS_AT_MOST: u64 = 100;
const DECISIONS_SHOWN: usize = 12;
const MANY: [&str; 12] = [
    "links",
    "pages",
    "languages",
    "parts",
    "born",
    "ideas",
    "tags",
    "files",
    "cites",
    "mentions",
    "cited_by",
    "mentioned_by",
];
const OUTLINE_SAYS: [&str; 11] = [
    "words",
    "id",
    "kind",
    "title",
    "state",
    "sections",
    "book",
    "pages",
    "languages",
    "bytes",
    "lines",
];
const NO_BASE: &str = "The base that links things is not ready on this machine, so this card holds what the thing says alone.";

fn of_kind(kind: &str) -> &'static [&'static str] {
    match kind {
        "cite" => &["cites", "cited_by"],
        "mention" => &["mentions", "mentioned_by"],
        "structure" => &[
            "book",
            "pages",
            "languages",
            "part_of",
            "parts",
            "origin",
            "born",
            "ideas_from",
            "ideas",
        ],
        "tag" => &["tags"],
        "uses" => &["files"],
        _ => &[
            "book",
            "pages",
            "languages",
            "part_of",
            "parts",
            "origin",
            "born",
            "ideas_from",
            "ideas",
            "tags",
            "files",
            "cites",
            "mentions",
            "cited_by",
            "mentioned_by",
        ],
    }
}

fn card_of(paths: &Paths, state: &State, base: Option<&Brain>, id: &str) -> Option<Card> {
    let gate = Gate::new(paths, state);
    let today = jiff::Zoned::now().date();
    base?.card(id, today, |one| gate.sees(one)).ok().flatten()
}

pub(super) fn of_doc(paths: &Paths, which: &str, args: &Value) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    if let Some(at) = args.get("section") {
        let Some(at) = at.as_u64() else {
            return Err(Refused::Tool(
                "`section` takes the `at` of a heading on the document's card, a whole number."
                    .into(),
            ));
        };
        return walked(paths, &state, which, at as usize);
    }
    if let Some(kind) = text(args, "links") {
        let Some(kept) = state.docs.values().find(|one| one.file == which) else {
            return Err(Refused::Missing(no_document_called(&state, which)));
        };
        in_sight(&state, kept, which)?;
        let held = base(paths);
        let card = card_of(paths, &state, held.as_ref(), &node_of(which));
        return linked(which, card, &kind, args);
    }
    let held = base(paths);
    let mut answer = outlined(paths, &state, which, held.as_ref())?;
    match card_of(paths, &state, held.as_ref(), &node_of(which)) {
        Some(card) => with_neighbours(&mut answer, card),
        None if held.is_none() => answer["structuredContent"]["base"] = json!(NO_BASE),
        None => {}
    }
    Ok(answer)
}

// A task's page is no node of its own: its links are its task's, so its card shows the task's neighbours.
fn node_of(which: &str) -> String {
    tisty_core::task_docs::task_of_file(which)
        .map_or_else(|| which.to_string(), |task| task.to_string())
}

fn with_neighbours(answer: &mut Value, card: Card) {
    let mut said = to_map(&card);
    if card.kind == "task" {
        let near = of_kind("all");
        said.retain(|key, _| near.contains(&key.as_str()));
    }
    for key in OUTLINE_SAYS {
        said.remove(key);
    }
    let told = neighbours_told(&said);
    let capped = capped(&mut said);
    let Some(shown) = answer["structuredContent"].as_object_mut() else {
        return;
    };
    if said.contains_key("cited_by") || said.contains_key("mentioned_by") {
        shown.remove("pointed_at");
    }
    shown.extend(said);
    if capped {
        shown.insert(
            "next".into(),
            json!("`links` with a kind brings every neighbour of that kind."),
        );
    }
    if let Some(text) = answer["content"][0]["text"].as_str() {
        answer["content"][0]["text"] = json!(format!("{text}{told}"));
    }
}

pub(super) fn of_task(paths: &Paths, which: &str, args: &Value) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let Ok(id) = which.parse::<TaskId>() else {
        return Err(Refused::Tool(not_a_task_id(&state, which)));
    };
    let Some(task) = state.tasks.get(&id).filter(|one| !one.folded()) else {
        if state.is_erased(id) {
            return Err(gone(&state, which));
        }
        return Err(Refused::Tool(format!(
            "no task here has the id {which}. Look it up again with `brain` and a `query`."
        )));
    };
    let held = base(paths);
    let card = card_of(paths, &state, held.as_ref(), which);
    if let Some(kind) = text(args, "links") {
        return linked(which, card, &kind, args);
    }
    let today = jiff::Zoned::now().date();
    let line = state_line(paths, &state, task);
    let (mut said, told) = match card {
        Some(card) => {
            let mut said = to_map(&card);
            said.remove("sections");
            if let Some(status) = said.remove("state") {
                said.insert("status".into(), status);
            }
            let told = neighbours_told(&said);
            if capped(&mut said) {
                said.insert(
                    "next".into(),
                    json!("`links` with a kind brings every neighbour of that kind."),
                );
            }
            (said, told)
        }
        None => {
            let mut said = brief(task, &state, today)
                .as_object()
                .cloned()
                .unwrap_or_default();
            if held.is_none() {
                said.insert("base".into(), json!(NO_BASE));
            }
            (said, String::new())
        }
    };
    if task.is_open() {
        said.insert("stage".into(), json!(stage(task, line.as_deref()).said()));
    }
    said.insert("moved".into(), json!(day(last_moved(task))));
    if let Some(line) = &line {
        said.insert("state".into(), json!(line));
        if tisty_core::task_state::since(task).is_some_and(|since| since.old()) {
            said.insert("state_old".into(), json!(true));
        }
    }
    said.entry("next")
        .or_insert_with(|| json!("`read` with `task` brings its steps, page and notes."));
    let text = format!(
        "{:?}, {}{}{}",
        task.title,
        said.get("status")
            .and_then(Value::as_str)
            .unwrap_or_else(|| super::super::named(task.status)),
        line.map(|one| format!("\nState: {one}"))
            .unwrap_or_default(),
        told
    );
    Ok(mirrored(text, Value::Object(said)))
}

fn linked(which: &str, card: Option<Card>, kind: &str, args: &Value) -> Result<Value, Refused> {
    let Some(card) = card else {
        return Err(Refused::Tool(format!(
            "{NO_BASE} Its links are in its text: `read_doc` or `read` with {which}."
        )));
    };
    let most = how_many(args, LINKS_BY_DEFAULT, LINKS_AT_MOST);
    let all = to_map(&card);
    let mut kept = Map::new();
    kept.insert("id".into(), json!(card.id));
    kept.insert("title".into(), json!(card.title));
    kept.insert("links".into(), json!(kind));
    let mut lines = Vec::new();
    for key in of_kind(kind) {
        match all.get(*key) {
            Some(Value::Array(many)) => {
                kept.insert(
                    (*key).into(),
                    json!(many.iter().take(most).collect::<Vec<_>>()),
                );
                if many.len() > most {
                    kept.insert(format!("{key}_more"), json!(many.len() - most));
                }
                lines.push(format!(
                    "{}: {}",
                    key.replace('_', " "),
                    many.iter()
                        .take(most)
                        .map(near_told)
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
            Some(one @ Value::Object(_)) => {
                kept.insert((*key).into(), one.clone());
                lines.push(format!("{}: {}", key.replace('_', " "), near_told(one)));
            }
            _ => {}
        }
    }
    let text = match lines.is_empty() {
        true => format!("{:?} has no neighbours of that kind.", card.title),
        false => format!("{:?}\n{}", card.title, lines.join("\n")),
    };
    Ok(mirrored(text, Value::Object(kept)))
}

fn to_map(card: &Card) -> Map<String, Value> {
    match serde_json::to_value(card) {
        Ok(Value::Object(said)) => said,
        _ => Map::new(),
    }
}

fn capped(said: &mut Map<String, Value>) -> bool {
    let mut cut = false;
    for key in MANY {
        let Some(Value::Array(many)) = said.get_mut(key) else {
            continue;
        };
        if many.len() > NEAR_SHOWN {
            let more = many.len() - NEAR_SHOWN;
            many.truncate(NEAR_SHOWN);
            said.insert(format!("{key}_more"), json!(more));
            cut = true;
        }
    }
    if let Some(Value::Array(decisions)) = said.get_mut("decisions")
        && decisions.len() > DECISIONS_SHOWN
    {
        let more = decisions.len() - DECISIONS_SHOWN;
        decisions.truncate(DECISIONS_SHOWN);
        let before = said
            .get("decisions_more")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        said.insert("decisions_more".into(), json!(before + more as u64));
    }
    cut
}

fn neighbours_told(said: &Map<String, Value>) -> String {
    let counted: Vec<String> = MANY
        .iter()
        .filter_map(|key| match said.get(*key) {
            Some(Value::Array(many)) if !many.is_empty() => {
                Some(format!("{} {}", key.replace('_', " "), many.len()))
            }
            _ => None,
        })
        .collect();
    match counted.is_empty() {
        true => String::new(),
        false => format!("\n\nNeighbours: {}.", counted.join(", ")),
    }
}

fn near_told(one: &Value) -> String {
    match one {
        Value::Object(near) => {
            let title = near
                .get("title")
                .or_else(|| near.get("file"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let id = near
                .get("id")
                .or_else(|| near.get("doc"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            match near.get("state").and_then(Value::as_str) {
                Some(state) => format!("{title:?} [{id}] ({state})"),
                None => format!("{title:?} [{id}]"),
            }
        }
        Value::String(one) => one.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
#[path = "cards_test.rs"]
mod tests;
