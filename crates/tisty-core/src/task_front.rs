use jiff::civil::{Date, DateTime};

use crate::model::{Cadence, DateSpec, From, Priority, Repeat, Task, TaskId, Unit};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Front {
    pub task: Option<TaskId>,
    pub scheduled: Option<DateSpec>,
    pub due: Option<DateSpec>,
    pub remind: Vec<DateSpec>,
    pub list: Option<String>,
    pub priority: Option<Priority>,
    pub repeat: Option<Repeat>,
    pub tags: Vec<String>,
}

const KEYS: [&str; 9] = [
    crate::task_docs::KEY,
    "scheduled",
    "due",
    "remind",
    "list",
    "priority",
    "repeat",
    "tags",
    "tag",
];

pub fn day(one: &DateSpec) -> String {
    let date = one.at.date();
    match (one.has_time, one.floating) {
        (false, _) => date.to_string(),
        (true, true) => format!("{date}T{}", one.at.time().strftime("%H:%M")),
        (true, false) => format!("{date}T{}[{}]", one.at.time().strftime("%H:%M"), one.tz),
    }
}

pub fn day_in(said: &str, tz: &str) -> Option<DateSpec> {
    let said = said.trim();
    let (when, zone) = match said.split_once('[') {
        Some((when, rest)) => (when, Some(rest.strip_suffix(']')?)),
        None => (said, None),
    };
    if !when.contains('T') {
        return Some(DateSpec::all_day(when.parse::<Date>().ok()?, tz));
    }
    let at = when.parse::<DateTime>().ok()?;
    match zone {
        Some(zone) => jiff::tz::TimeZone::get(zone)
            .is_ok()
            .then(|| DateSpec::fixed(at, zone)),
        None => Some(DateSpec::floating(at, tz)),
    }
}

pub fn repeat(one: Repeat) -> String {
    let unit = match one.each.unit {
        Unit::Day => "day",
        Unit::Week => "week",
        Unit::Month => "month",
        Unit::Year => "year",
    };
    let each = match one.each.every {
        1 => format!("every {unit}"),
        many => format!("every {many} {unit}s"),
    };
    let from = match one.from {
        From::Due => "due",
        From::Done => "done",
    };
    let until = one
        .until
        .map(|at| format!(" until {at}"))
        .unwrap_or_default();
    format!("{each} from {from}{until}")
}

pub fn repeat_in(said: &str) -> Option<Repeat> {
    let words: Vec<&str> = said.split_whitespace().collect();
    let mut at = 0;
    (words.first()?.eq_ignore_ascii_case("every")).then_some(())?;
    at += 1;
    let every = match words.get(at)?.parse::<u16>() {
        Ok(every) if every > 0 => {
            at += 1;
            every
        }
        Ok(_) => return None,
        Err(_) => 1,
    };
    let unit = match words.get(at)?.to_ascii_lowercase().trim_end_matches('s') {
        "day" => Unit::Day,
        "week" => Unit::Week,
        "month" => Unit::Month,
        "year" => Unit::Year,
        _ => return None,
    };
    at += 1;
    let mut from = From::Due;
    let mut until = None;
    while let Some(word) = words.get(at) {
        match (word.to_ascii_lowercase().as_str(), words.get(at + 1)) {
            ("from", Some(next)) if next.eq_ignore_ascii_case("done") => from = From::Done,
            ("from", Some(next)) if next.eq_ignore_ascii_case("due") => from = From::Due,
            ("until", Some(next)) => until = Some(next.parse::<Date>().ok()?),
            _ => return None,
        }
        at += 2;
    }
    Some(Repeat {
        from,
        each: Cadence { every, unit },
        until,
    })
}

// Words YAML would read as a number, a yes or no, nothing, or its own syntax keep their quotes.
fn quoted(said: &str) -> String {
    const WORDS: [&str; 11] = [
        "yes", "no", "true", "false", "on", "off", "null", "~", "y", "n", "",
    ];
    let lower = said.to_ascii_lowercase();
    let plain = !said.contains([':', '#', '[', ']', '{', '}', ',', '"', '\''])
        && said.trim() == said
        && !said.starts_with(['-', '&', '*', '!', '@', '|', '>', '%', '?', '`'])
        && !WORDS.contains(&lower.as_str())
        && lower.parse::<f64>().is_err();
    match plain {
        true => said.to_string(),
        false => tisty_markdown::front::quoted(said),
    }
}

fn listed(all: &[String]) -> String {
    let inside: Vec<String> = all.iter().map(|one| quoted(one)).collect();
    format!("[{}]", inside.join(", "))
}

