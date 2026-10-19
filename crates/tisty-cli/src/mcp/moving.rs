use serde_json::{Value, json};
use tisty_core::event::{Op, TaskMove};
use tisty_core::model::{ListId, Task, TaskId};
use tisty_core::{Paths, State};

use super::asked::text;
use super::chores::{filling, still_filling};
use super::jsonrpc::told;
use super::{Refused, hitch, moved, opened};

enum Whole {
    Untouched,
    Under(TaskId),
    LetGo,
}

pub(super) fn move_task(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "task") else {
        return Err(Refused::Tool("moving needs a `task` id.".into()));
    };
    let whole = match args.get("part_of") {
        None => Whole::Untouched,
        Some(Value::Null) => Whole::LetGo,
        Some(_) => match text(args, "part_of").map(|one| one.parse::<TaskId>()) {
            Some(Ok(id)) => Whole::Under(id),
            _ => {
                return Err(Refused::Tool(
                    "`part_of` is the id of the task to hang this one under, or null to let it \
                     go. Use the `id` that `find` or `propose` gave you."
                        .into(),
                ));
            }
        },
    };
    let listed = text(args, "list");
    if listed.is_none() && matches!(whole, Whole::Untouched) {
        return Err(Refused::Tool(
            "moving needs a `list` to put the task in, or `part_of`: a task to hang it under, \
             or null to let it go."
                .into(),
        ));
    }
    let (state, mut store) = opened(paths)?;
    let (id, task) = filling(&state, &store, &said)?;
    let list = match &listed {
        Some(name) => Some(list_named(&state, task, name)?),
        None => None,
    };
    let part_of = match whole {
        Whole::Untouched => None,
        Whole::Under(up) => Some(Some(hung_under(&state, task, up)?)),
        Whole::LetGo => Some(None),
    };
    let held_by = task.part_of.and_then(|up| state.tasks.get(&up));
    let mut d = TaskMove {
        list: list.map(Some),
        part_of,
        ..Default::default()
    };
    if let (Some(Some(next)), Some(up)) = (part_of, held_by)
        && Some(next) != task.part_of
        && !state.attended_by_agents(up)
    {
        return Err(Refused::Tool(format!(
            "{:?} is a part of {:?}, which the person keeps closed to agents, so moving it to \
             another whole is theirs. Say where it belongs with `note`.",
            task.title, up.title
        )));
    }
    if part_of == Some(None) {
        let Some(up) = held_by else {
            return Ok(told(
                format!(
                    "{:?} stands on its own already; nothing changed.",
                    task.title
                ),
                json!({ "id": id.to_string(), "title": task.title, "moved": false }),
            ));
        };
        if !state.attended_by_agents(up) {
            return Err(Refused::Tool(format!(
                "{:?} is a part of {:?}, which the person keeps closed to agents, so letting it \
                 go is theirs. Say why it stands apart with `note`.",
                task.title, up.title
            )));
        }
        if d.list.is_none()
            && task.list.is_none()
            && up.list.is_some()
            && state.filed_by_agents(task)
        {
            d.list = Some(up.list);
        }
    }
    if part_of.is_none_or(|up| up == task.part_of) && d.list.is_none_or(|one| one == task.list) {
        return Ok(told(
            format!(
                "{:?} is already where you asked; nothing changed.",
                task.title
            ),
            json!({ "id": id.to_string(), "title": task.title, "moved": false }),
        ));
    }
    let written = store
        .append_batch_unless(vec![Op::TaskMove { id, d: d.clone() }], |events| {
            let held = State::replay(events);
            held.tasks
                .get(&id)
                .is_none_or(|now| !still_filling(&held, now))
        })
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }
    let list_name = |one: Option<ListId>| {
        one.and_then(|at| state.lists.get(&at)).map_or_else(
            || "the inbox".to_string(),
            |list| format!("the list {}", list.name),
        )
    };
    let mut said = Vec::new();
    match part_of {
        Some(Some(up)) => said.push(format!(
            "Hung {:?} under {:?} as one of its parts.",
            task.title,
            state.tasks.get(&up).map_or("", |one| one.title.as_str())
        )),
        Some(None) => said.push(format!(
            "Let {:?} go from {:?}: it stands on its own, in {}.",
            task.title,
            held_by.map_or("", |one| one.title.as_str()),
            list_name(d.list.unwrap_or(task.list))
        )),
        None => {}
    }
    if let Some(list) = list {
        said.push(format!(
            "Moved {:?} to {}.",
            task.title,
            list_name(Some(list))
        ));
    }
    Ok(told(
        said.join(" "),
        json!({
            "id": id.to_string(),
            "title": task.title,
            "moved": true,
            "part_of": part_of.map(|up| up.map(|one| one.to_string())),
            "list": listed,
        }),
    ))
}

fn list_named(state: &State, task: &Task, name: &str) -> Result<ListId, Refused> {
    if !state.filed_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{:?} is the person's own: its list is theirs, opened to agents or not. Say where \
             you would put it with `note`.",
            task.title
        )));
    }
    let all = state.find_list(name);
    match all.as_slice() {
        [one] if one.archived => Err(Refused::Tool(format!(
            "the list {name:?} is put away, so nothing new goes in it."
        ))),
        [one] => Ok(one.id),
        [] => {
            let mut names: Vec<&str> = state
                .lists
                .values()
                .filter(|one| !one.archived)
                .map(|one| one.name.as_str())
                .collect();
            names.sort_unstable();
            Err(Refused::Tool(match names.is_empty() {
                true => format!("there is no list called {name:?}, and there are none yet."),
                false => format!(
                    "there is no list called {name:?}, and you cannot make one. These exist: {}.",
                    names.join(", ")
                ),
            }))
        }
        _ => Err(Refused::Tool(format!(
            "{name:?} matches more than one list. Send the whole name."
        ))),
    }
}

fn hung_under(state: &State, task: &Task, up: TaskId) -> Result<TaskId, Refused> {
    let Some(whole) = state.tasks.get(&up).filter(|one| !one.folded()) else {
        return Err(Refused::Tool(format!("no task here has the id {up}.")));
    };
    let repeats = |one: &Task| one.repeat.is_some() || one.after.is_some();
    let why = if up == task.id {
        Some("is the task itself, and a task is never a part of itself".to_string())
    } else if !whole.is_open() {
        Some("is closed, and a closed task takes no new parts".into())
    } else if whole.part_of.is_some() {
        Some("is itself a part, and a part holds no parts: one level only".into())
    } else if state.holds_parts(task.id) {
        Some(format!(
            "would hold {:?}, which has parts of its own: one level only, so let its parts go \
             first or hang them there one by one",
            task.title
        ))
    } else if repeats(whole) || repeats(task) {
        Some("comes back, or the task does, and something that never ends holds no parts".into())
    } else if !state.attended_by_agents(whole) {
        Some(
            "is the person's, kept closed to agents: a part goes only under a task you filed or \
             one the person opened to you"
                .into(),
        )
    } else if whole.resolved.is_some() {
        Some(
            "has been said done or not to be done, and the person has not decided yet: a new \
             part would stand under that mark unlooked at"
                .into(),
        )
    } else {
        None
    };
    match why {
        Some(why) => Err(Refused::Tool(format!(
            "{:?} {why}. Nothing was moved.",
            whole.title
        ))),
        None => Ok(up),
    }
}
