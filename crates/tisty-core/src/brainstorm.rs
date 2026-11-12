use std::path::Path;

use crate::State;
use crate::model::Kept;

pub fn opening(title: &str) -> String {
    format!(
        "---\nview: notes\nbrainstorm: on\n---\n\n# {}\n",
        title.trim()
    )
}

pub fn jotted(idea: &str, at: jiff::civil::DateTime) -> String {
    format!("## {}\n\n{}", at.strftime("%Y-%m-%d %H:%M"), idea.trim())
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
