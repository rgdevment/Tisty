use serde_json::{Value, json};
use tisty_core::model::{DocId, Kept};
use tisty_core::{Paths, State};

use super::{Refused, TOOLS_STAY_FRESH, admitted, doc_titled, outline_doc, read_doc, trail};

const SCHEME: &str = "tisty:doc/";
const LISTED_AT_ONCE: usize = 50;
const MARKDOWN: &str = "text/markdown";
const NOT_FOUND: i32 = -32002;
const BAD_PARAMS: i32 = -32602;
const BROKE: i32 = -32603;
const LIST_STAYS_FRESH: i64 = 60_000;

pub(super) fn listed(paths: &Paths, params: &Value) -> Result<Value, (i32, String)> {
    let after = match params.get("cursor").and_then(Value::as_str) {
        None => None,
        Some(said) => Some(said.parse::<DocId>().map_err(|_| {
            (
                BAD_PARAMS,
                format!("{said:?} is not a cursor this server handed out."),
            )
        })?),
    };
    let Some((state, _)) = admitted(paths).map_err(refused)? else {
        return Ok(page(Vec::new(), None));
    };
    let mut kept: Vec<&Kept> = state
        .papers()
        .filter(|one| !state.held_away(one))
        .filter(|one| after.is_none_or(|after| one.id < after))
        .collect();
    kept.sort_by_key(|one| std::cmp::Reverse(one.id));
    let shown: Vec<&Kept> = kept.iter().take(LISTED_AT_ONCE).copied().collect();
    let held = tisty_core::cache::Cache::open(paths.cache()).ok().flatten();
    let files: Vec<String> = shown.iter().map(|one| one.file.clone()).collect();
    let cards = tisty_core::docs::cards_of(&paths.docs(), held.as_ref(), &files);
    let resources = shown
        .iter()
        .map(|one| {
            let title = cards
                .get(&one.file)
                .map(|card| card.title.trim())
                .filter(|said| !said.is_empty())
                .or_else(|| {
                    one.title
                        .as_deref()
                        .map(str::trim)
                        .filter(|said| !said.is_empty())
                })
                .unwrap_or("Untitled document")
                .to_string();
            let mut listed = json!({
                "uri": format!("{SCHEME}{}", one.file),
                "name": &title,
                "mimeType": MARKDOWN,
            });
            if let Some(where_at) = sits(&state, one) {
                listed["description"] = json!(format!("{title} — {where_at}"));
            }
            listed
        })
        .collect();
    let next = shown
        .last()
        .filter(|_| kept.len() > shown.len())
        .map(|last| last.id.to_string());
    Ok(page(resources, next))
}

fn page(resources: Vec<Value>, next: Option<String>) -> Value {
    let mut listed = json!({
        "resultType": "complete",
        "resources": resources,
        "ttlMs": LIST_STAYS_FRESH,
        "cacheScope": "private",
    });
    if let Some(next) = next {
        listed["nextCursor"] = json!(next);
    }
    listed
}

fn sits(state: &State, kept: &Kept) -> Option<String> {
    match (kept.page_of.and_then(|up| state.docs.get(&up)), kept.folder) {
        (Some(up), _) => Some(format!("a page of {}", doc_titled(state, &up.file))),
        (None, Some(at)) => Some(format!("in {}", trail(state, at))),
        (None, None) => None,
    }
}

pub(super) fn templates() -> Value {
    json!({
        "resultType": "complete",
        "resourceTemplates": [{
            "uriTemplate": format!("{SCHEME}{{doc}}{{?from,to}}"),
            "name": "document",
            "title": "A document, or some of its lines",
            "description": "A document by its id. A long one comes as its outline; `from` and \
                            `to` read the lines between.",
            "mimeType": MARKDOWN,
        }],
        "ttlMs": TOOLS_STAY_FRESH,
        "cacheScope": "public",
    })
}

struct Address {
    doc: String,
    from: Option<u64>,
    to: Option<u64>,
}

pub(super) fn read(paths: &Paths, params: &Value) -> Result<Value, (i32, String)> {
    let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
    let asked = addressed(uri)?;
    let mut args = json!({ "doc": asked.doc });
    if let Some(from) = asked.from {
        args["from"] = json!(from);
    }
    if let Some(to) = asked.to {
        args["to"] = json!(to);
    }
    let said = read_doc(paths, &args).map_err(refused)?;
    let held = &said["structuredContent"];
    let (text, mime) = match held.get("body") {
        Some(_) => (with_the_rest(&said, &asked), MARKDOWN),
        None => (outlined(paths, &asked, held)?, "text/plain"),
    };
    Ok(json!({
        "resultType": "complete",
        "contents": [{ "uri": uri, "mimeType": mime, "text": text }],
    }))
}

fn addressed(uri: &str) -> Result<Address, (i32, String)> {
    let nowhere = || {
        (
            NOT_FOUND,
            format!("nothing here is at {uri:?}: a document is at {SCHEME} and its id."),
        )
    };
    let rest = uri.strip_prefix(SCHEME).ok_or_else(nowhere)?;
    let (doc, query) = rest.split_once('?').unwrap_or((rest, ""));
    if doc.is_empty() || doc.contains(['/', '#']) {
        return Err(nowhere());
    }
    let mut asked = Address {
        doc: doc.to_string(),
        from: None,
        to: None,
    };
    for pair in query.split('&').filter(|one| !one.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let line = value.parse::<u64>().ok().filter(|one| *one > 0);
        let line = || {
            line.ok_or_else(|| {
                (
                    BAD_PARAMS,
                    format!("`{key}` is a line number, from 1, not {value:?}."),
                )
            })
        };
        match key {
            "from" => asked.from = Some(line()?),
            "to" => asked.to = Some(line()?),
            _ => {
                return Err((
                    BAD_PARAMS,
                    format!("a document's address takes `from` and `to`, not `{key}`."),
                ));
            }
        }
    }
    Ok(asked)
}

fn with_the_rest(said: &Value, asked: &Address) -> String {
    let text = said["content"][0]["text"].as_str().unwrap_or_default();
    let held = &said["structuredContent"];
    match (held["next"].as_u64(), held["to"].as_u64()) {
        (Some(next), Some(to)) => {
            let end = asked.to.or_else(|| held["lines"].as_u64()).unwrap_or(next);
            format!(
                "{text}\n\nThis part stops at line {to}; it goes on at \
                 {SCHEME}{}?from={next}&to={end}.",
                asked.doc
            )
        }
        _ => text.to_string(),
    }
}

fn outlined(paths: &Paths, asked: &Address, held: &Value) -> Result<String, (i32, String)> {
    let outline = outline_doc(paths, &json!({ "doc": asked.doc })).map_err(refused)?;
    let shown = outline["content"][0]["text"].as_str().unwrap_or_default();
    let notice = match held["notice"].as_str() {
        Some(notice) => format!("({notice})\n\n"),
        None => String::new(),
    };
    Ok(format!(
        "{notice}{shown}\n\nIt holds {} characters, so this is its outline rather than the whole \
         of it. Read the lines under a heading at {SCHEME}{}?from=FIRST&to=LAST, or with \
         `read_doc`.",
        held["chars"].as_u64().unwrap_or_default(),
        asked.doc
    ))
}

fn refused(why: Refused) -> (i32, String) {
    match why {
        Refused::Protocol(code, why) => (code, why),
        Refused::Missing(why) => (NOT_FOUND, why),
        Refused::Unable(why) => (BROKE, why),
        Refused::Tool(why) => (BAD_PARAMS, why),
    }
}
