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

// Chosen once, when the idea is born, and never the colour of the idea just before it.
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

pub fn roll() -> u32 {
    use rand_core::TryRngCore;
    rand_core::OsRng
        .try_next_u32()
        .unwrap_or_else(|_| jiff::Timestamp::now().subsec_nanosecond().unsigned_abs())
}

// The oldest wins, so two machines that each made one before they met keep dropping ideas in the same.
pub fn find<'a>(root: &Path, state: &'a State) -> Option<&'a Kept> {
    let mut open: Vec<&Kept> = state
        .docs
        .values()
        .filter(|one| one.page_of.is_none() && !state.held_away(one) && !state.written_shut(one.id))
        .collect();
    open.sort_by_key(|one| one.id);
    open.into_iter().find(|one| {
        crate::docs::read(root, &one.file)
            .is_ok_and(|body| tisty_markdown::front::brainstorm(&body))
    })
}

#[cfg(test)]
#[path = "brainstorm_test.rs"]
mod tests;
