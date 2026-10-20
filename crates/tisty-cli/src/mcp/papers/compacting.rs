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

pub(super) struct Original {
    body: String,
    print: Option<String>,
}

/// Correcting a compaction still last keeps going back leading to the body before the first.
pub(super) fn correcting(paths: &Paths, kept: &Kept, args: &Value) -> Option<Original> {
    if !compacting(args) || flag(args, "keep_compaction") {
        return None;
    }
    let first = kept.compacted.as_ref()?;
    let now = tisty_core::docs::read(&paths.docs(), &kept.file).ok()?;
    if !first.still_last(&now) {
        return None;
    }
    let body = tisty_core::docs::read_before(paths.data(), &kept.file)?;
    let print = Some(tisty_core::attach::printed(body.as_bytes()));
    match &first.before {
        Some(before) if Some(before) != print.as_ref() => None,
        _ => Some(Original { body, print }),
    }
}

pub(super) fn compacted(
    paths: &Paths,
    args: &Value,
    state: &State,
    store: &mut Store,
    which: &str,
    body: &str,
    first: Option<Original>,
) -> Result<Value, Refused> {
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    if !agents_wrote(state, kept) && !flag(args, "person_asked") {
        return Err(Refused::Tool(format!(
            "{} was begun by the person, so it is theirs to have compacted: nothing was written. \
             Do it only when they ask you in this conversation, and then send `person_asked`.",
            doc_named(state, which)
        )));
    }
    let (id, loose) = (kept.id, kept.folder.is_none() && kept.page_of.is_none());
    let was = match &first {
        Some(one) => one.body.clone(),
        None => tisty_core::docs::read(&paths.docs(), which).map_err(hitch)?,
    };
    unsaid(state, kept, &was, body)?;
    let mut said = over_again(paths, args, state, store, which, body)?;
    let whole = tisty_core::docs::read(&paths.docs(), which).unwrap_or_else(|_| body.to_string());
    if let Some(one) = &first
        && let Err(e) = tisty_core::docs::kept_before(paths.data(), which, &one.body, &whole)
    {
        tisty_core::witness::warn(
            tisty_core::witness::channel::DOOR,
            "a compaction was written again but the body before the first could not be kept",
            &[("code", tisty_core::witness::Fact::Code(e.coded()))],
        );
    }
    let print = Some(tisty_core::attach::printed(whole.as_bytes()));
    let before = match first {
        Some(one) => one.print,
        None => Some(tisty_core::attach::printed(was.as_bytes())),
    };
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
    let unlinked = unlinked(&was, &whole);
    let (before, after) = (was.chars().count(), whole.chars().count());
    let title = tisty_core::docs::titled(&whole);
    let more = format!(
        " Compacted from {before} to {after} characters; the window shows the person that you \
         did, with one click to undo, and `restore_doc` brings the old body back. Leave what you worked out \
         with `sum_up`, so the next reader does not read it all again.{}{}{}",
        match after > before {
            true => format!(
                " The body grew from {before} to {after} characters: compacting folds and \
                 moves, it should not add; if this is right, say why to the person."
            ),
            false => String::new(),
        },
        match dropped.is_empty() {
            true => String::new(),
            false => format!(
                " These files were shown before and are only mentioned now: {}. Put each back \
                 where it belongs unless losing it was the point.",
                dropped.join(", ")
            ),
        },
        match loose {
            true => super::super::placing::loose_doc(state, &title, &whole),
            false => String::new(),
        }
    );
    let unlinked: Vec<String> = unlinked
        .into_iter()
        .filter(|one| live(state, one))
        .collect();
    let more = match unlinked.is_empty() {
        true => more,
        false => format!(
            "{more} These links were in the old body and are only mentioned in this one: {}.",
            unlinked.join(", ")
        ),
    };
    if let Some(text) = said["content"][0]["text"].as_str() {
        let text = format!("{text}{more}");
        said["content"][0]["text"] = json!(text);
        if said["structuredContent"].get("said").is_some() {
            said["structuredContent"]["said"] = json!(text);
        }
    }
    said["structuredContent"]["size"] = json!({ "before": before, "after": after });
    said["structuredContent"]["dropped"] = json!(dropped);
    said["structuredContent"]["unlinked"] = json!(unlinked);
    said["structuredContent"]["compacted"] = json!(true);
    Ok(said)
}

