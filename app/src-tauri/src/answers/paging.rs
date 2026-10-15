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

pub fn paged(shelf: &Path, body: &str) -> String {
    let kept = shelf.join(DRAWN_BY).join(format!(
        "{}.html",
        tisty_core::attach::printed(body.as_bytes())
    ));
    if let Ok(html) = std::fs::read_to_string(&kept) {
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

// Every version of every document opened leaves a page, so the oldest go once there are too many.
fn thinned(drawer: &Path) {
    let mut pages: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(drawer)
        .into_iter()
        .flatten()
        .flatten()
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
