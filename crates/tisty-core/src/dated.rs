use std::collections::HashMap;

use crate::event::{Event, Op};
use crate::model::{StepId, Task};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Dated {
    pub text: String,
    pub born: Option<jiff::Timestamp>,
    pub done: Option<jiff::Timestamp>,
}

/// When each of a task's steps was added and, while it stays ticked, when it was ticked.
pub fn dated(log: &[Event], task: &Task) -> Vec<Dated> {
    let mut born: HashMap<StepId, jiff::Timestamp> = HashMap::new();
    let mut done: HashMap<StepId, jiff::Timestamp> = HashMap::new();
    for event in log {
        match &event.op {
            Op::StepAdd { id, d } if *id == task.id => {
                born.entry(d.step).or_insert(event.timestamp);
            }
            Op::StepDone { id, d } if *id == task.id => {
                done.insert(d.step, event.timestamp);
            }
            Op::StepUndone { id, d } if *id == task.id => {
                done.remove(&d.step);
            }
            _ => {}
        }
    }
    crate::stages::ordered(task)
        .into_iter()
        .map(|step| Dated {
            text: step.text.clone(),
            born: born.get(&step.id).copied(),
            done: done.get(&step.id).copied().filter(|_| step.done),
        })
        .collect()
}

#[cfg(test)]
#[path = "dated_test.rs"]
mod tests;
