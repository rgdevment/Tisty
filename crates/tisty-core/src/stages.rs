use std::ops::Range;

use tisty_markdown::{Block, Kind};
use ulid::Ulid;

use crate::event::{Op, StepAdd, StepRef, StepReorder, StepText};
use crate::model::{STEP_AT_MOST, Step, Task};

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

// An item wrapped over several lines by another editor is still one stage, until a blank line or a list below it.
fn said(item: &str) -> Option<String> {
    let mut lines = item.lines();
    let (_, first) = lines.next()?.split_once(']')?;
    let mut words = vec![first.trim()];
    for line in lines {
        let bare = line.trim_start_matches(|one: char| one == '>' || one.is_whitespace());
        let listed = bare.starts_with(['-', '*', '+'])
            || bare.split_once('.').is_some_and(|(head, _)| {
                !head.is_empty() && head.chars().all(|one| one.is_ascii_digit())
            });
        if bare.is_empty() || listed {
            break;
        }
        words.push(bare.trim_end());
    }
    Some(flat(&words.join(" "))).filter(|text| !text.is_empty())
}

fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn title(text: &str) -> String {
    let flat = flat(text);
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

// A lone pair left between two that agree is a rename; more than one each way cannot be told apart.
fn paired(from: &[String], to: &[String]) -> Vec<Option<usize>> {
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

fn words(stages: &[Stage]) -> Vec<String> {
    stages.iter().map(|one| one.text.clone()).collect()
}

// Without an earlier text, what the steps say is taken as what the document said before.
fn earlier(steps: &[&Step], before: Option<&str>) -> Vec<Stage> {
    match before {
        Some(body) => stages(body),
        None => steps
            .iter()
            .map(|one| Stage {
                text: flat(&one.text),
                done: one.done,
                at: 0..0,
            })
            .collect(),
    }
}

/// Only what changed between the two texts reaches the log, so a step never written into the
/// document, or ticked elsewhere since, is left as it is.
pub fn reconciled(task: &Task, before: Option<&str>, now: &str) -> Vec<Op> {
    let steps = ordered(task);
    let said: Vec<String> = steps.iter().map(|one| flat(&one.text)).collect();
    let was = earlier(&steps, before);
    let is = stages(now);
    let step_of = paired(&said, &words(&was));
    let was_of = paired(&words(&was), &words(&is));
    let bound: Vec<Option<(usize, usize)>> = was_of
        .iter()
        .map(|one| one.and_then(|w| step_of[w].map(|s| (w, s))))
        .collect();
    let mut ops = Vec::new();

    let held: Vec<(usize, &str)> = bound
        .iter()
        .enumerate()
        .filter_map(|(at, one)| one.map(|(_, s)| (at, steps[s].order.as_str())))
        .collect();
    let keys: Vec<&str> = held.iter().map(|(_, key)| *key).collect();
    let mut placed: Vec<Option<String>> = vec![None; is.len()];
    for ((at, key), fresh) in held.iter().zip(crate::order::resequenced(&keys)) {
        placed[*at] = Some(fresh.clone().unwrap_or_else(|| key.to_string()));
        if let (Some(order), Some((_, s))) = (fresh, bound[*at]) {
            ops.push(Op::StepReorder {
                id: task.id,
                d: StepReorder {
                    step: steps[s].id,
                    order,
                },
            });
        }
    }

    let mut before_key: Option<String> = None;
    for (at, one) in is.iter().enumerate() {
        let Some((w, s)) = bound[at] else {
            let after = placed[at + 1..].iter().flatten().next().cloned();
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
            continue;
        };
        let step = steps[s];
        if one.text != was[w].text && title(&one.text) != flat(&step.text) {
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
        before_key.clone_from(&placed[at]);
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

/// The document with every box ticked as its step is, since the file's own boxes may be stale.
pub fn ticked(task: &Task, body: &str) -> String {
    let steps = ordered(task);
    let said: Vec<String> = steps.iter().map(|one| flat(&one.text)).collect();
    let found = stages(body);
    let of = paired(&said, &words(&found));
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
