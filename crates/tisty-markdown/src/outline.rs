use std::collections::HashMap;
use std::ops::Range;

use crate::{Block, Doc, Kind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Head {
    pub level: u8,
    pub said: String,
    pub id: String,
    pub at: Range<usize>,
}

pub fn outline(doc: &Doc) -> Vec<Head> {
    let mut found = Vec::new();
    let mut taken: HashMap<String, usize> = HashMap::new();
    walk(doc.blocks(), doc.text(), &mut found, &mut taken);
    found
}

fn walk(blocks: &[Block], text: &str, found: &mut Vec<Head>, taken: &mut HashMap<String, usize>) {
    for block in blocks {
        if let Kind::Heading(level) = block.kind {
            let source: Vec<&str> = text[block.at.clone()]
                .lines()
                .map(crate::unquoted)
                .collect();
            let said = crate::reading::plain(&source.join("\n"));
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

// The same shape GitHub gives an anchor, so a link copied from either place keeps working.
fn slug(said: &str) -> String {
    said.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

fn unique(id: String, taken: &mut HashMap<String, usize>) -> String {
    let mut named = id.clone();
    while taken.contains_key(&named) {
        let seen = taken.entry(id.clone()).or_insert(0);
        *seen += 1;
        named = format!("{id}-{seen}");
    }
    taken.insert(named.clone(), 0);
    named
}
