use serde_json::{Value, json};
use tisty_core::docs::Card;
use tisty_core::model::Kept;
use tisty_core::{Paths, State};

use super::super::said;
use super::compacting::agents_wrote;

pub(super) fn by(state: &State, kept: &Kept) -> &'static str {
    match agents_wrote(state, kept) {
        true => "agent",
        false => "person",
    }
}

pub(super) fn alike(state: &State, kept: &Kept, pages: &[&Kept]) -> Vec<Value> {
    let inside: Vec<&str> = std::iter::once(kept)
        .chain(pages.iter().copied())
        .map(|one| one.file.as_str())
        .collect();
    let subjects: Vec<(String, &str)> = std::iter::once(kept)
        .chain(pages.iter().copied())
        .filter_map(|one| one.title.as_deref())
        .map(|title| (tisty_core::text::subject(title), title))
        .filter(|(subject, _)| !subject.is_empty())
        .collect();
    let mut found: Vec<Value> = state
        .papers()
        .filter(|one| !inside.contains(&one.file.as_str()))
        .filter(|one| one.flagged.is_none() && !state.held_away(one))
        .filter_map(|one| {
            let title = one.title.as_deref()?;
            let subject = tisty_core::text::subject(title);
            let (_, like) = subjects.iter().find(|(had, _)| near(had, &subject))?;
            Some(json!({ "doc": one.file, "title": title, "like": like }))
        })
        .collect();
    found.sort_by(|a, b| a["doc"].as_str().cmp(&b["doc"].as_str()));
    found
}

// A hint, not a refusal: titles sharing most of their words are worth a look, however they are dressed.
fn near(one: &str, other: &str) -> bool {
    let words = |subject: &str| -> std::collections::BTreeSet<String> {
        tisty_core::text::telling(subject).into_iter().collect()
    };
    if one == other {
        return true;
    }
    let (a, b) = (words(one), words(other));
    let shared = a.intersection(&b).count();
    let fewer = a.len().min(b.len());
    shared >= 2 && shared * 3 >= fewer * 2
}

pub(super) fn repeated(own: (&str, &Card), pages: &[(&str, &Card)]) -> Vec<Value> {
    let mut seen: Vec<(String, String, Vec<String>)> = Vec::new();
    for (file, card) in std::iter::once(&own).chain(pages.iter()) {
        let mut here: Vec<String> = Vec::new();
        for heading in &card.outline {
            let key = tisty_core::text::subject(&heading.title);
            if key.is_empty()
                || here.contains(&key)
                || key == tisty_core::text::subject(&card.title)
            {
                continue;
            }
            here.push(key.clone());
            match seen.iter_mut().find(|(had, _, _)| *had == key) {
                Some((_, _, all)) => all.push((*file).to_string()),
                None => seen.push((key, heading.title.clone(), vec![(*file).to_string()])),
            }
        }
    }
    // A heading every page carries is the pages' template, as minutes have, not a repetition.
    let pages_held = 1 + pages.len();
    seen.into_iter()
        .filter(|(_, _, all)| all.len() > 1 && (pages_held < 3 || all.len() < pages_held))
        .map(|(_, heading, all)| json!({ "heading": heading, "in": all }))
        .collect()
}

// Past this a card's words are a pasted body, not a name, and only their start is worth showing.
const LABEL_SHOWN: usize = 80;

pub(super) fn mislabeled(whole: &str, pages: &[(&str, &Card)]) -> Vec<Value> {
    tisty_core::refs::paper_labels(whole)
        .into_iter()
        .filter_map(|(file, shows)| {
            let (_, card) = pages.iter().find(|(had, _)| *had == file)?;
            let title = card.title.as_str();
            let (said, named) = (
                tisty_core::text::worded(&shows),
                tisty_core::text::worded(title),
            );
            let differs = tisty_core::docs::another_name(&said, &named, title);
            differs.then(|| {
                let shown: String = shows.chars().take(LABEL_SHOWN).collect();
                let cut = match shown.len() < shows.len() {
                    true => "…",
                    false => "",
                };
                json!({ "doc": file, "shows": format!("{shown}{cut}"), "title": title })
            })
        })
        .collect()
}

pub(super) fn told_mislabeled(mislabeled: &[Value]) -> String {
    if mislabeled.is_empty() {
        return String::new();
    }
    format!(
        "\n\nPage lines that show another name than the page's title ({}): {}. If the window \
         should show the title, each line is worth writing again with it.",
        mislabeled.len(),
        mislabeled
            .iter()
            .map(|one| format!(
                "{:?} for {:?} [{}]",
                said(one, "shows"),
                said(one, "title"),
                said(one, "doc")
            ))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub(super) fn told_dead(dead: &[Value]) -> String {
    if dead.is_empty() {
        return String::new();
    }
    format!(
        "\n\nLinks that lead nowhere ({}): {}. Nothing here answers them — mend each or take \
         it out.",
        dead.len(),
        dead.iter()
            .map(|one| format!("{} in {}", said(one, "to"), said(one, "in")))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub(super) fn dead(paths: &Paths, state: &State, bodies: &[(&str, String)]) -> Vec<Value> {
    let mut found = Vec::new();
    for (file, body) in bodies {
        for one in tisty_core::refs::extract(body) {
            let gone = match one.target.strip_prefix(tisty_core::refs::DOC) {
                Some(at) => {
                    let at = at.split(['#', '?']).next().unwrap_or_default();
                    !state.docs.values().any(|kept| kept.file == at)
                }
                None => {
                    tisty_core::attach::names_an_attachment(&one.target)
                        && !tisty_core::attach::resolve(&one.target, paths.data())
                            .is_ok_and(|at| at.is_file())
                }
            };
            if gone {
                found.push(json!({ "in": file, "to": one.target }));
            }
        }
        for id in task_ids(body) {
            if !state.tasks.contains_key(&id) {
                found.push(json!({ "in": file, "to": format!("task {id}") }));
            }
        }
    }
    found
}

// A task named by its id in the text outlives nothing: once erased, the id points nowhere.
fn task_ids(body: &str) -> Vec<tisty_core::model::TaskId> {
    let mut ids: Vec<tisty_core::model::TaskId> = body
        .split(|one: char| !one.is_ascii_alphanumeric())
        .filter(|word| {
            word.len() == 26
                && word
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        })
        .filter_map(|word| word.parse().ok())
        .collect();
    ids.sort();
    ids.dedup();
    ids
}

#[cfg(test)]
#[path = "alike_test.rs"]
mod tests;
