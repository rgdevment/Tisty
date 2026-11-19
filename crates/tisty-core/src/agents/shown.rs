use tisty_markdown::front;

use super::{Card, Kind};

const SKILL_FILE: &str = "SKILL.md";
const SKILL_NAME: &str = "name";

/// How an agent document is called where people see it: never stored, only read from its words.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    pub kind: &'static str,
    pub level: String,
    pub name: String,
    pub label: String,
    /// The skill folder a file belongs to, so the files of one skill are drawn together.
    pub skill: Option<String>,
}

pub fn shown(body: &str) -> Option<Shown> {
    let card = Card::of(body)?;
    let past = front::span(body)?.end;
    let words = &body[past..];
    let own = front::span(words).map_or(0, |at| at.end);
    let skill = match card.kind {
        Kind::Skill => card
            .name
            .split_once('/')
            .map(|(folder, _)| folder.to_string()),
        _ => None,
    };
    let leaf = card.name.rsplit('/').next().unwrap_or(&card.name);
    let skill_file = skill.is_some() && leaf.eq_ignore_ascii_case(SKILL_FILE);
    let label = match skill_file {
        true => named_in(&words[..own]).or_else(|| skill.clone()),
        false if card.code() => None,
        false => headed(&words[own..]),
    }
    .unwrap_or_else(|| leaf.to_string());
    Some(Shown {
        kind: card.kind.word(),
        level: card.level,
        name: card.name,
        label,
        skill,
    })
}

fn named_in(front_matter: &str) -> Option<String> {
    front::fields(front_matter)
        .into_iter()
        .find(|one| one.key.eq_ignore_ascii_case(SKILL_NAME))
        .and_then(|one| one.values.into_iter().next())
        .map(|said| crate::text::plainly(said.trim()))
        .filter(|said| !said.is_empty())
}

fn headed(words: &str) -> Option<String> {
    crate::docs::headings(words)
        .into_iter()
        .map(|(_, _, said)| crate::text::plainly(said.trim_end_matches('#').trim()))
        .find(|said| !said.is_empty())
}

#[cfg(test)]
#[path = "shown_test.rs"]
mod tests;
