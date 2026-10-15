pub(crate) use tisty_markdown::refs::shuts;
pub use tisty_markdown::refs::{DOC, Kind, Ref, alerted, card, extract, paper_lines, papers};

#[cfg(test)]
#[path = "refs_lines.rs"]
mod lines;

#[cfg(test)]
#[path = "refs_test.rs"]
mod tests;
