use serde_json::{Value, json};

use super::{pointed_at, reachable_or};
use tisty_core::{Paths, State, Store};

use super::super::asked::strings;
use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::{
    NEWEST_SHOWN, Part, Refused, Spot, card_alone, card_moved, carried_off, doc_named, gist_of,
    hitch, line_of, named_all, named_doc, opened, part_asked, retold, said, shortened, trail,
    up_named, when,
};

pub(in crate::mcp) fn export_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "exporting needs the `doc` to take out.".into(),
        ));
    };
    let Some(said) = text(args, "into") else {
        return Err(Refused::Tool(
            "exporting needs an `into` folder on this machine to leave the files in.".into(),
        ));
    };
    let (state, _) = opened(paths)?;
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };

    let asked = std::path::Path::new(&said);
    let into = reachable_or(paths, &said, asked, "leave files")?;
    if !into.is_dir() {
        return Err(Refused::Tool(format!(
            "{said:?} is not a folder. Name one that exists, and the files are left inside it."
        )));
    }

    let body = tisty_core::docs::read(&paths.docs(), &which).unwrap_or_default();
    let pages: Vec<String> = state
        .pages_read(kept.id, &body)
        .iter()
        .map(|one| one.file.clone())
        .collect();
    let beside = match tisty_core::Config::load_or_init(paths).map_err(hitch)?.sync {
        Some(tisty_core::config::Sync::Folder(at)) => Some(at),
        _ => None,
    };
    let taken =
        tisty_core::docs::with_pages(paths.data(), &which, &pages, &into, beside.as_deref())
            .map_err(hitch)?;

    Ok(told(
        format!(
            "Took {} out to {} — its cover, {} page(s) and {} file(s) beside them{}{}. Nothing here changed: an export is a copy.",
            doc_named(&state, &which),
            into.display(),
            pages.len(),
            taken.files,
            match taken.missed {
                0 => String::new(),
                many => format!(", and {many} page(s) could not be read, so they are not there"),
            },
            match taken.left.len() {
                0 => String::new(),
                many => format!(
                    ", and {many} file(s) it points at are not in the store, so they did not come along: {}",
                    taken.left.join(", ")
                ),
            }
        ),
        json!({
            "doc": which,
            "into": into.display().to_string(),
            "pages_out": pages.len(),
            "files": taken.files,
            "missed": taken.missed,
            "left_behind": taken.left,
            "pages": pages,
        }),
    ))
}

