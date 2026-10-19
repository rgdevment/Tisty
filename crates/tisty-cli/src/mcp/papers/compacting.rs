use serde_json::{Value, json};
use tisty_core::model::{Kept, TaskId};
use tisty_core::{Op, Paths, State, Store};

use super::super::{Refused, doc_named, hitch, opened};
use super::writing::over_again;

fn flag(args: &Value, key: &str) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(false)
}

pub(super) fn compacting(args: &Value) -> bool {
    flag(args, "compact")
}

pub(super) fn not_alone(args: &Value) -> Result<(), Refused> {
    match compacting(args) {
        true => Err(Refused::Tool(
            "`compact` writes an existing document back into shape, so it needs the `doc` and \
             the `print` you read it at. A new document is written without it."
                .into(),
        )),
        false => Ok(()),
    }
}

pub(in crate::mcp) fn agents_wrote(state: &State, kept: &Kept) -> bool {
    kept.made_by
        .as_ref()
        .is_some_and(|who| state.assistants.contains(who))
}

/// A second document on a subject that has one is how a store fills with copies of itself.
pub(super) fn subject_free(state: &State, args: &Value, body: &str) -> Result<(), Refused> {
    if flag(args, "anyway") {
        return Ok(());
    }
    let wanted = tisty_core::text::subject(&tisty_core::docs::titled(body));
    if wanted.is_empty() {
        return Ok(());
    }
    let Some(taken) = state.papers().find(|one| {
        one.page_of.is_none()
            && one.flagged.is_none()
            && !state.held_away(one)
            && one
                .title
                .as_deref()
                .is_some_and(|title| tisty_core::text::subject(title) == wanted)
    }) else {
        return Ok(());
    };
    Err(Refused::Tool(format!(
        "{} already holds that subject, so nothing was written: a second document on it is a \
         copy that drifts. Update that one — `edit_doc`, or `write_doc` with its `doc` and \
         `print` — or, if this really is another subject, give it a title that says how, or \
         send `anyway`.",
        doc_named(state, &taken.file)
    )))
}

pub(super) fn compacted(
    paths: &Paths,
    args: &Value,
    state: &State,
    store: &mut Store,
    which: &str,
    body: &str,
) -> Result<Value, Refused> {
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    if !agents_wrote(state, kept) && !flag(args, "asked") {
        return Err(Refused::Tool(format!(
            "{} was begun by the person, so it is theirs to have compacted: nothing was written. \
             Do it only when they ask you in this conversation, and then send `asked`.",
            doc_named(state, which)
        )));
    }
    let (id, loose) = (kept.id, kept.folder.is_none() && kept.page_of.is_none());
    let was = tisty_core::docs::read(&paths.docs(), which).map_err(hitch)?;
    let mut said = over_again(paths, args, state, store, which, body)?;
    let whole = tisty_core::docs::read(&paths.docs(), which).unwrap_or_else(|_| body.to_string());
    let print = Some(tisty_core::attach::printed(whole.as_bytes()));
    let before = Some(tisty_core::attach::printed(was.as_bytes()));
    // The body is already rewritten, so a mark that fails to land must not read as a write that did not happen.
    if let Err(e) = store.append(Op::DocCompacted { id, print, before }) {
        tisty_core::witness::warn(
            tisty_core::witness::channel::DOOR,
            "a compaction was written but its mark could not be kept",
            &[
                ("at", tisty_core::witness::Fact::Id(id.to_string())),
                ("code", tisty_core::witness::Fact::Code(e.coded())),
            ],
        );
    }

    let dropped = dropped(&was, &whole);
    let (before, after) = (was.chars().count(), whole.chars().count());
    let title = tisty_core::docs::titled(&whole);
    let more = format!(
        " Compacted from {before} to {after} characters; the window shows the person that you \
         did, with one click to undo, and `restore_doc` brings the old body back.{}{}",
        match dropped.is_empty() {
            true => String::new(),
            false => format!(
                " These files were named before and are not now, so the body no longer shows \
                 them: {}. Put each back where it belongs unless losing it was the point.",
                dropped.join(", ")
            ),
        },
        match loose {
            true => super::super::placing::loose_doc(state, &title),
            false => String::new(),
        }
    );
    if let Some(text) = said["content"][0]["text"].as_str() {
        let text = format!("{text}{more}");
        said["content"][0]["text"] = json!(text);
        if said["structuredContent"].get("said").is_some() {
            said["structuredContent"]["said"] = json!(text);
        }
    }
    said["structuredContent"]["shrunk"] = json!({ "before": before, "after": after });
    said["structuredContent"]["dropped"] = json!(dropped);
    said["structuredContent"]["compacted"] = json!(true);
    Ok(said)
}

fn dropped(was: &str, whole: &str) -> Vec<String> {
    let files = |body: &str| -> Vec<String> {
        tisty_core::refs::extract(body)
            .into_iter()
            .map(|one| one.target)
            .filter(|one| tisty_core::attach::names_an_attachment(one))
            .collect()
    };
    let now = files(whole);
    let mut gone: Vec<String> = files(was)
        .into_iter()
        .filter(|one| !now.contains(one))
        .collect();
    gone.sort();
    gone.dedup();
    gone
}

pub(super) fn page_to_restore(paths: &Paths, which: &str) -> Result<(String, bool), Refused> {
    let Ok(id) = which.parse::<TaskId>() else {
        return Ok((which.to_string(), false));
    };
    let (state, _) = opened(paths)?;
    let Some(task) = state.tasks.get(&id).filter(|one| !one.folded()) else {
        return Ok((which.to_string(), false));
    };
    if !task.is_open() || !state.attended_by_agents(task) {
        return Err(Refused::Tool(format!(
            "{:?} is not a task you may work on, open and opened to agents, so its page is not \
             yours to put back.",
            task.title
        )));
    }
    let owner = tisty_core::task_docs::owner(&state, id);
    match tisty_core::task_docs::of(&state, owner) {
        Some(page) => Ok((page.file.clone(), true)),
        None => Err(Refused::Tool(format!(
            "{:?} has no page yet, so there is nothing to put back.",
            task.title
        ))),
    }
}

pub(super) fn only_the_notes(now: &str, was: &str) -> Result<(), Refused> {
    use tisty_core::assistant_notes::outside;
    match tisty_core::docs::unchanged(&outside(now), &outside(was)) {
        true => Ok(()),
        false => Err(Refused::Tool(
            "going back on a task's page reaches only the assistant's notes, and what is kept \
             beside it differs elsewhere too — its steps, or the person's words. Nothing was \
             changed. Write the notes again with `append_doc` and `replace` instead."
                .into(),
        )),
    }
}

// A compaction is a proposal until the person keeps or undoes it, and only one step back is kept.
pub(super) fn awaiting(state: &State, kept: &tisty_core::model::Kept) -> Result<(), Refused> {
    match kept.compacted {
        None => Ok(()),
        Some(_) => Err(Refused::Tool(format!(
            "{} was compacted and waits for the person to keep it or undo it in the window; \
             another write now would cost them that one-click undo, so nothing was written. \
             Tell them what you would still change, and write once they have kept it.",
            doc_named(state, &kept.file)
        ))),
    }
}
