use serde_json::{Value, json};
use tisty_core::Paths;

use tisty_core::State;
use tisty_core::event::{LogEdit, Op, StepRef, StepText, TaskPatch};
use tisty_core::model::{StepId, Task};

use super::asked::text;
use super::chores::{
    already_said_done, filling, steps_asked, steps_fit, steps_named, still_filling,
};
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
    let (id, task) = filling(&state, &store, &said)?;
    if !state.filed_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{:?} is the person's own: opening it to agents lets you mend its description, its \
             open steps and its journal, never its title. Say what you would call it with `note`.",
            task.title
        )));
    }
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
                    .is_none_or(|now| unsteady(&held, now) || now.title != had)
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
    let (id, task) = filling(&state, &store, &said)?;
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
                    unsteady(&held, now)
                        || now.step(step).is_none_or(|one| one.done || one.text != was)
                })
            },
        )
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    Ok(told(
        format!("Reworded {was:?} to {words:?} on {:?}.", task.title),
        json!({ "id": id.to_string(), "title": task.title, "reworded": 1 }),
    ))
}

pub(super) fn unplan(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (said, wanted) = steps_asked(args, "unplanning", "each step to take off")?;
    let (state, mut store) = opened(paths)?;
    let (id, task) = filling(&state, &store, &said)?;
    if let Some(why) = already_said_done(task, "taking a step off") {
        return Err(why);
    }
    let ids: Vec<StepId> = open_steps(task, &wanted, "taken off")?
        .into_iter()
        .map(|(step, _)| step)
        .collect();
    let ops = ids
        .iter()
        .map(|step| Op::StepRemove {
            id,
            d: StepRef { step: *step },
        })
        .collect();
    let written = store
        .append_batch_unless(ops, |events| {
            let held = State::replay(events);
            held.tasks.get(&id).is_none_or(|now| {
                unsteady(&held, now)
                    || !ids
                        .iter()
                        .all(|one| now.step(*one).is_some_and(|step| !step.done))
            })
        })
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    let left = task.steps.len() - ids.len();
    Ok(told(
        format!(
            "Took {} step(s) off {:?}. {left} left.",
            ids.len(),
            task.title
        ),
        json!({ "id": id.to_string(), "title": task.title, "removed": ids.len(), "left": left }),
    ))
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
    let (id, task) = filling(&state, &store, &said)?;
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
                    unsteady(&held, now)
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

fn unsteady(held: &State, now: &Task) -> bool {
    !still_filling(held, now) || now.resolved.is_some()
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
