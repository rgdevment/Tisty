use serde_json::{Value, json};

use super::reading::{beside_ready, in_this_order, left_named, named_at_end, placed, where_said};

use super::{many_docs, said_docs};
use tisty_core::{Op, Paths};

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::{Held, Refused, Spot, doc_named, hitch, named_all, named_doc, opened, up_named};

pub(in crate::mcp) fn page_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    if args.get("order").is_some_and(|one| !one.is_null()) {
        return in_this_order(paths, args);
    }
    let (many, listed) = many_docs(args, "hang or unhang")?;
    let (state, mut store) = opened(paths)?;

    for side in ["after", "before", "at"] {
        if args.get(side).is_some() && text(args, side).is_none() {
            return Err(Refused::Tool(format!(
                "`{side}` came empty. A page goes where another page is named, or first or \
                 last, and an empty name says neither — leave it out to hang the page without \
                 writing its line."
            )));
        }
    }
    let named: Vec<(&str, Option<String>)> = ["after", "before", "at"]
        .into_iter()
        .map(|key| (key, text(args, key)))
        .filter(|(_, said)| said.is_some())
        .collect();
    if named.len() > 1 {
        return Err(Refused::Tool(format!(
            "send one of `after`, `before` and `at`, not {}: a page goes in one place.",
            named
                .iter()
                .map(|(key, _)| format!("`{key}`"))
                .collect::<Vec<_>>()
                .join(" and ")
        )));
    }
    let side = match text(args, "at").as_deref() {
        None => None,
        Some("first") => Some(Spot::First),
        Some("last") => Some(Spot::Last),
        Some(other) => {
            return Err(Refused::Tool(format!(
                "`at` is either \"first\" or \"last\", and {other:?} is neither. Name a page with \
                 `after` or `before` to put it anywhere else."
            )));
        }
    };
    let beside = match (text(args, "after"), text(args, "before")) {
        (Some(one), None) => Some(Held::After(one)),
        (None, Some(one)) => Some(Held::Before(one)),
        _ => side.map(Held::At),
    };
    if beside.is_some() {
        if many.len() != 1 {
            return Err(Refused::Tool(
                "`after`, `before` and `at` place one page, so `doc` takes a single name \
                 here. Hang them together first, then place them one at a time."
                    .into(),
            ));
        }
        if text(args, "page_of").is_none() {
            return Err(Refused::Tool(
                "`after`, `before` and `at` say where a page sits inside the document that \
                 holds it, so they need `page_of`. Left out, the page leaves that document \
                 altogether and there is no order to give it."
                    .into(),
            ));
        }
    }

    let up = match text(args, "page_of") {
        None => None,
        Some(said) => {
            let Some(up) = state.docs.values().find(|one| one.file == said) else {
                return Err(Refused::Tool(format!(
                    "no document here is called {said:?}. `docs` lists them all."
                )));
            };
            if up.page_of.is_some() {
                return Err(Refused::Tool(format!(
                    "{said} is a page itself, and a page holds no pages. Name the document it \
                     belongs to."
                )));
            }
            if state.held_away(up) {
                return Err(Refused::Tool(format!(
                    "{} is put away, and a page of it is put away with it. Leave {} where it is.",
                    doc_named(&state, &said),
                    named_all(&state, &many)
                )));
            }
            if state.shut(up.id) {
                return Err(Refused::Tool(format!(
                    "{said} is locked, and hanging a page off it writes the line that names \
                     it. Ask the person to unlock it first."
                )));
            }
            Some(up.id)
        }
    };
    if let (Some(held), Some(over)) = (&beside, up) {
        beside_ready(paths, &state, over, &many[0], held.spot())?;
    }

    let mut moving = Vec::new();
    let mut already = Vec::new();
    for which in &many {
        let Some(kept) = state.docs.values().find(|one| one.file == *which) else {
            return Err(Refused::Tool(format!(
                "no document here is called {which:?}. `docs` lists them all."
            )));
        };
        if state.shut(kept.id) {
            return Err(Refused::Tool(format!(
                "{which} is locked. Where a locked document sits is part of what the person shut \
                 away, so it neither becomes a page nor leaves the one that holds it. Ask them to \
                 unlock it first."
            )));
        }
        if up.is_none() && state.held_by_another(kept) {
            return Err(Refused::Tool(format!(
                "{} is in the archive with the folder that holds it, and taking it out of its \
                 document would leave it outside the archive with nobody's hand on it. The person \
                 brings the folder back from the window first.",
                doc_named(&state, which)
            )));
        }
        if let Some(over) = up {
            if over == kept.id {
                return Err(Refused::Tool(format!(
                    "{which} cannot be a page of itself."
                )));
            }
            if state.docs.values().any(|one| one.page_of == Some(kept.id)) {
                return Err(Refused::Tool(format!(
                    "{which} has pages of its own, so it cannot become a page. Move its pages \
                     first."
                )));
            }
            if state.held_away(kept) {
                return Err(Refused::Tool(format!(
                    "{which} is put away. Bring it back before making it a page, or it leaves \
                     the archive with no way of returning."
                )));
            }
        }
        match kept.page_of == up {
            true => already.push(which.clone()),
            false => moving.push((which.clone(), kept.id)),
        }
    }

    let under = up.and_then(|one| named_doc(&state, one));
    if let (Some(held), Some(over), true) = (&beside, up, moving.is_empty()) {
        placed(paths, &state, &mut store, over, &many[0], held.spot())?;
        return Ok(told(
            format!(
                "Moved {} {}.",
                named_all(&state, &many),
                where_said(&state, held.spot())
            ),
            json!({ "doc": said_docs(&many, listed), "page_of": under, "left": already }),
        ));
    }
    if moving.is_empty() {
        return Ok(told(
            match (up.and_then(|one| up_named(&state, one)), already.len() == 1) {
                (Some(named), true) => format!(
                    "{} was already a page of {named}.",
                    named_all(&state, &already)
                ),
                (Some(named), false) => format!(
                    "{} were already pages of {named}.",
                    named_all(&state, &already)
                ),
                (None, true) => format!(
                    "{} was already a document of its own.",
                    named_all(&state, &already)
                ),
                (None, false) => format!(
                    "{} were already documents of their own.",
                    named_all(&state, &already)
                ),
            },
            json!({ "doc": said_docs(&many, listed), "page_of": under, "left": already }),
        ));
    }

    let events = match up {
        Some(_) => Vec::new(),
        None => store.read_all().map_err(hitch)?,
    };
    let mut last: std::collections::BTreeMap<Option<tisty_core::model::FolderId>, String> =
        Default::default();
    let doing: Vec<Op> = moving
        .iter()
        .map(|(_, id)| Op::DocMove {
            id: *id,
            d: match up {
                Some(_) => tisty_core::event::Filed {
                    folder: None,
                    page_of: Some(up),
                    order: None,
                },
                None => {
                    let mut out = tisty_core::undo::unhung(&events, &state, *id);
                    let home = out.folder.unwrap_or_default();
                    if let Some(order) = out.order.as_ref() {
                        let order = match last.get(&home) {
                            Some(before) if before >= order => tisty_core::order::after(before),
                            _ => order.clone(),
                        };
                        last.insert(home, order.clone());
                        out.order = Some(order);
                    }
                    out
                }
            },
        })
        .collect();
    let stayed = left_named(paths, &state, &moving, up);
    store.append_batch(doing).map_err(hitch)?;
    let mut put = String::new();
    if let (None, Some(over)) = (&beside, up) {
        let (now, mut store) = opened(paths)?;
        let hung: Vec<String> = moving.iter().map(|(which, _)| which.clone()).collect();
        put = match named_at_end(paths, &now, &mut store, over, &hung) {
            Ok((given, walled_off)) => match (given.is_empty(), walled_off) {
                (false, _) => format!(
                    " Wrote {} at the end, which is where it is read.",
                    match given.len() {
                        1 => "its line".to_string(),
                        many => format!("the {many} lines naming them"),
                    }
                ),
                (true, true) => format!(
                    " {} ends inside a fence, so no line was written for it there: a line in \
                     code is not a way in. Close the fence and name it with `after`, `before` or \
                     `at`.",
                    up_named(&state, over).unwrap_or_default()
                ),
                (true, false) => String::new(),
            },
            Err(Refused::Tool(why)) => format!(
                " No line was written for {}, so {} loose: {why}",
                match hung.len() {
                    1 => "it",
                    _ => "them",
                },
                match hung.len() {
                    1 => "it is",
                    _ => "they are",
                }
            ),
            Err(other) => return Err(other),
        };
    }
    if let (Some(held), Some(over)) = (&beside, up) {
        let (now, mut store) = opened(paths)?;
        put = match placed(paths, &now, &mut store, over, &many[0], held.spot()) {
            Ok(()) => format!(" Its line sits {}.", where_said(&state, held.spot())),
            Err(Refused::Tool(why)) => {
                format!(" It is a page now, but no line was written for it, so it is loose: {why}")
            }
            Err(other) => return Err(other),
        };
    }

    let over = match stayed.is_empty() {
        true => String::new(),
        false => format!(
            " The line naming {} is still written in {}, which no longer holds it: take it out \
             with `edit_doc` when you are done.",
            named_all(
                &state,
                &stayed
                    .iter()
                    .map(|(_, one)| one.clone())
                    .collect::<Vec<_>>()
            ),
            named_all(
                &state,
                &stayed.iter().map(|(up, _)| up.clone()).collect::<Vec<_>>()
            )
        ),
    };
    let put = format!("{put}{over}");

    let names: Vec<String> = moving.iter().map(|(which, _)| which.clone()).collect();
    let one_of_them = names.len() == 1;
    let over = match (already.is_empty(), already.len() == 1, up.is_some()) {
        (true, _, _) => String::new(),
        (false, true, true) => format!(" {} was already one.", named_all(&state, &already)),
        (false, false, true) => {
            format!(" {} were already pages of it.", named_all(&state, &already))
        }
        (false, true, false) => {
            format!(
                " {} was already a document of its own.",
                named_all(&state, &already)
            )
        }
        (false, false, false) => format!(
            " {} were already documents of their own.",
            named_all(&state, &already)
        ),
    };
    Ok(told(
        match (up.and_then(|one| up_named(&state, one)), one_of_them) {
            (Some(named), true) => format!(
                "{} is now a page of {named}.{over}{put}",
                named_all(&state, &names)
            ),
            (Some(named), false) => format!(
                "{} are now pages of {named}, in that order.{over}{put}",
                named_all(&state, &names)
            ),
            (None, true) => format!(
                "{} is now a document of its own.{over}",
                named_all(&state, &names)
            ),
            (None, false) => format!(
                "{} are now documents of their own.{over}",
                named_all(&state, &names)
            ),
        },
        json!({ "doc": said_docs(&names, listed), "page_of": under, "left": already }),
    ))
}
