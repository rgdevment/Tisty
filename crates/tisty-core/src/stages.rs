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
        gathered(block, body, &mut out);
    }
    out
}

fn gathered(block: &Block, body: &str, out: &mut Vec<Stage>) {
    match block.kind {
        Kind::Code { .. } => return,
        Kind::Item { done: Some(done) } => {
            if let Some(text) = body.get(block.at.clone()).and_then(said) {
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
        gathered(inner, body, out);
    }
}

fn listed(line: &str) -> bool {
    let numbered = line.find(['.', ')']).is_some_and(|at| {
        at > 0
            && line[..at].chars().all(|one| one.is_ascii_digit())
            && line[at + 1..].starts_with(' ')
    });
    numbered || ["- ", "* ", "+ "].iter().any(|mark| line.starts_with(mark))
}

// Words another editor wrapped onto more lines are still the stage's, up to a gap, a list or code.
fn said(item: &str) -> Option<String> {
    let mut lines = item.lines();
    let (_, first) = lines.next()?.split_once(']')?;
    let mut words = vec![first.trim()];
    for line in lines {
        let bare = line.trim_start_matches(|one: char| one == '>' || one.is_whitespace());
        if bare.is_empty() || listed(bare) || bare.starts_with("```") || bare.starts_with("~~~") {
            break;
        }
        words.push(bare.trim_end());
    }
    Some(one_line(&words.join(" "))).filter(|text| !text.is_empty())
}

pub fn title(text: &str) -> String {
    let flat = one_line(text);
    if flat.chars().count() <= STEP_AT_MOST {
        return flat;
    }
    let room: String = flat.chars().take(STEP_AT_MOST - 1).collect();
    let cut = room.rfind(' ').filter(|at| *at > 0).unwrap_or(room.len());
    format!("{}…", room[..cut].trim_end())
}

fn ordered(task: &Task) -> Vec<&Step> {
    let mut steps: Vec<&Step> = task.steps.iter().collect();
    steps.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));
    steps
}

// A lone pair left between two that agree is a rename, but only between two texts: a step and a
// line that merely sit in the same place are never taken for each other.
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

fn titles(steps: &[&Step]) -> Vec<String> {
    steps.iter().map(|one| title(&one.text)).collect()
}

fn titled(stages: &[Stage]) -> Vec<String> {
    stages.iter().map(|one| title(&one.text)).collect()
}

fn texts(stages: &[Stage]) -> Vec<String> {
    stages.iter().map(|one| one.text.clone()).collect()
}

fn earlier(steps: &[&Step], before: Option<&str>) -> Vec<Stage> {
    match before {
        Some(body) => stages(body),
        None => steps
            .iter()
            .map(|one| Stage {
                text: one_line(&one.text),
                done: one.done,
                at: 0..0,
            })
            .collect(),
    }
}

// The steps the document holds take the places they already had, in the document's order, so a
// step never written into it keeps its place among them.
fn reordered(task: &Task, steps: &[&Step], in_doc: &[usize]) -> (Vec<Op>, Vec<String>) {
    let mut keys: Vec<String> = steps.iter().map(|one| one.order.clone()).collect();
    let slots: Vec<usize> = (0..steps.len()).filter(|at| in_doc.contains(at)).collect();
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

/// Only what changed between the two texts reaches the log.
pub fn reconciled(task: &Task, before: Option<&str>, now: &str) -> Vec<Op> {
    let steps = ordered(task);
    let was = earlier(&steps, before);
    let is = stages(now);
    let step_of = paired(&titles(&steps), &titled(&was), false);
    let was_of = paired(&texts(&was), &texts(&is), true);
    let bound: Vec<Option<(usize, usize)>> = was_of
        .iter()
        .map(|one| one.and_then(|w| step_of[w].map(|s| (w, s))))
        .collect();
    let mut in_doc: Vec<usize> = Vec::new();
    for (_, s) in bound.iter().flatten() {
        if !in_doc.contains(s) {
            in_doc.push(*s);
        }
    }
    let (mut ops, keys) = reordered(task, &steps, &in_doc);

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
        if was_of[at].is_some() {
            continue;
        }
        let after = bound[at + 1..]
            .iter()
            .flatten()
            .next()
            .map(|(_, s)| keys[*s].clone())
            .filter(|next| before_key.as_deref().is_none_or(|low| low < next.as_str()));
        let order = crate::order::between(before_key.as_deref(), after.as_deref());
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

    let kept: std::collections::BTreeSet<usize> = was_of.iter().flatten().copied().collect();
    for (w, s) in step_of.iter().enumerate() {
        if let Some(s) = s
            && !kept.contains(&w)
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
    let steps = ordered(task);
    let found = stages(body);
    let of = paired(&titles(&steps), &titled(&found), false);
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
    out
}

#[cfg(test)]
#[path = "stages_test.rs"]
mod tests;
