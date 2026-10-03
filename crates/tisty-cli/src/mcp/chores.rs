use serde_json::{Value, json};
use tisty_core::Paths;

use tisty_core::capture::{Draft, Rejected};
use tisty_core::event::{Body, LogAdd, Op, Resolve, StepAdd, StepRef, TaskPatch};
use tisty_core::model::DateSpec;
use tisty_core::model::{Tag, Task, TaskId};
use tisty_core::{State, Store};
use ulid::Ulid;

use super::asked::{
    day, in_order, moments, only_what_it_takes, ranked, short_and_plain, strings, text,
};
use super::jsonrpc::{speaking_through, told};
use super::{
    INBOX_TAG, Refused, already, gone, history, hitch, how_it_ended, moved, opened, order, said,
    unticked, when,
};

/// Eight tasks in one call instead of eight calls: what the door costs an agent is mostly the
/// conversation it has to send again each time, not the writing.
pub(super) fn propose(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(many) = args.get("tasks") else {
        return proposed(paths, args);
    };
    let Some(many) = many.as_array() else {
        return Err(Refused::Tool(
            "`tasks` is a list of what you want to propose. One on its own needs no list at all."
                .into(),
        ));
    };
    if many.is_empty() {
        return Err(Refused::Tool(
            "`tasks` came empty, so nothing was written.".into(),
        ));
    }
    if many.len() > DRAFTS_AT_MOST {
        return Err(Refused::Tool(format!(
            "{} is more than the {DRAFTS_AT_MOST} tasks one call takes. Send them in rounds.",
            many.len()
        )));
    }
    if many.iter().any(|one| !one.is_object()) {
        return Err(Refused::Tool(
            "every entry in `tasks` is a task of its own, written the same way a single one is."
                .into(),
        ));
    }

    let mut done: Vec<Value> = Vec::with_capacity(many.len());
    let (mut written, mut already, mut turned) = (0, 0, 0);
    for one in many {
        match one_of_many(paths, one) {
            Ok(said) => {
                let kept = said
                    .get("structuredContent")
                    .cloned()
                    .unwrap_or(Value::Null);
                match kept.get("proposed") == Some(&json!(true)) {
                    true => written += 1,
                    false => already += 1,
                }
                done.push(kept);
            }
            Err(Refused::Tool(said)) => {
                turned += 1;
                done.push(json!({
                    "title": text(one, "title"),
                    "proposed": false,
                    "refused": said,
                }));
            }
            Err(other) => return Err(other),
        }
    }

    let stepped = done
        .iter()
        .filter(|one| {
            one.get("steps")
                .and_then(Value::as_u64)
                .is_some_and(|n| n > 0)
        })
        .count();
    Ok(told(
        format!(
            "{written} written, {already} already there, {turned} turned away.\n{}{}",
            done.iter()
                .map(|one| match one.get("refused").and_then(Value::as_str) {
                    Some(why) => format!("{} — {why}", said(one, "title")),
                    None => format!("{} — {}", said(one, "id"), said(one, "title")),
                })
                .collect::<Vec<_>>()
                .join("\n"),
            match stepped {
                0 => String::new(),
                n => format!(
                    "\n{n} of them carry steps: `tick` each as you do it, `say_done` when all are."
                ),
            }
        ),
        json!({ "tasks": done, "written": written, "refused": turned }),
    ))
}

