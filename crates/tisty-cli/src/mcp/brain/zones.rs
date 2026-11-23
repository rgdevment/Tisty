use serde_json::{Value, json};
use tisty_core::{Paths, State};

use super::super::jsonrpc::mirrored;
use super::super::looking::counted_tags;
use super::super::{Refused, lists, opened, papers, trail};

const TAGS_SHOWN: usize = 40;

pub(super) fn documents(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    papers::listing::docs(paths, args)
}

pub(super) fn all(paths: &Paths) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let listed = listed(&state);
    let folders = folders(&state);
    let tags = counted_tags(&state);
    let inbox = lists::open_in_the_inbox(&state);
    let inbox_named = lists::inbox_named(&state);
    let shown_tags: Vec<Value> = tags
        .iter()
        .take(TAGS_SHOWN)
        .map(|(tag, times)| json!({ "tag": tag, "times": times }))
        .collect();
    let mut kept = json!({
        "lists": listed,
        "inbox": inbox,
        "folders": folders,
        "tags": shown_tags,
        "next": format!("`brain` with `list`, `folder` or `tag` shows what one holds, `list` «{inbox_named}» the tasks in no list; `scope` «open» leaves out the archived ones, which these counts do not hold. With `query`, what says some words."),
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
        "Lists, with their open tasks: {}; the inbox ({inbox}), which `list` «{inbox_named}» brings.\nFolders, with their documents: {}.\nTags, most used first: {}{}.",
        told(&listed, "list", "open"),
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

fn listed(state: &State) -> Vec<Value> {
    lists::open_by_name(state)
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
