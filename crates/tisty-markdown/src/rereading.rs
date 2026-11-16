use std::collections::HashSet;
use std::ops::Range;

use crate::{Block, Doc, Kind};

const CROSSING: [&str; 3] = ["]:", "[^", ":::"];
const TAGGING: [char; 3] = ['#', '`', '~'];

pub(crate) fn crossing(text: &str) -> bool {
    CROSSING.iter().any(|one| text.contains(one))
}

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
    let changed = changed(was.text(), text);
    rebuilt(was, text.to_string(), &changed)
}

// The page says where it changed the text, so nothing is compared to find it.
pub fn reread_edit(was: Doc, at: Range<usize>, insert: &str) -> Result<Doc, Doc> {
    let Some(gone) = was.text().get(at.clone()) else {
        return Err(was);
    };
    if gone == insert {
        return Ok(was);
    }
    let old = was.text();
    let mut text = String::with_capacity(old.len() - gone.len() + insert.len());
    text.push_str(&old[..at.start]);
    text.push_str(insert);
    text.push_str(&old[at.end..]);
    let changed = Changed {
        start: at.start,
        old_end: at.end,
        new_end: at.start + insert.len(),
    };
    Ok(rebuilt(was, text, &changed))
}

fn rebuilt(was: Doc, text: String, changed: &Changed) -> Doc {
    let Some(plan) = planned(&was, &text, changed) else {
        return crate::read(&text);
    };
    let mut blocks = was.blocks;
    let mut after: Vec<Block> = blocks.drain(plan.after..).collect();
    blocks.truncate(plan.keep);
    blocks.extend(plan.part);
    for one in &mut after {
        shift(one, plan.old_end, plan.new_end);
    }
    blocks.append(&mut after);
    // A plan is refused for a crossing near the edit, so the text it was made for holds none.
    Doc {
        text,
        blocks,
        crossing: false,
    }
}

#[cfg(test)]
pub(crate) fn plan(was: &Doc, text: &str) -> Option<Plan> {
    planned(was, text, &changed(was.text(), text))
}

fn changed(old: &str, text: &str) -> Changed {
    let start = shared_head(old, text);
    let tail = shared_tail(&old[start..], &text[start..]);
    Changed {
        start,
        old_end: old.len() - tail,
        new_end: text.len() - tail,
    }
}

// Only the blocks around an edit are read again, and the edges must read as before or the whole text is.
fn planned(was: &Doc, text: &str, changed: &Changed) -> Option<Plan> {
    let old = was.text();
    if was.crossing || text.starts_with('\u{feff}') {
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
    let Changed {
        start,
        old_end,
        new_end,
    } = *changed;
    let near = &text.as_bytes()[start.saturating_sub(2)..(new_end + 2).min(text.len())];
    if CROSSING
        .iter()
        .any(|one| near.windows(one.len()).any(|at| at == one.as_bytes()))
    {
        return None;
    }
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
    let filed = filed(old, text, changed, &text[from..to])?;
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

#[derive(Clone, Copy)]
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
