use serde_json::Value;
use tisty_core::State;
use tisty_core::event::Op;
use tisty_core::model::TaskId;

use super::asked::{only_what_it_takes, short_and_plain, text};
use super::chores::{Drafted, drafted, standing};
use super::order;
use super::{Refused, alike};

const PARTS_AT_MOST: usize = 32;

/// Refused here, with the reason, rather than let go quietly when the log is read back.
pub(super) fn whole_named(state: &State, args: &Value) -> Result<Option<TaskId>, Refused> {
    let Some(said) = text(args, "part_of") else {
        return Ok(None);
    };
    if args.get("parts").is_some_and(|one| !one.is_null()) {
        return Err(Refused::Tool(
            "a task that holds `parts` is not itself a part: send one or the other.".into(),
        ));
    }
    let Ok(id) = said.parse::<TaskId>() else {
        return Err(Refused::Tool(format!(
            "{said:?} is not a task id. Use the `id` that `find` or `propose` gave you."
        )));
    };
    let Some(whole) = state.tasks.get(&id).filter(|one| !one.folded()) else {
        return Err(Refused::Tool(format!("no task here has the id {said}.")));
    };
    let why = if !whole.is_open() {
        Some("is closed, and a closed task takes no new parts")
    } else if whole.part_of.is_some() {
        Some("is itself a part, and a part holds no parts: one level only")
    } else if whole.repeat.is_some() || whole.after.is_some() {
        Some("comes back, and something that never ends holds no parts")
    } else if !state.attended_by_agents(whole) {
        Some(
            "is the person's, kept closed to agents: a part goes only under a task you filed or \
             one the person opened to you",
        )
    } else {
        None
    };
    match why {
        Some(why) => Err(Refused::Tool(format!(
            "{:?} {why}. Nothing was written.",
            whole.title
        ))),
        None => Ok(Some(id)),
    }
}

pub(super) fn parts_drafted(
    state: &State,
    args: &Value,
    whole: &Drafted,
    again: bool,
) -> Result<Vec<Drafted>, Refused> {
    let Some(given) = args.get("parts").filter(|one| !one.is_null()) else {
        return Ok(Vec::new());
    };
    let Some(given) = given.as_array() else {
        return Err(Refused::Tool(
            "`parts` is a list of tasks, each written the way a single one is.".into(),
        ));
    };
    if given.len() > PARTS_AT_MOST {
        return Err(Refused::Tool(format!(
            "{} is more than the {PARTS_AT_MOST} parts one call takes. Nothing was written. Send \
             the first {PARTS_AT_MOST} here, and the rest in rounds of their own, each with \
             `part_of` set to the id this call gives back.",
            given.len()
        )));
    }
    let (list, mut place) = whole
        .ops
        .iter()
        .find_map(|op| match op {
            Op::TaskAdd { id, d } if *id == whole.id => Some((d.list, d.order.clone())),
            _ => None,
        })
        .unwrap_or_else(|| (None, order::first()));

    let mut drafts: Vec<Drafted> = Vec::with_capacity(given.len());
    for one in given {
        if !one.is_object() {
            return Err(Refused::Tool(
                "every entry in `parts` is a task of its own, written the way a single one is."
                    .into(),
            ));
        }
        if let Some(key) = ["parts", "tasks", "part_of", "again"]
            .into_iter()
            .find(|key| one.get(*key).is_some())
        {
            return Err(Refused::Tool(format!(
                "a part cannot carry `{key}`: there is one level only, and it belongs to the \
                 task around it. Nothing was written."
            )));
        }
        only_what_it_takes("propose", one)?;
        short_and_plain(one)?;
        let Some(title) = text(one, "title") else {
            return Err(Refused::Tool(
                "every part needs a `title`. Nothing was written.".into(),
            ));
        };
        if let Some(source) = text(one, "source") {
            let spelled = alike(&source);
            let twice = whole
                .source
                .iter()
                .chain(drafts.iter().filter_map(|drafted| drafted.source.as_ref()))
                .any(|other| alike(other) == spelled);
            if twice || standing(state, &source, again).is_some() {
                return Err(Refused::Tool(format!(
                    "the part {title:?} comes from {source:?}, which {} Nothing was written.",
                    match twice {
                        true => "the task or another part in this call comes from too.",
                        false => "was proposed already: `find` it with `source`.",
                    }
                )));
            }
        }
        let mut part = drafted(state, one, &title, Some(whole.id))?;
        place = order::after(&place);
        let its_own_list = text(one, "list").is_some();
        for op in &mut part.ops {
            if let Op::TaskAdd { id, d } = op
                && *id == part.id
            {
                d.order.clone_from(&place);
                if !its_own_list {
                    d.list = list;
                }
            }
        }
        drafts.push(part);
    }
    Ok(drafts)
}
