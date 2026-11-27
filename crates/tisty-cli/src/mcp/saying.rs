use serde_json::{Value, json};
use tisty_core::event::{LogAdd, Op, Resolve};
use tisty_core::{Paths, State};
use ulid::Ulid;

use super::asked::text;
use super::chores::{seen_task, still_in_sight};
use super::jsonrpc::{speaking_through, told};
use super::{Refused, history, hitch, moved, opened, unticked, when};

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
    let (branch, pr) = match drop {
        true => (None, None),
        false => (landed(args, "branch")?, landed(args, "pr")?),
    };
    let (state, mut store) = opened(paths)?;
    let (id, task) = seen_task(&state, &said)?;
    let me = store.device().clone();
    if !task.is_open() {
        return Err(history(task));
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
    let open_parts = state
        .parts_of(id)
        .filter(|part| {
            part.is_open() && !part.folded() && part.resolved.is_none() && !part.for_person
        })
        .count();
    if !drop && open_parts > 0 {
        return Err(Refused::Tool(format!(
            "{:?} still has {open_parts} part(s) open, and a whole is done only when its parts \
             are. Say each of yours done as you finish it. What only the person can do goes to \
             them with `unplan` and `to_person`, and their part never holds the whole; then \
             `say_done`. Whether to let the rest go is the person's call: say what is left with \
             `note`.",
            task.title
        )));
    }
    if drop && (task.repeat.is_some() || task.after.is_some()) {
        return Err(Refused::Tool(format!(
            "{:?} is a turn of something that repeats, and dropping it would end the repeat. \
             Whether it goes on is the person's call: say why with `note`.",
            task.title
        )));
    }
    if !drop && let Some(refusal) = unticked(task, state.agents_own(task)) {
        return Err(refusal);
    }

    let attended = state.agents_own(task);
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
                        .dropping(drop)
                        .as_word(!attended)
                        .landing(branch.clone(), pr.clone()),
                },
            ],
            |events| {
                let held = State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    !still_in_sight(now)
                        || held.agents_own(now) != attended
                        || now.resolved.is_some()
                        || (!drop && now.steps.iter().any(|step| !step.done && !step.said))
                })
            },
        )
        .map_err(hitch)?;
    if written.is_none() {
        return Err(moved(task));
    }

    let mut kept = json!({ "id": id.to_string(), "title": task.title, "open": true });
    if let Some(branch) = &branch {
        kept["branch"] = json!(branch);
    }
    if let Some(pr) = &pr {
        kept["pr"] = json!(pr);
    }
    Ok(told(
        format!(
            "{} {:?}{}. It stays open until the person decides.",
            if drop {
                "Said it will not be done:"
            } else {
                "Said done:"
            },
            task.title,
            match (&branch, &pr) {
                (_, Some(pr)) => format!(", on {pr}"),
                (Some(branch), None) => format!(", on the branch {branch}"),
                (None, None) => String::new(),
            }
        ),
        kept,
    ))
}

const LANDED_AT_MOST: usize = 200;

fn landed(args: &Value, key: &str) -> Result<Option<String>, Refused> {
    let Some(said) = text(args, key) else {
        return Ok(None);
    };
    if said.chars().count() > LANDED_AT_MOST || said.contains(char::is_control) {
        return Err(Refused::Tool(format!(
            "`{key}` is one line of at most {LANDED_AT_MOST} characters: the branch name, or the \
             pull request's number or link. Nothing was said."
        )));
    }
    Ok(Some(said))
}
