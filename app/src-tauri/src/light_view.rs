use std::path::Path;

use tauri::{AppHandle, Manager};

use crate::viewing::{Shown, Views, named, place_of, shown_in};
use crate::{Answer, Refusal};

const PIECE_ABOUT: usize = 64 * 1024;

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Heading {
    level: u8,
    said: String,
    id: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Reading {
    Drawn {
        name: String,
        place: String,
        path: String,
        first: String,
        more: bool,
        heads: Vec<Heading>,
    },
    Missed {
        name: String,
        place: String,
        path: String,
        refusal: Refusal,
    },
}

fn file_of(app: &AppHandle, label: &str) -> Answer<std::path::PathBuf> {
    match shown_in(&app.state::<Views>(), label)? {
        Shown::File(at) => Ok(at),
        _ => Err(Refusal::of("notAllowed")),
    }
}

fn body_of(at: &Path) -> Answer<String> {
    tisty_core::docs::read_to_show(at).map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { limit, .. } => {
            Refusal::about("documentTooBig", crate::weighed(limit))
        }
        _ => Refusal::about("cannotRead", at.display().to_string()),
    })
}

// What is drawn here goes nowhere else: a stranger's file stays out of the cache of the person's own pages.
fn drawn_from(body: &str) -> (Vec<String>, Vec<Heading>) {
    let doc = tisty_markdown::read(body);
    let heads = tisty_markdown::outline(&doc)
        .into_iter()
        .map(|head| Heading {
            level: head.level,
            said: head.said,
            id: head.id,
        })
        .collect();
    (tisty_markdown::pieces(&doc, PIECE_ABOUT), heads)
}

#[tauri::command(async)]
pub fn view_read(app: AppHandle, window: tauri::WebviewWindow) -> Answer<Reading> {
    let label = window.label();
    let at = file_of(&app, label)?;
    let (name, place, path) = (named(&at), place_of(&at), at.display().to_string());
    let body = match body_of(&at) {
        Ok(body) => body,
        Err(refusal) => {
            return Ok(Reading::Missed {
                name,
                place,
                path,
                refusal,
            });
        }
    };
    let (mut pieces, heads) = drawn_from(&body);
    let first = if pieces.is_empty() {
        String::new()
    } else {
        pieces.remove(0)
    };
    let more = !pieces.is_empty();
    if let Ok(mut held) = app.state::<Views>().waiting_pieces.lock() {
        held.insert(label.to_string(), pieces);
    }
    Ok(Reading::Drawn {
        name,
        place,
        path,
        first,
        more,
        heads,
    })
}

#[tauri::command(async)]
pub fn view_rest(app: AppHandle, window: tauri::WebviewWindow) -> Vec<String> {
    app.state::<Views>()
        .waiting_pieces
        .lock()
        .ok()
        .and_then(|mut held| held.remove(window.label()))
        .unwrap_or_default()
}

#[tauri::command(async)]
pub fn view_source(app: AppHandle, window: tauri::WebviewWindow) -> Answer<String> {
    body_of(&file_of(&app, window.label())?)
}

#[cfg(test)]
#[path = "light_view_test.rs"]
mod tests;
