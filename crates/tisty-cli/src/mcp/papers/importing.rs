use serde_json::{Value, json};

use super::writing::{beside_the_file, said_of, where_it_lands, write_doc};

use super::reachable_or;
use tisty_core::Paths;
use tisty_core::model::{Step, Task, TaskId};

use super::super::asked::{body_at_most, short_and_plain, text};
use super::super::chores::{already_said_done, filling, still_filling};
use super::super::jsonrpc::told;
use super::super::{Refused, hitch, moved, opened, retold};

pub(in crate::mcp) fn import_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "path") else {
        return Err(Refused::Tool(
            "importing needs a `path` to a markdown file on this machine.".into(),
        ));
    };
    let asked = std::path::Path::new(&said);
    let at = reachable_or(paths, &said, asked, "take files from")?;
    if !at.is_file() {
        return Err(Refused::Tool(format!(
            "{said:?} is not a file. `import_doc` takes one markdown file at a time; call it once per file when you are bringing a whole export across."
        )));
    }
    if !at
        .extension()
        .is_some_and(|one| one.eq_ignore_ascii_case("md") || one.eq_ignore_ascii_case("markdown"))
    {
        return Err(Refused::Tool(format!(
            "{said:?} is not markdown. A document is markdown; anything else goes in with `attach`, which keeps it beside a document or a task."
        )));
    }
    let big = std::fs::metadata(&at).map(|one| one.len()).unwrap_or(0);
    if big > tisty_core::docs::BODY_AT_MOST {
        return Err(Refused::Tool(format!(
            "{said:?} is {big} bytes, past the {} a file may be to be read at all. Split it \
             before bringing it in.",
            tisty_core::docs::BODY_AT_MOST
        )));
    }
    let raw = std::fs::read(&at)
        .map_err(|why| Refused::Tool(format!("{said:?} could not be read: {why}.")))?;
    let raw = String::from_utf8(raw).map_err(|_| {
        Refused::Tool(format!(
            "{said:?} is not text this can read. Tisty keeps documents as UTF-8."
        ))
    })?;

    let looks =
        tisty_core::agent::secret_in(raw.as_bytes()).map(|one| match one.named.is_empty() {
            true => one.why.to_string(),
            false => one.named,
        });

    let titles = tisty_core::arriving::Titles::of(&tisty_core::docs::all(&paths.docs()));
    let made = tisty_core::arriving::tidied_among(&raw, &titles);
    tisty_core::docs::survives(&made.body).map_err(|eats| {
        Refused::Tool(format!(
            "{said:?} still holds {eats} after being tidied, which Tisty does not keep in its own words. Nothing was written."
        ))
    })?;
    short_and_plain(&json!({ "body": made.body }))?;
    // The files beside it are copied in below, and a document turned away after that would
    // leave them on the person's disk with nothing pointing at them.
    let (state, mut store) = opened(paths)?;
    let home = task_named(&state, &store, &made.body)?;
    match home {
        Some(id) => coming_home(paths, &state, id, &made.body)?,
        None => {
            where_it_lands(&state, args)?;
            super::compacting::subject_free(&state, args, &titled(args, &at, &made.body))?;
        }
    }
    let (whole, brought) = beside_the_file(paths, &at, &made.body);
    if let Some(id) = home {
        if !brought.told.is_empty() {
            store.append_batch(brought.told).map_err(hitch)?;
        }
        drop(store);
        return came_home(paths, id, &whole);
    }
    let made = tisty_core::arriving::Tidied {
        body: whole,
        changed: made.changed,
    };

    let body = titled(args, &at, &made.body);

    let mut asked_again = args.clone();
    if let Some(one) = asked_again.as_object_mut() {
        one.remove("path");
        one.remove("title");
        one.insert("body".into(), json!(body));
    }
    let long = body.chars().count();
    if long > body_at_most() {
        return Err(Refused::Tool(format!(
            "{said:?} reads as {long} characters once it is tidied, and a document is kept up \
             to {}. Split it before bringing it in.",
            body_at_most()
        )));
    }
    short_and_plain(&asked_again)?;
    if !brought.told.is_empty() {
        store.append_batch(brought.told).map_err(hitch)?;
    }
    drop(store);
    let written = write_doc(paths, &asked_again)?;

    let changed = made.changed.join(", ");
    Ok(told(
        format!(
            "{}{}{}{}{}{}",
            said_of(&written),
            match made.changed.is_empty() {
                true => " Nothing had to be changed on the way in.".to_string(),
                false => format!(
                    " On the way in it was tidied: {changed}. What the file said is still on disk, untouched."
                ),
            },
            match brought.kept {
                0 => String::new(),
                one => format!(
                    " {one} file(s) beside it came in too, and the text now points at the copies Tisty keeps."
                ),
            },
            match brought.missed.is_empty() {
                true => String::new(),
                false => format!(
                    " These could not come in, so their links were taken out rather than left pointing outside Tisty — the words that named them are still in the text: {}.",
                    brought.missed.join("; ")
                ),
            },
            match &looks {
                None => String::new(),
                Some(said) => format!(
                    " Nothing was held back: what reads like a live credential ({said}) came in as written, under a warning the person sees when they open it."
                ),
            },
            match brought.papers.is_empty() {
                true => String::new(),
                false => format!(
                    " It also names {} other markdown file(s) — those are documents, not files to keep: import each one and the links will still read as text until you tie them together.",
                    brought.papers.len()
                ),
            }
        ),
        match written["structuredContent"].clone() {
            Value::Object(mut one) => {
                one.insert("from".into(), json!(at.display().to_string()));
                one.insert("changed".into(), json!(made.changed));
                one.insert("files".into(), json!(brought.kept));
                one.insert("left_behind".into(), json!(brought.missed));
                one.insert("names_markdown".into(), json!(brought.papers));
                one.insert("reads_like_a_credential".into(), json!(looks));
                Value::Object(one)
            }
            other => other,
        },
    ))
}

