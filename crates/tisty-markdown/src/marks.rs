#[cfg(feature = "tags")]
use std::collections::HashSet;
use std::ops::Range;

#[cfg(feature = "tags")]
use crate::Block;
use crate::{Mark, Span};

#[cfg(feature = "refs")]
// The tokenizer splits `[[name]]` around its brackets, so the pair is found in the bytes.
pub fn named(spans: &mut Vec<Span>, text: &str) {
    let (Some(from), Some(to)) = (
        spans.iter().map(|one| one.at.start).min(),
        spans.iter().map(|one| one.at.end).max(),
    ) else {
        return;
    };
    let mut found = Vec::new();
    let mut at = from;
    while let Some(open) = text[at..to].find("[[").map(|step| at + step) {
        let Some((end, name)) = crate::refs::wiki(&text[open..to]) else {
            break;
        };
        let whole = open..open + end + 2;
        match name {
            Some(name) if written(spans, &whole) => {
                found.push(Span {
                    mark: Mark::Ref {
                        name: name.to_string(),
                    },
                    at: whole.clone(),
                });
                at = whole.end;
            }
            _ => at = open + 2,
        }
    }
    spans.extend(found);
}

#[cfg(feature = "tags")]
// Only what the core files is marked, so a tag past its sixty-fourth never looks filed.
pub fn tagged(blocks: &mut [Block], text: &str) {
    let filed: HashSet<String> = crate::tags::tags_in(text).into_iter().collect();
    walk(blocks, text, &filed);
}

#[cfg(feature = "tags")]
fn walk(blocks: &mut [Block], text: &str, filed: &HashSet<String>) {
    for block in blocks {
        mark_lines(block, text, filed);
        walk(&mut block.inner, text, filed);
    }
}

#[cfg(feature = "tags")]
fn mark_lines(block: &mut Block, text: &str, filed: &HashSet<String>) {
    let Some(first) = block.spans.iter().map(|one| one.at.start).min() else {
        return;
    };
    let last = block
        .spans
        .iter()
        .map(|one| one.at.end)
        .max()
        .unwrap_or(first);
    let mut from = text[..first].rfind('\n').map_or(0, |at| at + 1);
    let mut found = Vec::new();
    while from < last {
        let to = text[from..].find('\n').map_or(text.len(), |at| from + at);
        for (at, name) in crate::tags::spots_as_written(&text[from..to]) {
            let at = from + at.start..from + at.end;
            if filed.contains(&name) && written(&block.spans, &at) && !referred(&block.spans, &at) {
                found.push(Span {
                    mark: Mark::Tag { name },
                    at,
                });
            }
        }
        from = to + 1;
    }
    block.spans.extend(found);
    block
        .spans
        .sort_by_key(|one| (one.at.start, std::cmp::Reverse(one.at.end)));
}

// Plain words from end to end: nothing in code, html or the markup between two words.
fn written(spans: &[Span], at: &Range<usize>) -> bool {
    at.clone().all(|byte| {
        spans
            .iter()
            .any(|one| one.mark == Mark::Text && one.at.start <= byte && byte < one.at.end)
    })
}

#[cfg(feature = "tags")]
fn referred(spans: &[Span], at: &Range<usize>) -> bool {
    spans.iter().any(|one| {
        matches!(one.mark, Mark::Ref { .. }) && one.at.start <= at.start && at.end <= one.at.end
    })
}
