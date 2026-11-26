use serde_json::{Value, json};
use tisty_core::model::{StepCount, Task};
use tisty_core::{Paths, State};

use super::super::asked::{how_many, text};
use super::super::chores::seen_task;
use super::super::jsonrpc::mirrored;
use super::super::{Refused, opened};
use super::SIFTS;

const PARTS_BY_DEFAULT: u64 = 50;
const PARTS_AT_MOST: u64 = 100;
const BLOCKED: [&str; 2] = ["bloqueada:", "blocked:"];
const NEXT_SHOWN: usize = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Stage {
    Blocked,
    UnderWay,
    Waiting,
    NotStarted,
}

impl Stage {
    pub(super) fn said(self) -> &'static str {
        match self {
            Stage::Blocked => "blocked",
            Stage::UnderWay => "under_way",
            Stage::Waiting => "waiting",
            Stage::NotStarted => "not_started",
        }
    }

    fn told(self) -> &'static str {
        match self {
            Stage::Blocked => "blocked",
            Stage::UnderWay => "under way",
            Stage::Waiting => "waiting on the person",
            Stage::NotStarted => "not started",
        }
    }
}

pub(super) fn asked(args: &Value) -> bool {
    text(args, "part_of").is_some()
        && text(args, "query").is_none()
        && ["scope", "lines"]
            .iter()
            .all(|key| args.get(*key).is_none())
        && SIFTS
            .iter()
            .all(|key| *key == "part_of" || args.get(*key).is_none())
}

pub(super) fn stage(task: &Task, state_line: Option<&str>) -> Stage {
    if state_line.is_some_and(blocked) {
        return Stage::Blocked;
    }
    if task.resolved.is_some() {
        return Stage::Waiting;
    }
    let count = task.step_count();
    match count.done + count.said > 0 || state_line.is_some() {
        true => Stage::UnderWay,
        false => Stage::NotStarted,
    }
}

fn blocked(line: &str) -> bool {
    let bare = tisty_core::text::folded(line.trim_start_matches(['*', '_', ' ', '>', '-']));
    BLOCKED.iter().any(|one| bare.starts_with(one))
}

pub(super) fn state_line(paths: &Paths, state: &State, task: &Task) -> Option<String> {
    task.stated_at?;
    let owner = tisty_core::task_docs::owner(state, task.id);
    let kept = tisty_core::task_docs::of(state, owner)?;
    let body = tisty_core::docs::read(&paths.docs(), &kept.file).ok()?;
    let said = tisty_core::task_state::said(&body)?;
    let first = said
        .lines()
        .map(str::trim)
        .find(|one| !one.is_empty() && tisty_core::assistant_notes::signed(one).is_empty())?;
    Some(tisty_core::view::clipped(first))
}

pub(in crate::mcp) struct Progress {
    stage: Stage,
    count: StepCount,
    next: Option<String>,
    state: Option<String>,
    old: bool,
}

pub(in crate::mcp) fn progress(paths: &Paths, state: &State, task: &Task) -> Progress {
    let line = state_line(paths, state, task);
    Progress {
        stage: stage(task, line.as_deref()),
        count: task.step_count(),
        next: task
            .steps
            .iter()
            .find(|step| !step.done && !step.said)
            .map(|step| cut(step.text.trim())),
        old: line.is_some() && tisty_core::task_state::since(task).is_some_and(|since| since.old()),
        state: line,
    }
}

impl Progress {
    pub(in crate::mcp) fn how_far(&self) -> String {
        let mut said = self.stage.told().to_string();
        let StepCount {
            done,
            said: word,
            all,
        } = self.count;
        match (&self.next, all, word) {
            (_, 0, _) => {}
            (None, all, 0) => said.push_str(&format!(", all {all} steps ticked")),
            (next, all, _) => {
                said.push_str(&format!(
                    ", {done} of {all} steps ticked{}",
                    word_waiting(word)
                ));
                if let Some(next) = next {
                    said.push_str(&format!(", next {next:?}"));
                }
            }
        }
        said
    }

    pub(in crate::mcp) fn told(&self) -> String {
        let mut said = self.how_far();
        if let Some(line) = &self.state {
            said.push_str(&format!("; state: {line}"));
            if self.old {
                said.push_str(" (older than its steps or notes)");
            }
        }
        said
    }

    pub(in crate::mcp) fn put(&self, into: &mut Value) {
        into["stage"] = json!(self.stage.said());
        counted(into, self.count);
        if let Some(line) = &self.state {
            into["state"] = json!(line);
        }
        if self.old {
            into["state_old"] = json!(true);
        }
    }
}

fn counted(into: &mut Value, count: StepCount) {
    if count.all > 0 {
        into["steps"] = json!([count.done, count.all]);
    }
    if count.said > 0 {
        into["steps_said"] = json!(count.said);
    }
}