pub(super) fn proposed(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(title) = text(args, "title") else {
        return Err(Refused::Tool("a task needs a `title`.".into()));
    };
    let (state, mut store) = opened(paths)?;

    let again = args.get("again").and_then(Value::as_bool).unwrap_or(false);
    if let Some(source) = text(args, "source")
        && let Some(held) = already(&state, &source)
        && state.is_erased(held)
        && !again
    {
        return Ok(told(
            "Already proposed from that source, and erased since: the person let it go. Nothing \
             was written. Only if they want it back, propose it once more with `again` set."
                .into(),
            json!({ "id": held.to_string(), "proposed": false, "erased": true }),
        ));
    }
    if let Some(source) = text(args, "source")
        && let Some(held) = already(&state, &source)
        && let Some(task) = state.tasks.get(&held)
        && !(again && !task.is_open())
    {
        let (said, kept) = match task.completed_at.filter(|_| !task.is_open()) {
            Some(at) => (
                format!(
                    "Already proposed from that source, and closed on {}: {:?}. {}. If the \
                     person wants it done again, propose it once more with `again` set, saying \
                     in the description how this one ended. Nothing was written.",
                    when(at),
                    task.title,
                    how_it_ended(task)
                ),
                json!({ "id": task.id.to_string(), "title": task.title, "proposed": false, "closed": when(at) }),
            ),
            None => (
                format!(
                    "Already proposed from that source: {:?}. Nothing was written.",
                    task.title
                ),
                json!({ "id": task.id.to_string(), "title": task.title, "proposed": false }),
            ),
        };
        return Ok(told(said, kept));
    }

    let on = day(args, "date")?;
    let owed = day(args, "deadline")?;
    in_order(on.as_ref(), owed.as_ref())?;

    let mut tags: Vec<Tag> = Vec::new();
    for one in strings(args, "tags")?
        .iter()
        .filter_map(|said| Tag::written(said).ok())
    {
        if !tags.contains(&one) {
            tags.push(one);
        }
    }
    if let Ok(mine) = Tag::new(INBOX_TAG)
        && !tags.contains(&mine)
    {
        tags.push(mine);
    }

    let draft = Draft {
        title: title.clone(),
        date: on,
        deadline: owed,
        priority: ranked(args)?,
        filing: text(args, "list").map(tisty_core::capture::Filing::Named),
        tags,
        repeat: None,
        source: text(args, "source"),
    };
    let plan =
        tisty_core::capture::plan(&state, draft).map_err(|e| with_the_names(refused(e), &state))?;
    let id = plan.task;
    let mut ops = plan.ops;
    if let Some(body) = text(args, "description") {
        ops.push(Op::TaskDescribe {
            id,
            d: Body { body: Some(body) },
        });
    }
    let bells = moments(args, "remind")?;
    if !bells.is_empty() {
        ops.push(Op::TaskUpdate {
            id,
            d: TaskPatch {
                reminders: Some(bells),
                ..Default::default()
            },
        });
    }
    let mut step = order::first();
    let mut planned = 0;
    for one in strings(args, "steps")? {
        planned += 1;
        ops.push(Op::StepAdd {
            id,
            d: StepAdd {
                step: Ulid::generate(),
                text: one,
                order: step.clone(),
            },
        });
        step = order::after(&step);
    }

    let source = text(args, "source");
    let taken = source.clone();
    // Checked again under the lock: two agents reading the same thread at once must not
    // both get through. A closed task from that source stands aside only when `again` says so.
    let written = store
        .append_batch_unless(ops, move |events| match &taken {
            None => false,
            Some(one) => {
                let held = State::replay(events);
                already(&held, one).is_some_and(|id| {
                    held.is_erased(id) && !again
                        || held
                            .tasks
                            .get(&id)
                            .is_some_and(|task| task.is_open() || !again)
                })
            }
        })
        .map_err(hitch)?;

    let Some(_) = written else {
        let held = State::replay(&store.read_all().map_err(hitch)?);
        let task = source
            .as_deref()
            .and_then(|one| already(&held, one))
            .and_then(|id| held.tasks.get(&id));
        return Ok(told(
            match task {
                Some(task) => format!(
                    "Already proposed from that source, as {}: {:?}. Nothing was written.",
                    task.id, task.title
                ),
                None => "Already proposed from that source. Nothing was written.".into(),
            },
            json!({
                "id": task.map(|one| one.id.to_string()),
                "title": task.map(|one| one.title.clone()),
                "proposed": false,
            }),
        ));
    };
    let landed = text(args, "list");
    let where_at = match &landed {
        Some(name) => format!("in {name}"),
        None => "in the inbox".to_string(),
    };
    let mut kept = serde_json::Map::new();
    kept.insert("id".into(), json!(id.to_string()));
    kept.insert("title".into(), json!(title));
    if let Some(landed) = &landed {
        kept.insert("list".into(), json!(landed));
    }
    kept.insert("proposed".into(), json!(true));
    if planned > 0 {
        kept.insert("steps".into(), json!(planned));
    }
    Ok(told(
        match planned {
            0 => format!("Proposed {title:?} as {id} {where_at}, tagged #{INBOX_TAG}."),
            n => format!(
                "Proposed {title:?} as {id} {where_at}, tagged #{INBOX_TAG}, with {n} step(s): \
                 `tick` each as you do it, `say_done` when all are."
            ),
        },
        Value::Object(kept),
    ))
}

