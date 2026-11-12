use std::path::Path;

use crate::State;
use crate::model::Kept;

pub fn opening(title: &str) -> String {
    format!(
        "---\nview: notes\nbrainstorm: on\n---\n\n# {}\n",
        title.trim()
    )
}

pub const IDEA_AT_MOST: usize = 140;

pub const COLOURS: [&str; 12] = [
    "pink", "coral", "orange", "yellow", "lime", "green", "teal", "cyan", "blue", "lavender",
    "violet", "magenta",
];

pub fn jotted(idea: &str, at: jiff::civil::DateTime, colour: &str) -> String {
    format!(
        "## {} · {colour}\n\n{}",
        at.strftime("%Y-%m-%d %H:%M"),
        idea.trim()
    )
}

pub fn coloured(body: &str, roll: u32) -> &'static str {
    let last = body
        .lines()
        .rev()
        .find(|line| line.starts_with("## "))
        .and_then(|line| line.rsplit_once(" · "))
        .map(|(_, colour)| colour.trim());
    let many = COLOURS.len();
    let mut at = roll as usize % many;
    if last == Some(COLOURS[at]) {
        at = (at + 1 + (roll as usize / many) % (many - 1)) % many;
    }
    COLOURS[at]
}

pub fn filed(idea: &str, from: &str, on: jiff::civil::Date) -> String {
    let quoted: Vec<String> = idea
        .trim()
        .lines()
        .map(|line| format!("> {line}").trim_end().to_string())
        .collect();
    format!(
        "> [!NOTE] {} · {}\n{}",
        from.trim(),
        on.strftime("%Y-%m-%d"),
        quoted.join("\n")
    )
}

pub fn ideas(body: &str) -> usize {
    let mut fenced = false;
    body.lines()
        .filter(|line| {
            if line.starts_with("```") || line.starts_with("~~~") {
                fenced = !fenced;
                return false;
            }
            !fenced && line.starts_with("## ")
        })
        .count()
}

pub fn roll() -> u32 {
    use rand_core::TryRngCore;
    rand_core::OsRng
        .try_next_u32()
        .unwrap_or_else(|_| jiff::Timestamp::now().subsec_nanosecond().unsigned_abs())
}

// The oldest wins, so two machines that each made one before they met keep dropping ideas in the same.
pub fn find<'a>(root: &Path, state: &'a State, last: Option<&str>) -> Option<(&'a Kept, String)> {
    let mut open: Vec<&Kept> = state
        .docs
        .values()
        .filter(|one| one.page_of.is_none() && !state.held_away(one))
        .collect();
    open.sort_by_key(|one| one.id);
    let holding = |one: &'a Kept| {
        crate::docs::read(root, &one.file)
            .ok()
            .filter(|body| tisty_markdown::front::brainstorm(body))
            .map(|body| (one, body))
    };
    open.iter()
        .find(|one| last.is_some_and(|file| one.file == file))
        .and_then(|one| holding(one))
        .or_else(|| open.into_iter().find_map(holding))
}

#[cfg(test)]
#[path = "brainstorm_test.rs"]
mod tests;