pub(in crate::mcp) fn word_waiting(said: usize) -> String {
    match said {
        0 => String::new(),
        said => format!(", {said} said done and waiting on the person"),
    }
}

fn cut(step: &str) -> String {
    match step.char_indices().nth(NEXT_SHOWN) {
        Some((at, _)) => format!("{}…", &step[..at]),
        None => step.to_string(),
    }
}

pub(super) fn last_moved(task: &Task) -> jiff::Timestamp {
    let made = jiff::Timestamp::from_millisecond(task.id.timestamp_ms() as i64)
        .unwrap_or(jiff::Timestamp::UNIX_EPOCH);
    task.log
        .iter()
        .map(|one| one.at)
        .chain(task.completed_at)
        .chain(task.stated_at)
        .chain(task.steps_moved_at)
        .fold(made, std::cmp::max)
}

pub(super) fn view(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let said = text(args, "part_of").unwrap_or_default();
    let (whole_id, whole) = seen_task(&state, &said)?;
    let most = how_many(args, PARTS_BY_DEFAULT, PARTS_AT_MOST);
    let past = args.get("after").and_then(Value::as_u64).unwrap_or(0) as usize;
    let parts: Vec<&Task> = state
        .tasks
        .values()
        .filter(|one| one.part_of == Some(whole_id) && !one.folded())
        .collect();
    let closed = parts.iter().filter(|one| !one.is_open()).count();
    let mut open: Vec<(Stage, jiff::Timestamp, &Task, Option<String>)> = parts
        .iter()
        .filter(|one| one.is_open())
        .map(|one| {
            let line = state_line(paths, &state, one);
            (stage(one, line.as_deref()), last_moved(one), *one, line)
        })
        .collect();
    open.sort_by_key(|(stage, moved, task, _)| (*stage, std::cmp::Reverse(*moved), task.id));
    let mut counts = serde_json::Map::new();
    for stage in [
        Stage::Blocked,
        Stage::UnderWay,
        Stage::Waiting,
        Stage::NotStarted,
    ] {
        counts.insert(
            stage.said().into(),
            json!(open.iter().filter(|one| one.0 == stage).count()),
        );
    }
    let total = open.len();
    let shown: Vec<&(Stage, jiff::Timestamp, &Task, Option<String>)> =
        open.iter().skip(past).take(most).collect();
    let rows: Vec<Value> = shown
        .iter()
        .map(|(stage, moved, task, line)| row(*stage, *moved, task, line.as_deref()))
        .collect();
    let mut kept = json!({
        "project": { "id": whole.id.to_string(), "title": whole.title },
        "stages": counts,
        "closed": closed,
        "parts": rows,
        "total": total,
    });
    if past + rows.len() < total {
        kept["next"] = json!(format!(
            "`after` {} brings the next of the {total} open parts.",
            past + rows.len()
        ));
    }
    let lines: Vec<String> = shown
        .iter()
        .map(|(stage, moved, task, line)| {
            let steps = match task.step_count() {
                StepCount { all: 0, .. } => String::new(),
                StepCount { done, said, all } => {
                    format!(", {done}/{all} steps{}", word_waiting(said))
                }
            };
            let said = line
                .as_deref()
                .map(|one| format!("\n  {one}"))
                .unwrap_or_default();
            format!(
                "{} — {:?} [{}], last moved {}{steps}{said}",
                stage.told(),
                task.title,
                task.id,
                day(*moved)
            )
        })
        .collect();
    let text = format!(
        "{:?} has {total} open part(s): {}; {closed} closed.\n{}",
        whole.title,
        [
            Stage::Blocked,
            Stage::UnderWay,
            Stage::Waiting,
            Stage::NotStarted
        ]
        .iter()
        .map(|stage| format!(
            "{} {}",
            open.iter().filter(|one| one.0 == *stage).count(),
            stage.told()
        ))
        .collect::<Vec<_>>()
        .join(", "),
        lines.join("\n")
    );
    super::super::each::said_once(&mut kept, &["parts"]);
    Ok(mirrored(text, kept))
}

fn row(stage: Stage, moved: jiff::Timestamp, task: &Task, line: Option<&str>) -> Value {
    let mut one = json!({
        "id": task.id.to_string(),
        "title": task.title,
        "stage": stage.said(),
        "moved": day(moved),
    });
    counted(&mut one, task.step_count());
    if let Some(line) = line {
        one["state"] = json!(line);
    }
    one
}

pub(super) fn day(at: jiff::Timestamp) -> String {
    at.to_zoned_datetime(jiff::tz::TimeZone::system())
        .date()
        .to_string()
}

#[cfg(test)]
#[path = "project_test.rs"]
mod tests;
