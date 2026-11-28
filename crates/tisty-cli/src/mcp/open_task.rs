use serde_json::{Value, json};
use tisty_core::model::{Task, TaskId};
use tisty_core::{Config, Paths};

use super::jsonrpc::with_ask;

// Long enough that reading around a task before starting it is never taken for work done on it.
const QUIET_FOR_SECONDS: i64 = 600;
const WRITES_ELSEWHERE: usize = 3;
const STEP_SHOWN: usize = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Touch {
    Read(TaskId),
    Proposed(TaskId),
    Worked(TaskId),
    Ended,
    Elsewhere,
    Looked,
}

pub(super) fn aimed(args: &Value) -> Option<TaskId> {
    if let Some(task) = args.get("task").and_then(Value::as_str) {
        return task.trim().parse().ok();
    }
    let doc = args.get("doc").and_then(Value::as_str)?.trim();
    doc.parse()
        .ok()
        .or_else(|| tisty_core::task_docs::task_of_file(doc))
}

pub(super) fn touch(name: &str, aimed: Option<TaskId>, writes: bool, said: &Value) -> Touch {
    let shown = &said["structuredContent"];
    match name {
        "task_resolve" => Touch::Ended,
        // Only a task's own read opens it: its page read as a document is a look, as it always was.
        "workspace_read" => match aimed {
            Some(task) if still_open(shown) && shown["id"] == task.to_string() => Touch::Read(task),
            _ => Touch::Looked,
        },
        // A task filed with nothing to tick is a note for the person, never the agent's own work.
        "task_create" => match (shown["proposed"].as_bool(), shown["id"].as_str()) {
            (Some(true), Some(id)) if shown["steps"].as_u64().is_some_and(|steps| steps > 0) => {
                id.parse().map_or(Touch::Looked, Touch::Proposed)
            }
            _ => Touch::Looked,
        },
        _ if !writes => Touch::Looked,
        _ => aimed.map_or(Touch::Elsewhere, Touch::Worked),
    }
}

fn still_open(shown: &Value) -> bool {
    shown["status"]
        .as_str()
        .is_none_or(|status| status == "open")
        && shown.get("said_done").is_none()
        && shown.get("said_not_doing").is_none()
}

struct Stretch {
    task: TaskId,
    since: jiff::Timestamp,
    elsewhere: usize,
    asked: bool,
    only_proposed: bool,
    others_read: bool,
}