pub(super) fn remind(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("a reminder needs a `task` id.".into()));
    };
    let bells = moments(args, "at")?;
    if bells.is_empty() {
        return Err(Refused::Tool(
            "a reminder needs `at`, a day and an hour like 2026-08-31T09:00.".into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id).filter(|one| !one.folded()) else {
        return Err(gone(&state, &said));
    };
    if !task.is_open() {
        return Err(history(task));
    }
    let mut all = task.reminders.clone();
    let mut added = 0;
    for one in bells {
        if !all.iter().any(|kept| kept.at == one.at) {
            all.push(one);
            added += 1;
        }
    }
    if added == 0 {
        return Ok(told(
            format!("{:?} was already set to ring then.", task.title),
            json!({ "id": id.to_string(), "title": task.title, "added": false }),
        ));
    }
    all.sort_by_key(|one| one.at);
    store
        .append(Op::TaskUpdate {
            id,
            d: TaskPatch {
                reminders: Some(all.clone()),
                ..Default::default()
            },
        })
        .map_err(hitch)?;
    Ok(told(
        format!("{:?} will ring {added} time(s) more.", task.title),
        json!({
            "id": id.to_string(),
            "title": task.title,
            "added": true,
            "reminders": all.iter().map(|one| one.at.to_string()).collect::<Vec<_>>(),
        }),
    ))
}

pub(super) fn reschedule(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("moving a day needs a `task` id.".into()));
    };
    let on = day(args, "date")?;
    let owed = day(args, "deadline")?;
    let clears = |key: &str| args.get(key).is_some_and(Value::is_null);
    if on.is_none() && owed.is_none() && !clears("date") && !clears("deadline") {
        return Err(Refused::Tool(
            "moving a day needs a `date` or a `deadline`. Send null to take one off.".into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id) else {
        return Err(gone(&state, &said));
    };
    if !state.filed_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{:?} is the person's own, so its day is theirs to move — opened to agents or not. \
             You can only move what an agent filed. Say what you have learnt with `note` and \
             leave the day alone.",
            task.title
        )));
    }
    if !task.is_open() {
        return Err(history(task));
    }
    if let Some(why) = already_said_done(task, "moving its day") {
        return Err(why);
    }
    let standing =
        |given: &Option<DateSpec>, key: &str, held: &Option<DateSpec>| match (given, clears(key)) {
            (Some(one), _) => Some(one.clone()),
            (None, true) => None,
            (None, false) => held.clone(),
        };
    in_order(on.as_ref(), owed.as_ref())?;

    let was = (
        task.date.as_ref().map(|one| one.date().to_string()),
        task.deadline.as_ref().map(|one| one.date().to_string()),
    );
    let patch = TaskPatch {
        date: match (on.clone(), clears("date")) {
            (Some(one), _) => Some(Some(one)),
            (None, true) => Some(None),
            (None, false) => None,
        },
        deadline: match (owed.clone(), clears("deadline")) {
            (Some(one), _) => Some(Some(one)),
            (None, true) => Some(None),
            (None, false) => None,
        },
        ..Default::default()
    };
    store
        .append(Op::TaskUpdate { id, d: patch })
        .map_err(hitch)?;

    let now = (
        on.as_ref().map(|one| one.date().to_string()),
        owed.as_ref().map(|one| one.date().to_string()),
    );
    let moved = |what: &str, was: &Option<String>, now: &Option<String>, cleared: bool| match (
        now, cleared,
    ) {
        (Some(one), _) => Some(match was {
            Some(was) => format!("{what} {was} \u{2192} {one}"),
            None => format!("{what} {one}"),
        }),
        (None, true) => Some(format!("{what} taken off")),
        (None, false) => None,
    };
    let said = [
        moved("on", &was.0, &now.0, clears("date")),
        moved("owed", &was.1, &now.1, clears("deadline")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ");

    Ok(told(
        format!("Moved {:?}: {said}.", task.title),
        json!({
            "id": id.to_string(),
            "title": task.title,
            "date": standing(&on, "date", &task.date).map(|one| one.date().to_string()),
            "deadline": standing(&owed, "deadline", &task.deadline)
                .map(|one| one.date().to_string()),
        }),
    ))
}

pub(super) fn describe(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("describing needs a `task` id.".into()));
    };
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool(
            "describing needs a `body`, in markdown.".into(),
        ));
    };
    let (state, mut store) = opened(paths)?;
    let (id, task) = filling(&state, &store, &said)?;
    if let Some(why) = already_said_done(task, "writing a description") {
        return Err(why);
    }
    if task
        .description
        .as_deref()
        .is_some_and(|had| !had.trim().is_empty())
    {
        return Err(Refused::Tool(format!(
            "{:?} is already described, and a description is not yours to write over. Add what \
             you have learnt with `note`.",
            task.title
        )));
    }
    let written = store
        .append_batch_unless(
            vec![Op::TaskDescribe {
                id,
                d: Body { body: Some(body) },
            }],
            |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    !still_filling(&held, now)
                        || now
                            .description
                            .as_deref()
                            .is_some_and(|had| !had.trim().is_empty())
                })
            },
        )
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    Ok(told(
        format!("Described {:?}.", task.title),
        json!({ "id": id.to_string(), "title": task.title }),
    ))
}

