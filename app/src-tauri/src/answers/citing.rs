use std::sync::Mutex;

use tisty_core::brain::Citing;
use tisty_core::model::DocId;
use tisty_markdown::Kind;

use crate::{Answer, Refusal, Session, elsewhere, held};

const GLANCE_AT_MOST: usize = 1200;

#[derive(serde::Serialize)]
pub struct Heading {
    id: String,
    title: String,
    level: u8,
}

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
pub struct Glance {
    title: String,
    heading: Option<String>,
    html: String,
    more: bool,
}

#[tauri::command]
pub async fn doc_cited_by(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Vec<Citing>>> {
    let paths = held(&session).paths.clone();
    elsewhere(move || tisty_core::brain::cited_by(&paths, &id)).await
}

#[tauri::command(async)]
pub fn doc_heads(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<Heading>> {
    let body = body_of(&session, id)?;
    Ok(tisty_markdown::sections(&tisty_markdown::read(&body))
        .into_iter()
        .map(|head| Heading {
            id: head.id,
            title: head.said,
            level: head.level,
        })
        .collect())
}

#[tauri::command(async)]
pub fn doc_glance(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    at: Option<String>,
) -> Answer<Glance> {
    let body = body_of(&session, id)?;
    Ok(glanced(&body, at.as_deref()))
}

fn body_of(session: &tauri::State<'_, Mutex<Session>>, id: String) -> Answer<String> {
    let (root, file) = {
        let session = held(session);
        let by_ulid = id.parse::<DocId>().ok();
        let file = by_ulid
            .and_then(|one| session.state.docs.get(&one))
            .map_or_else(|| id.clone(), |kept| kept.file.clone());
        (session.paths.docs(), file)
    };
    tisty_core::docs::read(&root, &file).map_err(|_| Refusal::about("noSuchDoc", id))
}

// A heading nobody wrote any more opens on the document's first words, as following the link does.
fn glanced(body: &str, at: Option<&str>) -> Glance {
    let doc = tisty_markdown::read(body);
    let text = doc.text();
    let heads = tisty_markdown::outline(&doc);
    let chosen = at.and_then(|at| heads.iter().position(|head| head.id == at));
    let (from, to, heading) = match chosen {
        Some(n) => {
            let head = &heads[n];
            let to = heads[n + 1..]
                .iter()
                .find(|next| next.level <= head.level)
                .map_or(text.len(), |next| next.at.start);
            (head.at.end, to, Some(head.said.clone()))
        }
        None => (opening(&doc), text.len(), None),
    };
    let said = text.get(from..to).unwrap_or_default().trim();
    let (said, more) = capped(said);
    Glance {
        title: tisty_core::docs::titled(body),
        heading,
        html: tisty_markdown::html(&tisty_markdown::read(said)),
        more,
    }
}

fn opening(doc: &tisty_markdown::Doc) -> usize {
    let mut blocks = doc
        .blocks()
        .iter()
        .skip_while(|block| block.kind == Kind::Front);
    match blocks.next() {
        Some(first) if first.kind == Kind::Heading(1) => first.at.end,
        Some(first) => first.at.start,
        None => doc.text().len(),
    }
}

fn capped(said: &str) -> (&str, bool) {
    if said.len() <= GLANCE_AT_MOST {
        return (said, false);
    }
    let mut end = GLANCE_AT_MOST;
    while !said.is_char_boundary(end) {
        end -= 1;
    }
    let cut = said[..end]
        .rfind("\n\n")
        .filter(|at| *at > GLANCE_AT_MOST / 3)
        .unwrap_or(end);
    (said[..cut].trim_end(), true)
}

#[cfg(test)]
#[path = "citing_test.rs"]
mod tests;
