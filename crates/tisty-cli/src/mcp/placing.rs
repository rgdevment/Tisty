use tisty_core::State;

use super::trail;

const NAMED_AT_MOST: usize = 3;
const ALIKE_AT_LEAST: usize = 2;

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

fn said_of(kept: &tisty_core::model::Kept) -> std::collections::BTreeSet<String> {
    let mut all: std::collections::BTreeSet<String> =
        words(kept.title.as_deref().unwrap_or_default())
            .into_iter()
            .collect();
    all.extend(kept.tags.iter().map(|tag| tag.as_str().to_string()));
    all
}

// A project's folder is named after the project, not after what its documents say, so they are asked too.
fn alike_folders(state: &State, title: &str, body: &str) -> Vec<String> {
    let mut mine: std::collections::BTreeSet<String> = words(title).into_iter().collect();
    if let Some(tags) = tisty_core::event::Said::of(body).tags {
        mine.extend(tags.iter().map(|tag| tag.as_str().to_string()));
    }
    let mut held: std::collections::BTreeMap<_, usize> = Default::default();
    for kept in state
        .papers()
        .filter(|one| one.page_of.is_none() && one.flagged.is_none() && !state.held_away(one))
    {
        let Some(folder) = kept.folder.filter(|one| !state.folder_away(*one)) else {
            continue;
        };
        if said_of(kept).intersection(&mine).count() >= ALIKE_AT_LEAST {
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
    let alike = alike_folders(state, title, body);
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
