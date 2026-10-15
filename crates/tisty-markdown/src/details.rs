use crate::{Block, Kind};

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
        // One written whole on a single line closes itself and holds no blocks to fold.
        Kind::Html if !said.contains("</details>") => {
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

// A colon fence closes whichever colon container opened last, `:::tip` as much as `:::details`.
fn colon_opens(block: &Block, text: &str) -> bool {
    let said = text[block.at.clone()].trim();
    block.kind == Kind::Paragraph
        && !said.contains('\n')
        && said
            .strip_prefix(":::")
            .is_some_and(|rest| rest.starts_with(|c: char| c.is_alphanumeric()))
}

fn closing(blocks: &[Block], from: usize, text: &str) -> Option<usize> {
    let colon = blocks[from].kind == Kind::Paragraph;
    let mut deep = 0usize;
    for (at, block) in blocks.iter().enumerate().skip(from) {
        let said = text[block.at.clone()].trim();
        let (opens, shuts) = if colon {
            (
                colon_opens(block, text),
                block.kind == Kind::Paragraph && said == ":::",
            )
        } else {
            (
                opened(block, text).is_some(),
                block.kind == Kind::Html && said == "</details>",
            )
        };
        if opens {
            deep += 1;
        } else if shuts {
            deep = deep.saturating_sub(1);
            if deep == 0 {
                return Some(at);
            }
        }
    }
    None
}
