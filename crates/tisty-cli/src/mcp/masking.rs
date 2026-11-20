use std::collections::BTreeMap;
use std::ops::Range;

use serde_json::Value;
use tisty_core::model::TaskId;
use tisty_core::{Paths, State};

const TASK: &str = tisty_markdown::refs::TASK;
const HTML_LINK: &str = "<a href=\"tisty:task/";
const HTML_END: &str = "</a>";

// A task the agent may not see and one that is not there read alike, so neither tells the other apart.
pub(super) const OUT_OF_SIGHT: &str = "a task out of sight";

struct Spot {
    label: Range<usize>,
    id: String,
}

fn in_sight(state: &State, id: &str) -> bool {
    id.parse::<TaskId>()
        .ok()
        .and_then(|id| state.tasks.get(&id))
        .is_some_and(|task| !task.folded())
}

fn spots(text: &str) -> Vec<Spot> {
    let mut all: Vec<Spot> = linked(text).chain(anchored(text)).collect();
    all.sort_by_key(|one| one.label.start);
    all
}

fn linked(text: &str) -> impl Iterator<Item = Spot> + '_ {
    let opening = format!("]({TASK}");
    text.match_indices(opening.as_str())
        .map(|(at, _)| at)
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(move |at| {
            let from = at + opening.len();
            let id_len = text[from..].find(|c: char| !c.is_ascii_alphanumeric())?;
            if id_len == 0 || !text[from + id_len..].starts_with(')') {
                return None;
            }
            Some(Spot {
                label: label_start(&text[..at])?..at,
                id: text[from..from + id_len].to_string(),
            })
        })
}

fn label_start(before: &str) -> Option<usize> {
    let bytes = before.as_bytes();
    let mut at = bytes.len();
    let mut inner = 0;
    while at > 0 {
        at -= 1;
        match bytes[at] {
            b'\n' if at > 0 && bytes[at - 1] == b'\n' => return None,
            b']' if !escaped(bytes, at) => inner += 1,
            b'[' if !escaped(bytes, at) && inner > 0 => inner -= 1,
            b'[' if !escaped(bytes, at) => return Some(at + 1),
            _ => {}
        }
    }
    None
}

fn escaped(bytes: &[u8], at: usize) -> bool {
    bytes[..at]
        .iter()
        .rev()
        .take_while(|one| **one == b'\\')
        .count()
        % 2
        == 1
}

fn anchored(text: &str) -> impl Iterator<Item = Spot> + '_ {
    text.match_indices(HTML_LINK).filter_map(|(at, _)| {
        let from = at + HTML_LINK.len();
        let id_len = text[from..].find('"')?;
        let rest = &text[from + id_len..];
        let start = from + id_len + rest.find('>')? + 1;
        let end = start + text[start..].find(HTML_END)?;
        Some(Spot {
            label: start..end,
            id: text[from..from + id_len].to_string(),
        })
    })
}

pub(super) fn masked(state: &State, text: &str) -> Option<String> {
    let hidden: Vec<Spot> = spots(text)
        .into_iter()
        .filter(|one| !in_sight(state, &one.id) && text[one.label.clone()] != *OUT_OF_SIGHT)
        .collect();
    if hidden.is_empty() {
        return None;
    }
    let mut out = text.to_string();
    for one in hidden.iter().rev() {
        out.replace_range(one.label.clone(), OUT_OF_SIGHT);
    }
    Some(out)
}

pub(super) fn unmasked(state: &State, sent: &str, was: &str) -> Option<String> {
    let mut named: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for one in spots(was) {
        if !in_sight(state, &one.id) {
            named
                .entry(one.id.to_ascii_uppercase())
                .or_default()
                .push(&was[one.label.clone()]);
        }
    }
    if named.is_empty() {
        return None;
    }
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let mut back: Vec<(Range<usize>, &str)> = Vec::new();
    for one in spots(sent) {
        if sent[one.label.clone()] != *OUT_OF_SIGHT {
            continue;
        }
        let id = one.id.to_ascii_uppercase();
        let Some(labels) = named.get(&id) else {
            continue;
        };
        let turn = seen.entry(id).or_default();
        back.push((one.label, labels[(*turn).min(labels.len() - 1)]));
        *turn += 1;
    }
    if back.is_empty() {
        return None;
    }
    let mut out = sent.to_string();
    for (at, label) in back.iter().rev() {
        out.replace_range(at.clone(), label);
    }
    Some(out)
}