pub(super) fn plan(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("planning needs a `task` id.".into()));
    };
    let steps = strings(args, "steps")?;
    if steps.is_empty() {
        return Err(Refused::Tool(
            "planning needs `steps`, the checklist to add, one string each.".into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let (id, task) = filling(&state, &store, &said)?;
    if let Some(said) = &task.resolved {
        return Err(Refused::Tool(format!(
            "{:?} has already been said {}, and a step added now would stand under that mark \
             unlooked at. Say what is still left with `note` and leave the task to the person.",
            task.title,
            marked_as(said)
        )));
    }
    let mut ops = Vec::with_capacity(steps.len());
    let mut order = state.step_order_between(id, task.steps.last().map(|s| s.id), None);
    for one in &steps {
        ops.push(Op::StepAdd {
            id,
            d: StepAdd {
                step: Ulid::generate(),
                text: one.clone(),
                order: order.clone(),
            },
        });
        order = order::after(&order);
    }
    let written = store
        .append_batch_unless(ops, |events| {
            let held = State::replay(events);
            held.tasks
                .get(&id)
                .is_none_or(|now| !still_filling(&held, now) || now.resolved.is_some())
        })
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    Ok(told(
        format!("Planned {} step(s) on {:?}.", steps.len(), task.title),
        json!({ "id": id.to_string(), "title": task.title, "added": steps.len() }),
    ))
}

pub(super) fn tick(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("ticking needs a `task` id.".into()));
    };
    let mut wanted = strings(args, "steps")?;
    if let Some(one) = text(args, "step") {
        wanted.push(one);
    }
    if wanted.is_empty() {
        return Err(Refused::Tool(
            "ticking needs `steps`: the text of each step you did, as `read` shows it \
             (`step` for one)."
                .into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let (id, task) = filling(&state, &store, &said)?;
    let mut chosen: Vec<&tisty_core::model::Step> = Vec::new();
    for one in &wanted {
        let sought = tisty_core::text::folded(one.trim());
        let alike: Vec<&tisty_core::model::Step> = task
            .steps
            .iter()
            .filter(|step| tisty_core::text::folded(step.text.trim()) == sought)
            .collect();
        // Two steps written alike are a list with a line repeated: naming it ticks the next
        // one still open, so the checklist is walked down rather than stuck on the name.
        let Some(step) = alike
            .iter()
            .find(|step| !step.done && !chosen.iter().any(|had| had.id == step.id))
            .or_else(|| alike.first())
            .copied()
        else {
            return Err(Refused::Tool(format!(
                "no step of {:?} reads {one:?}; nothing was ticked. `read` shows them as they \
                 are written.",
                task.title
            )));
        };
        if !chosen.iter().any(|had| had.id == step.id) {
            chosen.push(step);
        }
    }
    let (already, fresh): (Vec<&tisty_core::model::Step>, Vec<&tisty_core::model::Step>) =
        chosen.iter().partition(|step| step.done);
    if !fresh.is_empty() {
        let ids: Vec<tisty_core::model::StepId> = fresh.iter().map(|step| step.id).collect();
        let ops = ids
            .iter()
            .map(|step| Op::StepDone {
                id,
                d: StepRef { step: *step },
            })
            .collect();
        // Judged again under the lock: a step the person ticked meanwhile is not ticked twice.
        let written = store
            .append_batch_unless(ops, |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    !still_filling(&held, now)
                        || now
                            .steps
                            .iter()
                            .any(|step| step.done && ids.contains(&step.id))
                })
            })
            .map_err(hitch)?;
        if written.is_none() {
            return Err(moved(task));
        }
    }
    let left = task.steps.iter().filter(|step| !step.done).count() - fresh.len();
    let mut said = match fresh.len() {
        0 => format!("Nothing new ticked on {:?}: ", task.title),
        n => format!("Ticked {n} step(s) on {:?}. ", task.title),
    };
    if !already.is_empty() {
        said.push_str(&format!("{} ticked already. ", already.len()));
    }
    said.push_str(&match left {
        0 => "None left: `say_done` when the work is done.".to_string(),
        n => format!("{n} still unticked."),
    });
    Ok(told(
        said,
        json!({
            "id": id.to_string(),
            "title": task.title,
            "ticked": fresh.len(),
            "already": already.len(),
            "left": left,
        }),
    ))
}

