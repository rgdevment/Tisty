use crate::{Block, Callout, Kind};

pub fn gather(blocks: &mut Vec<Block>, text: &str) {
    let mut at = 0;
    while at < blocks.len() {
        let Some(head) = opened(&blocks[at], text) else {
            at += 1;
            continue;
        };
        let Some(shut) = closing(blocks, at, text) else {
            at += 1;
            continue;
        };
        let mut inner: Vec<Block> = blocks.drain(at + 1..shut).collect();
        let (summary, open) = match head {
            Opened::Details(summary, open) => (summary, open),
            Opened::Callout(kind) => {
                let whole = blocks[at].at.start..blocks[at + 1].at.end;
                blocks.splice(
                    at..at + 2,
                    [Block {
                        kind: Kind::Callout(kind),
                        at: whole,
                        inner,
                        spans: Vec::new(),
                    }],
                );
                at += 1;
                continue;
            }
        };
        let summary = match (summary.is_empty(), inner.first()) {
            (true, Some(first)) => match summoned(first, text) {
                Some(said) => {
                    inner.remove(0);
                    said
                }
                None => summary,
            },
            _ => summary,
        };
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

enum Opened {
    Details(String, bool),
    Callout(Callout),
}

fn opened(block: &Block, text: &str) -> Option<Opened> {
    let said = text[block.at.clone()].trim();
    match block.kind {
        // One written whole on a single line closes itself and holds no blocks to fold.
        Kind::Html if !said.contains("</details>") => {
            let tag = said.strip_prefix("<details")?;
            if !tag.starts_with(['>', ' ']) {
                return None;
            }
            let (attributes, rest) = tag.split_once('>')?;
            let open = attributes
                .split_whitespace()
                .any(|one| one == "open" || one.starts_with("open="));
            Some(Opened::Details(titled(rest).unwrap_or_default(), open))
        }
        Kind::Paragraph if said.contains('\n') => None,
        Kind::Paragraph => match said.strip_prefix(":::details") {
            Some(summary) => Some(Opened::Details(summary.trim().to_string(), false)),
            None => said
                .strip_prefix(":::")
                .filter(|rest| rest.starts_with(char::is_alphabetic))
                .and_then(called)
                .map(Opened::Callout),
        },
        _ => None,
    }
}

fn called(said: &str) -> Option<Callout> {
    Some(match said.trim().to_ascii_lowercase().as_str() {
        "note" => Callout::Note,
        "tip" => Callout::Tip,
        "important" => Callout::Important,
        "warning" => Callout::Warning,
        "caution" => Callout::Caution,
        _ => return None,
    })
}

fn titled(said: &str) -> Option<String> {
    let tag = said.trim().strip_prefix("<summary")?;
    if !tag.starts_with(['>', ' ']) {
        return None;
    }
    let (_, inside) = tag.split_once('>')?;
    let (summary, _) = inside.split_once("</summary>")?;
    Some(summary.trim().to_string())
}

// A blank line between the two tags leaves the summary in a block of its own.
fn summoned(block: &Block, text: &str) -> Option<String> {
    (block.kind == Kind::Html)
        .then(|| titled(&text[block.at.clone()]))
        .flatten()
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