fn mentions(value: &Value) -> bool {
    match value {
        Value::String(text) => text.contains(TASK),
        Value::Array(many) => many.iter().any(mentions),
        Value::Object(all) => all.values().any(mentions),
        _ => false,
    }
}

fn masked_all(state: &State, value: &mut Value) {
    match value {
        Value::String(text) => {
            if let Some(out) = masked(state, text) {
                *text = out;
            }
        }
        Value::Array(many) => many.iter_mut().for_each(|one| masked_all(state, one)),
        Value::Object(all) => {
            let aims_out_of_sight = all
                .get("target")
                .and_then(Value::as_str)
                .and_then(|to| to.strip_prefix(TASK))
                .is_some_and(|id| !in_sight(state, id));
            if aims_out_of_sight && all.contains_key("label") {
                all.insert("label".into(), Value::String(OUT_OF_SIGHT.into()));
            }
            all.values_mut().for_each(|one| masked_all(state, one));
        }
        _ => {}
    }
}

pub(super) fn text_masked(paths: &Paths, text: String) -> String {
    if !text.contains(TASK) {
        return text;
    }
    let Ok(state) = tisty_core::cache::project(&paths.store(), paths.cache()) else {
        return text;
    };
    masked(&state, &text).unwrap_or(text)
}

pub(super) fn answer_masked(paths: &Paths, mut said: Value) -> Value {
    if !mentions(&said) {
        return said;
    }
    let Ok(state) = tisty_core::cache::project(&paths.store(), paths.cache()) else {
        return said;
    };
    masked_all(&state, &mut said);
    said
}

// Tisty's own guides name example tasks that were never there, and must reach the agent as written.
pub(super) fn reads_the_persons_words(name: &str) -> bool {
    !matches!(name, "rules" | "help" | "language")
}

fn rewritten_keys(name: &str) -> &'static [&'static str] {
    match name {
        "write_doc" | "edit_doc" | "append_doc" => &["body", "old", "new"],
        "describe" | "reword_note" => &["body"],
        "reword_step" => &["text"],
        _ => &[],
    }
}

pub(super) fn args_unmasked(paths: &Paths, name: &str, mut args: Value) -> Value {
    let keys = rewritten_keys(name);
    let sends_a_mask = keys
        .iter()
        .filter_map(|key| args.get(*key).and_then(Value::as_str))
        .any(|text| text.contains(OUT_OF_SIGHT));
    if !sends_a_mask {
        return args;
    }
    let Ok(state) = tisty_core::cache::project(&paths.store(), paths.cache()) else {
        return args;
    };
    let Some(was) = rewritten(paths, &state, name, &args) else {
        return args;
    };
    for key in keys {
        let Some(sent) = args.get(*key).and_then(Value::as_str) else {
            continue;
        };
        if let Some(back) = unmasked(&state, sent, &was) {
            args[*key] = Value::String(back);
        }
    }
    args
}

fn rewritten(paths: &Paths, state: &State, name: &str, args: &Value) -> Option<String> {
    let said = |key: &str| args.get(key).and_then(Value::as_str);
    if matches!(name, "write_doc" | "edit_doc" | "append_doc") {
        let which = said("doc")?;
        let file = match which.parse::<TaskId>() {
            Ok(task) => {
                let owner = tisty_core::task_docs::owner(state, task);
                tisty_core::task_docs::of(state, owner)?.file.clone()
            }
            Err(_) => which.to_string(),
        };
        return tisty_core::docs::read(&paths.docs(), &file).ok();
    }
    let task = state.tasks.get(&said("task")?.parse::<TaskId>().ok()?)?;
    let one_of = |all: Vec<(String, &str)>, key: &str| match said(key)
        .and_then(|asked| all.iter().find(|(id, _)| id == asked))
    {
        Some((_, text)) => text.to_string(),
        None => all
            .iter()
            .map(|(_, text)| *text)
            .collect::<Vec<_>>()
            .join("\n"),
    };
    Some(match name {
        "describe" => task.description.clone().unwrap_or_default(),
        "reword_note" => one_of(
            task.log
                .iter()
                .map(|one| (one.id.to_string(), one.body.as_str()))
                .collect(),
            "note",
        ),
        _ => one_of(
            task.steps
                .iter()
                .map(|one| (one.id.to_string(), one.text.as_str()))
                .collect(),
            "step",
        ),
    })
}

#[cfg(test)]
#[path = "masking_test.rs"]
mod tests;
