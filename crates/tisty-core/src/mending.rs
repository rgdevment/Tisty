use serde::Serialize;
use tisty_markdown::Kind;

use crate::arriving::tidied;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mend {
    pub from: usize,
    pub to: usize,
    pub fixed: String,
}

/// The blocks a person wrote outside the Tisty language, each with what the translator would put
/// in its place. Code keeps its own words, so a widget's HTML is never one of them.
pub fn mends(body: &str) -> Vec<Mend> {
    let doc = tisty_markdown::read(body);
    doc.blocks()
        .iter()
        .filter(|block| !matches!(block.kind, Kind::Front | Kind::Code { .. }))
        .filter_map(|block| {
            let said = body.get(block.at.clone())?.trim_end();
            let windows = said.contains("\r\n");
            let even = said.replace("\r\n", "\n");
            let fixed = tidied(&even).body;
            let fixed = fixed.trim_end();
            (fixed != even).then(|| Mend {
                from: block.at.start,
                to: block.at.start + said.len(),
                fixed: match windows {
                    true => fixed.replace('\n', "\r\n"),
                    false => fixed.to_string(),
                },
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "mending_test.rs"]
mod tests;