pub fn lines(task: &Task, list: Option<&str>) -> Vec<String> {
    let mut out = vec![format!("{}: {}", crate::task_docs::KEY, task.id)];
    if let Some(one) = &task.date {
        out.push(format!("scheduled: {}", day(one)));
    }
    if let Some(one) = &task.deadline {
        out.push(format!("due: {}", day(one)));
    }
    if !task.reminders.is_empty() {
        let all: Vec<String> = task.reminders.iter().map(day).collect();
        out.push(format!("remind: {}", listed(&all)));
    }
    if let Some(name) = list {
        out.push(format!("list: {}", quoted(name)));
    }
    if task.priority.set() {
        out.push(format!("priority: {}", task.priority.name()));
    }
    if let Some(one) = task.repeat {
        out.push(format!("repeat: {}", repeat(one)));
    }
    if !task.tags.is_empty() {
        let all: Vec<String> = task.tags.iter().map(ToString::to_string).collect();
        out.push(format!("tags: {}", listed(&all)));
    }
    out
}

pub fn dressed(body: &str, task: &Task, list: Option<&str>) -> String {
    let ticked = crate::stages::ticked(task, body);
    let body = ticked.as_str();
    let (others, rest) = match tisty_markdown::front::span(body) {
        Some(at) => (theirs(&body[at.clone()]), &body[at.end..]),
        None => (Vec::new(), body),
    };
    let mut block = lines(task, list);
    block.extend(others);
    format!("---\n{}\n---\n{rest}", block.join("\n"))
}

fn theirs(block: &str) -> Vec<String> {
    let inner: Vec<&str> = block.lines().collect();
    let inner = inner
        .get(1..inner.len().saturating_sub(1))
        .unwrap_or_default();
    let mut out = Vec::new();
    let mut skipping = false;
    for line in inner {
        let nested = line.starts_with([' ', '\t']) || line.trim_start().starts_with("- ");
        if nested && !line.trim().is_empty() && skipping {
            continue;
        }
        let key = line
            .split_once(':')
            .map(|(key, _)| key.trim().to_ascii_lowercase());
        skipping = !nested && key.is_some_and(|key| KEYS.contains(&key.as_str()));
        if !skipping {
            out.push(line.to_string());
        }
    }
    out
}

pub fn read(body: &str, tz: &str) -> Front {
    let mut front = Front::default();
    for field in tisty_markdown::front::fields(body) {
        let first = field.values.first().map(String::as_str).unwrap_or_default();
        match field.key.to_ascii_lowercase().as_str() {
            crate::task_docs::KEY => front.task = first.trim().parse().ok(),
            "scheduled" => front.scheduled = day_in(first, tz),
            "due" => front.due = day_in(first, tz),
            "remind" => {
                front.remind = field
                    .values
                    .iter()
                    .filter_map(|one| day_in(one, tz))
                    .collect()
            }
            "list" => front.list = Some(first.trim().to_string()).filter(|one| !one.is_empty()),
            "priority" => front.priority = first.trim().to_ascii_lowercase().parse().ok(),
            "repeat" => front.repeat = repeat_in(first),
            "tags" | "tag" => front.tags = tisty_markdown::front::tags(body),
            _ => {}
        }
    }
    front
}

pub fn patch(task: &Task, front: &Front) -> Option<crate::event::TaskPatch> {
    let mut out = crate::event::TaskPatch::default();
    // Compared as written, since the zone and the seconds a document cannot carry are no change.
    let shown = |one: &Option<DateSpec>| one.as_ref().map(day);
    let differs = |now: &Option<DateSpec>, said: &Option<DateSpec>| {
        said.is_some() && shown(said) != shown(now)
    };
    if differs(&task.date, &front.scheduled) {
        out.date = Some(front.scheduled.clone());
    }
    if differs(&task.deadline, &front.due) {
        out.deadline = Some(front.due.clone());
    }
    let written = |all: &[DateSpec]| all.iter().map(day).collect::<Vec<_>>();
    if !front.remind.is_empty() && written(&front.remind) != written(&task.reminders) {
        out.reminders = Some(front.remind.clone());
    }
    if let Some(priority) = front.priority.filter(|one| *one != task.priority) {
        out.priority = Some(priority);
    }
    if front.repeat.is_some() && front.repeat != task.repeat {
        out.repeat = Some(front.repeat);
    }
    let tags: Vec<crate::model::Tag> = front
        .tags
        .iter()
        .filter_map(|one| crate::model::Tag::new(one).ok())
        .collect();
    if !tags.is_empty() && set(&tags) != set(&task.tags) {
        out.tags = Some(tags);
    }
    (out != crate::event::TaskPatch::default()).then_some(out)
}

fn set(all: &[crate::model::Tag]) -> std::collections::BTreeSet<&crate::model::Tag> {
    all.iter().collect()
}

