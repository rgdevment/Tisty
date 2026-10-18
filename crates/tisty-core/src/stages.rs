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

/// Every box a document ticks, in reading order, wherever it sits: a quote, a callout, a fold.
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

fn said(item: &str) -> Option<String> {
    let line = item.lines().next()?;
    let (_, rest) = line.split_once(']')?;
    Some(rest.trim().to_string())
}

/// What the log keeps of a stage: its words, cut at a word when they run past a step's length.
pub fn title(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
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

fn longest(a: &[String], b: &[String]) -> Vec<(usize, usize)> {
    let mut kept = vec![vec![0u32; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            kept[i][j] = match a[i] == b[j] {
                true => kept[i + 1][j + 1] + 1,
                false => kept[i + 1][j].max(kept[i][j + 1]),
            };
        }
    }
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            out.push((i, j));
            i += 1;
            j += 1;
        } else if kept[i + 1][j] >= kept[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

/// Which step each stage is, by its words where they still agree, then by the same words
/// moved elsewhere, then by place: one stage left between two that agree is the one renamed.
pub fn matched(steps: &[&Step], stages: &[Stage]) -> Vec<Option<usize>> {
    let said: Vec<String> = steps.iter().map(|one| one.text.clone()).collect();
    let titled: Vec<String> = stages.iter().map(|one| title(&one.text)).collect();
    let mut of = vec![None; stages.len()];
    let mut taken = vec![false; steps.len()];
    let anchors = longest(&said, &titled);
    for (step, stage) in &anchors {
        of[*stage] = Some(*step);
        taken[*step] = true;
    }
    for (stage, words) in titled.iter().enumerate() {
        if of[stage].is_some() {
            continue;
        }
        if let Some(step) = (0..steps.len()).find(|at| !taken[*at] && said[*at] == *words) {
            of[stage] = Some(step);
            taken[step] = true;
        }
    }
    let mut bounds: Vec<(usize, usize)> = vec![(0, 0)];
    bounds.extend(anchors.iter().map(|(step, stage)| (step + 1, stage + 1)));
    let ends: Vec<(usize, usize)> = anchors
        .iter()
        .copied()
        .chain(std::iter::once((steps.len(), stages.len())))
        .collect();
    for ((step_from, stage_from), (step_to, stage_to)) in bounds.into_iter().zip(ends) {
        let loose_steps: Vec<usize> = (step_from..step_to).filter(|at| !taken[*at]).collect();
        let loose_stages: Vec<usize> = (stage_from..stage_to)
            .filter(|at| of[*at].is_none())
            .collect();
        if loose_steps.len() == loose_stages.len() {
            for (step, stage) in loose_steps.into_iter().zip(loose_stages) {
                of[stage] = Some(step);
                taken[step] = true;
            }
        }
    }
    of
}

/// What the log has to learn so the task's steps say what its document says. Whether a step is
/// done is the log's own: a box left unticked in the file never undoes a tick made elsewhere.
pub fn reconciled(task: &Task, body: &str) -> Vec<Op> {
    let steps = ordered(task);
    let found = stages(body);
    let of = matched(&steps, &found);
    let mut ops = Vec::new();

    let held: Vec<(usize, &str)> = of
        .iter()
        .enumerate()
        .filter_map(|(stage, step)| step.map(|step| (stage, steps[step].order.as_str())))
        .collect();
    let keys: Vec<&str> = held.iter().map(|(_, key)| *key).collect();
    let fresh = crate::order::resequenced(&keys);
    let mut placed: Vec<Option<String>> = vec![None; found.len()];
    for ((stage, key), new) in held.iter().zip(fresh) {
        placed[*stage] = Some(new.clone().unwrap_or_else(|| key.to_string()));
        if let (Some(order), Some(step)) = (new, of[*stage]) {
            ops.push(Op::StepReorder {
                id: task.id,
                d: StepReorder {
                    step: steps[step].id,
                    order,
                },
            });
        }
    }

    let mut before: Option<String> = None;
    for (stage, one) in found.iter().enumerate() {
        if let Some(step) = of[stage] {
            let words = title(&one.text);
            if steps[step].text != words {
                ops.push(Op::StepText {
                    id: task.id,
                    d: StepText {
                        step: steps[step].id,
                        text: words,
                    },
                });
            }
            before.clone_from(&placed[stage]);
            continue;
        }
        let after = placed[stage + 1..].iter().flatten().next().cloned();
        let order = crate::order::between(before.as_deref(), after.as_deref());
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
        before = Some(order);
    }

    let kept: Vec<usize> = of.iter().flatten().copied().collect();
    for (at, step) in steps.iter().enumerate() {
        if !kept.contains(&at) {
            ops.push(Op::StepRemove {
                id: task.id,
                d: StepRef { step: step.id },
            });
        }
    }
    ops
}

/// The document with every box ticked as its step is, since the file's own boxes may be stale.
pub fn ticked(task: &Task, body: &str) -> String {
    let steps = ordered(task);
    let found = stages(body);
    let of = matched(&steps, &found);
    let mut out = body.to_string();
    for (stage, step) in found.iter().zip(&of).rev() {
        let Some(step) = step else { continue };
        if stage.done == steps[*step].done {
            continue;
        }
        let Some(open) = body[stage.at.clone()].find('[') else {
            continue;
        };
        let at = stage.at.start + open + 1;
        let mark = if steps[*step].done { "x" } else { " " };
        out.replace_range(at..at + 1, mark);
    }
    out
}

#[cfg(test)]
#[path = "stages_test.rs"]
mod tests;