fn titled(args: &Value, at: &std::path::Path, whole: &str) -> String {
    let past = tisty_markdown::front::span(whole).map_or(0, |at| at.end);
    let (front, rest) = whole.split_at(past);
    let front = match front.is_empty() || front.ends_with('\n') {
        true => front.to_string(),
        false => format!("{front}\n"),
    };
    let named_above = tisty_markdown::front::title(&front).is_some();
    let headed = named_above
        || rest
            .lines()
            .find(|one| !one.trim().is_empty())
            .is_some_and(|one| one.starts_with("# "));
    match (text(args, "title"), headed) {
        (Some(said), _) if named_above => format!(
            "{}{rest}",
            tisty_markdown::front::titled_as(&front, &said).unwrap_or(front.clone())
        ),
        (Some(said), _) => format!("{front}# {said}\n\n{}", rest.trim_start()),
        (None, true) => whole.to_string(),
        (None, false) => {
            let named = at.file_stem().unwrap_or_default().to_string_lossy();
            format!("{front}# {named}\n\n{}", rest.trim_start())
        }
    }
}

fn zone() -> String {
    jiff::tz::TimeZone::system()
        .iana_name()
        .unwrap_or("UTC")
        .to_string()
}

// A page naming a task this assistant may fill is that task's; one it may not is refused whole.
fn task_named(
    state: &tisty_core::State,
    store: &tisty_core::Store,
    body: &str,
) -> Result<Option<TaskId>, Refused> {
    let Some(id) = tisty_core::task_front::read(body, &zone()).task else {
        return Ok(None);
    };
    if !state.tasks.contains_key(&id) {
        return Ok(None);
    }
    filling(state, store, &id.to_string()).map(|(id, _)| Some(id))
}

fn page_of(paths: &Paths, state: &tisty_core::State, id: TaskId) -> Option<(String, String)> {
    let owner = tisty_core::task_docs::owner(state, id);
    let file = tisty_core::task_docs::of(state, owner)?.file.clone();
    let body = tisty_core::docs::read(&paths.docs(), &file).ok()?;
    Some((file, body))
}

// Judged before the files beside it are copied in, and again under the lock before anything is written.
fn coming_home(
    paths: &Paths,
    state: &tisty_core::State,
    id: TaskId,
    body: &str,
) -> Result<(), Refused> {
    let Some(task) = state.tasks.get(&id) else {
        return Ok(());
    };
    if let Some(why) = already_said_done(task, "bringing its page back") {
        return Err(why);
    }
    let was = page_of(paths, state, id).map(|(_, body)| body);
    let Some(taken) = tisty_core::task_front::taken_in(state, body, was.as_deref(), &zone()) else {
        return Ok(());
    };
    if was
        .as_deref()
        .is_some_and(|was| tisty_core::task_front::left_out(was, &taken.body))
    {
        return Err(Refused::Tool(format!(
            "{:?}'s page holds lines this file leaves out or words differently — written since \
             the file was taken out, or changed in the copy — so bringing it in would erase them; \
             nothing was changed. `read` the task for its page as it is now and bring your changes \
             in with `tick`, `plan`, `unplan`, `reword_step`, `describe`, `note` or `append_doc`.",
            task.title
        )));
    }
    match broken_rule(task, &taken.ops) {
        Some(why) => Err(Refused::Tool(format!(
            "{why}; nothing was brought in. Say what you found with `note`."
        ))),
        None => Ok(()),
    }
}