pub(super) fn untick(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("unticking needs a `task` id.".into()));
    };
    let mut wanted = strings(args, "steps")?;
    if let Some(one) = text(args, "step") {
        wanted.push(one);
    }
    if wanted.is_empty() {
        return Err(Refused::Tool(
            "unticking needs `steps`: the text of each step an agent ticked by mistake, as \
             `read` shows it (`step` for one)."
                .into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let (id, task) = filling(&state, &store, &said)?;
    let mut chosen: Vec<&tisty_core::model::Step> = Vec::new();
    for one in &wanted {
        let sought = tisty_core::text::folded(one.trim());
        let alike: Vec<&tisty_core::model::Step> = task
            .steps
            .iter()
            .filter(|step| tisty_core::text::folded(step.text.trim()) == sought)
            .collect();
        let Some(step) = alike
            .iter()
            .find(|step| step.by_agent && !chosen.iter().any(|had| had.id == step.id))
            .or_else(|| alike.first())
            .copied()
        else {
            return Err(Refused::Tool(format!(
                "no step of {:?} reads {one:?}; nothing was unticked. `read` shows them as they \
                 are written.",
                task.title
            )));
        };
        if step.done && !step.by_agent {
            return Err(Refused::Tool(format!(
                "{:?} was ticked by the person, and taking it back is theirs; nothing was \
                 unticked. Say what you found with `note`.",
                step.text
            )));
        }
        if !chosen.iter().any(|had| had.id == step.id) {
            chosen.push(step);
        }
    }
    let (fresh, already): (Vec<&tisty_core::model::Step>, Vec<&tisty_core::model::Step>) =
        chosen.iter().partition(|step| step.done);
    if !fresh.is_empty() {
        let ids: Vec<tisty_core::model::StepId> = fresh.iter().map(|step| step.id).collect();
        let ops = ids
            .iter()
            .map(|step| Op::StepUndone {
                id,
                d: StepRef { step: *step },
            })
            .collect();
        let written = store
            .append_batch_unless(ops, |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    !still_filling(&held, now)
                        || now
                            .steps
                            .iter()
                            .any(|step| ids.contains(&step.id) && !(step.done && step.by_agent))
                })
            })
            .map_err(hitch)?;
        if written.is_none() {
            return Err(moved(task));
        }
    }
    let left = task.steps.iter().filter(|step| !step.done).count() + fresh.len();
    let mut said = match fresh.len() {
        0 => format!("Nothing unticked on {:?}: ", task.title),
        n => format!("Unticked {n} step(s) on {:?}. ", task.title),
    };
    if !already.is_empty() {
        said.push_str(&format!("{} were not ticked. ", already.len()));
    }
    said.push_str(&format!("{left} unticked now."));
    Ok(told(
        said,
        json!({
            "id": id.to_string(),
            "title": task.title,
            "unticked": fresh.len(),
            "already": already.len(),
            "left": left,
        }),
    ))
}