pub fn moved(state: &crate::State, task: &Task, front: &Front) -> Option<crate::event::TaskMove> {
    let name = front.list.as_deref()?.trim_start_matches('@');
    let open = || state.lists.iter().filter(|(_, one)| !one.archived);
    let (id, _) = open()
        .find(|(_, one)| one.name == name)
        .or_else(|| open().find(|(_, one)| one.name.eq_ignore_ascii_case(name)))?;
    (task.list != Some(*id)).then(|| crate::event::TaskMove {
        list: Some(Some(*id)),
        ..crate::event::TaskMove::default()
    })
}

pub struct TakenIn {
    pub task: TaskId,
    pub ops: Vec<crate::Op>,
    pub body: String,
    /// It would take words off the page or a step or a tick off the task: an old copy may be speaking.
    pub takes_away: bool,
}

/// A page naming a task hands it its properties and, against `was`, its boxes as steps.
pub fn taken_in(state: &crate::State, body: &str, was: Option<&str>, tz: &str) -> Option<TakenIn> {
    let front = read(body, tz);
    let id = front.task?;
    let task = state.tasks.get(&id)?;
    let mut ops = Vec::new();
    if let Some(d) = patch(task, &front).and_then(|d| allowed(state, task, d)) {
        ops.push(crate::Op::TaskUpdate { id, d });
    }
    if let Some(d) = moved(state, task, &front) {
        ops.push(crate::Op::TaskMove { id, d });
    }
    let (others, rest) = match tisty_markdown::front::span(body) {
        Some(at) => (theirs(&body[at.clone()]), &body[at.end..]),
        None => (Vec::new(), body),
    };
    let mut block = vec![format!("{}: {id}", crate::task_docs::KEY)];
    block.extend(others);
    let body = format!("---\n{}\n---\n{rest}", block.join("\n"));
    let before = was.map(|one| crate::stages::ticked(task, one));
    let shown = was.map_or_else(Vec::new, |one| crate::stages::shown(task, one));
    let steps: Vec<crate::Op> = crate::stages::reconciled(task, before.as_deref(), &body)
        .into_iter()
        .filter(|op| seen(op, &shown))
        .collect();
    let takes_away = was.is_some_and(|was| left_out(was, &body))
        || steps.iter().any(|op| {
            matches!(
                op,
                crate::Op::StepRemove { .. } | crate::Op::StepUndone { .. }
            )
        });
    ops.extend(steps);
    if let Some(was) = was {
        ops.extend(crate::paged::paged(id, was, &body));
    }
    Some(TakenIn {
        task: id,
        ops,
        body,
        takes_away,
    })
}

// A copy cannot have taken off a step the page never showed, nor unticked one it showed open.
fn seen(op: &crate::Op, shown: &[(crate::model::StepId, bool)]) -> bool {
    match op {
        crate::Op::StepRemove { d, .. } => shown.iter().any(|(one, _)| *one == d.step),
        crate::Op::StepUndone { d, .. } => shown.iter().any(|(one, done)| *one == d.step && *done),
        _ => true,
    }
}

// A page coming in is held to what the window lets a person set by hand.
fn allowed(
    state: &crate::State,
    task: &Task,
    mut d: crate::event::TaskPatch,
) -> Option<crate::event::TaskPatch> {
    if state.repeat_refused(task.id).is_some() {
        d.repeat = None;
    }
    let now = jiff::Zoned::now();
    if matches!(&d.deadline, Some(Some(due)) if due.passed(&now)) {
        d.deadline = None;
    }
    (d != crate::event::TaskPatch::default()).then_some(d)
}

/// Whether the page holds a line `body` leaves out, as it does once it moved on since a copy went out.
pub fn left_out(was: &str, body: &str) -> bool {
    let lines = |one: &str| -> Vec<String> {
        let rest = match tisty_markdown::front::span(one) {
            Some(at) => &one[at.end..],
            None => one,
        };
        rest.lines()
            .map(unlinked)
            .filter(|line| !line.is_empty())
            .collect()
    };
    let mut theirs = lines(body);
    lines(was)
        .into_iter()
        .any(|line| match theirs.iter().position(|one| *one == line) {
            Some(at) => {
                theirs.swap_remove(at);
                false
            }
            None => true,
        })
}

// A link's target changes on every trip out and back and a tick is weighed as a step, so neither counts.
fn unlinked(line: &str) -> String {
    let mut out = String::new();
    let line = line.trim().replace("[x]", "[ ]").replace("[X]", "[ ]");
    let mut rest = line.as_str();
    while let Some(at) = rest.find("](") {
        out.push_str(&rest[..=at]);
        rest = &rest[at + 2..];
        rest = rest.find(')').map_or("", |end| &rest[end + 1..]);
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
#[path = "task_front_test.rs"]
mod tests;