pub(in crate::mcp) fn outline_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "looking into a document needs its `doc` name.".into(),
        ));
    };
    let (state, _) = opened(paths)?;
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    let held = tisty_core::cache::Cache::open(paths.cache()).ok().flatten();
    let Some(card) = tisty_core::docs::card_of(&paths.docs(), held.as_ref(), &which) else {
        return Err(Refused::Tool(format!(
            "{which:?} is named in the log but its text is not on this machine yet. It may still \
             be arriving from another one."
        )));
    };
    let whole = tisty_core::docs::read(&paths.docs(), &which).unwrap_or_default();
    let pages = state.pages_read(kept.id, &whole);
    let told_of = tisty_core::refs::papers(&whole);
    let names: Vec<String> = pages.iter().map(|one| one.file.clone()).collect();
    let cards = tisty_core::docs::cards_of(&paths.docs(), held.as_ref(), &names);
    let rows: Vec<Value> = pages
        .iter()
        .map(|one| {
            let mut row = serde_json::Map::new();
            row.insert("doc".into(), json!(one.file));
            if let Some(card) = cards.get(&one.file) {
                row.insert("title".into(), json!(card.title));
                row.insert("words".into(), json!(card.words));
                if !card.outline.is_empty() {
                    row.insert("sections".into(), json!(card.outline.len()));
                }
                if !card.keywords.is_empty() {
                    row.insert("about".into(), json!(card.keywords));
                }
                if let Some(said) = gist_of(held.as_ref(), &one.file, &card.print) {
                    row.insert("gist".into(), shortened(said));
                }
            }
            if state.held_away(one) {
                row.insert("archived".into(), json!(true));
            }
            if one.archived {
                row.insert("apart".into(), json!(true));
            }
            if one.flagged.is_some() && !state.held_away(one) {
                row.insert("flagged".into(), json!(true));
            }
            if !told_of.contains(&one.file) {
                row.insert("loose".into(), json!(true));
            }
            Value::Object(row)
        })
        .collect();

    let pointing = pointed_at(paths, &state, &which);
    let mut kept_of = serde_json::Map::new();
    kept_of.insert("doc".into(), json!(which));
    kept_of.insert("title".into(), json!(card.title));
    kept_of.insert("chars".into(), json!(card.chars));
    kept_of.insert("lines".into(), json!(card.lines));
    kept_of.insert("words".into(), json!(card.words));
    kept_of.insert("print".into(), json!(card.print));
    kept_of.insert("outline".into(), json!(card.outline));
    if !pointing.is_empty() {
        kept_of.insert("pointed_at".into(), json!(pointing));
    }
    if !card.keywords.is_empty() {
        kept_of.insert("about".into(), json!(card.keywords));
    }
    if card.pictures > 0 {
        kept_of.insert("pictures".into(), json!(card.pictures));
    }
    if card.links > 0 {
        kept_of.insert("links".into(), json!(card.links));
    }
    if let Some(said) = gist_of(held.as_ref(), &which, &card.print) {
        kept_of.insert("gist".into(), json!(said));
    }
    if let Some(up) = kept.page_of.and_then(|up| named_doc(&state, up)) {
        kept_of.insert("page_of".into(), json!(up));
    }
    if !rows.is_empty() {
        kept_of.insert("pages".into(), json!(rows));
    }
    if state.held_away(kept) {
        kept_of.insert("archived".into(), json!(true));
    }
    if state.shut(kept.id) {
        kept_of.insert("locked".into(), json!(true));
    }

    let mut shown = card
        .outline
        .iter()
        .map(|one| {
            format!(
                "{}{}  ·  lines {}-{}, {} chars",
                "  ".repeat(one.level - 1),
                one.title,
                one.line,
                one.to,
                one.chars
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    if shown.is_empty() {
        shown = format!(
            "{:?} holds no headings — {} words in all.",
            card.title, card.words
        );
    }
    if !rows.is_empty() {
        let away = rows
            .iter()
            .filter(|row| row["apart"] == json!(true))
            .count();
        let marked = rows
            .iter()
            .filter(|row| row["flagged"] == json!(true))
            .count();
        let adrift = rows
            .iter()
            .filter(|row| row["loose"] == json!(true))
            .count();
        shown.push_str(&format!(
            "\n\nPages, in the order they are read ({}{}{}{}):",
            rows.len(),
            match away {
                0 => String::new(),
                _ => format!(", {away} of them put away on their own"),
            },
            match marked {
                0 => String::new(),
                _ => format!(", {marked} an agent gave up for old"),
            },
            match adrift {
                0 => String::new(),
                _ => format!(", {adrift} no line in the document names"),
            }
        ));
        for row in &rows {
            let holds = match (row["words"].as_u64(), row["sections"].as_u64()) {
                (Some(words), Some(sections)) => format!(" ({words} words, {sections} sections)"),
                (Some(words), None) => format!(" ({words} words)"),
                _ => String::new(),
            };
            let state_of = match (
                row["archived"] == json!(true),
                row["apart"] == json!(true),
                row["flagged"] == json!(true),
            ) {
                (true, true, _) => " — in the archive on its own, read-only",
                (true, false, _) => " — in the archive with the document, read-only",
                (false, _, true) => " — an agent says it has had its day",
                (false, _, false) => "",
            };
            let adrift = match row["loose"] == json!(true) {
                true => " — loose: no line names it, so the window gives it no place",
                false => "",
            };
            shown.push_str(&format!(
                "\n  {} — {}{holds}{state_of}{adrift}",
                said(row, "doc"),
                said(row, "title")
            ));
        }
    }
    if !pointing.is_empty() {
        let few: Vec<String> = pointing.iter().take(NEWEST_SHOWN).cloned().collect();
        shown.push_str(&format!(
            "\n\nPointing at it: {}{}. Putting it away leaves those pointing into the archive.",
            named_all(&state, &few),
            match pointing.len() > few.len() {
                true => format!(" and {} more", pointing.len() - few.len()),
                false => String::new(),
            }
        ));
    }
    Ok(told(shown, Value::Object(kept_of)))
}

pub(in crate::mcp) fn read_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "reading a document needs its `doc` name.".into(),
        ));
    };
    let (state, _) = opened(paths)?;
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    let folder = kept.folder.map(|at| trail(&state, at));
    let body = tisty_core::docs::read(&paths.docs(), &which).map_err(hitch)?;
    let pages: Vec<String> = state
        .pages_read(kept.id, &body)
        .iter()
        .map(|one| one.file.clone())
        .collect();
    let away = state.held_away(kept);

    let mut kept_of = serde_json::Map::new();
    kept_of.insert("doc".into(), json!(which));
    kept_of.insert("title".into(), json!(tisty_core::docs::titled(&body)));
    kept_of.insert("chars".into(), json!(body.chars().count()));
    kept_of.insert("lines".into(), json!(body.lines().count()));
    kept_of.insert(
        "print".into(),
        json!(tisty_core::attach::printed(body.as_bytes())),
    );
    if let Some(folder) = folder {
        kept_of.insert("folder".into(), json!(folder));
    }
    if let Some(up) = kept.page_of.and_then(|up| named_doc(&state, up)) {
        kept_of.insert("page_of".into(), json!(up));
    }
    if !pages.is_empty() {
        kept_of.insert("pages".into(), json!(pages));
        let held = state.pages_read(kept.id, &body);
        let told_of = tisty_core::refs::papers(&body);
        let adrift: Vec<String> = held
            .iter()
            .filter(|one| !told_of.contains(&one.file))
            .map(|one| one.file.clone())
            .collect();
        if !adrift.is_empty() {
            kept_of.insert("pages_loose".into(), json!(adrift));
        }
        let shelved: Vec<String> = held
            .iter()
            .filter(|one| one.archived)
            .map(|one| one.file.clone())
            .collect();
        if !shelved.is_empty() {
            kept_of.insert("pages_archived".into(), json!(shelved));
        }
        let marked: Vec<String> = held
            .iter()
            .filter(|one| one.flagged.is_some() && !state.held_away(one))
            .map(|one| one.file.clone())
            .collect();
        if !marked.is_empty() {
            kept_of.insert("pages_flagged".into(), json!(marked));
        }
    }
    if away {
        kept_of.insert("archived".into(), json!(true));
    }
    if state.shut(kept.id) {
        kept_of.insert("locked".into(), json!(true));
    }
    if let Some(mark) = &kept.flagged {
        kept_of.insert(
            "flagged".into(),
            json!({ "at": mark.at.to_string(), "said": mark.body }),
        );
    }

    match part_asked(&body, args)? {
        Part::Outline => {
            kept_of.insert("whole".into(), json!(false));
            kept_of.insert("outline".into(), json!(outline_of(&body)));
            Ok(told(
                format!(
                    "{}\n\nThis one holds {} characters, so here is what is in it rather than the \
                     whole of it. Ask for a part with `section`, with `from` and `to`, or with \
                     `chars`. To change a passage, `edit_doc` takes the words to replace and the \
                     ones to put there: reading it whole to send it back whole costs many times \
                     more, and risks writing over what the person did meanwhile.",
                    tisty_core::docs::titled(&body),
                    body.chars().count()
                ),
                Value::Object(kept_of),
            ))
        }
        Part::Held {
            body: part,
            from,
            to,
            next,
        } => {
            kept_of.insert("body".into(), json!(part));
            kept_of.insert("from".into(), json!(from));
            kept_of.insert("to".into(), json!(to));
            if let Some(next) = next {
                kept_of.insert("next".into(), json!(next));
                kept_of.insert("whole".into(), json!(false));
            }
            let said = match (away, kept.flagged.as_ref()) {
                (true, _) => format!("({})\n\n{part}", away_words(&state, kept)),
                (false, Some(mark)) => format!(
                    "(An assistant marked this as one that has had its day on {}: {})\n\n{part}",
                    when(mark.at),
                    mark.body
                ),
                (false, None) => part,
            };
            Ok(told(said, Value::Object(kept_of)))
        }
    }
}