pub(super) fn note(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool("a note needs a `body`.".into()));
    };
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("a note needs a `task` id.".into()));
    };
    let (state, mut store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id) else {
        return Err(gone(&state, &said));
    };
    if !task.is_open() {
        return Err(history(task));
    }

    let zone = jiff::tz::TimeZone::system();
    store
        .append(Op::TaskLog {
            id,
            d: LogAdd::new(Ulid::generate(), body).in_zone(zone.iana_name().map(str::to_string)),
        })
        .map_err(hitch)?;
    Ok(told(
        format!("Noted on {:?}.", task.title),
        json!({ "id": id.to_string(), "title": task.title }),
    ))
}

pub(super) fn say_done(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    spoken_for(paths, args, false)
}

pub(super) fn say_not_doing(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    spoken_for(paths, args, true)
}

fn spoken_for(paths: &Paths, args: &Value, drop: bool) -> Result<Value, Refused> {
    let saying = if drop {
        "saying a task will not be done"
    } else {
        "saying a task is done"
    };
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool(format!("{saying} needs the `task` id.")));
    };
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool(format!(
            "{saying} needs a `body`: {} Without it the person has only your word and nothing \
             to check it against.",
            if drop {
                "why it should not be done, and what you found that says so."
            } else {
                "what you did and how you know it holds."
            }
        )));
    };
    let (state, mut store) = opened(paths)?;
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id) else {
        return Err(gone(&state, &said));
    };
    let me = store.device().clone();
    if !state.attended_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{:?} is the person's own, and they have not opened it to you: whether it is done \
             is theirs to say. You can only speak for what an agent filed, or for what they \
             opened to agents. Say what you have learnt with `note` instead.",
            task.title
        )));
    }
    if !task.is_open() {
        return Err(history(task));
    }
    if task.folded() {
        return Err(Refused::Tool(format!(
            "{:?} was put out of sight by the person, so it is not yours to speak for. Say what \
             you have learnt with `note` instead.",
            task.title
        )));
    }
    if let Some(already) = &task.resolved {
        let who = match already.by == me && already.via == speaking_through() {
            true => "you".to_string(),
            false => already
                .via
                .as_deref()
                .map(tisty_core::agent::client_named)
                .unwrap_or_else(|| "an assistant".to_string()),
        };
        return Err(Refused::Tool(format!(
            "{who} already said {:?} {} on {}, and the person has not looked yet. Saying \
             it again would only stack another entry on the journal — add what is new with \
             `note`.",
            task.title,
            if already.drop {
                "would not be done"
            } else {
                "was done"
            },
            when(already.at)
        )));
    }
    if !drop && let Some(refusal) = unticked(task) {
        return Err(refusal);
    }

    let zone = jiff::tz::TimeZone::system();
    let entry = Ulid::generate();
    // Judged again under the lock: the mark must not land beside a step the person just
    // unticked, nor on top of one another agent just left.
    let written = store
        .append_batch_unless(
            vec![
                Op::TaskLog {
                    id,
                    d: LogAdd::new(entry, body).in_zone(zone.iana_name().map(str::to_string)),
                },
                Op::TaskResolve {
                    id,
                    d: Resolve::new(entry)
                        .said_by(jiff::Timestamp::now(), me.clone())
                        .dropping(drop),
                },
            ],
            |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    !still_filling(&held, now)
                        || now.resolved.is_some()
                        || (!drop && now.steps.iter().any(|step| !step.done))
                })
            },
        )
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }

    Ok(told(
        format!(
            "{} {:?}. It stays open until the person decides.",
            if drop {
                "Said it will not be done:"
            } else {
                "Said done:"
            },
            task.title
        ),
        json!({ "id": id.to_string(), "title": task.title, "open": true }),
    ))
}

