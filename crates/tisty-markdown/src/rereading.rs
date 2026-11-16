use std::collections::HashSet;
use std::ops::Range;

use crate::{Block, Doc, Kind};

const CROSSING: [&str; 3] = ["]:", "[^", ":::"];
const TAGGING: [char; 3] = ['#', '`', '~'];

pub(crate) struct Plan {
    keep: usize,
    after: usize,
    part: Vec<Block>,
    old_end: usize,
    new_end: usize,
}

pub fn reread(was: Doc, text: &str) -> Doc {
    if was.text() == text {
        return was;
    }
    let Some(plan) = plan(&was, text) else {
        return crate::read(text);
    };
    let mut blocks = was.blocks;
    let mut after: Vec<Block> = blocks.drain(plan.after..).collect();
    blocks.truncate(plan.keep);
    blocks.extend(plan.part);
    for one in &mut after {
        shift(one, plan.old_end, plan.new_end);
    }
    blocks.append(&mut after);
    Doc {
        text: text.to_string(),
        blocks,
    }
}

// Only the blocks around an edit are read again, and the edges must read as before or the whole text is.
pub(crate) fn plan(was: &Doc, text: &str) -> Option<Plan> {
    let old = was.text();
    let crossing = CROSSING
        .iter()
        .any(|one| old.contains(one) || text.contains(one));
    if crossing || text.starts_with('\u{feff}') {
        return None;
    }
    let blocks = was.blocks();
    let front_end = blocks
        .first()
        .filter(|one| one.kind == Kind::Front)
        .map(|one| one.at.end);
    if crate::front::span(text).map(|one| one.end) != front_end {
        return None;
    }
    let start = shared_head(old, text);
    let tail = shared_tail(&old[start..], &text[start..]);
    let (old_end, new_end) = (old.len() - tail, text.len() - tail);
    if front_end.is_some_and(|end| start <= end) {
        return None;
    }
    let front = usize::from(front_end.is_some());
    let first = blocks.partition_point(|one| one.at.end < start).max(front);
    let last = blocks
        .partition_point(|one| one.at.start <= old_end)
        .max(first);
    let keep = if first > front { first - 1 } else { front };
    let after = (last + 1).min(blocks.len());
    let (edge_before, edge_after) = (keep < first, after > last);
    let from = if edge_before {
        blocks[keep].at.start
    } else {
        front_end.unwrap_or(0)
    };
    let to = if edge_after {
        shifted(blocks[after - 1].at.end, old_end, new_end)
    } else {
        text.len()
    };
    let changed = Changed {
        start,
        old_end,
        new_end,
    };
    let filed = filed(old, text, &changed, &text[from..to])?;
    let part = crate::reading::part(text, from..to, &filed);
    if edge_before && part.first() != blocks.get(keep) {
        return None;
    }
    if edge_after {
        let mut edge = blocks[after - 1].clone();
        shift(&mut edge, old_end, new_end);
        if part.last() != Some(&edge) {
            return None;
        }
    }
    Some(Plan {
        keep,
        after,
        part,
        old_end,
        new_end,
    })
}

struct Changed {
    start: usize,
    old_end: usize,
    new_end: usize,
}

fn filed(old: &str, text: &str, changed: &Changed, part: &str) -> Option<HashSet<String>> {
    let tags_of =
        |said: &str| -> HashSet<String> { crate::tags::tags_in(said).into_iter().collect() };
    let tagging = |said: &str, end: usize| {
        let from = said[..changed.start]
            .rfind(char::is_whitespace)
            .map_or(0, |at| at + 1);
        let to = said[end..]
            .find(char::is_whitespace)
            .map_or(said.len(), |at| end + at);
        said[from..to].contains(TAGGING)
    };
    if tagging(old, changed.old_end) || tagging(text, changed.new_end) {
        let now = tags_of(text);
        return (tags_of(old) == now).then_some(now);
    }
    let tag_like = part
        .as_bytes()
        .windows(2)
        .any(|pair| pair[0] == b'#' && !pair[1].is_ascii_whitespace() && pair[1] != b'#');
    Some(if tag_like {
        tags_of(text)
    } else {
        HashSet::new()
    })
}

fn shared_head(old: &str, text: &str) -> usize {
    let mut at = old
        .bytes()
        .zip(text.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while !old.is_char_boundary(at) || !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

fn shared_tail(old: &str, text: &str) -> usize {
    let mut at = old
        .bytes()
        .rev()
        .zip(text.bytes().rev())
        .take_while(|(a, b)| a == b)
        .count();
    while !old.is_char_boundary(old.len() - at) || !text.is_char_boundary(text.len() - at) {
        at -= 1;
    }
    at
}

fn shifted(at: usize, old_end: usize, new_end: usize) -> usize {
    if at >= old_end {
        at + new_end - old_end
    } else {
        at
    }
}

fn shift(block: &mut Block, old_end: usize, new_end: usize) {
    let moved = |range: &mut Range<usize>| {
        range.start = shifted(range.start, old_end, new_end);
        range.end = shifted(range.end, old_end, new_end);
    };
    moved(&mut block.at);
    if let Kind::Table { columns } = &mut block.kind {
        for one in columns.iter_mut().filter_map(|one| one.typed.as_mut()) {
            moved(one);
        }
    }
    for one in &mut block.spans {
        moved(&mut one.at);
    }
    for one in &mut block.inner {
        shift(one, old_end, new_end);
    }
}

#[cfg(test)]
#[path = "rereading_test.rs"]
mod tests;
