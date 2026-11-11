use serde_json::{Value, json};
use tisty_core::Paths;

use super::super::asked::text;
use super::super::jsonrpc::told;
use super::super::mistaken::not_a_document;
use super::super::{
    LISTED_AT_MOST, Refused, folder_found, gist_of, named_doc, opened, placing, said, scoped,
    shortened, trail,
};

pub(in crate::mcp) fn docs(paths: &Paths, args: &Value) -> Result<Value, Refused> {
    let (state, _) = opened(paths)?;
    let scope = scoped(args)?;
    let within = match text(args, "folder") {
        Some(said) => Some(folder_found(&state, &said)?),
        None => match args.get("folder").is_some_and(|one| !one.is_null()) {
            true => {
                return Err(Refused::Tool(
                    "`folder` names one folder to list. Leave it out to list them all.".into(),
                ));
            }
            false => None,
        },
    };
    if args.get("page_of").is_some() && text(args, "page_of").is_none() {
        return Err(Refused::Tool(
            "`page_of` names one document whose pages to list. Leave it out to list them all."
                .into(),
        ));
    }
    let under = match text(args, "page_of") {
        Some(said) => Some(
            state
                .docs
                .values()
                .find(|one| one.file == said)
                .map(|one| one.id)
                .ok_or_else(|| {
                    Refused::Tool(not_a_document(&state, &said).unwrap_or_else(|| {
                        format!(
                            "no document here is called {said:?}, so nothing hangs from it. \
                             `docs` lists them all."
                        )
                    }))
                })?,
        ),
        None => None,
    };
    let most = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .clamp(1, LISTED_AT_MOST as u64) as usize;
    let past = args.get("after").and_then(Value::as_u64).unwrap_or(0) as usize;

    let reading = under.is_some();
    let deep = args.get("deep").and_then(Value::as_bool).unwrap_or(false);
    let mut kept: Vec<&tisty_core::model::Kept> = state
        .papers()
        .filter(|one| state.doc_kept_by(scope, one))
        .filter(|one| within.is_none_or(|at| placing::listed_in(&state, one, at, deep)))
        .filter(|one| under.is_none_or(|up| one.page_of == Some(up)))
        .collect();
    match reading {
        // Pages answer in the order they are read, which is the order they are named in.
        true => kept.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id))),
        // What moved last, not what was made last: an agent coming back asks what has changed.
        false => kept.sort_by_key(|one| std::cmp::Reverse((one.wrote, one.id))),
    }

    let all = kept.len();
    let shown_of: Vec<&tisty_core::model::Kept> =
        kept.iter().skip(past).take(most).copied().collect();
    let held = tisty_core::cache::Cache::open(paths.cache()).ok().flatten();
    // Listing them is the one moment that already knows which files are really there.
    tisty_core::docs::forget_stray_cards(&paths.docs(), held.as_ref());
    let cards = tisty_core::docs::cards_of(
        &paths.docs(),
        held.as_ref(),
        &shown_of
            .iter()
            .map(|one| one.file.clone())
            .collect::<Vec<_>>(),
    );

    let spoken = super::tongues::spoken(&state);
    let shown: Vec<Value> = shown_of
        .iter()
        .map(|one| {
            let card = cards.get(&one.file);
            let mut kept_of = serde_json::Map::new();
            kept_of.insert("doc".into(), json!(one.file));
            kept_of.insert(
                "title".into(),
                json!(card.map(|one| one.title.clone()).unwrap_or_default()),
            );
            if let Some(at) = one.folder {
                kept_of.insert("folder".into(), json!(trail(&state, at)));
            }
            if let Some(up) = one.page_of.and_then(|up| named_doc(&state, up)) {
                kept_of.insert("page_of".into(), json!(up));
            }
            let tongues = spoken.get(&one.id).map_or(&[][..], Vec::as_slice);
            super::tongues::said_into(one, tongues, &mut kept_of);
            let pages = state.pages_of(one.id);
            if !pages.is_empty() {
                kept_of.insert("pages".into(), json!(pages.len()));
                let away = pages.iter().filter(|page| page.archived).count();
                if away > 0 {
                    kept_of.insert("pages_archived".into(), json!(away));
                }
                let marked = pages
                    .iter()
                    .filter(|page| page.flagged.is_some() && !state.held_away(page))
                    .count();
                if marked > 0 {
                    kept_of.insert("pages_flagged".into(), json!(marked));
                }
            }
            if state.held_away(one) {
                kept_of.insert("archived".into(), json!(true));
            }
            if one.archived {
                kept_of.insert("apart".into(), json!(true));
            }
            if state.shut(one.id) {
                kept_of.insert("locked".into(), json!(true));
            }
            if one.flagged.is_some() && !state.held_away(one) {
                kept_of.insert("flagged".into(), json!(true));
            }
            if let Some(card) = card {
                kept_of.insert("words".into(), json!(card.words));
                if !card.outline.is_empty() {
                    kept_of.insert("sections".into(), json!(card.outline.len()));
                }
                if !card.keywords.is_empty() {
                    kept_of.insert("about".into(), json!(card.keywords));
                }
                if card.pictures > 0 {
                    kept_of.insert("pictures".into(), json!(card.pictures));
                }
            }
            if let Some(at) = one.wrote {
                kept_of.insert("wrote".into(), json!(at.to_string()));
            }
            if let Some(card) = card
                && let Some(said) = gist_of(held.as_ref(), &one.file, &card.print)
            {
                kept_of.insert("gist".into(), json!(shortened(said)));
            }
            Value::Object(kept_of)
        })
        .collect();

    let mut folders: Vec<Value> = state
        .folders
        .values()
        .map(|one| {
            json!({
                "folder": one.name,
                "id": one.id.to_string(),
                "path": trail(&state, one.id),
                "icon": one.icon,
                "docs": state
                    .docs
                    .values()
                    .filter(|kept| kept.folder == Some(one.id) && kept.page_of.is_none())
                    .count(),
            })
        })
        .collect();
    folders.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let below = placing::below(&state, scope, within.filter(|_| !deep && under.is_none()));

    let mut lines: Vec<String> = shown
        .iter()
        .map(|one| {
            let where_at = match one["page_of"].as_str() {
                Some(up) => format!("page of {up}"),
                None => one["folder"].as_str().unwrap_or("no folder").to_string(),
            };
            let holds = match one["pages"].as_u64().unwrap_or(0) {
                0 => String::new(),
                1 => ", 1 page".into(),
                many => format!(", {many} pages"),
            };
            let put_away = if one["archived"] == json!(true) {
                ", put away"
            } else {
                ""
            };
            let shut = if one["locked"] == json!(true) {
                ", locked"
            } else {
                ""
            };
            let marked = if one["flagged"] == json!(true) {
                ", marked as one that has had its day"
            } else {
                ""
            };
            format!(
                "{} — {} ({where_at}{holds}{put_away}{shut}{marked})",
                said(one, "doc"),
                said(one, "title")
            )
        })
        .collect();
    lines.extend(placing::told_below(&below));
    lines.push(match folders.is_empty() {
        true => "No folders here yet.".into(),
        false => format!(
            "Folders: {}.",
            folders
                .iter()
                .filter_map(|one| one["path"].as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    });

    let mut kept = json!({ "docs": shown, "total": all });
    if !below.is_empty() {
        kept["below"] = json!(below);
    }
    if args
        .get("folders")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        kept["folders"] = json!(folders);
    }

    Ok(told(
        format!(
            "{all} document(s){}; showing {}.\n{}",
            placing::counted(args, within.is_none() && under.is_none()),
            shown.len(),
            lines.join("\n")
        ),
        kept,
    ))
}
