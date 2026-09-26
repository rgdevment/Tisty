use serde_json::{Value, json};

use super::{FOLDERS_AT_MOST, many_docs, pointed_at, said_docs};
use tisty_core::{Op, Paths};
use ulid::Ulid;

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::{
    FOLDER_NAME_AT_MOST, Refused, as_path, doc_named, folder_named, hitch, named_all, named_doc,
    opened, speaking_through, trail, up_named, when,
};

pub(in crate::mcp) fn archive_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "putting a document away needs its `doc` name.".into(),
        ));
    };
    let away = match args.get("archived") {
        None | Some(Value::Null) => true,
        Some(Value::Bool(one)) => *one,
        Some(other) => {
            return Err(Refused::Tool(format!(
                "`archived` is true or false, and {other} is neither. Leave it out to put the document away."
            )));
        }
    };
    let (state, mut store) = opened(paths)?;
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    if state.held_by_another(kept) {
        let holds = match kept.folder.is_some_and(|at| state.folder_away(at)) {
            true => "the folder that holds it".to_string(),
            false => match kept.page_of.and_then(|up| up_named(&state, up)) {
                Some(up) => format!("the document that holds it, {up}"),
                None => "the folder that holds it".to_string(),
            },
        };
        return Err(Refused::Tool(match away {
            true => format!(
                "{} is in the archive already, held there by {holds}. A mark of its own would \
                 outlive that and keep it away after {holds} comes back, which is not what \
                 putting something away means here.",
                doc_named(&state, &which)
            ),
            false => format!(
                "{} is in the archive with {holds}, so it does not come back on its own and \
                 nothing here takes it out. The person brings back what holds it, from the \
                 window.",
                doc_named(&state, &which)
            ),
        }));
    }
    let folder = match text(args, "folder") {
        None => None,
        Some(said) => match kept.page_of {
            Some(_) => {
                return Err(Refused::Tool(format!(
                    "{which} is a page, and a page is kept where its document is, so there is no \
                     folder to put it in."
                )));
            }
            None => Some(folder_named(&state, &said)?),
        },
    };
    let filing = folder.filter(|at| kept.folder != Some(*at));
    if kept.archived == away && filing.is_none() {
        return Ok(told(
            match away {
                true => format!("{} was already put away.", doc_named(&state, &which)),
                false => format!(
                    "{} was already out of the archive.",
                    doc_named(&state, &which)
                ),
            },
            json!({ "doc": which, "archived": away }),
        ));
    }
    let pages: Vec<String> = match kept.archived == away {
        true => Vec::new(),
        false => state
            .pages_of(kept.id)
            .iter()
            .filter(|one| !one.archived)
            .map(|one| one.file.clone())
            .collect(),
    };
    let pointing = match away {
        true => {
            let up = kept.page_of.and_then(|up| named_doc(&state, up));
            pointed_at(paths, &state, &which)
                .into_iter()
                .filter(|one| Some(one) != up.as_ref())
                .collect()
        }
        false => Vec::new(),
    };
    let mut doing = Vec::new();
    if let Some(at) = filing {
        doing.push(Op::DocMove {
            id: kept.id,
            d: tisty_core::event::Filed {
                page_of: None,
                folder: Some(Some(at)),
                order: None,
            },
        });
    }
    if kept.archived != away {
        doing.push(match away {
            true => Op::DocArchive { id: kept.id },
            false => Op::DocUnarchive { id: kept.id },
        });
    }
    store.append_batch(doing).map_err(hitch)?;

    Ok(told(
        format!(
            "{}{}{}",
            match (kept.archived == away, away) {
                (true, true) => format!("{} was already put away.", doc_named(&state, &which)),
                (true, false) => format!(
                    "{} was already out of the archive.",
                    doc_named(&state, &which)
                ),
                (false, true) => format!(
                    "Put {} away. It is not gone: `docs` and `find` still reach it with `scope`, and this same call with `archived` false brings it back.",
                    doc_named(&state, &which)
                ),
                (false, false) => format!(
                    "Brought {} back out of the archive.{}",
                    doc_named(&state, &which),
                    match (
                        kept.page_of.is_some(),
                        kept.folder.map(|at| trail(&state, at))
                    ) {
                        (true, _) => String::new(),
                        (false, Some(named)) => format!(" It sits in {named}."),
                        (false, None) => " It sits in no folder.".to_string(),
                    }
                ),
            },
            match filing.map(|at| trail(&state, at)) {
                Some(named) => format!(" Filed in {named}."),
                None => String::new(),
            },
            match pages.is_empty() {
                true => String::new(),
                false => match away {
                    true => format!(" Its pages went with it: {}.", named_all(&state, &pages)),
                    false => format!(
                        " Its pages came back with it: {}.",
                        named_all(&state, &pages)
                    ),
                },
            }
        ) + &match pointing.is_empty() {
            true => String::new(),
            false => format!(
                " Still pointing at it, and now pointing into the archive: {}.",
                pointing.join(", ")
            ),
        },
        json!({
            "doc": which,
            "archived": away,
            "pages": pages,
            "pointed_at": pointing,
        }),
    ))
}

