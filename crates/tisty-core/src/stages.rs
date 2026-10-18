use std::ops::Range;

use tisty_markdown::{Block, Kind};
use ulid::Ulid;

use crate::event::{Op, StepAdd, StepRef, StepReorder, StepText};
use crate::model::{STEP_AT_MOST, Step, Task};
use crate::text::one_line;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    pub text: String,
    pub done: bool,
    pub at: Range<usize>,
}

pub fn stages(body: &str) -> Vec<Stage> {
    let doc = tisty_markdown::read(body);
    let mut out = Vec::new();
    for block in doc.blocks() {
        gathered(block, body, 0, &mut out);
    }
    out
}

fn gathered(block: &Block, body: &str, quoted: usize, out: &mut Vec<Stage>) {
    let quoted = quoted + usize::from(matches!(block.kind, Kind::Quote | Kind::Callout(_)));
    match block.kind {
        Kind::Code { .. } => return,
        Kind::Item { done: Some(done) } => {
            if let Some(text) = body.get(own(block)).and_then(|item| said(item, quoted)) {
                out.push(Stage {
                    text,
                    done,
                    at: block.at.clone(),
                });
            }
        }
        _ => {}
    }
    for inner in &block.inner {
        gathered(inner, body, quoted, out);
    }
}

// The item's own words end where the parser opens its first block, or its first paragraph closes.
fn own(item: &Block) -> Range<usize> {
    let end = match item.inner.first() {
        Some(first) if first.kind == Kind::Paragraph => first.at.end,
        Some(first) => first.at.start,
        None => item.at.end,
    };
    item.at.start..end.max(item.at.start)
}

fn said(item: &str, quoted: usize) -> Option<String> {
    let mut lines = item.lines();
    let (_, first) = lines.next()?.split_once(']')?;
    let mut words = vec![first.trim().trim_end_matches('\\')];
    for line in lines {
        let mut bare = line.trim_start();
        for _ in 0..quoted {
            bare = bare.strip_prefix('>').unwrap_or(bare).trim_start();
        }
        words.push(bare.trim_end().trim_end_matches('\\'));
    }
    Some(one_line(&words.join(" "))).filter(|text| !text.is_empty())
}

pub fn title(text: &str) -> String {
    let flat = one_line(text);
    if flat.chars().count() <= STEP_AT_MOST {
        return flat;
    }
    let room: String = flat.chars().take(STEP_AT_MOST).collect();
    let cut = match room.rfind(' ').filter(|at| *at > 0) {
        Some(at) => at,
        None => room.char_indices().last().map_or(0, |(at, _)| at),
    };
    format!("{}…", room[..cut].trim_end())
}

/// Each step written as the stage a document holds for it, one line apiece.
pub fn lines(steps: &[&Step]) -> String {
    steps
        .iter()
        .map(|step| format!("- [{}] {}\n", if step.done { "x" } else { " " }, step.text))
        .collect()
}

pub fn ordered(task: &Task) -> Vec<&Step> {
    let mut steps: Vec<&Step> = task.steps.iter().collect();
    steps.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));
    steps
}

// A lone pair between two that agree is a rename, never a step and a line merely side by side.
fn paired(from: &[String], to: &[String], renames: bool) -> Vec<Option<usize>> {
    let mut of = vec![None; to.len()];
    let mut taken = vec![false; from.len()];
    let anchors = crate::lcs::pairs(from, to);
    for (one, other) in &anchors {
        of[*other] = Some(*one);
        taken[*one] = true;
    }
    for (other, words) in to.iter().enumerate() {
        if of[other].is_some() {
            continue;
        }
        if let Some(one) = (0..from.len()).find(|at| !taken[*at] && from[*at] == *words) {
            of[other] = Some(one);
            taken[one] = true;
        }
    }
    if !renames {
        return of;
    }
    let starts = std::iter::once((0, 0)).chain(anchors.iter().map(|(a, b)| (a + 1, b + 1)));
    let ends = anchors
        .iter()
        .copied()
        .chain(std::iter::once((from.len(), to.len())));
    for ((from_at, to_at), (from_end, to_end)) in starts.zip(ends) {
        let loose: Vec<usize> = (from_at..from_end).filter(|at| !taken[*at]).collect();
        let open: Vec<usize> = (to_at..to_end).filter(|at| of[*at].is_none()).collect();
        if let ([one], [other]) = (loose.as_slice(), open.as_slice()) {
            of[*other] = Some(*one);
            taken[*one] = true;
        }
    }
    of
}

fn titled<'a>(texts: impl IntoIterator<Item = &'a String>) -> Vec<String> {
    texts.into_iter().map(|one| title(one)).collect()
}

fn texts(stages: &[Stage]) -> Vec<String> {
    stages.iter().map(|one| one.text.clone()).collect()
}

// The document's steps refill the places they held, so a step it never had keeps its own.
fn reordered(task: &Task, steps: &[&Step], in_doc: &[usize]) -> (Vec<Op>, Vec<String>) {
    let mut keys: Vec<String> = steps.iter().map(|one| one.order.clone()).collect();
    let mut held = vec![false; steps.len()];
    for at in in_doc {
        held[*at] = true;
    }
    let slots: Vec<usize> = (0..steps.len()).filter(|at| held[*at]).collect();
    let mut sequence: Vec<usize> = (0..steps.len()).collect();
    for (slot, step) in slots.iter().zip(in_doc) {
        sequence[*slot] = *step;
    }
    let wanted: Vec<&str> = sequence
        .iter()
        .map(|at| steps[*at].order.as_str())
        .collect();
    let mut ops = Vec::new();
    for (at, fresh) in sequence.iter().zip(crate::order::resequenced(&wanted)) {
        let Some(order) = fresh else { continue };
        keys[*at].clone_from(&order);
        ops.push(Op::StepReorder {
            id: task.id,
            d: StepReorder {
                step: steps[*at].id,
                order,
            },
        });
    }
    (ops, keys)
}

