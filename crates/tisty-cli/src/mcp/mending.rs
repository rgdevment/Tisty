use serde_json::{Value, json};
use tisty_core::Paths;

use tisty_core::State;
use tisty_core::event::{LogEdit, Op, StepAdd, StepRef, StepText, TaskPatch};
use tisty_core::model::{StepId, Task, TaskId};

use super::asked::text;
use super::chores::{
    already_said_done, reaching, steps_asked, steps_fit, steps_named, still_in_sight,
};
use super::companion::{Companion, companion};
use super::jsonrpc::told;
use super::{Refused, hitch, moved, opened};

pub(super) fn rename(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("renaming needs a `task` id.".into()));
    };
    let Some(title) = text(args, "title") else {
        return Err(Refused::Tool("renaming needs the new `title`.".into()));
    };
    let (state, mut store) = opened(paths)?;
    let (id, task) = reaching(&state, &said)?;
    if let Some(why) = already_said_done(task, "renaming it") {
        return Err(why);
    }
    if task.title == title {
        return Ok(told(
            format!("{title:?} is already its title; nothing changed."),
            json!({ "id": id.to_string(), "title": title }),
        ));
    }
    let had = task.title.clone();
    let written = store
        .append_batch_unless(
            vec![Op::TaskUpdate {
                id,
                d: TaskPatch {
                    title: Some(title.clone()),
                    ..Default::default()
                },
            }],
            |events| {
                let held = State::replay(events);
                held.tasks
                    .get(&id)
                    .is_none_or(|now| unsteady(now) || now.title != had)
            },
        )
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    Ok(told(
        format!("Renamed {had:?} to {title:?}."),
        json!({ "id": id.to_string(), "title": title }),
    ))
}

pub(super) fn reword_step(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("rewording needs a `task` id.".into()));
    };
    let (Some(step), Some(words)) = (text(args, "step"), text(args, "text")) else {
        return Err(Refused::Tool(
            "rewording needs the `step` as `read` shows it and the new `text`.".into(),
        ));
    };
    steps_fit(std::slice::from_ref(&words))?;
    let (state, mut store) = opened(paths)?;
    let (id, task) = reaching(&state, &said)?;
    if let Some(why) = already_said_done(task, "rewording a step") {
        return Err(why);
    }
    let Some((step, was)) = open_steps(task, &[step], "reworded")?.pop() else {
        return Err(moved(task));
    };
    if was == words {
        return Ok(told(
            format!("{words:?} already reads that way; nothing changed."),
            json!({ "id": id.to_string(), "title": task.title, "reworded": 0 }),
        ));
    }
    let written = store
        .append_batch_unless(
            vec![Op::StepText {
                id,
                d: StepText {
                    step,
                    text: words.clone(),
                },
            }],
            |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    unsteady(now) || now.step(step).is_none_or(|one| one.done || one.text != was)
                })
            },
        )
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    let page = super::worded::worded(paths, &mut store, id, |body, _| {
        tisty_core::stages::reworded(body, task, step, &words)
    });
    let unsaid = match task.step(step).is_some_and(|one| one.said) {
        true => {
            " Your word that it was done went with the old words: `tick` it again if the new \
                  ones are done too."
        }
        false => "",
    };
    Ok(told(
        format!(
            "Reworded {was:?} to {words:?} on {:?}.{unsaid}{page}",
            task.title
        ),
        json!({ "id": id.to_string(), "title": task.title, "reworded": 1 }),
    ))
}

