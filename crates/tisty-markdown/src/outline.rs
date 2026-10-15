use std::collections::HashMap;
use std::ops::Range;

use crate::{Block, Doc, Kind, Mark};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    pub level: u8,
    pub said: String,
    pub id: String,
    pub at: Range<usize>,
}

/// Every heading in reading order, with an id that stays the same while its words do.
pub fn outline(doc: &Doc) -> Vec<Head> {
    let mut found = Vec::new();
    let mut taken: HashMap<String, usize> = HashMap::new();
    walk(doc.blocks(), doc.text(), &mut found, &mut taken);
    found
}

fn walk(blocks: &[Block], text: &str, found: &mut Vec<Head>, taken: &mut HashMap<String, usize>) {
    for block in blocks {
        if let Kind::Heading(level) = block.kind {
            let said = words(block, text);
            if !said.is_empty() {
                let id = unique(slug(&said), taken);
                found.push(Head {
                    level,
                    said,
                    id,
                    at: block.at.clone(),
                });
            }
        }
        walk(&block.inner, text, found, taken);
    }
}

fn words(block: &Block, text: &str) -> String {
    let mut said = String::new();
    for span in &block.spans {
        match span.mark {
            Mark::Text => said.push_str(&text[span.at.clone()]),
            Mark::Code => said.push_str(text[span.at.clone()].trim_matches('`')),
            _ => {}
        }
    }
    said.trim().to_string()
}

// The same shape GitHub gives an anchor, so a link copied from either place keeps working.
fn slug(said: &str) -> String {
    said.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

fn unique(id: String, taken: &mut HashMap<String, usize>) -> String {
    let seen = taken.entry(id.clone()).or_insert(0);
    let named = if *seen == 0 {
        id
    } else {
        format!("{id}-{seen}")
    };
    *seen += 1;
    named
}
