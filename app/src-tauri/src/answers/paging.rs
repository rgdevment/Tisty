use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

use crate::{Session, held};

const DRAWN_BY: &str = env!("CARGO_PKG_VERSION");
const KEPT_AT_MOST: usize = 256;

#[tauri::command(async)]
pub fn doc_drawn(session: tauri::State<'_, Mutex<Session>>, body: String) -> tauri::ipc::Response {
    let shelf = held(&session).paths.cache().join("pages");
    tauri::ipc::Response::new(paged(&shelf, &body).into_bytes())
}

// A note changes with every key, so it is drawn fresh and never left on the shelf; a journal is
// never rewritten, so what it said before Tisty had its own words is turned as it is drawn.
#[tauri::command(async)]
pub fn note_drawn(body: String) -> String {
    tisty_markdown::html(&tisty_markdown::read(&tisty_markdown::to_tisty(&body)))
}

#[tauri::command(async)]
pub fn doc_printed(body: String) -> serde_json::Value {
    tisty_markdown::printed(&tisty_markdown::read(&body))
}

pub fn paged(shelf: &Path, body: &str) -> String {
    // A build under work keeps one version number while the drawing changes under it.
    if cfg!(debug_assertions) && !cfg!(test) {
        return tisty_markdown::html(&tisty_markdown::read(body));
    }
    let kept = shelf.join(DRAWN_BY).join(format!(
        "{}.html",
        tisty_core::attach::printed(body.as_bytes())
    ));
    if let Ok(html) = std::fs::read_to_string(&kept) {
        touched(&kept);
        return html;
    }
    let html = tisty_markdown::html(&tisty_markdown::read(body));
    if let Err(e) = keep(shelf, &kept, &html) {
        witness::warn(
            channel::CACHE,
            "a drawn page was not kept",
            &[("why", Fact::Why(e.to_string()))],
        );
    }
    html
}

fn keep(shelf: &Path, kept: &Path, html: &str) -> tisty_core::Result<()> {
    let drawer = shelf.join(DRAWN_BY);
    for old in std::fs::read_dir(shelf).into_iter().flatten().flatten() {
        if old.file_name() != DRAWN_BY {
            let _ = std::fs::remove_dir_all(old.path());
        }
    }
    std::fs::create_dir_all(&drawer)?;
    thinned(&drawer);
    tisty_core::store::write_atomic(kept, html.as_bytes())
}

// A page read again moves to the back of the line, so the shelf drops the ones nobody opens.
fn touched(kept: &Path) {
    let _ = std::fs::File::options()
        .write(true)
        .open(kept)
        .and_then(|page| page.set_modified(std::time::SystemTime::now()));
}

// Every version of every document opened leaves a page, so the oldest go once there are too many.
fn thinned(drawer: &Path) {
    let mut pages: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(drawer)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|one| one.path().extension().is_some_and(|ext| ext == "html"))
        .filter_map(|one| Some((one.metadata().ok()?.modified().ok()?, one.path())))
        .collect();
    if pages.len() < KEPT_AT_MOST {
        return;
    }
    pages.sort();
    for (_, page) in pages.iter().take(pages.len() + 1 - KEPT_AT_MOST / 2) {
        let _ = std::fs::remove_file(page);
    }
}

#[cfg(test)]
#[path = "paging_test.rs"]
mod tests;
