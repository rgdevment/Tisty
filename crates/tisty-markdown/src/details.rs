use crate::{Block, Kind};

/// What sits between `<details>` and `</details>` (or `:::details` and `:::`) folds under one block.
pub fn gather(blocks: &mut Vec<Block>, text: &str) {
    let mut at = 0;
    while at < blocks.len() {
        let Some((summary, open)) = opened(&blocks[at], text) else {
            at += 1;
            continue;
        };
        let Some(shut) = closing(blocks, at, text) else {
            at += 1;
            continue;
        };
        let inner: Vec<Block> = blocks.drain(at + 1..shut).collect();
        let whole = blocks[at].at.start..blocks[at + 1].at.end;
        blocks.splice(
            at..at + 2,
            [Block {
                kind: Kind::Details { summary, open },
                at: whole,
                inner,
                spans: Vec::new(),
            }],
        );
        at += 1;
    }
}

fn opened(block: &Block, text: &str) -> Option<(String, bool)> {
    let said = text[block.at.clone()].trim();
    match block.kind {
        Kind::Html => {
            let tag = said.strip_prefix("<details")?;
            if !tag.starts_with(['>', ' ']) {
                return None;
            }
            let (attributes, rest) = tag.split_once('>')?;
            let summary = rest
                .trim()
                .strip_prefix("<summary>")
                .and_then(|inside| inside.split_once("</summary>"))
                .map_or_else(String::new, |(summary, _)| summary.trim().to_string());
            Some((
                summary,
                attributes.split_whitespace().any(|one| one == "open"),
            ))
        }
        Kind::Paragraph => {
            let summary = said.strip_prefix(":::details")?;
            (!summary.contains('\n')).then(|| (summary.trim().to_string(), false))
        }
        _ => None,
    }
}

fn closing(blocks: &[Block], from: usize, text: &str) -> Option<usize> {
    let mut deep = 0usize;
    for (at, block) in blocks.iter().enumerate().skip(from) {
        let said = text[block.at.clone()].trim();
        if opened(block, text).is_some() {
            deep += 1;
        } else if (block.kind == Kind::Html && said == "</details>")
            || (block.kind == Kind::Paragraph && said == ":::")
        {
            deep -= 1;
            if deep == 0 {
                return Some(at);
            }
        }
    }
    None
}
