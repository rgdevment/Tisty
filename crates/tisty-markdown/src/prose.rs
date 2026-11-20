use std::ops::Range;

use crate::inline::{atom, walls};
use crate::ranges::{Nearby, outside};
use crate::{Block, Kind, Mark};

// The words as the page shows them: code, html, properties and what Tisty draws in a mark's place say nothing.
pub fn prose(source: &str) -> String {
    let doc = crate::read(source);
    let mut said = String::new();
    gathered(doc.blocks(), doc.text(), &[], &mut said);
    said
}

fn gathered(blocks: &[Block], text: &str, typed: &[Range<usize>], said: &mut String) {
    for block in blocks {
        match &block.kind {
            Kind::Code { .. } | Kind::Html | Kind::Front | Kind::Rule | Kind::Definition { .. } => {
                continue;
            }
            Kind::Details { summary, .. } => spoken(summary, said),
            _ => {}
        }
        let mut silent: Vec<Range<usize>> = block
            .spans
            .iter()
            .filter(|one| atom(&one.mark))
            .map(|one| one.at.clone())
            .collect();
        silent.extend(walls(&block.spans, text));
        silent.extend(typed.iter().cloned());
        let silent = Nearby::merged(silent);
        for span in block.spans.iter().filter(|one| one.mark == Mark::Text) {
            for piece in outside(span.at.clone(), silent.near(&span.at)) {
                spoken(&text[piece], said);
            }
        }
        let typed: Vec<Range<usize>> = match &block.kind {
            Kind::Table { columns } => columns.iter().filter_map(|one| one.typed.clone()).collect(),
            _ => typed.to_vec(),
        };
        gathered(&block.inner, text, &typed, said);
    }
}

fn spoken(piece: &str, said: &mut String) {
    if !said.is_empty() {
        said.push(' ');
    }
    said.push_str(piece);
}

#[cfg(test)]
#[path = "prose_test.rs"]
mod tests;
