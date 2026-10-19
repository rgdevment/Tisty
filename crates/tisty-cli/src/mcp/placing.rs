use tisty_core::State;

use super::trail;

const NAMED_AT_MOST: usize = 3;

/// Words too short to say what something is about would make every folder look close.
fn words(said: &str) -> Vec<String> {
    tisty_core::text::folded(said)
        .split(|one: char| !one.is_alphanumeric())
        .filter(|one| one.chars().count() >= 3)
        .map(str::to_string)
        .collect()
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

pub(super) fn loose_doc(state: &State, title: &str) -> String {
    match closest_folders(state, title).as_slice() {
        [] if state.folders.is_empty() => " It sits in no folder, and there are none yet: ask \
             the person where it belongs, suggesting a name, and make the folder once they agree."
            .to_string(),
        [] => " It sits in no folder, and no folder's name is near its title: ask the person \
             where it belongs, suggesting a name, rather than leaving it loose."
            .to_string(),
        near => format!(
            " It sits in no folder; the closest by name: {}. `file_doc` puts it in one.",
            near.join(", ")
        ),
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