pub(in crate::mcp) fn flag_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(which) = text(args, "doc") else {
        return Err(Refused::Tool(
            "marking a document needs its `doc` name.".into(),
        ));
    };
    let Some(body) = text(args, "body") else {
        return Err(Refused::Tool(
            "marking a document needs a `body`: what makes it old and how you know. Without it \
             the person has only your word and nothing to weigh it against."
                .into(),
        ));
    };
    let (state, mut store) = opened(paths)?;
    let Some(kept) = state.docs.values().find(|one| one.file == which) else {
        return Err(Refused::Tool(format!(
            "no document here is called {which:?}. `docs` lists them all."
        )));
    };
    if state.held_away(kept) {
        return Err(Refused::Tool(format!(
            "{} is already in the archive, so it is out of the way. Nothing to mark.",
            doc_named(&state, &which)
        )));
    }
    if let Some(already) = &kept.flagged {
        let who = match already.by == *store.device() && already.via == speaking_through() {
            true => "you".to_string(),
            false => already
                .via
                .as_deref()
                .map(tisty_core::agent::client_named)
                .unwrap_or_else(|| "an assistant".to_string()),
        };
        return Err(Refused::Tool(format!(
            "{who} already marked {} on {}, and the person has not looked yet. Marking \
             it again would only say the same thing twice.",
            doc_named(&state, &which),
            when(already.at)
        )));
    }
    let id = kept.id;
    let me = store.device().clone();
    store
        .append(Op::DocFlag {
            id,
            d: tisty_core::event::Flag::new(body)
                .said_by(jiff::Timestamp::now(), me)
                .through(speaking_through()),
        })
        .map_err(hitch)?;

    Ok(told(
        format!(
            "Marked {} as one that has had its day. It is untouched and still reads the \
             same: the person sees the mark when they open it, and archiving it, deleting \
             it or taking the mark off are all theirs.",
            doc_named(&state, &which)
        ),
        json!({ "doc": which, "flagged": true }),
    ))
}

pub(in crate::mcp) fn file_doc(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (many, listed) = many_docs(args, "file")?;
    let (state, mut store) = opened(paths)?;
    let folder = match text(args, "folder") {
        Some(said) => Some(folder_named(&state, &said)?),
        None => None,
    };

    let mut moving = Vec::new();
    let mut already = Vec::new();
    for which in &many {
        let Some(kept) = state.docs.values().find(|one| one.file == *which) else {
            return Err(Refused::Tool(format!(
                "no document here is called {which:?}. `docs` lists them all."
            )));
        };
        if let Some(up) = kept.page_of.and_then(|up| up_named(&state, up)) {
            return Err(Refused::Tool(format!(
                "{} is a page of {up}, and a page is kept where its document is. `page_doc` \
                 takes it out as a document of its own first.",
                doc_named(&state, which)
            )));
        }
        if state.held_by_another(kept) {
            return Err(Refused::Tool(format!(
                "{} is in the archive with the folder that holds it, and moving it out from \
                 under that folder would take it out of the archive with nobody's hand on it. \
                 The person brings the folder back from the window first.",
                doc_named(&state, which)
            )));
        }
        match kept.folder == folder {
            true => already.push(which.clone()),
            false => moving.push((which.clone(), kept.id, kept.archived)),
        }
    }

    let where_at = folder.map(|at| trail(&state, at));
    if moving.is_empty() {
        let away_now = already.iter().any(|one| {
            state
                .docs
                .values()
                .any(|kept| kept.file == *one && kept.archived)
        });
        return Ok(told(
            match (&where_at, already.len() == 1) {
                (Some(named), true) => {
                    format!("{} was already in {named}.", named_all(&state, &already))
                }
                (Some(named), false) => {
                    format!("{} were already in {named}.", named_all(&state, &already))
                }
                (None, true) => {
                    format!("{} was already in no folder.", named_all(&state, &already))
                }
                (None, false) => {
                    format!("{} were already in no folder.", named_all(&state, &already))
                }
            },
            json!({
                "doc": said_docs(&many, listed),
                "folder": where_at,
                "archived": away_now,
                "left": already,
            }),
        ));
    }

    store
        .append_batch(
            moving
                .iter()
                .map(|(_, id, _)| Op::DocMove {
                    id: *id,
                    d: tisty_core::event::Filed {
                        page_of: None,
                        folder: Some(folder),
                        order: None,
                    },
                })
                .collect(),
        )
        .map_err(hitch)?;

    let names: Vec<String> = moving.iter().map(|(which, _, _)| which.clone()).collect();
    let still = match moving.iter().any(|(_, _, away)| *away) {
        true => {
            " What the archive holds stays there: filing it says where it belongs, not that it is back."
        }
        false => "",
    };
    let over = match (already.is_empty(), already.len() == 1) {
        (true, _) => String::new(),
        (false, true) => format!(" {} was already there.", named_all(&state, &already)),
        (false, false) => format!(" {} were already there.", named_all(&state, &already)),
    };
    Ok(told(
        match &where_at {
            Some(named) => format!(
                "Filed {} in {named}.{over}{still}",
                named_all(&state, &names)
            ),
            None => format!(
                "Took {} out of every folder.{over}{still}",
                named_all(&state, &names)
            ),
        },
        json!({
            "doc": said_docs(&names, listed),
            "folder": where_at,
            "archived": moving.iter().any(|(_, _, away)| *away),
            "left": already,
        }),
    ))
}

