use std::path::Path;
use std::sync::Mutex;

use tisty_core::history::{self, Compared};
use tisty_core::witness::channel;

use crate::{Answer, Refusal, Session, blamed, held};

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Told {
    Drawn {
        html: String,
        gone: usize,
        came: usize,
    },
    TooLong,
}

// The body compared with a version is the one on disk: a window with words unsaved saves before it asks.
#[tauri::command(async)]
pub fn doc_compared(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    was: String,
    now: Option<String>,
) -> Answer<Told> {
    let data = held(&session).paths.data().to_path_buf();
    told(&data, &id, &was, now.as_deref())
}

pub(crate) fn told(data: &Path, id: &str, was: &str, now: Option<&str>) -> Answer<Told> {
    match history::compared(data, id, was, now) {
        Ok(Compared::Told(page)) => Ok(Told::Drawn {
            html: page.html,
            gone: page.gone,
            came: page.came,
        }),
        Ok(Compared::TooLong) => Ok(Told::TooLong),
        Ok(Compared::NotKept) => Err(Refusal::of("versionGone")),
        Err(tisty_core::Error::Io(_) | tisty_core::Error::OutsideTheStore(_)) => {
            Err(Refusal::about("noSuchDoc", id))
        }
        Err(e) => Err(blamed(
            channel::WINDOW,
            "a document could not be read to compare it",
            e,
        )),
    }
}

#[cfg(test)]
#[path = "comparing_test.rs"]
mod tests;
