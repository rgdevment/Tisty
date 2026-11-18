use serde_json::{Value, json};
use tisty_core::{Paths, State};

use super::super::jsonrpc::mirrored;
use super::super::looking::counted_tags;
use super::super::{Refused, opened, papers, trail};

const TAGS_SHOWN: usize = 40;

pub(super) fn documents(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    papers::listing::docs(paths, args)
}

pub(super) fn all(paths: &Paths) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let lists = lists(&state);
    let folders = folders(&state);
    let tags = counted_tags(&state);
    let inbox = state
        .tasks
        .values()
        .filter(|one| one.list.is_none() && in_sight(one))
        .count();
    let shown_tags: Vec<Value> = tags
        .iter()
        .take(TAGS_SHOWN)
        .map(|(tag, times)| json!({ "tag": tag, "times": times }))
        .collect();
    let mut kept = json!({
        "lists": lists,
        "inbox": inbox,
        "folders": folders,
        "tags": shown_tags,
        "next": "`brain` with `list`, `folder` or `tag` shows what one holds; with `query`, what says some words.",
    });
    if tags.len() > TAGS_SHOWN {
        kept["tags_more"] = json!(tags.len() - TAGS_SHOWN);
    }
    let told = |all: &[Value], key: &str, count: &str| {
        all.iter()
            .map(|one| format!("{} ({})", one[key].as_str().unwrap_or_default(), one[count]))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let text = format!(
        "Lists, with their open tasks: {}; the inbox ({inbox}).\nFolders, with their documents: {}.\nTags, most used first: {}{}.",
        told(&lists, "list", "open"),
        match folders.is_empty() {
            true => "none yet".to_string(),
            false => told(&folders, "folder", "docs"),
        },
        shown_tags
            .iter()
            .map(|one| format!(
                "#{} ({})",
                one["tag"].as_str().unwrap_or_default(),
                one["times"]
            ))
            .collect::<Vec<_>>()
            .join(", "),
        match tags.len() > TAGS_SHOWN {
            true => format!(", and {} more", tags.len() - TAGS_SHOWN),
            false => String::new(),
        }
    );
    Ok(mirrored(text, kept))
}

fn in_sight(task: &tisty_core::model::Task) -> bool {
    task.is_open() && !task.folded()
}

fn lists(state: &State) -> Vec<Value> {
    let mut named: Vec<(&str, usize)> = state
        .lists
        .values()
        .filter(|one| !one.archived)
        .map(|one| {
            let open = state
                .tasks
                .values()
                .filter(|task| task.list == Some(one.id) && in_sight(task))
                .count();
            (one.name.as_str(), open)
        })
        .collect();
    named.sort_unstable();
    named
        .into_iter()
        .map(|(list, open)| json!({ "list": list, "open": open }))
        .collect()
}

fn folders(state: &State) -> Vec<Value> {
    let mut all: Vec<(String, usize)> = state
        .folders
        .values()
        .map(|one| {
            let docs = state
                .papers()
                .filter(|kept| kept.folder == Some(one.id) && kept.page_of.is_none())
                .count();
            (trail(state, one.id), docs)
        })
        .collect();
    all.sort();
    all.into_iter()
        .map(|(folder, docs)| json!({ "folder": folder, "docs": docs }))
        .collect()
}