pub(in crate::mcp) fn folder(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let Some(said) = text(args, "name") else {
        return Err(Refused::Tool("a folder needs a `name`.".into()));
    };
    let steps: Vec<&str> = said
        .split('/')
        .map(str::trim)
        .filter(|one| !one.is_empty())
        .collect();
    let under = match steps.len() > 1 {
        true => Some(steps[..steps.len() - 1].join(" / ")),
        false => None,
    };
    let Some(said) = steps.last().map(|one| (*one).to_string()) else {
        return Err(Refused::Tool(
            "a folder needs a `name` with something in it.".into(),
        ));
    };
    let name = tisty_core::text::plainly(&said);
    if name.chars().count() > FOLDER_NAME_AT_MOST {
        return Err(Refused::Tool(format!(
            "a folder name is at most {FOLDER_NAME_AT_MOST} characters, and {name:?} is longer. \
             It has to fit a rail on screen: name it in a word or two."
        )));
    }
    if name.is_empty() {
        return Err(Refused::Tool("a folder needs a `name`.".into()));
    }
    let icon = match text(args, "icon") {
        Some(key) => Some(tisty_core::model::icon::kept(&key).ok_or_else(|| {
            Refused::Tool(format!(
                "there is no icon called {key:?}. Send a single emoji, or one name from the drawn \
                         catalogue — home, work, money, study, travel, health, food, \
                         shopping, family, code, folder, archive are some of them. Leave \
                         `icon` out if none fits."
            ))
        })?),
        None => None,
    };
    let color = match text(args, "color") {
        Some(key) => Some(
            tisty_core::model::hue::kept(&key)
                .map(str::to_string)
                .ok_or_else(|| {
                    Refused::Tool(format!(
                        "there is no colour called {key:?}. The palette is {}.",
                        tisty_core::model::hue::HUES.join(", ")
                    ))
                })?,
        ),
        None => None,
    };
    let (state, mut store) = opened(paths)?;

    let parent = match (text(args, "inside"), &under) {
        (Some(inside), Some(path)) if as_path(&inside) != as_path(path) => {
            return Err(Refused::Tool(format!(
                "`name` reads as a path under {path:?} and `inside` says {inside:?}, which \
                 are two different places. Send the name alone with `inside`, or the whole path \
                 without it."
            )));
        }
        (Some(inside), _) => Some(folder_named(&state, &inside)?),
        (None, Some(path)) => Some(folder_named(&state, path).map_err(|why| match why {
            Refused::Tool(said) => Refused::Tool(format!(
                "a folder's name is its own and never a path, so {path:?} was read as the folder \
                 to nest it in, and {said} Make each step of the path first, and nest with \
                 `inside`."
            )),
            other => other,
        })?),
        (None, None) => None,
    };
    if let Some(at) = parent
        && state.depth(Some(at)) >= tisty_core::model::DEEPEST
    {
        return Err(Refused::Tool(format!(
            "folders only nest {} deep here. Make it beside that one instead.",
            tisty_core::model::DEEPEST
        )));
    }

    let wanted = tisty_core::text::folded(&name);
    if let Some(one) = state.folders.values().find(|one| {
        tisty_core::text::folded(&one.name) == wanted && (parent.is_none() || one.parent == parent)
    }) {
        if state.folder_away(one.id) && (icon.is_some() || color.is_some()) {
            return Err(Refused::Tool(format!(
                "{:?} is in the archive, and what it looks like is not changed while it is there.",
                one.name
            )));
        }
        if icon.is_some() || color.is_some() {
            store
                .append(Op::FolderLook {
                    id: one.id,
                    d: tisty_core::event::Look {
                        icon: icon.map(Some),
                        color: color.map(Some),
                    },
                })
                .map_err(hitch)?;
        }
        return Ok(told(
            format!(
                "{:?} already exists, at {}.",
                one.name,
                trail(&state, one.id)
            ),
            json!({ "folder": one.name, "id": one.id.to_string(), "made": false }),
        ));
    }
    if state.folders.len() >= FOLDERS_AT_MOST {
        return Err(Refused::Tool(format!(
            "there are already {FOLDERS_AT_MOST} folders here. File the document in one of them \
             instead of making another."
        )));
    }
    let id = Ulid::generate();
    let order =
        tisty_core::order::last_of(state.under(parent).iter().map(|one| one.order.as_str()));
    store
        .append(Op::FolderAdd {
            id,
            d: tisty_core::event::FolderAdd {
                name: name.clone(),
                order,
                parent,
                icon,
                color,
            },
        })
        .map_err(hitch)?;

    Ok(told(
        match parent.map(|at| trail(&state, at)) {
            Some(under) => format!("Made the folder {name:?} inside {under}."),
            None => format!("Made the folder {name:?}."),
        },
        json!({ "folder": name, "id": id.to_string(), "made": true }),
    ))
}