/// Pages the write names again by itself are not lost, so only what nothing carries is refused.
fn unsaid(state: &State, kept: &Kept, was: &str, body: &str) -> Result<(), Refused> {
    let put_back: Vec<String> = state
        .pages_of(kept.id)
        .into_iter()
        .filter(|one| one.flagged.is_none())
        .map(|one| format!("{}{}", tisty_core::refs::DOC, one.file))
        .collect();
    // A link to nothing, or to a document marked in this same pass, already has its trace.
    let settled = |target: &str| -> bool {
        let Some(file) = target.strip_prefix(tisty_core::refs::DOC) else {
            return false;
        };
        let file = file.split(['#', '?']).next().unwrap_or(file);
        match state.docs.values().find(|one| one.file == file) {
            None => true,
            Some(one) => {
                one.flagged.is_some()
                    || one.title.as_deref().is_some_and(|title| {
                        !title.trim().is_empty() && body.contains(title.trim())
                    })
            }
        }
    };
    let lost = |gone: Vec<String>| -> Vec<String> {
        gone.into_iter()
            .filter(|one| !put_back.contains(one) && !mentioned(body, one) && !settled(one))
            .collect()
    };
    let (links, files) = (lost(unlinked(was, body)), lost(dropped(was, body)));
    let mut told = Vec::new();
    if !links.is_empty() {
        told.push(format!(
            "These links are in the old body and nowhere in the new one: {}.",
            links.join(", ")
        ));
    }
    if !files.is_empty() {
        told.push(format!(
            "These files are named in the old body and nowhere in the new one: {}.",
            files.join(", ")
        ));
    }
    match told.is_empty() {
        true => Ok(()),
        false => Err(Refused::Tool(format!(
            "{} Name each in the history of the body you send — by its address, or a \
             document that still exists by its title — so a link taken out is never a link \
             lost; links to nothing, and to documents marked in this pass, need no line. \
             Nothing was written.",
            told.join(" ")
        ))),
    }
}

fn mentioned(body: &str, target: &str) -> bool {
    body.contains(target)
        || target
            .strip_prefix(tisty_core::refs::DOC)
            .is_some_and(|file| body.contains(file))
        || tisty_core::attach::leaf_of(target).is_some_and(|leaf| body.contains(leaf))
}

fn dropped(was: &str, whole: &str) -> Vec<String> {
    gone_from(was, whole, true)
}

fn unlinked(was: &str, whole: &str) -> Vec<String> {
    gone_from(was, whole, false)
}

fn gone_from(was: &str, whole: &str, files: bool) -> Vec<String> {
    let named = |body: &str| -> Vec<String> {
        tisty_core::refs::extract(body)
            .into_iter()
            .map(|one| one.target)
            .filter(|one| tisty_core::attach::names_an_attachment(one) == files)
            .collect()
    };
    let now = named(whole);
    let mut gone: Vec<String> = named(was)
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
pub(in crate::mcp) fn awaiting(state: &State, kept: &Kept) -> Result<(), Refused> {
    match kept.compacted {
        None => Ok(()),
        Some(_) => Err(Refused::Tool(format!(
            "{} was compacted and waits for the person to keep it or undo it in the window; \
             another write now would cost them that one-click undo, so nothing was written. \
             While it is still the last write you may correct it with one more `write_doc` \
             with `compact`; otherwise tell them what you would still change, and when they \
             tell you in this conversation to keep it, send `keep_compaction` with your next \
             write.",
            doc_named(state, &kept.file)
        ))),
    }
}

pub(in crate::mcp) fn released(
    state: &State,
    kept: &Kept,
    args: &Value,
    store: &mut Store,
) -> Result<(), Refused> {
    if kept.compacted.is_some() && flag(args, "keep_compaction") {
        return store
            .append(Op::DocUncompacted { id: kept.id })
            .map(|_| ())
            .map_err(hitch);
    }
    awaiting(state, kept)
}

fn live(state: &State, target: &str) -> bool {
    let Some(file) = target.strip_prefix(tisty_core::refs::DOC) else {
        return true;
    };
    let file = file.split(['#', '?']).next().unwrap_or(file);
    state
        .docs
        .values()
        .find(|one| one.file == file)
        .is_some_and(|one| one.flagged.is_none())
}
