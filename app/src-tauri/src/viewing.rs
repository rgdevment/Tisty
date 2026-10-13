use std::collections::HashMap;
use std::path::{Component, Path, PathBuf, Prefix};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Emitter, Manager};
use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Refusal, Session, held, widgets};

const OPENS: [&str; 3] = ["md", "markdown", "txt"];

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Lent {
    Source(String),
    Reference(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Paper {
    pub id: String,
    pub file: String,
    pub title: String,
    pub place: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shown {
    File(PathBuf),
    Doc(Paper),
    Widget { lent: Lent, from: Option<Paper> },
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Showing {
    File {
        name: String,
        place: String,
        path: String,
        body: String,
    },
    Missed {
        name: String,
        place: String,
        path: String,
        refusal: Refusal,
    },
    Doc {
        paper: Paper,
        body: String,
    },
    Widget {
        lent: Lent,
        from: Option<Paper>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Imported {
    paper: Paper,
    stamp: Option<(u64, std::time::SystemTime)>,
}

#[derive(Default)]
pub struct Views {
    shown: Mutex<HashMap<String, Shown>>,
    kept: Mutex<HashMap<PathBuf, Imported>>,
    importing: Mutex<()>,
    made: AtomicU64,
    ready: std::sync::atomic::AtomicBool,
    early: Mutex<Vec<PathBuf>>,
}

impl Views {
    // Finder hands over the file a launch was for before the app has finished starting, so it waits for the start.
    pub fn started(&self, wanted: &mut Vec<PathBuf>) {
        let mut held = self.early.lock().unwrap_or_else(|e| e.into_inner());
        wanted.append(&mut held);
        self.ready.store(true, Ordering::SeqCst);
    }

    // A file from outside is someone else's word: its window borrows no widget, whatever its body names.
    pub fn stranger(&self, label: &str) -> bool {
        self.shown
            .lock()
            .ok()
            .is_some_and(|held| matches!(held.get(label), Some(Shown::File(_))))
    }
}

fn stamp_of(at: &Path) -> Option<(u64, std::time::SystemTime)> {
    let facts = std::fs::metadata(at).ok()?;
    Some((facts.len(), facts.modified().ok()?))
}

// Whatever reaches Tisty from outside is a third party's word: only a local path to a readable text file passes.
pub fn asked(said: &str, cwd: Option<&Path>) -> Option<PathBuf> {
    let said = said.trim();
    let at = match url::Url::parse(said) {
        Ok(link) if link.scheme() == "tisty" => {
            if link.host_str() != Some("open") || !matches!(link.path(), "" | "/") {
                return None;
            }
            let mut paths = link.query_pairs().filter(|(key, _)| key == "path");
            let (_, path) = paths.next()?;
            if paths.next().is_some() {
                return None;
            }
            PathBuf::from(path.into_owned())
        }
        Ok(link) if link.scheme() == "file" => link.to_file_path().ok()?,
        _ => match (PathBuf::from(said), cwd) {
            (at, Some(cwd)) if at.is_relative() => cwd.join(at),
            (at, _) => at,
        },
    };
    readable(&at).then_some(at)
}

fn readable(at: &Path) -> bool {
    let local = match at.components().next() {
        Some(Component::Prefix(prefix)) => {
            matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_))
        }
        _ => true,
    };
    local
        && at.is_absolute()
        && !at.components().any(|one| one == Component::ParentDir)
        && at
            .extension()
            .and_then(|one| one.to_str())
            .is_some_and(|one| OPENS.contains(&one.to_ascii_lowercase().as_str()))
        && at.is_file()
}

fn named(at: &Path) -> String {
    at.file_name()
        .map(|one| one.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn place_of(at: &Path) -> String {
    at.parent()
        .and_then(|up| up.file_name())
        .map(|one| one.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn wanted(argv: &[String], cwd: Option<&Path>) -> Vec<PathBuf> {
    argv.iter()
        .skip(1)
        .filter_map(|one| asked(one, cwd))
        .collect()
}

// Windows wedges a window built from inside the event loop, so the windows are built from a thread of their own.
pub fn show(app: &AppHandle, wanted: Vec<PathBuf>) {
    if wanted.is_empty() {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        for at in wanted {
            open(&app, Shown::File(at));
        }
    });
}

pub fn heard(app: &AppHandle, argv: &[String], cwd: Option<&Path>) -> bool {
    let found = wanted(argv, cwd);
    let any = !found.is_empty();
    show(app, found);
    any
}

#[cfg(target_os = "macos")]
pub fn heard_links(app: &AppHandle, links: &[url::Url]) {
    let said: Vec<String> = std::iter::once(String::new())
        .chain(links.iter().map(|one| one.to_string()))
        .collect();
    let views = app.state::<Views>();
    let found = wanted(&said, None);
    if let Ok(mut held) = views.early.lock()
        && !views.ready.load(Ordering::SeqCst)
    {
        held.extend(found);
        return;
    }
    let any = !found.is_empty();
    show(app, found);
    if !any {
        witness::warn(
            channel::WINDOW,
            "a link Tisty was asked to open was not one it opens",
            &[],
        );
    }
}

fn spanish(app: &AppHandle) -> bool {
    app.try_state::<Mutex<Session>>()
        .and_then(|session| session.lock().ok().map(|one| one.locale.clone()))
        .map(|locale| {
            tisty_core::model::spoken(locale.as_deref())
                .to_lowercase()
                .starts_with("es")
        })
        .unwrap_or_else(crate::refusing::speaks_spanish)
}

pub fn open(app: &AppHandle, shown: Shown) {
    let views = app.state::<Views>();
    let label = format!("view-{}", views.made.fetch_add(1, Ordering::Relaxed));
    let title = match &shown {
        Shown::File(at) => named(at),
        Shown::Doc(paper) => paper.title.clone(),
        Shown::Widget { .. } => "Tisty".into(),
    };
    if let Ok(mut held) = views.shown.lock() {
        held.insert(label.clone(), shown);
    }
    let made = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::App(crate::refusing::page(spanish(app)).into()),
    )
    .title(title)
    .inner_size(1180.0, 820.0)
    .min_inner_size(520.0, 380.0)
    .decorations(false)
    .transparent(true)
    .shadow(true)
    .center()
    .build();
    match made {
        Ok(window) => {
            let gone = app.clone();
            let named = label.clone();
            window.on_window_event(move |event| {
                if !matches!(event, tauri::WindowEvent::Destroyed) {
                    return;
                }
                if let Ok(mut held) = gone.state::<Views>().shown.lock() {
                    held.remove(&named);
                }
                gone.state::<widgets::Lent>().release(&named);
            });
        }
        Err(why) => {
            if let Ok(mut held) = views.shown.lock() {
                held.remove(&label);
            }
            witness::warn(
                channel::WINDOW,
                "a window to show something on its own would not open",
                &[("why", Fact::Why(why.to_string()))],
            );
        }
    }
}

fn shown_in(views: &Views, label: &str) -> Answer<Shown> {
    views
        .shown
        .lock()
        .ok()
        .and_then(|held| held.get(label).cloned())
        .ok_or_else(|| Refusal::of("notAllowed"))
}

fn session_of(app: &AppHandle) -> Answer<tauri::State<'_, Mutex<Session>>> {
    app.try_state::<Mutex<Session>>()
        .ok_or_else(|| Refusal::of("internal"))
}

// Read without being minded: the main window's guard against overwriting a changed file must stay its own.
fn doc_body(app: &AppHandle, file: &str) -> Answer<String> {
    let root = held(&session_of(app)?).paths.docs();
    tisty_core::docs::read(&root, file).map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { limit, .. } => {
            Refusal::about("documentTooBig", crate::weighed(limit))
        }
        _ => Refusal::about("noSuchDoc", file.to_string()),
    })
}

#[tauri::command(async)]
pub fn view_shown(
    app: AppHandle,
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
) -> Answer<Showing> {
    Ok(match shown_in(&views, window.label())? {
        Shown::File(at) => match crate::answers::carrying::read_outside(&at) {
            Ok(body) => Showing::File {
                name: named(&at),
                place: place_of(&at),
                path: at.display().to_string(),
                body,
            },
            Err(refusal) => Showing::Missed {
                name: named(&at),
                place: place_of(&at),
                path: at.display().to_string(),
                refusal,
            },
        },
        Shown::Doc(paper) => Showing::Doc {
            body: doc_body(&app, &paper.file)?,
            paper,
        },
        Shown::Widget { lent, from } => Showing::Widget { lent, from },
    })
}

#[tauri::command]
pub fn view_facts(
    session: tauri::State<'_, Mutex<Session>>,
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
) -> Answer<crate::answers::papers::Facts> {
    match shown_in(&views, window.label())? {
        Shown::File(at) => crate::answers::papers::outside_facts(&at),
        Shown::Doc(paper) => crate::answers::papers::doc_facts(session, paper.file),
        Shown::Widget { .. } => Err(Refusal::of("notAllowed")),
    }
}

fn bring(app: &AppHandle, file: &str) {
    let _ = app.emit_to("main", "showdoc", file.to_string());
    crate::tray::surface(app);
}

#[tauri::command(async)]
pub fn view_import(
    app: AppHandle,
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
    open: bool,
) -> Answer<Paper> {
    let _one_at_a_time = views.importing.lock().unwrap_or_else(|e| e.into_inner());
    let paper = match shown_in(&views, window.label())? {
        Shown::Doc(paper) => paper,
        Shown::File(at) => {
            let paper = imported_once(&app, &views, &at)?;
            if let Ok(mut held) = views.shown.lock() {
                held.insert(window.label().to_string(), Shown::Doc(paper.clone()));
            }
            paper
        }
        Shown::Widget { .. } => return Err(Refusal::of("notAllowed")),
    };
    if open {
        bring(&app, &paper.file);
    } else {
        let _ = app.emit_to("main", "papered", ());
    }
    Ok(paper)
}

// The same file brought in twice is the same document, as long as both are still what they were.
fn imported_once(app: &AppHandle, views: &Views, at: &Path) -> Answer<Paper> {
    let session = session_of(app)?;
    let stamp = stamp_of(at);
    let known = views
        .kept
        .lock()
        .ok()
        .and_then(|held| held.get(at).cloned());
    if let Some(known) = known
        && known.stamp.is_some()
        && known.stamp == stamp
        && {
            let session = held(&session);
            session
                .state
                .docs
                .values()
                .any(|kept| kept.file == known.paper.file && !session.state.held_away(kept))
        }
    {
        return Ok(known.paper);
    }
    let body = crate::answers::carrying::read_outside(at)?;
    let (made, id) = crate::answers::carrying::imported(&mut held(&session), &body, None)?;
    let paper = Paper {
        id: id.to_string(),
        file: made.id,
        title: made.title,
        place: String::new(),
    };
    if let Ok(mut held) = views.kept.lock() {
        held.insert(
            at.to_path_buf(),
            Imported {
                paper: paper.clone(),
                stamp,
            },
        );
    }
    Ok(paper)
}

#[tauri::command(async)]
pub fn view_go(app: AppHandle, file: String) {
    bring(&app, &file);
}

#[tauri::command(async)]
pub fn view_widget(app: AppHandle, lent: Lent, from: Option<Paper>) -> Answer<()> {
    if let Lent::Reference(reference) = &lent
        && !widgets::a_page(reference)
    {
        return Err(Refusal::of("notAllowed"));
    }
    open(&app, Shown::Widget { lent, from });
    Ok(())
}

#[tauri::command(async)]
pub fn view_doc(app: AppHandle, paper: Paper) {
    open(&app, Shown::Doc(paper));
}

#[cfg(test)]
#[path = "viewing_test.rs"]
mod tests;