pub(super) fn left_named(
    paths: &Paths,
    state: &State,
    moving: &[(String, tisty_core::model::DocId)],
    up: Option<tisty_core::model::DocId>,
) -> Vec<(String, String)> {
    let up = match up {
        Some(one) => one,
        None => return Vec::new(),
    };
    moving
        .iter()
        .filter_map(|(which, id)| {
            let was = state.docs.get(id)?.page_of?;
            if was == up {
                return None;
            }
            let parent = state.docs.get(&was)?;
            let body = tisty_core::docs::read(&paths.docs(), &parent.file).ok()?;
            tisty_core::refs::papers(&body)
                .contains(which)
                .then(|| (parent.file.clone(), which.clone()))
        })
        .collect()
}

pub(super) fn in_this_order(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let order = strings(args, "order")?;
    if let Some(also) = ["doc", "after", "before", "at"]
        .into_iter()
        .find(|key| args.get(*key).is_some_and(|one| !one.is_null()))
    {
        return Err(Refused::Tool(format!(
            "`order` says where every page it names goes, so it takes no `{also}`: one call \
             says an order, another puts one page somewhere. Send them apart."
        )));
    }
    if order.len() < 2 {
        return Err(Refused::Tool(format!(
            "`order` is the pages in the order they are to be read, and {} names no order at \
             all. Two or more, or nothing to do.",
            match order.len() {
                0 => "an empty list",
                _ => "one page",
            }
        )));
    }
    let mut seen = std::collections::BTreeSet::new();
    if let Some(twice) = order.iter().find(|one| !seen.insert((*one).clone())) {
        return Err(Refused::Tool(format!(
            "`order` names {twice:?} twice, and a page is read in one place. Send it once."
        )));
    }
    let Some(said) = text(args, "page_of") else {
        return Err(Refused::Tool(
            "`order` says how the pages of one document are read, so it needs `page_of`.".into(),
        ));
    };
    let (state, mut store) = opened(paths)?;
    let Some(up) = state.docs.values().find(|one| one.file == said) else {
        return Err(Refused::Tool(format!(
            "no document here is called {said:?}. Ids are opaque, like q7ntmzbm-0001, and a title \
             is not one — `docs` prints the id beside the title of every document."
        )));
    };
    if state.shut(up.id) {
        return Err(Refused::Tool(format!(
            "{said} is locked, and ordering its pages writes in it. Ask the person to unlock it \
             first."
        )));
    }
    if state.held_away(up) {
        return Err(Refused::Tool(format!(
            "{} is put away, so it is not written in any more. Bring it back with \
             `archive_doc` first if its order truly has to change.",
            doc_named(&state, &said)
        )));
    }
    let over = up.id;
    ordered(paths, &state, &mut store, over, &order)?;
    Ok(told(
        format!(
            "Read in that order now, in {}: {}. What it said before is kept beside the \
             documents.",
            doc_named(&state, &said),
            named_all(&state, &order)
        ),
        json!({ "doc": order, "page_of": said }),
    ))
}

