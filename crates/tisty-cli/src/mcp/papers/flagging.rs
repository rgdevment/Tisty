use serde_json::{Value, json};
use tisty_core::model::{FLAG_REASONS, Kept, TaskId, reason_points_elsewhere};
use tisty_core::{Op, State, Store};

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::{Refused, doc_named, hitch, named_all, opened, speaking_through, when};
use super::{many_docs, said_docs};

pub(in crate::mcp) fn flag_doc(paths: &tisty_core::Paths, args: &Value) -> Result<Value, Refused> {
    let (many, listed) = many_docs(args, "mark")?;
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool(
            "marking a document needs a `body`: what makes it old and how you know. Without it \
             the person has only your word and nothing to weigh it against."
                .into(),
        ));
    };
    let reason = match text(args, "reason") {
        Some(said) if FLAG_REASONS.contains(&said.as_str()) => Some(said),
        Some(said) => {
            return Err(Refused::Tool(format!(
                "`reason` is one of {} — not {said:?}.",
                FLAG_REASONS.join(", ")
            )));
        }
        None => None,
    };
    let (state, mut store) = opened(paths)?;
    let instead = instead_of(&state, args, reason.as_deref(), &many)?;

    let mut marking: Vec<(&Kept, &str)> = Vec::new();
    let mut already: Vec<String> = Vec::new();
    for which in &many {
        let Some(kept) = state.docs.values().find(|one| one.file == *which) else {
            return Err(Refused::Tool(format!(
                "no document here is called {which:?}. `docs` lists them all; nothing was marked."
            )));
        };
        if state.held_away(kept) {
            return Err(Refused::Tool(format!(
                "{} is already in the archive, so it is out of the way. Nothing was marked.",
                doc_named(&state, which)
            )));
        }
        match &kept.flagged {
            Some(_) if listed => already.push(which.clone()),
            Some(had) => return Err(marked_twice(&state, &store, which, had)),
            None => marking.push((kept, which)),
        }
    }
    let at = jiff::Timestamp::now();
    let me = store.device().clone();
    let ops: Vec<Op> = marking
        .iter()
        .map(|(kept, _)| Op::DocFlag {
            id: kept.id,
            d: tisty_core::event::Flag::new(body.clone())
                .said_by(at, me.clone())
                .through(speaking_through())
                .because(reason.clone(), instead.clone()),
        })
        .collect();
    if !ops.is_empty() {
        store.append_batch(ops).map_err(hitch)?;
    }

    let marked: Vec<String> = marking
        .iter()
        .map(|(_, which)| (*which).to_string())
        .collect();
    let skipped = match already.is_empty() {
        true => String::new(),
        false => format!(
            " Already marked and waiting for the person, so left as they were: {}.",
            named_all(&state, &already)
        ),
    };
    let said = match marked.is_empty() {
        true => format!("Nothing new was marked.{skipped}"),
        false => format!(
            "Marked {} as {}. Untouched, it still reads the same: the person sees the mark when \
             they open it, and deleting, archiving or taking the mark off are theirs{}.{skipped}",
            named_all(&state, &marked),
            reason_said(&state, reason.as_deref(), instead.as_deref()),
            match marked.len() {
                1 => "",
                _ => " — one click settles the set",
            }
        ),
    };
    Ok(told(
        said,
        json!({
            "doc": said_docs(&marked, listed),
            "flagged": !marked.is_empty(),
            "reason": reason,
            "instead": instead,
            "skipped": already,
        }),
    ))
}

fn instead_of(
    state: &State,
    args: &Value,
    reason: Option<&str>,
    many: &[String],
) -> Result<Option<String>, Refused> {
    let Some(instead) = text(args, "instead") else {
        return match reason.is_some_and(reason_points_elsewhere) {
            true => Err(Refused::Tool(format!(
                "a document marked {} points at what holds it now: send `instead`, the id of \
                 that document or task.",
                reason.unwrap_or_default()
            ))),
            false => Ok(None),
        };
    };
    if many.contains(&instead) {
        return Err(Refused::Tool(
            "`instead` names one of the documents being marked; it has to be what holds them \
             now."
                .into(),
        ));
    }
    let a_doc = state.docs.values().any(|one| one.file == instead);
    let a_task = instead
        .parse::<TaskId>()
        .ok()
        .is_some_and(|id| state.tasks.contains_key(&id));
    match a_doc || a_task {
        true => Ok(Some(instead)),
        false => Err(Refused::Tool(format!(
            "`instead` is {instead:?}, and no document or task here has that id. `docs` and \
             `find` give them."
        ))),
    }
}

fn reason_said(state: &State, reason: Option<&str>, instead: Option<&str>) -> String {
    let there = instead.map(|one| match one.parse::<TaskId>() {
        Ok(id) => state.tasks.get(&id).map_or_else(
            || one.to_string(),
            |task| format!("the task {:?}", task.title),
        ),
        Err(_) => doc_named(state, one),
    });
    let there = there.unwrap_or_default();
    match reason {
        Some("duplicate_of") => format!("a duplicate of {there}"),
        Some("moved") => format!("moved to {there}"),
        Some("superseded") => format!("replaced by {there}"),
        Some("wrong") => "wrong".into(),
        Some("obsolete") => "obsolete".into(),
        Some("scratch") => "a scratch".into(),
        _ => "having had its day".into(),
    }
}

fn marked_twice(
    state: &State,
    store: &Store,
    which: &str,
    had: &tisty_core::model::Flagged,
) -> Refused {
    let who = match had.by == *store.device() && had.via == speaking_through() {
        true => "you".to_string(),
        false => had
            .via
            .as_deref()
            .map(tisty_core::agent::client_named)
            .unwrap_or_else(|| "an assistant".to_string()),
    };
    Refused::Tool(format!(
        "{who} already marked {} on {}, and the person has not looked yet. Marking it again \
         would only say the same thing twice.",
        doc_named(state, which),
        when(had.at)
    ))
}
