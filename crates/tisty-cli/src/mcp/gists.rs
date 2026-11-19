use serde_json::{Value, json};
use tisty_core::event::Gist;
use tisty_core::model::{AgentNote, Kept};
use tisty_core::{Op, Paths, State, Store};

use super::asked::text;
use super::jsonrpc::{speaking_through, told};
use super::mistaken::{named_document, no_document_called};
use super::{Refused, hitch, opened};

pub(super) const A_GIST_IN_A_LINE: usize = 120;

pub(super) fn sum_up(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "saying what a document is about needs its `doc` name.".into(),
        ));
    };
    let summary = text(args, "summary").unwrap_or_default();
    let notes = text(args, "notes").unwrap_or_default();
    if summary.is_empty() && notes.is_empty() {
        return Err(Refused::Tool(
            "this takes a `summary` of what the document says, `notes` for what the next agent \
             should know about it, or both."
                .into(),
        ));
    }
    if summary.chars().count() > tisty_core::docs::SUMMARY_AT_MOST {
        return Err(Refused::Tool(format!(
            "a summary past {} characters is not a summary. Say what the document is for and \
             what a reader would come to it for.",
            tisty_core::docs::SUMMARY_AT_MOST
        )));
    }
    if notes.chars().count() > tisty_core::docs::NOTES_AT_MOST {
        return Err(Refused::Tool(format!(
            "`notes` runs past {} characters. What does not fit belongs in the document itself, \
             where the person can see it.",
            tisty_core::docs::NOTES_AT_MOST
        )));
    }

    let (state, mut store) = opened(paths)?;
    let Some(kept) = named_document(&state, &which) else {
        return Err(Refused::Tool(no_document_called(&state, &which)));
    };
    if kept.task_of.is_some() {
        return Err(Refused::Tool(format!(
            "{} is a task's page. What a task stands at goes in `tick` with `state`, where the \
             person reads it, not in a note only agents see. Nothing was written.",
            super::doc_named(&state, &which)
        )));
    }
    let held = tisty_core::cache::Cache::open(paths.cache()).ok().flatten();
    let Some(card) = tisty_core::docs::card_of(&paths.docs(), held.as_ref(), &which) else {
        return Err(Refused::Tool(format!(
            "{which:?} is named in the log but its text is not on this machine yet."
        )));
    };

    let said = Gist {
        print: card.print.clone(),
        summary,
        notes,
        at: None,
    };
    let again = state
        .agent_notes
        .get(&kept.id)
        .is_some_and(|had| says_the_same(had, &said));
    if !again {
        store
            .append(Op::DocGist {
                id: kept.id,
                d: said,
            })
            .map_err(hitch)?;
    }

    Ok(told(
        format!(
            "Kept what {} is about, against the text as it reads now. It goes with Tisty to the \
             person's other computers, never shown to the person, and the next reader is told if \
             the document has moved since.",
            super::named_as(&state, &which, &card.title)
        ),
        json!({
            "doc": which,
            "title": super::title_of(&state, &which, card.title.clone()),
            "print": card.print,
            "kept": true,
        }),
    ))
}

// A retry that says the same again would only lengthen the log every computer carries.
fn says_the_same(had: &AgentNote, said: &Gist) -> bool {
    had.print == said.print && had.summary == said.summary && had.notes == said.notes
}

pub(super) fn gist_told(gist: &Value, summary_most: usize, notes_most: usize) -> String {
    let stale = match gist["stale"] == json!(true) {
        true => " (stale: written against an older version)",
        false => "",
    };
    let cut = |said: &str, most: usize| match said.chars().count() > most {
        true => format!("{}…", said.chars().take(most).collect::<String>()),
        false => said.to_string(),
    };
    let mut told = Vec::new();
    if let Some(summary) = gist["summary"].as_str() {
        told.push(format!(
            "An agent's summary: {}",
            cut(summary, summary_most)
        ));
    }
    if let Some(notes) = gist["notes"].as_str() {
        told.push(format!("Its notes: {}", cut(notes, notes_most)));
    }
    match told.is_empty() {
        true => String::new(),
        false => format!("{}{stale}", told.join("\n")),
    }
}

pub(super) fn gist_of(state: &State, kept: &Kept, print: &str) -> Option<Value> {
    let held = state.agent_notes.get(&kept.id)?;
    let mut said = serde_json::Map::new();
    if !held.summary.is_empty() {
        said.insert("summary".into(), json!(held.summary));
    }
    if !held.notes.is_empty() {
        said.insert("notes".into(), json!(held.notes));
    }
    said.insert("said_at".into(), json!(held.at.to_string()));
    said.insert("by_agent".into(), json!(true));
    if held.print != print {
        said.insert("stale".into(), json!(true));
    }
    Some(Value::Object(said))
}

// Run at every start, not once: an older build on this machine may still keep notes there.
pub(super) fn carried_over(paths: &Paths) {
    let Some(cache) = tisty_core::cache::Cache::open(paths.cache()).ok().flatten() else {
        return;
    };
    let kept_here = cache.gists();
    if kept_here.is_empty() {
        return;
    }
    let Some((state, mut store)) = as_the_agent(paths) else {
        return;
    };
    let mut ops = Vec::new();
    let mut settled = Vec::new();
    for (file, gist) in kept_here {
        if let Some(kept) = a_document(&state, &file)
            && !state
                .agent_notes
                .get(&kept.id)
                .is_some_and(|had| had.at >= gist.at)
        {
            ops.push(Op::DocGist {
                id: kept.id,
                d: Gist {
                    print: gist.print,
                    summary: gist.summary,
                    notes: gist.notes,
                    at: Some(gist.at),
                },
            });
        }
        settled.push(file);
    }
    if !ops.is_empty() && store.append_batch(ops).is_err() {
        return;
    }
    for file in settled {
        cache.forget_gist(&file);
    }
}

// The whole log, the shut diary too: a note written while it was open still belongs to it.
fn as_the_agent(paths: &Paths) -> Option<(State, Store)> {
    let agent = tisty_core::Config::load_or_init(paths).ok()?.agent_id?;
    let state = tisty_core::cache::project(&paths.store(), paths.cache()).ok()?;
    if !state.agents.contains(&agent) {
        return None;
    }
    let key = tisty_core::signing::mine(paths, &agent);
    let store = Store::open(paths.store(), agent)
        .ok()?
        .signing_with(key)
        .speaking_through(speaking_through());
    Some((state, store))
}

fn a_document<'a>(state: &'a State, file: &str) -> Option<&'a Kept> {
    state
        .docs
        .values()
        .find(|one| one.file == file && one.task_of.is_none())
}