/// Hanging says where a page belongs, and a page nothing names has no place to be read in, so the
/// lines of the ones that have none are written together rather than one write each.
pub(super) fn named_at_end(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    up: tisty_core::model::DocId,
    which: &[String],
) -> Result<(Vec<String>, bool), Refused> {
    let Some(parent) = state.docs.get(&up) else {
        return Ok((Vec::new(), false));
    };
    let held: Vec<&str> = which.iter().map(String::as_str).collect();
    match tisty_core::docs::name_at_end(&paths.docs(), paths.data(), &parent.file, &held)
        .map_err(hitch)?
    {
        // Hanging gives the page a key of its own, and where it is read comes from the text. A
        // book that already names it has nothing written in it, so nothing would bring the two
        // back together unless the settling runs anyway.
        tisty_core::docs::Naming::Nothing => {
            if let Ok(body) = tisty_core::docs::read(&paths.docs(), &parent.file) {
                retold(state, store, &parent.file, &body)?;
            }
            Ok((Vec::new(), false))
        }
        tisty_core::docs::Naming::Fenced => Ok((Vec::new(), true)),
        tisty_core::docs::Naming::WouldRename => Err(Refused::Tool(format!(
            "a line at the end of {} would become the first thing it says, and a document takes \
             its title from that, so it would be renamed. Write something above it first.",
            doc_named(state, &parent.file)
        ))),
        tisty_core::docs::Naming::Wrote { named, whole } => {
            retold(state, store, &parent.file, &whole)?;
            Ok((named, false))
        }
    }
}

pub(super) fn where_said(state: &State, spot: Spot) -> String {
    match spot.anchor() {
        Some(one) => format!("{} {}", spot.said(), doc_named(state, one)),
        None => spot.said().to_string(),
    }
}