const DRAFTS_AT_MOST: usize = 32;

/// A misspelt argument would otherwise be dropped in silence, teaching the model nothing.
pub(super) fn one_of_many(paths: &Paths, one: &Value) -> Result<Value, Refused> {
    only_what_it_takes("propose", one)?;
    short_and_plain(one)?;
    if one.get("tasks").is_some() {
        return Err(Refused::Tool(
            "a task inside `tasks` cannot carry `tasks` of its own.".into(),
        ));
    }
    proposed(paths, one)
}

/// What an assistant may fill in: a task it filed, or one the person opened to agents — open,
/// and not folded away. The refusal says which of the three it is not.
fn filling<'a>(state: &'a State, store: &Store, said: &str) -> Result<(TaskId, &'a Task), Refused> {
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(task) = state.tasks.get(&id).filter(|one| !one.folded()) else {
        return Err(gone(state, said));
    };
    if !task.is_open() {
        return Err(history(task));
    }
    if !state.attended_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{:?} is the person's own, and they have not opened it to you. You fill in what an \
             agent filed, or what they opened to agents; on the rest, say what you have learnt \
             with `note`.",
            task.title
        )));
    }
    let _ = store;
    Ok((id, task))
}

/// Still open, in sight, and open to this hand: what every fill-in checks again under the lock.
fn still_filling(held: &State, now: &Task) -> bool {
    now.is_open() && !now.folded() && held.attended_by_agents(now)
}

fn already_said_done(task: &Task, doing: &str) -> Option<Refused> {
    task.resolved.as_ref().map(|said| {
        Refused::Tool(format!(
            "{:?} has already been said {}, so {doing} now would speak over a mark nobody has \
             looked at yet. Say what you have learnt with `note` and leave the task to the \
             person.",
            task.title,
            marked_as(said)
        ))
    })
}

/// A refusal that sends the agent to another call for a handful of names costs it a whole turn.
fn with_the_names(said: Refused, state: &State) -> Refused {
    let Refused::Tool(said) = said else {
        return said;
    };
    if !said.contains("list") {
        return Refused::Tool(said);
    }
    let mut all: Vec<&str> = state
        .lists
        .values()
        .filter(|one| !one.archived)
        .map(|one| one.name.as_str())
        .collect();
    all.sort_unstable();
    Refused::Tool(match all.is_empty() {
        true => format!("{said} There are no lists at all yet."),
        false => format!("{said} These exist: {}.", all.join(", ")),
    })
}

fn refused(e: Rejected) -> Refused {
    match e {
        Rejected::Untitled => Refused::Tool("a task needs a title.".into()),
        Rejected::NoSuchList(said) => Refused::Tool(format!(
            "there is no list called {said:?}, and you cannot make one. Leave `list` out and it \
             lands in the inbox."
        )),
        Rejected::AmbiguousList(said) => Refused::Tool(format!(
            "{said:?} matches more than one list. Send the whole name."
        )),

        Rejected::ArchivedList(said) => Refused::Tool(format!(
            "the list {said:?} is put away, so nothing new goes in it. Leave `list` out and it \
             lands in the inbox."
        )),
        other => Refused::Protocol(-32603, format!("{other:?}")),
    }
}

fn marked_as(said: &tisty_core::model::Resolved) -> &'static str {
    if said.drop { "not to be done" } else { "done" }
}
