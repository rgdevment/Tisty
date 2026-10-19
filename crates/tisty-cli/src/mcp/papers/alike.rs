use serde_json::{Value, json};
use tisty_core::State;
use tisty_core::docs::Card;
use tisty_core::model::Kept;

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

#[cfg(test)]
#[path = "alike_test.rs"]
mod tests;