impl Stretch {
    fn opened(task: TaskId, now: jiff::Timestamp, only_proposed: bool) -> Self {
        Self {
            task,
            since: now,
            elsewhere: 0,
            asked: false,
            only_proposed,
            others_read: false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Due {
    pub(super) task: TaskId,
    pub(super) minutes: i64,
    pub(super) elsewhere: usize,
    pub(super) only_proposed: bool,
    pub(super) others_read: bool,
}

#[derive(Default)]
pub(super) struct OpenTask {
    open: Option<Stretch>,
}

impl OpenTask {
    pub(super) fn heard(&mut self, touch: Touch, now: jiff::Timestamp) -> Option<Due> {
        match touch {
            Touch::Worked(task) => {
                self.open = Some(Stretch::opened(task, now, false));
                return None;
            }
            Touch::Ended => {
                self.open = None;
                return None;
            }
            Touch::Read(task) | Touch::Proposed(task) => {
                let proposed = matches!(touch, Touch::Proposed(_));
                match &mut self.open {
                    Some(open) if open.task == task => open.only_proposed &= proposed,
                    Some(open) if !open.asked => open.others_read |= !proposed,
                    _ => {
                        self.open = Some(Stretch::opened(task, now, proposed));
                        return None;
                    }
                }
            }
            Touch::Elsewhere => {
                if let Some(open) = &mut self.open {
                    open.elsewhere += 1;
                }
            }
            Touch::Looked => {}
        }
        let open = self.open.as_ref()?;
        let quiet = now.as_second() - open.since.as_second();
        let due = quiet >= QUIET_FOR_SECONDS || open.elsewhere >= WRITES_ELSEWHERE;
        (!open.asked && due).then_some(Due {
            task: open.task,
            minutes: quiet / 60,
            elsewhere: open.elsewhere,
            only_proposed: open.only_proposed,
            others_read: open.others_read,
        })
    }

    fn asked(&mut self) {
        if let Some(open) = &mut self.open {
            open.asked = true;
        }
    }

    fn let_go(&mut self) {
        self.open = None;
    }

    pub(super) fn answered(
        &mut self,
        paths: &Paths,
        touch: Touch,
        mut said: Value,
        now: jiff::Timestamp,
    ) -> Value {
        let Some(due) = self.heard(touch, now) else {
            return said;
        };
        // Nothing structured is all text, and an answer asks one thing, so the question waits for the next.
        if !said["structuredContent"].is_object() || said["structuredContent"].get("ask").is_some()
        {
            return said;
        }
        match asked_about(paths, &due) {
            Asked::Ask(ask) => {
                if with_ask(&mut said, ask) {
                    self.asked();
                }
            }
            Asked::Off => self.asked(),
            Asked::Gone => self.let_go(),
            Asked::Later => {}
        }
        said
    }
}

enum Asked {
    Ask(Value),
    Off,
    Gone,
    Later,
}

fn asked_about(paths: &Paths, due: &Due) -> Asked {
    let off = Config::load(&paths.config_file())
        .ok()
        .flatten()
        .is_some_and(|config| !config.asks_open_task());
    if off {
        return Asked::Off;
    }
    let state = match super::admitted(paths) {
        Ok(Some((state, _))) => state,
        Ok(None) => return Asked::Gone,
        Err(_) => {
            // `hitch` logged it, and its flag belongs to a refused call, never to this answered one.
            super::TOLD.take();
            return Asked::Later;
        }
    };
    state
        .tasks
        .get(&due.task)
        .and_then(|task| question(task, due))
        .map_or(Asked::Gone, Asked::Ask)
}

pub(crate) const TURN_LEFT_UNWRITTEN: &str = "You edited files this turn and wrote nothing in \
    Tisty. If this was work on a Tisty task, leave it in one `task_edit_steps` (`action: \"tick\"` \
    or `\"set_state\"`): the steps you finished, if any, and `state` with what is done, what is \
    left and the decisions in force. If it was not work on a task, end your turn without writing \
    anything.";

pub(crate) const PUBLISHED_ASK: &str = "You just pushed or opened or merged a pull request. Did \
    you finish a step or the task? If it was a step of a Tisty task, call `task_edit_steps` \
    (`action: \"tick\"`) with `state`; if the work is done and proven, call `task_resolve` \
    (`outcome: \"done\"`) with the branch, the commit and how you verified it. Merging and \
    publishing after that are the person's, and need no waiting.";

pub(crate) fn ready_to_say_done(paths: &Paths, said: &str) -> Option<String> {
    let id = said.parse::<TaskId>().ok()?;
    let state = match super::admitted(paths) {
        Ok(Some((state, _))) => state,
        Ok(None) => return None,
        Err(_) => {
            super::TOLD.take();
            return None;
        }
    };
    let task = state.tasks.get(&id)?;
    let ready = task.is_open()
        && !task.folded()
        && task.resolved.is_none()
        && state.agents_own(task)
        && !task.steps.is_empty()
        && task.steps.iter().all(|step| step.done);
    ready.then(|| {
        format!(
            "Every step of {:?} is ticked, but it is not said done. If the work is done and \
             proven, call `task_resolve` (`outcome: \"done\"`) now with the branch, the commit \
             and how you verified it; what only the person can do goes to them with \
             `task_edit_steps` (`action: \"remove\"`, `to_person: true`). If it is not done, \
             end your turn.",
            task.title
        )
    })
}

pub(super) fn question(task: &Task, due: &Due) -> Option<Value> {
    if !task.is_open() || task.folded() || task.resolved.is_some() || task.for_person {
        return None;
    }
    let count = task.step_count();
    let all = count.all;
    if due.only_proposed && all == 0 {
        return None;
    }
    let left: Vec<&str> = task
        .steps
        .iter()
        .filter(|step| !step.done && !step.said)
        .map(|step| step.text.trim())
        .collect();
    let quiet = match due.minutes * 60 >= QUIET_FOR_SECONDS {
        true => format!("You have written nothing on it for {} minutes", due.minutes),
        false => format!(
            "You wrote {} other things without writing on it",
            due.elsewhere
        ),
    };
    let waiting = super::brain::project::word_waiting(count.said);
    let missing = match (all, left.first()) {
        (0, _) => String::new(),
        (_, None) if count.said == 0 => ", and every step is ticked but it is not said done".into(),
        (_, None) => format!(
            ", and {} of {all} steps are ticked{waiting}, but it is not said done",
            count.done
        ),
        (_, Some(next)) => format!(
            ", and {} of {all} steps {} still unticked, the next {:?}{waiting}",
            left.len(),
            match left.len() {
                1 => "is",
                _ => "are",
            },
            cut(next)
        ),
    };
    let stale = match tisty_core::task_state::since(task) {
        Some(since) if since.steps => "; its state is older than its steps",
        _ => "",
    };
    let another = match due.others_read {
        true => " If you were working on another task, update that one instead.",
        false => "",
    };
    let said = format!(
        "Is {:?} up to date? {quiet}{missing}{stale}. Leave it in one `task_edit_steps` \
         (`action: \"tick\"` or `\"set_state\"`): the steps you finished, if any, and `state` \
         with what is done, what is left and the decisions in force. If the work is done, call \
         `task_resolve` (`outcome: \"done\"`) with the proof. A decision, something ruled out or \
         a block goes in `task_note`.{another}",
        task.title
    );
    Some(json!({
        "task": task.id.to_string(),
        "title": task.title,
        "left": left.len(),
        "said": said,
    }))
}

fn cut(step: &str) -> String {
    match step.char_indices().nth(STEP_SHOWN) {
        Some((at, _)) => format!("{}…", &step[..at]),
        None => step.to_string(),
    }
}

#[cfg(test)]
#[path = "open_task_test.rs"]
mod tests;
