use serde_json::{Value, json};

use super::writing::{in_its_place, where_over};

use tisty_core::Paths;

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::spoken::spoken;
use super::super::{
    Refused, UNSETTLED, by_how_much, doc_named, for_the_person, grew, hitch, left_loose,
    loose_words, named_twice, nearest, opened, raw, retold, twice_words, with_echo, wrapped,
};

/// The document opened for an edit, refused when it is put away, waits on a compaction or is locked.
pub(super) fn writable(
    paths: &Paths,
    args: &Value,
    which: &str,
) -> Result<(tisty_core::State, tisty_core::Store, String), Refused> {
    let (state, mut store) = opened(paths)?;
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    let named = doc_named(&state, which);
    if state.held_away(kept) {
        return Err(Refused::Tool(format!(
            "{named} is put away, so it is not edited any more."
        )));
    }
    super::compacting::released(&state, kept, args, &mut store)?;
    if state.shut(kept.id) {
        return Err(Refused::Tool(format!(
            "{named} is locked. The person shut it so nothing writes in it — not the \
             window, not you. Ask them to unlock it if it truly has to change."
        )));
    }
    Ok((state, store, named))
}

/// A passage inside a fence is code: `new` goes in as sent, never read as prose.
pub(in crate::mcp) fn edit_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let body =
        text(args, "doc").and_then(|which| tisty_core::docs::read(&paths.docs(), &which).ok());
    if body.is_some_and(|body| in_a_fence(&body, args)) {
        return edited(paths, args, true);
    }
    spoken(args, "new", |args| edited(paths, args, false))
}

/// The passage sits wholly in one fence's code, and `new` brings no fence mark that would shut it.
fn in_a_fence(body: &str, args: &Value) -> bool {
    let mut fence = tisty_core::docs::fencing();
    if raw(args, "new").is_some_and(|new| new.lines().any(&mut fence)) {
        return false;
    }
    let line = |key| {
        args.get(key)
            .and_then(Value::as_u64)
            .map(|one| one as usize)
    };
    let (from, to) = match (raw(args, "old"), line("from"), line("to")) {
        (Some(old), None, None) => {
            let old = old.replace('\r', "");
            let mut found = body.match_indices(old.as_str()).map(|(at, _)| at);
            let (Some(start), None) = (found.next(), found.next()) else {
                return false;
            };
            let from = body[..start].matches('\n').count() + 1;
            (
                from,
                from + old.trim_end_matches('\n').matches('\n').count(),
            )
        }
        (None, Some(from), Some(to)) => (from, to),
        _ => return false,
    };
    tisty_core::docs::fenced_between(body, from, to)
}

