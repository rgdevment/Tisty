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

// A journal is never rewritten, so what it said before Tisty had its own words is turned as drawn.
#[tauri::command(async)]
pub fn note_drawn(body: String) -> String {
    tisty_markdown::html(&tisty_markdown::read(&tisty_markdown::to_tisty(&body)))
}

#[tauri::command(async)]
pub fn mends_in(body: String) -> Vec<tisty_core::mending::Mend> {
    tisty_core::mending::mends(&body)
}

const GUIDE: [&str; 2] = [
    "https://markdown.tisty.app/llms-full.txt",
    "https://markdown.tisty.app/es/llms-full.txt",
];
const PATIENCE: std::time::Duration = std::time::Duration::from_secs(4);

// The guide is read where it is published, so a change to it reaches the window without a release.
#[tauri::command]
pub async fn help_reference(spanish: bool) -> String {
    let at = GUIDE[usize::from(spanish)];
    tauri::async_runtime::spawn_blocking(move || published(at))
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| tisty_markdown::reference(spanish).to_string())
}

#[tauri::command]
pub fn help_reference_here(spanish: bool) -> String {
    tisty_markdown::reference(spanish).to_string()
}

pub(crate) fn published(at: &str) -> Option<String> {
    let asked = reqwest::blocking::Client::builder()
        .timeout(PATIENCE)
        .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?
        .get(at)
        .send()
        .ok()?;
    if !asked.status().is_success() {
        return None;
    }
    asked.text().ok().filter(|text| text.starts_with("# "))
}

// The guide is drawn past the page cache, which holds the person's own documents and would lose them to it.
#[tauri::command(async)]
pub fn help_drawn(body: String) -> String {
    tisty_markdown::html(&tisty_markdown::read(&body))
}

#[tauri::command(async)]
pub fn help_anchors(bodies: Vec<String>) -> Vec<Vec<String>> {
    bodies
        .iter()
        .map(|body| {
            tisty_markdown::outline(&tisty_markdown::read(body))
                .into_iter()
                .map(|head| head.id)
                .collect()
        })
        .collect()
}

#[tauri::command(async)]
pub fn doc_plain(body: String) -> String {
    tisty_markdown::to_standard(&body, &|_| None)
}

// The body comes back changed only if the table still sits where the page drew it; the window saves it.
#[tauri::command(async)]
pub fn table_edit(
    body: String,
    at: (usize, usize),
    edits: Vec<tisty_markdown::grid::Edit>,
) -> Result<String, String> {
    tisty_markdown::grid::edits(&body, at.0..at.1, &edits).map_err(|why| why.word().to_string())
}

// A date may be typed as it is said, «el viernes», and lands in the file as its day.
#[tauri::command(async)]
pub fn table_canon(
    session: tauri::State<'_, Mutex<Session>>,
    words: String,
    typed: String,
    locale: String,
) -> String {
    let spoken = held(&session).locale.clone().unwrap_or(locale);
    canon(&words, &typed, &jiff::Zoned::now(), &spoken)
}

pub fn canon(words: &str, typed: &str, now: &jiff::Zoned, spoken: &str) -> String {
    let made = tisty_markdown::canon::canonical(words, typed);
    let dated = words.split_whitespace().next() == Some("date");
    if !dated || made.parse::<jiff::civil::Date>().is_ok() {
        return made;
    }
    tisty_nl::parse_date(typed, now, spoken).map_or(made, |when| when.at.date().to_string())
}

// The text editor names the table by the caret's byte, so it is found and changed in one ask.
#[tauri::command(async)]
pub fn table_edit_near(
    body: String,
    at: usize,
    edits: Vec<tisty_markdown::grid::Edit>,
) -> Result<String, String> {
    tisty_markdown::grid::edits_near(&body, at, &edits).map_err(|why| why.word().to_string())
}

#[tauri::command(async)]
pub fn table_order(
    body: String,
    at: (usize, usize),
    column: usize,
    down: bool,
) -> Result<Vec<usize>, String> {
    tisty_markdown::grid::order(&body, at.0..at.1, column, down)
        .map_err(|why| why.word().to_string())
}

#[tauri::command(async)]
pub fn table_written(
    body: String,
    at: (usize, usize),
) -> Result<tisty_markdown::grid::Written, String> {
    tisty_markdown::grid::written(&body, at.0..at.1).map_err(|why| why.word().to_string())
}

pub fn paged(shelf: &Path, body: &str) -> String {
    let drawn = || tisty_markdown::html(&tisty_markdown::read(body));
    // A build under work keeps one version number while the drawing changes under it.
    if cfg!(debug_assertions) && !cfg!(test) {
        return drawn();
    }
    let kept = shelf.join(DRAWN_BY).join(format!(
        "{}.html",
        tisty_core::attach::printed(body.as_bytes())
    ));
    if let Ok(html) = std::fs::read_to_string(&kept) {
        touched(&kept);
        return html;
    }
    let html = drawn();
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
