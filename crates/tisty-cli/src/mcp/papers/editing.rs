use serde_json::{Value, json};

use super::writing::{in_its_place, where_over};

use tisty_core::Paths;

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::{
    Refused, UNSETTLED, by_how_much, grew, hitch, left_loose, loose_words, named_twice, nearest,
    opened, raw, retold, twice_words, with_echo, wrapped,
};

pub(in crate::mcp) fn edit_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
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
    let (state, mut store) = opened(paths)?;
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    if state.held_away(kept) {
        return Err(Refused::Tool(format!(
            "{which:?} is put away, so it is not edited any more."
        )));
    }
    if state.shut(kept.id) {
        return Err(Refused::Tool(format!(
            "{which:?} is locked. The person shut it so nothing writes in it — not the \
             window, not you. Ask them to unlock it if it truly has to change."
        )));
    }
    tisty_core::docs::survives(new).map_err(|eats| {
        Refused::Tool(format!(
            "Tisty's editor cannot keep {eats}, and would destroy it the first time the person \
             opens the document. Send plain markdown: headings, lists, emphasis, inline links, tables (aligned columns and all), fenced code with its language, and GitHub alerts written as a quote that opens with [!NOTE], [!TIP], [!IMPORTANT], [!WARNING] or [!CAUTION]. Four bits of HTML are kept as well, because the editor writes them itself and reads them back whole: <u>, <mark>, <mark data-pen=\"green\"> and its other colours, and the icon span. Any other tag is refused. Maths goes in a fence saying `math`, never between dollars: `$$` is not markdown, so the editor keeps it as words and escapes what looks like markup inside it. A fence carries its language and, if you want, one name: ```rust title=\"src/walk.rs\", and the same for `mermaid` and `math`, which the window draws with that name above them. Nothing else after the language: a second word is dropped when the person opens the document, so it is refused here instead."
        ))
    })?;

    if by_place {
        return in_its_place(paths, &state, &mut store, &which, args, new);
    }

    let (old, new) = (&old.replace('\r', ""), &new.replace('\r', ""));
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
                    "nothing in {which:?} reads exactly like that `old`, so nothing was changed. \
                     The nearest it has is line {line}:\n\n{near}\n\nCopy it character for \
                     character as `read_doc` hands it back, or name the place with `section` \
                     or `from` and `to`."
                ),
                None => format!(
                    "nothing in {which:?} reads like that `old`, so nothing was changed. Ask \
                     `find` with this `doc` where the words are, or `read_doc` for the part you \
                     mean."
                ),
            }
        })),
        tisty_core::docs::Change::TheLot => Err(Refused::Tool(format!(
            "that `old` is the whole of {which:?}, which `edit_doc` will not take: a passage it \
             cannot tell from the document is a rewrite wearing an edit's clothes. Nothing was \
             changed. Edit the passage that differs, or replace the body with `write_doc`, naming \
             {which:?} and the `print` `read_doc` gave you."
        ))),
        tisty_core::docs::Change::Twice(many) => Err(Refused::Tool({
            let body = tisty_core::docs::read(&paths.docs(), &which).unwrap_or_default();
            let at = where_over(&body, old);
            format!(
                "that `old` fits {many} places in {which:?}, and Tisty will not choose for you, \
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
            Ok(told(
                format!(
                    "Changed that passage in {:?}, {}. What it was is kept beside the \
                     documents.{}{}{}{}",
                    tisty_core::docs::titled(&whole),
                    by_how_much(&was, &whole),
                    loose_words(&state, &loose),
                    twice_words(&state, &twice),
                    if settled { "" } else { UNSETTLED },
                    wrapped(new)
                ),
                with_echo(
                    json!({
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
