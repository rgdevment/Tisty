use jiff::Timestamp;

use crate::event::{Op, PageChange, Paged};
use crate::model::{Task, TaskId};

pub const STATE: [&str; 2] = ["## State", "## Estado"];

// Every pickup of the task reads it whole, so it says what stands and nothing of how it got there.
pub const STATE_AT_MOST: usize = 3_000;

pub fn named(section: &str) -> bool {
    STATE
        .iter()
        .any(|one| one.strip_prefix("## ") == Some(section.trim()))
}

pub fn section(body: &str) -> Option<&str> {
    crate::closing::held(body, &STATE).map(|(_, words)| &body[words])
}

pub fn heading_in(body: &str) -> Option<&'static str> {
    STATE
        .iter()
        .find(|one| crate::closing::held(body, &[**one]).is_some())
        .map(|one| one.trim_start_matches("## "))
}

pub fn said(body: &str) -> Option<&str> {
    section(body)
        .map(str::trim)
        .filter(|words| !words.is_empty())
}

const SPANISH: [&str; 5] = [
    "## Descripción",
    "## Pasos",
    "## Notas",
    "## Notas del asistente",
    "## Cada vuelta",
];
const ENGLISH: [&str; 5] = [
    "## Description",
    "## Steps",
    "## Notes",
    "## Assistant notes",
    "## Every round",
];

fn spoken_in(body: &str) -> Option<bool> {
    let holds = |heads: &[&str]| body.lines().any(|line| heads.contains(&line.trim_end()));
    match (holds(&SPANISH), holds(&ENGLISH)) {
        (true, false) => Some(true),
        (false, true) => Some(false),
        _ => None,
    }
}

pub fn stated(body: &str, state: &str, spanish: bool) -> String {
    let line = if body.contains("\r\n") { "\r\n" } else { "\n" };
    let state =
        crate::assistant_notes::kept_inside(state.replace("\r\n", "\n").trim()).replace('\n', line);
    if let Some((_, words)) = crate::closing::held(body, &STATE) {
        let head = body[..words.start].trim_end_matches(['\n', '\r']);
        let rest = &body[words.end..];
        let tail = if rest.is_empty() { "" } else { line };
        return format!("{head}{line}{line}{state}{line}{tail}{rest}");
    }
    let heading = STATE[usize::from(spoken_in(body).unwrap_or(spanish))];
    let before = crate::closing::held(body, &crate::task_docs::NOTES)
        .or_else(|| crate::closing::held(body, &crate::assistant_notes::ASSISTANT_NOTES))
        .map(|(head, _)| head.start);
    match before {
        Some(at) => {
            let top = body[..at].trim_end_matches(['\n', '\r']);
            format!(
                "{top}{line}{line}{heading}{line}{line}{state}{line}{line}{}",
                &body[at..]
            )
        }
        None => {
            let top = body.trim_end_matches(['\n', '\r']);
            format!("{top}{line}{line}{heading}{line}{line}{state}{line}")
        }
    }
}

pub(crate) fn paged(task: &mut Task, d: &Paged, at: Timestamp) {
    let into = named(&d.section);
    match &d.change {
        PageChange::Removed if into => task.stated_at = None,
        PageChange::Renamed { from } if named(from) && !into => task.stated_at = None,
        PageChange::Added | PageChange::Written | PageChange::Renamed { .. } if into => {
            task.stated_at = Some(at);
        }
        _ => {}
    }
}

pub(crate) fn steps_moved(op: &Op) -> Option<TaskId> {
    match op {
        Op::StepAdd { id, .. }
        | Op::StepDone { id, .. }
        | Op::StepUndone { id, .. }
        | Op::StepSaid { id, .. }
        | Op::StepUnsaid { id, .. }
        | Op::StepText { id, .. }
        | Op::StepRemove { id, .. } => Some(*id),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Since {
    pub steps: bool,
    pub notes: usize,
}

impl Since {
    pub fn old(self) -> bool {
        self.steps || self.notes > 0
    }
}

pub fn since(task: &Task) -> Option<Since> {
    let at = task.stated_at?;
    Some(Since {
        steps: task.steps_moved_at.is_some_and(|moved| moved > at),
        notes: task.journal().filter(|one| one.at > at).count(),
    })
}

#[cfg(test)]
#[path = "task_state_test.rs"]
mod tests;