// A page brings in only what tick, untick, unplan and reword_step would let an assistant do.
fn broken_rule(task: &Task, ops: &[tisty_core::Op]) -> Option<String> {
    ops.iter().find_map(|op| {
        let (step, wrong): (_, fn(&Step) -> Option<&'static str>) = match op {
            tisty_core::Op::StepRemove { d, .. } => (d.step, |one| {
                one.done
                    .then_some("is ticked, and a closed step is not yours to take off")
            }),
            tisty_core::Op::StepUndone { d, .. } => (d.step, |one| {
                (!one.by_agent).then_some("was ticked by the person, and taking it back is theirs")
            }),
            tisty_core::Op::StepText { d, .. } => (d.step, |one| {
                one.done
                    .then_some("is ticked, and a closed step is not yours to reword")
            }),
            _ => return None,
        };
        match task.step(step) {
            Some(one) => wrong(one).map(|why| format!("{:?} {why}", one.text)),
            None => Some(format!(
                "a step of {:?} changed since its page was read",
                task.title
            )),
        }
    })
}

// A task's page coming back hands the task its properties and its words, and is no document of its own.
fn came_home(paths: &Paths, id: TaskId, body: &str) -> Result<Value, Refused> {
    let (state, mut store) = opened(paths)?;
    coming_home(paths, &state, id, body)?;
    let Some(task) = state.tasks.get(&id) else {
        return Err(Refused::Tool(
            "that page names no task here any more.".into(),
        ));
    };
    let owner = tisty_core::task_docs::owner(&state, id);
    let kept = page_of(paths, &state, id);
    let was = kept.as_ref().map(|(_, body)| body.clone());
    let Some(taken) = tisty_core::task_front::taken_in(&state, body, was.as_deref(), &zone())
    else {
        return Err(Refused::Tool(
            "that page names no task here any more.".into(),
        ));
    };
    // A page an assistant brings back moves no day or list but on a task an agent filed.
    let ours = state.filed_by_agents(task);
    let mut ops: Vec<tisty_core::Op> = taken
        .ops
        .into_iter()
        .filter(|op| {
            ours || !matches!(
                op,
                tisty_core::Op::TaskUpdate { .. } | tisty_core::Op::TaskMove { .. }
            )
        })
        .collect();
    let mut file = kept.map(|(file, _)| file);
    if file.is_none() {
        let (made, created) = tisty_core::task_docs::made_with(
            &paths.docs(),
            &state,
            owner,
            &taken.body,
            state.signed.alias.clone(),
        )
        .map_err(hitch)?;
        // A file already on disk kept its own words, and the page brought in goes over them below.
        if let (false, tisty_core::Op::DocAdd { d, .. }) = (created, &made) {
            file = Some(d.file.clone());
        }
        ops.push(made);
    }
    if !ops.is_empty() {
        let asked = ops.clone();
        let written = store
            .append_batch_unless(ops, |events| {
                let held = tisty_core::State::replay(events);
                held.tasks.get(&id).is_none_or(|now| {
                    !still_filling(&held, now)
                        || now.resolved.is_some()
                        || broken_rule(now, &asked).is_some()
                })
            })
            .map_err(hitch)?;
        if written.is_none() {
            return Err(moved(task));
        }
    }
    let mut kept_as_it_was = false;
    if let Some(file) = &file {
        let (state, _) = opened(paths)?;
        let whole = tisty_core::docs::amend(&paths.docs(), paths.data(), file, |now| {
            // Words written on the page since it was read are not written over.
            kept_as_it_was = was.as_deref().is_some_and(|was| now != was);
            (!kept_as_it_was && now != taken.body).then(|| taken.body.clone())
        })
        .map_err(hitch)?;
        if let Some(whole) = whole {
            retold(&state, &mut store, file, &whole)?;
        }
    }
    if kept_as_it_was {
        tisty_core::witness::note(
            tisty_core::witness::channel::DOOR,
            "a task's page changed while a copy came in, so its words were kept",
            &[("task", tisty_core::witness::Fact::Id(task.id.to_string()))],
        );
    }
    let title = task.title.clone();
    let words = match kept_as_it_was {
        true => " Its page changed while this came in, so the page's words were left as they are.",
        false => "",
    };
    Ok(told(
        format!(
            "{title:?} came back as its task's own page: its properties and its boxes went to the task and its words to the task's page. No new document was made.{words}"
        ),
        json!({ "task": id.to_string(), "title": title }),
    ))
}
