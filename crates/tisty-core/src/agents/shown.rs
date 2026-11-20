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

const SKILL_ABOUT: &str = "description";

/// The file's own properties, listed on the card so the page shows only the words below them.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnFront {
    pub about: Option<String>,
    pub fields: Vec<[String; 2]>,
    pub raw: Option<String>,
}

pub fn own_front(body: &str) -> OwnFront {
    let Some(card) = Card::of(body) else {
        return OwnFront::default();
    };
    let Some(past) = front::span(body).map(|at| at.end) else {
        return OwnFront::default();
    };
    let words = &body[past..];
    let Some(own) = front::span(words) else {
        return OwnFront::default();
    };
    let said = &words[own];
    let fields = front::fields(said);
    if fields.is_empty() {
        return OwnFront {
            raw: as_written(said),
            ..OwnFront::default()
        };
    }
    let leaf = card.name.rsplit('/').next().unwrap_or(&card.name);
    let titled = card.kind == Kind::Skill && leaf.eq_ignore_ascii_case(SKILL_FILE);
    let mut out = OwnFront::default();
    for field in fields {
        let about = field.key.eq_ignore_ascii_case(SKILL_ABOUT);
        let value = joined(&field.values, about);
        if about && out.about.is_none() {
            out.about = Some(value.trim().to_string()).filter(|said| !said.is_empty());
        } else if titled && field.key.eq_ignore_ascii_case(SKILL_NAME) {
            continue;
        } else if let Some(seen) = out.fields.iter_mut().find(|one| one[0] == field.key) {
            seen[1] = format!("{}, {value}", seen[1]);
        } else {
            out.fields.push([field.key, value]);
        }
    }
    out
}

// A YAML block scalar keeps its lines under a `>` or `|` mark; prose wraps onto lines of its own.
fn joined(values: &[String], prose: bool) -> String {
    let block = |mark: &str| {
        let style = mark
            .trim_end_matches(['+', '-'])
            .trim_end_matches(|one: char| one.is_ascii_digit());
        style == ">" || style == "|"
    };
    match values.split_first() {
        Some((mark, rest)) if block(mark) && mark.starts_with('|') => rest.join("\n"),
        Some((mark, rest)) if block(mark) => rest.join(" "),
        _ if prose => values.join(" "),
        _ => values.join(", "),
    }
}

fn as_written(front_matter: &str) -> Option<String> {
    let lines: Vec<&str> = front_matter.trim_end().lines().collect();
    let inner = lines
        .get(1..lines.len().saturating_sub(1))
        .unwrap_or_default()
        .join("\n");
    (!inner.trim().is_empty()).then_some(inner)
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