pub fn reconciled(task: &Task, before: Option<&str>, now: &str) -> Vec<Op> {
    let steps = ordered(task);
    let is = stages(now);
    // With no text before, a box may be stale, so it ticks no step and only one with no step is added.
    let was = before.map_or_else(|| is.clone(), stages);
    let step_of = paired(
        &titled(steps.iter().map(|one| &one.text)),
        &titled(was.iter().map(|one| &one.text)),
        false,
    );
    let was_of = paired(&texts(&was), &texts(&is), true);
    let bound: Vec<Option<(usize, usize)>> = was_of
        .iter()
        .map(|one| one.and_then(|w| step_of[w].map(|s| (w, s))))
        .collect();
    let in_doc: Vec<usize> = bound.iter().flatten().map(|(_, s)| *s).collect();
    let mut by_was: Vec<(usize, usize)> = bound.iter().flatten().copied().collect();
    by_was.sort_unstable();
    // Only a move made in the text reorders, so one made in the app is not undone by a stale file.
    let (mut ops, keys) = match by_was.iter().map(|(_, s)| *s).eq(in_doc.iter().copied()) {
        true => (
            Vec::new(),
            steps.iter().map(|one| one.order.clone()).collect(),
        ),
        false => reordered(task, &steps, &in_doc),
    };

    let mut before_key: Option<String> = None;
    for (at, one) in is.iter().enumerate() {
        if let Some((w, s)) = bound[at] {
            let step = steps[s];
            if one.text != was[w].text && title(&one.text) != title(&step.text) {
                ops.push(Op::StepText {
                    id: task.id,
                    d: StepText {
                        step: step.id,
                        text: title(&one.text),
                    },
                });
            }
            if one.done != was[w].done && one.done != step.done {
                let d = StepRef { step: step.id };
                ops.push(match one.done {
                    true => Op::StepDone { id: task.id, d },
                    false => Op::StepUndone { id: task.id, d },
                });
            }
            before_key = Some(keys[s].clone());
            continue;
        }
        // A box with no step that nobody touched may be one whose step was taken away on purpose.
        let touched = was_of[at].is_none_or(|w| was[w].text != one.text || was[w].done != one.done);
        if before.is_some() && !touched {
            continue;
        }
        // Every step bounds the new key, the ones the document never had included, so none is shared.
        let after = keys
            .iter()
            .filter(|key| before_key.as_deref().is_none_or(|low| low < key.as_str()))
            .min();
        let order = crate::order::between(before_key.as_deref(), after.map(String::as_str));
        let step = Ulid::generate();
        ops.push(Op::StepAdd {
            id: task.id,
            d: StepAdd {
                step,
                text: title(&one.text),
                order: order.clone(),
            },
        });
        if one.done {
            ops.push(Op::StepDone {
                id: task.id,
                d: StepRef { step },
            });
        }
        before_key = Some(order);
    }

    let mut kept = vec![false; was.len()];
    for w in was_of.iter().flatten() {
        kept[*w] = true;
    }
    for (w, s) in step_of.iter().enumerate() {
        if let Some(s) = s
            && !kept[w]
        {
            ops.push(Op::StepRemove {
                id: task.id,
                d: StepRef { step: steps[*s].id },
            });
        }
    }
    ops
}

/// The file's own boxes may be stale, so a document going out is ticked as the log has it.
pub fn ticked(task: &Task, body: &str) -> String {
    if task.steps.is_empty() {
        return body.to_string();
    }
    let steps = ordered(task);
    let found = stages(body);
    let of = paired(
        &titled(steps.iter().map(|one| &one.text)),
        &titled(found.iter().map(|one| &one.text)),
        false,
    );
    let mut out = body.to_string();
    for (stage, step) in found.iter().zip(&of).rev() {
        let Some(step) = step.map(|at| steps[at]) else {
            continue;
        };
        if stage.done == step.done {
            continue;
        }
        let Some(open) = body[stage.at.clone()].find('[') else {
            continue;
        };
        let at = stage.at.start + open + 1;
        out.replace_range(at..at + 1, if step.done { "x" } else { " " });
    }
    // A step filed from elsewhere has no line yet, and a step nobody can see cannot be ticked.
    let missing: Vec<&Step> = (0..steps.len())
        .filter(|at| !of.contains(&Some(*at)))
        .map(|at| steps[at])
        .collect();
    if missing.is_empty() {
        return out;
    }
    let mut grown = out.trim_end_matches(['\n', '\r']).to_string();
    if !grown.is_empty() {
        grown.push_str("\n\n");
    }
    grown.push_str(&lines(&missing));
    // Lines swallowed by what the text leaves open, a fence say, would be added again on every open.
    match stages(&grown).len() == found.len() + missing.len() {
        true => grown,
        false => out,
    }
}

#[cfg(test)]
#[path = "stages_test.rs"]
mod tests;
