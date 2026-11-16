use std::ops::Range;

use crate::filing::Filing;
use crate::{Block, Doc, Kind};

// A details tag pairs with its partner however far, as a definition reaches every reference.
const CROSSING: [&str; 5] = ["]:", "[^", ":::", "<details", "</details>"];

pub(crate) fn crossing(text: &str) -> bool {
    CROSSING.iter().any(|one| text.contains(one))
}

pub(crate) struct Plan {
    keep: usize,
    after: usize,
    part: Vec<Block>,
    old_end: usize,
    new_end: usize,
    filing: Filing,
    closers: usize,
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
    // A plan is refused for a crossing in the stretch it read, so the text keeps its definitions and notes.
    let mut defined = was.defined;
    defined.recount(plan.closers);
    Doc {
        text,
        blocks,
        crossing: was.crossing,
        labels: defined.labels(&plan.filing.far()),
        defined,
        filing: Box::new(plan.filing),
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
    if text.starts_with('\u{feff}') {
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
    let reach = CROSSING.iter().map(|one| one.len()).max().unwrap_or(1) - 1;
    let near = &text.as_bytes()[start.saturating_sub(reach)..(new_end + reach).min(text.len())];
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
    let old_to = if edge_after {
        blocks[after - 1].at.end
    } else {
        old.len()
    };
    let to = shifted(old_to, old_end, new_end);
    if was.crossing && (crossing(&old[from..old_to]) || crossing(&text[from..to])) {
        return None;
    }
    let closers = was
        .defined
        .after(&old[start..old_end], &text[start..new_end]);
    let skip = front_end.unwrap_or(0);
    if !was
        .defined
        .affords(&old[skip..], &text[skip..], closers, &text[from..to])
    {
        return None;
    }
    let (filing, settled) = was.filing.after(old, text, changed);
    if !filing.alone(&was.filing, settled) {
        return None;
    }
    let part = crate::reading::part(text, from..to, &filing.filed, &was.defined);
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
        filing,
        closers,
    })
}

#[derive(Clone, Copy)]
pub(crate) struct Changed {
    pub(crate) start: usize,
    pub(crate) old_end: usize,
    pub(crate) new_end: usize,
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