pub(super) fn beside_ready(
    paths: &Paths,
    state: &State,
    up: tisty_core::model::DocId,
    which: &str,
    spot: Spot,
) -> Result<(), Refused> {
    let Some(parent) = state.docs.get(&up) else {
        return Err(Refused::Tool(
            "the document that holds this page is not here any more.".into(),
        ));
    };
    let body = tisty_core::docs::read(&paths.docs(), &parent.file).map_err(hitch)?;
    let held: Vec<String> = body.lines().map(str::to_string).collect();
    if let Some(at) = line_of(&held, which)
        && !card_alone(&held[at], which)
    {
        return Err(carried_off(state, &parent.file, which, at, &held[at]));
    }

    let Some(anchor) = spot.anchor() else {
        return Ok(());
    };

    let Some(mark) = state.docs.values().find(|one| one.file == anchor) else {
        return Err(Refused::Tool(format!(
            "no document here is called {anchor:?}. Ids are opaque, like q7ntmzbm-0001, and a \
             title is not one — `docs` prints the id beside the title of every document."
        )));
    };
    if mark.page_of != Some(up) {
        return Err(Refused::Tool(format!(
            "{} is not a page of {}, so it says nothing about where this one goes. \
             `outline_doc` lists the pages in the order they are read, and says which of them \
             no line names.",
            doc_named(state, anchor),
            doc_named(state, &parent.file)
        )));
    }
    if mark.file == which {
        return Err(Refused::Tool(
            "a page cannot be placed before or after itself. Name another page of the same \
             document, or send `at` as \"first\" or \"last\"."
                .into(),
        ));
    }
    let Some(sits) = line_of(&held, anchor) else {
        return Err(Refused::Tool(format!(
            "{} is a page of {}, but no line in it names {}, so there is nothing to place this \
             one beside. Send `at` as \"first\" or \"last\" instead, and the line is written \
             without an anchor.",
            doc_named(state, anchor),
            doc_named(state, &parent.file),
            doc_named(state, anchor)
        )));
    };
    if !card_alone(&held[sits], anchor) {
        return Err(Refused::Tool(format!(
            "line {} of {} names {} in the middle of something else: {:?}. A page is placed \
             beside a line that names one page and nothing more, so that writing beside it \
             cannot break what that line is part of. Name a page whose line stands on its own.",
            sits + 1,
            doc_named(state, &parent.file),
            doc_named(state, anchor),
            held[sits].trim()
        )));
    }
    let _ = sits;
    Ok(())
}

fn away_words(state: &State, kept: &tisty_core::model::Kept) -> String {
    match (
        kept.archived,
        kept.page_of.and_then(|up| up_named(state, up)),
    ) {
        (true, Some(up)) => format!("This page is put away on its own, inside {up}"),
        (true, None) => "This document is put away".to_string(),
        (false, Some(up)) => format!("This page is in the archive with {up}, which holds it"),
        (false, None) => {
            "This document is in the archive with the folder that holds it".to_string()
        }
    }
}

fn outline_of(body: &str) -> Vec<Value> {
    tisty_core::docs::outlined(body)
        .iter()
        .map(|one| json!(one))
        .collect()
}

pub(super) fn placed(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    up: tisty_core::model::DocId,
    which: &str,
    spot: Spot,
) -> Result<(), Refused> {
    let Some(parent) = state.docs.get(&up) else {
        return Err(Refused::Tool(
            "the document that holds this page is not here any more.".into(),
        ));
    };
    let body = tisty_core::docs::read(&paths.docs(), &parent.file).map_err(hitch)?;
    let print = tisty_core::attach::printed(body.as_bytes());
    let title = tisty_core::docs::read(&paths.docs(), which)
        .map(|one| tisty_core::docs::titled(&one))
        .unwrap_or_default();
    if let Some(said) = card_moved(&body, which, &title, spot)
        && tisty_core::docs::titled(&said) != tisty_core::docs::titled(&body)
    {
        return Err(Refused::Tool(format!(
            "putting it there would make its line the first thing {} says, and a document takes \
             its title from what it says first, so {} would be renamed. Put this page after \
             another one instead.",
            doc_named(state, &parent.file),
            doc_named(state, &parent.file)
        )));
    }
    let Some(whole) = card_moved(&body, which, &title, spot) else {
        return Err(Refused::Tool(format!(
            "no line of {} names {} any more, so there was nowhere to put this one.",
            doc_named(state, &parent.file),
            doc_named(state, spot.anchor().unwrap_or_default())
        )));
    };
    match tisty_core::docs::rewrite(&paths.docs(), paths.data(), &parent.file, &whole, &print)
        .map_err(hitch)?
    {
        tisty_core::docs::Rewrite::Moved => Err(Refused::Tool(format!(
            "{} was written by somebody else in the same moment, so its line was left where it \
             was. Read it again and say where the page goes.",
            doc_named(state, &parent.file)
        ))),
        tisty_core::docs::Rewrite::Made { whole, .. } => {
            retold(state, store, &parent.file, &whole)?;
            Ok(())
        }
    }
}

