use serde_json::{Value, json};
use tisty_core::State;

use super::trail;

const NAMED_AT_MOST: usize = 3;
const ALIKE_AT_LEAST: usize = 2;
const WORDED_AT_LEAST: usize = 3;

fn words(said: &str) -> Vec<String> {
    tisty_core::text::telling(said)
}

fn closest(title: &str, named: Vec<String>) -> Vec<String> {
    let wanted = words(title);
    let mut scored: Vec<(usize, String)> = named
        .into_iter()
        .map(|name| {
            let shared = words(&name)
                .iter()
                .filter(|one| wanted.contains(one))
                .count();
            (shared, name)
        })
        .filter(|(shared, _)| *shared > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    scored.dedup_by(|a, b| a.1 == b.1);
    scored
        .into_iter()
        .take(NAMED_AT_MOST)
        .map(|(_, name)| name)
        .collect()
}

pub(super) fn closest_folders(state: &State, title: &str) -> Vec<String> {
    let all = state
        .folders
        .keys()
        .filter(|id| !state.folder_away(**id))
        .map(|id| trail(state, *id))
        .collect();
    closest(title, all)
}

pub(super) fn closest_lists(state: &State, title: &str) -> Vec<String> {
    let all = state
        .lists
        .values()
        .filter(|one| !one.archived)
        .map(|one| one.name.clone())
        .collect();
    closest(title, all)
}

type Told = (
    std::collections::BTreeSet<String>,
    std::collections::BTreeSet<String>,
);

fn told(title: &str, tags: impl Iterator<Item = String>) -> Told {
    (words(title).into_iter().collect(), tags.collect())
}

// Two plain words like «meeting notes» say little; a shared tag, or three words, say the subject.
fn alike((words, tags): &Told, (theirs, labels): &Told) -> bool {
    let tagged = tags.intersection(labels).count();
    let worded = words.intersection(theirs).count();
    (tagged >= 1 && tagged + worded >= ALIKE_AT_LEAST) || worded >= WORDED_AT_LEAST
}

// A project's folder is named after the project, not after what its documents say, so they are asked too.
fn alike_folders(state: &State, title: &str, body: &str) -> Vec<String> {
    let mine = told(
        title,
        tisty_core::tagging::tags_in(body)
            .into_iter()
            .map(|tag| tag.as_str().to_string()),
    );
    let mut held: std::collections::BTreeMap<_, usize> = Default::default();
    for kept in state
        .papers()
        .filter(|one| one.page_of.is_none() && one.flagged.is_none() && !state.held_away(one))
    {
        let Some(folder) = kept.folder else {
            continue;
        };
        let theirs = told(
            kept.title.as_deref().unwrap_or_default(),
            kept.tags.iter().map(|tag| tag.as_str().to_string()),
        );
        if alike(&mine, &theirs) {
            *held.entry(folder).or_default() += 1;
        }
    }
    let mut ranked: Vec<(String, usize)> = held
        .into_iter()
        .map(|(folder, many)| (trail(state, folder), many))
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(NAMED_AT_MOST)
        .map(|(folder, many)| format!("{folder} ({many} alike)"))
        .collect()
}

pub(super) fn loose_doc(state: &State, title: &str, body: &str) -> String {
    let named = closest_folders(state, title);
    let alike: Vec<String> = alike_folders(state, title, body)
        .into_iter()
        .filter(|one| {
            !named
                .iter()
                .any(|name| one.starts_with(&format!("{name} (")))
        })
        .collect();
    match (named.as_slice(), alike.as_slice()) {
        ([], []) if state.folders.is_empty() => " It sits in no folder, and there are none yet: \
             ask the person where it belongs, suggesting a name, and make the folder once they agree."
            .to_string(),
        ([], []) => " It sits in no folder, and nothing kept in one reads like it: ask the person \
             where it belongs, suggesting a name, rather than leaving it loose."
            .to_string(),
        _ => {
            let mut said = String::from(" It sits in no folder;");
            if !named.is_empty() {
                said.push_str(&format!(" the closest by name: {}.", named.join(", ")));
            }
            if !alike.is_empty() {
                said.push_str(&format!(
                    " What it says is closest to what these hold: {}.",
                    alike.join(", ")
                ));
            }
            said.push_str(" `file_doc` puts it in one, or ask the person.");
            said
        }
    }
}

pub(super) fn loose_task(state: &State, title: &str) -> String {
    let mut all: Vec<&str> = state
        .lists
        .values()
        .filter(|one| !one.archived)
        .map(|one| one.name.as_str())
        .collect();
    if all.is_empty() {
        return String::new();
    }
    match closest_lists(state, title).as_slice() {
        [] => {
            all.sort_unstable();
            format!(
                " It waits in the inbox; the lists are {}. `move` with `list` puts it in one.",
                all.join(", ")
            )
        }
        near => format!(
            " It waits in the inbox; the closest list by name: {}. `move` with `list` puts it \
             in one.",
            near.join(", ")
        ),
    }
}

// Asked for one folder, a parent that holds nothing itself would otherwise read as empty.
pub(super) fn held_below(
    state: &State,
    scope: tisty_core::view::Scope,
    at: tisty_core::model::FolderId,
) -> Vec<Value> {
    let beneath = |from: tisty_core::model::FolderId| {
        let mut walk = state.folders.get(&from).and_then(|one| one.parent);
        for _ in 0..=tisty_core::model::DEEPEST {
            match walk {
                Some(up) if up == at => return true,
                Some(up) => walk = state.folders.get(&up).and_then(|one| one.parent),
                None => return false,
            }
        }
        false
    };
    let mut held: std::collections::HashMap<tisty_core::model::FolderId, usize> =
        std::collections::HashMap::new();
    for kept in state.papers() {
        if let Some(folder) = kept.folder
            && kept.page_of.is_none()
            && state.doc_kept_by(scope, kept)
        {
            *held.entry(folder).or_default() += 1;
        }
    }
    let mut found: Vec<Value> = state
        .folders
        .values()
        .filter(|one| beneath(one.id))
        .filter_map(|one| {
            let docs = held.get(&one.id).copied().unwrap_or_default();
            (docs > 0).then(|| json!({ "folder": trail(state, one.id), "docs": docs }))
        })
        .collect();
    found.sort_by(|a, b| a["folder"].as_str().cmp(&b["folder"].as_str()));
    found
}

pub(super) fn told_below(below: &[Value]) -> Option<String> {
    (!below.is_empty()).then(|| {
        format!(
            "Folders below it hold documents of their own, listed only when asked by their path: {}.",
            below
                .iter()
                .map(|one| format!("{} ({})", super::said(one, "folder"), one["docs"]))
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}