fn edited(paths: &Paths, args: &Value, as_sent: bool) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool("editing needs the `doc` name.".into()));
    };
    let by_place = args.get("section").is_some() || args.get("from").is_some();
    let Some(new) = raw(args, "new") else {
        return Err(Refused::Tool(
            "an edit needs `new`, what goes there. Send \"\" to take the text out.".into(),
        ));
    };
    let old =
        match (raw(args, "old"), by_place) {
            (Some(old), false) => old,
            (Some(_), true) => return Err(Refused::Tool(
                "name the passage with `old`, or name where it is with `section` or with `from` \
                 and `to` — not both, since the two could point at different places."
                    .into(),
            )),
            (None, true) => "",
            (None, false) => {
                return Err(Refused::Tool(
                    "an edit needs `old`, the text to replace — or `section`, or `from` and `to`, \
                 which name a passage by where it sits. `outline_doc` gives both numbers."
                        .into(),
                ));
            }
        };
    let (state, mut store, named) = writable(paths, args, &which)?;
    let new = match as_sent {
        true => new.to_string(),
        false => tisty_core::speaking::turned(new),
    };
    let new = new.as_str();
    if !as_sent {
        tisty_core::docs::speaks(new)
            .map_err(|unsaid| Refused::Tool(super::writing::not_spoken(unsaid)))?;
    }

    if by_place {
        return in_its_place(paths, &state, &mut store, &which, args, new);
    }

    let (old, new) = (&old.replace('\r', ""), &new.replace('\r', ""));
    let before = tisty_core::docs::read(&paths.docs(), &which).unwrap_or_default();
    // The window may have written between the look at the fence and this read.
    if as_sent && !in_a_fence(&before, args) {
        return Err(Refused::Tool(format!(
            "{named} changed while this edit was on its way, so nothing was written. Read the \
             passage again and send the edit anew."
        )));
    }
    if before.matches(old.as_str()).count() == 1 {
        super::writing::still_speaks(&before, &before.replacen(old.as_str(), new, 1))?;
    }
    let made = tisty_core::docs::edit(&paths.docs(), paths.data(), &which, old, new).map_err(
        |e| match e {
            tisty_core::Error::DocumentTooBig { limit, .. } => Refused::Tool(format!(
                "that would take the document past the {limit} bytes Tisty can open."
            )),
            other => hitch(other),
        },
    )?;

    match made {
        tisty_core::docs::Change::Missing => Err(Refused::Tool({
            let body = tisty_core::docs::read(&paths.docs(), &which).unwrap_or_default();
            match nearest(&body, old) {
                Some((line, near)) => format!(
                    "nothing in {named} reads exactly like that `old`, so nothing was changed. \
                     The nearest it has is line {line}:\n\n{near}\n\nCopy it character for \
                     character as `read_doc` hands it back, or name the place with `section` \
                     or `from` and `to`."
                ),
                None => format!(
                    "nothing in {named} reads like that `old`, so nothing was changed. Ask \
                     `find` with this `doc` where the words are, or `read_doc` for the part you \
                     mean."
                ),
            }
        })),
        tisty_core::docs::Change::TheLot => Err(Refused::Tool(format!(
            "that `old` is the whole of {named}, which `edit_doc` will not take: a passage it \
             cannot tell from the document is a rewrite wearing an edit's clothes. Nothing was \
             changed. Edit the passage that differs, or replace the body with `write_doc`, naming \
             {named} and the `print` `read_doc` gave you."
        ))),
        tisty_core::docs::Change::Twice(many) => Err(Refused::Tool({
            let body = tisty_core::docs::read(&paths.docs(), &which).unwrap_or_default();
            let at = where_over(&body, old);
            format!(
                "that `old` fits {many} places in {named}, and Tisty will not choose for you, \
                 so nothing was changed. It starts on lines {}. Send more of the lines around \
                 the one you mean, or name it with `from` and `to`.",
                at.iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })),
        tisty_core::docs::Change::Made { was, whole } => {
            let loose = left_loose(&state, &which, &was, &whole);
            let twice = named_twice(&state, &which, &whole);
            let settled = retold(&state, &mut store, &which, &whole).is_ok();
            let anew = super::named_anew(paths, &state, &mut store, &which, &whole);
            Ok(told(
                format!(
                    "Changed that passage in {:?}, {}. What it was is kept beside the \
                     documents.{}{}{}{}",
                    tisty_core::docs::titled(&whole),
                    by_how_much(&was, &whole),
                    loose_words(&state, &loose),
                    twice_words(&state, &twice) + &anew,
                    if settled { "" } else { UNSETTLED },
                    wrapped(new)
                ),
                with_echo(
                    json!({
                        "for_the_person": for_the_person(&state, &which, &tisty_core::docs::titled(&whole)),
                        "doc": which,
                        "title": tisty_core::docs::titled(&whole),
                        "chars": whole.chars().count(),
                        "lines": whole.lines().count(),
                        "grew": grew(&was, &whole),
                        "loose": loose,
                        "print": tisty_core::attach::printed(whole.as_bytes()),
                    }),
                    args,
                    &was,
                    &whole,
                ),
            ))
        }
    }
}