fn ordered(
    paths: &Paths,
    state: &State,
    store: &mut Store,
    up: tisty_core::model::DocId,
    order: &[String],
) -> Result<(), Refused> {
    let Some(parent) = state.docs.get(&up) else {
        return Err(Refused::Tool(
            "the document that holds these pages is not here any more.".into(),
        ));
    };
    let body = tisty_core::docs::read(&paths.docs(), &parent.file).map_err(hitch)?;
    let held: Vec<String> = body.lines().map(str::to_string).collect();
    for id in order {
        let Some(page) = state.docs.values().find(|one| one.file == *id) else {
            return Err(Refused::Tool(format!(
                "no document here is called {id:?}. Ids are opaque, like q7ntmzbm-0001, and a \
                 title is not one — `docs` prints the id beside the title of every document."
            )));
        };
        if page.page_of != Some(up) {
            return Err(Refused::Tool(format!(
                "{} is not a page of {}, so it has no place in its order. Hang it there \
                 first with `page_of`, then say the order.",
                doc_named(state, id),
                doc_named(state, &parent.file)
            )));
        }
        let Some(at) = line_of(&held, id) else {
            return Err(Refused::Tool(format!(
                "no line of {} names {}, so there is no place of its own to move it between. \
                 Give it one with `after`, `before` or `at`, and then order them.",
                doc_named(state, &parent.file),
                doc_named(state, id)
            )));
        };
        if !card_alone(&held[at], id) {
            return Err(carried_off(state, &parent.file, id, at, &held[at]));
        }
    }
    let print = tisty_core::attach::printed(body.as_bytes());
    let Some(whole) = cards_ordered(&body, order) else {
        return Err(Refused::Tool(
            "the lines naming those pages are not all there to move between. Ask \
             `outline_doc` which pages this document names, and order those."
                .to_string(),
        ));
    };
    if renamed(&body, &whole) {
        return Err(Refused::Tool(format!(
            "that order would make another page's line the first thing {} says, and a document \
             takes its title from that, so it would be renamed. Write something above them \
             first.",
            doc_named(state, &parent.file)
        )));
    }
    match tisty_core::docs::rewrite(&paths.docs(), paths.data(), &parent.file, &whole, &print)
        .map_err(hitch)?
    {
        tisty_core::docs::Rewrite::Moved => Err(Refused::Tool(format!(
            "{} was written by somebody else in the same moment, so nothing was reordered. Read \
             it again and say the order.",
            doc_named(state, &parent.file)
        ))),
        tisty_core::docs::Rewrite::Made { whole, .. } => {
            retold(state, store, &parent.file, &whole)?;
            Ok(())
        }
    }
}

fn renamed(was: &str, now: &str) -> bool {
    tisty_core::docs::titled(was) != tisty_core::docs::titled(now)
}

fn cards_ordered(body: &str, order: &[String]) -> Option<String> {
    let ending = match body.contains("\r\n") {
        true => "\r\n",
        false => "\n",
    };
    let mut lines: Vec<String> = body.lines().map(str::to_string).collect();
    let mut slots: Vec<usize> = Vec::with_capacity(order.len());
    for id in order {
        let at = line_of(&lines, id)?;
        if !card_alone(&lines[at], id) {
            return None;
        }
        slots.push(at);
    }
    let held: Vec<String> = slots.iter().map(|at| lines[*at].clone()).collect();
    let mut places = slots;
    places.sort_unstable();
    for (place, line) in places.iter().zip(&held) {
        lines[*place] = line.clone();
    }
    let mut out = lines.join(ending);
    if !out.ends_with('\n') {
        out.push_str(ending);
    }
    Some(out)
}