pub(super) fn unplan(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (said, wanted) = steps_asked(args, "unplanning", "each step to take off")?;
    let to_person = args.get("to_person").and_then(Value::as_bool) == Some(true);
    if to_person && text(args, "to").is_some() {
        return Err(Refused::Tool(
            "`to` names a task and `to_person` hands the steps to the person's: send one of the \
             two. Nothing was moved."
                .into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let (id, task) = reaching(&state, &said)?;
    if let Some(why) = already_said_done(task, "taking a step off") {
        return Err(why);
    }
    let chosen = open_steps(task, &wanted, "taken off")?;
    let ids: Vec<StepId> = chosen.iter().map(|(step, _)| *step).collect();
    let words: Vec<String> = chosen.iter().map(|(_, words)| words.clone()).collect();
    let handing = match (text(args, "to"), to_person) {
        (Some(other), _) => {
            let (there, other) = handed_to(&state, id, &other)?;
            Some(Companion::Held(there, other))
        }
        (None, true) => {
            let lang = tisty_core::Config::load(&paths.config_file())
                .ok()
                .flatten()
                .map_or_else(crate::i18n::Lang::default, |config| {
                    crate::i18n::Lang::detect(config.locale.as_deref())
                });
            Some(companion(&state, task, text(args, "title"), &words, lang)?)
        }
        (None, false) => None,
    };
    let mut ops: Vec<Op> = ids
        .iter()
        .map(|step| Op::StepRemove {
            id,
            d: StepRef { step: *step },
        })
        .collect();
    let (there, fresh_under) = match handing {
        Some(Companion::Held(there, other)) => {
            let mut order = state.step_order_between(there, other.steps.last().map(|s| s.id), None);
            for words in &words {
                ops.push(Op::StepAdd {
                    id: there,
                    d: StepAdd {
                        step: ulid::Ulid::generate(),
                        text: words.clone(),
                        order: order.clone(),
                    },
                });
                order = tisty_core::order::after(&order);
            }
            (Some((there, other.title.clone())), None)
        }
        Some(Companion::Fresh(drafted, anchor)) => {
            let there = (drafted.id, drafted.title.clone());
            ops.extend(drafted.ops);
            (Some(there), Some(anchor))
        }
        None => (None, None),
    };
    let target = there.as_ref().map(|(one, _)| *one);
    let written = store
        .append_batch_unless(ops, |events| {
            let held = State::replay(events);
            let gone = |one: TaskId| held.tasks.get(&one).is_none_or(unsteady);
            gone(id)
                || (fresh_under.is_none() && target.is_some_and(gone))
                || fresh_under.is_some_and(|anchor| {
                    gone(anchor) || super::companion::held(&held, anchor).is_some()
                })
                || held.tasks.get(&id).is_some_and(|now| {
                    !ids.iter()
                        .all(|one| now.step(*one).is_some_and(|step| !step.done))
                })
        })
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    let mut page = super::worded::worded(paths, &mut store, id, |body, _| {
        Some(tisty_core::stages::unplanned(body, task, &ids))
    });
    if let Some(there) = target.filter(|_| fresh_under.is_none()) {
        let also = super::worded::worded(paths, &mut store, there, |was, now| {
            Some(tisty_core::stages::ticked(now, was))
        });
        if page.is_empty() {
            page = also;
        }
    }
    let left = task.steps.len() - ids.len();
    let said = match (&there, to_person, fresh_under) {
        (Some((_, other)), true, Some(_)) => format!(
            "Handed {} step(s) from {:?} to the person, in a new task of theirs, {other:?}, \
             filed as its part: it never holds yours open. {left} left on {:?}; `say_done` it \
             when they are done.{page}",
            ids.len(),
            task.title,
            task.title
        ),
        (Some((_, other)), true, None) => format!(
            "Handed {} step(s) from {:?} to the person, at the end of {other:?}, their task. \
             {left} left on {:?}; `say_done` it when they are done.{page}",
            ids.len(),
            task.title,
            task.title
        ),
        (Some((_, other)), false, _) => format!(
            "Moved {} step(s) from {:?} to the end of {other:?}'s steps. {left} left on {:?}.{page}",
            ids.len(),
            task.title,
            task.title
        ),
        (None, _, _) => format!(
            "Took {} step(s) off {:?}. {left} left.{page}",
            ids.len(),
            task.title
        ),
    };
    let mut kept = json!({
        "id": id.to_string(),
        "title": task.title,
        "removed": ids.len(),
        "left": left,
        "to": target.map(|one| one.to_string()),
    });
    if to_person {
        kept["to_person"] = json!(true);
        kept["companion_made"] = json!(fresh_under.is_some());
    }
    Ok(told(said, kept))
}

fn handed_to<'a>(
    state: &'a State,
    from: TaskId,
    said: &str,
) -> Result<(TaskId, &'a Task), Refused> {
    let (there, other) = reaching(state, said)?;
    if there == from {
        return Err(Refused::Tool(
            "`to` names the same task the steps are on; nothing was moved.".into(),
        ));
    }
    if let Some(why) = already_said_done(other, "adding steps to it") {
        return Err(why);
    }
    Ok((there, other))
}

pub(super) fn reword_note(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("rewording a note needs a `task` id.".into()));
    };
    let (Some(entry), Some(body)) = (text(args, "note"), text(args, "body")) else {
        return Err(Refused::Tool(
            "rewording a note needs the `note` id, as `read` shows it in the journal, and the \
             new `body`. Taking a note out is the person's."
                .into(),
        ));
    };
    let (state, mut store) = opened(paths)?;
    let (id, task) = reaching(&state, &said)?;
    if let Some(why) = already_said_done(task, "rewording a note") {
        return Err(why);
    }
    let Some(note) = task.journal().find(|one| one.id.to_string() == entry) else {
        return Err(Refused::Tool(format!(
            "no note of {:?} has the id {entry}. `read` it with `journal` to see each note's id.",
            task.title
        )));
    };
    let (entry, was) = (note.id, note.body.clone());
    if super::chores::note_length(&body) > super::chores::note_length(&was)
        && let Some(why) = super::chores::too_long_a_note(&state, task, &body, true)
    {
        return Err(why);
    }
    if was == body {
        return Ok(told(
            "The note already reads that way; nothing changed.".into(),
            json!({ "id": id.to_string(), "title": task.title, "note": entry.to_string() }),
        ));
    }
    let written = store
        .append_batch_unless(
            vec![Op::TaskLogEdit {
                id,
                d: LogEdit { entry, body },
            }],
            |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    unsteady(now)
                        || now
                            .journal()
                            .find(|one| one.id == entry)
                            .is_none_or(|one| one.body != was)
                })
            },
        )
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    Ok(told(
        format!("Reworded a note on {:?}.", task.title),
        json!({ "id": id.to_string(), "title": task.title, "note": entry.to_string() }),
    ))
}

fn unsteady(now: &Task) -> bool {
    !still_in_sight(now) || now.resolved.is_some()
}

fn open_steps(
    task: &Task,
    wanted: &[String],
    doing: &str,
) -> Result<Vec<(StepId, String)>, Refused> {
    let chosen = steps_named(task, wanted, doing, |step| !step.done)?;
    if let Some(step) = chosen.iter().find(|step| step.done) {
        return Err(Refused::Tool(format!(
            "{:?} is ticked, and a closed step is not yours to change; nothing was {doing}. If \
             you ticked it by mistake, `untick` it first.",
            step.text
        )));
    }
    Ok(chosen
        .into_iter()
        .map(|step| (step.id, step.text.clone()))
        .collect())
}
