use serde_json::json;
use tisty_core::model::{Task, TaskId};
use tisty_core::{Op, State, order};

use super::Refused;
use super::chores::{Drafted, already_said_done, drafted, still_in_sight};

pub(super) enum Companion<'a> {
    Held(TaskId, &'a Task),
    Fresh(Drafted, TaskId),
}

pub(super) fn anchor_of<'a>(state: &'a State, task: &'a Task) -> &'a Task {
    task.part_of
        .and_then(|whole| state.tasks.get(&whole))
        .unwrap_or(task)
}

pub(super) fn held(state: &State, anchor: TaskId) -> Option<&Task> {
    state
        .parts_of(anchor)
        .filter(|part| part.for_person && still_in_sight(part) && part.resolved.is_none())
        .min_by(|one, other| one.order.cmp(&other.order))
}

pub(super) fn companion<'a>(
    state: &'a State,
    task: &'a Task,
    title: Option<String>,
    steps: &[String],
    lang: crate::i18n::Lang,
) -> Result<Companion<'a>, Refused> {
    if !state.agents_own(task) {
        return Err(Refused::Tool(format!(
            "{:?} is the person's already, so its steps are theirs: nothing was moved. \
             `to_person` hands over steps from a task of yours.",
            task.title
        )));
    }
    let anchor = anchor_of(state, task);
    if anchor.id != task.id && !state.agents_own(anchor) && still_in_sight(anchor) {
        if let Some(why) = already_said_done(anchor, "adding steps to it") {
            return Err(why);
        }
        return Ok(Companion::Held(anchor.id, anchor));
    }
    if let Some(one) = held(state, anchor.id) {
        return Ok(Companion::Held(one.id, one));
    }
    if anchor.id != task.id
        && let Some(why) = already_said_done(anchor, "hanging the person's part on it")
    {
        return Err(why);
    }
    let repeats = anchor.repeat.is_some() || anchor.after.is_some();
    if repeats || !still_in_sight(anchor) {
        return Err(Refused::Tool(format!(
            "{:?} {} so it holds no part for the person: nothing was moved. `propose` their \
             task with `for` \"person\" in the same list, then hand the steps to it with `to`.",
            anchor.title,
            match repeats {
                true => "is a turn of something that repeats,",
                false => "is closed,",
            }
        )));
    }
    let title = title.unwrap_or_else(|| lang.fill("companion-title", &[("task", &anchor.title)]));
    let asked = json!({ "title": title, "steps": steps });
    let mut fresh = drafted(state, &asked, &title, Some(anchor.id), true)?;
    for op in &mut fresh.ops {
        if let Op::TaskAdd { id, d } = op
            && *id == fresh.id
        {
            d.list = anchor.list;
            d.order = order::after(&anchor.order);
        }
    }
    Ok(Companion::Fresh(fresh, anchor.id))
}
