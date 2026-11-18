use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub line: usize,
    pub said: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub replaced: bool,
}

const SAID_AT_MOST: usize = 100;
const NAMED_BY: [&str; 3] = ["decision", "decidid", "decided"];
const LEADS: [(&str, &str); 5] = [
    ("**~~", "~~**"),
    ("~~**", "**~~"),
    ("**", "**"),
    ("~~", "~~"),
    ("__", "__"),
];

// The convention the person follows: a section named for decisions, one item each, the old one struck.
pub(super) fn decisions(body: &str) -> Vec<Decision> {
    let deciding: Vec<(usize, usize)> = crate::docs::outlined(body)
        .into_iter()
        .filter(|head| names_decisions(&head.title))
        .map(|head| (head.line, head.to))
        .collect();
    let mut fenced = tisty_markdown::fences::fencing();
    let mut found = Vec::new();
    for (at, line) in body.lines().enumerate() {
        let line_no = at + 1;
        if fenced(line) {
            continue;
        }
        let within = deciding
            .iter()
            .any(|(from, to)| *from < line_no && line_no <= *to);
        let Some(marker) = within
            .then(|| tisty_markdown::fences::bullet(line))
            .flatten()
        else {
            continue;
        };
        let item = line[marker..].trim();
        if item.starts_with("[ ]") || item.starts_with("[x]") || item.starts_with("[X]") {
            continue;
        }
        let (said, replaced) = lead(item);
        if !said.is_empty() {
            found.push(Decision {
                line: line_no,
                said,
                replaced,
            });
        }
    }
    found
}

fn names_decisions(title: &str) -> bool {
    let folded = crate::text::folded(title);
    NAMED_BY.iter().any(|one| folded.contains(one))
}

fn lead(item: &str) -> (String, bool) {
    let replaced = item.starts_with("~~") || item.starts_with("**~~");
    let said = match LEADS
        .iter()
        .find_map(|(open, close)| item.strip_prefix(open).map(|rest| (rest, close)))
    {
        Some((rest, close)) => rest.find(close).map_or(rest, |end| &rest[..end]),
        None => item.find(". ").map_or(item, |end| &item[..=end]),
    };
    let said = said.replace("**", "").replace("~~", "");
    (cut(said.trim()), replaced)
}

fn cut(said: &str) -> String {
    match said.char_indices().nth(SAID_AT_MOST) {
        Some((at, _)) => format!("{}…", said[..at].trim_end()),
        None => said.to_string(),
    }
}

#[cfg(test)]
#[path = "decided_test.rs"]
mod tests;
