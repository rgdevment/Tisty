pub(crate) use tisty_markdown::refs::shuts;
pub use tisty_markdown::refs::{DOC, Kind, Ref, alerted, card, paper_labels, paper_lines, papers};

/// What a body points at, with an attachment named by its one key whichever way the body wrote it.
pub fn extract(text: &str) -> Vec<Ref> {
    tisty_markdown::refs::extract(text)
        .into_iter()
        .map(|mut one| {
            if let Some(at) = crate::attach::canonical(&one.target) {
                one.target = at;
            }
            one
        })
        .collect()
}

#[cfg(test)]
#[path = "refs_lines.rs"]
mod lines;

#[cfg(test)]
#[path = "refs_test.rs"]
mod tests;
