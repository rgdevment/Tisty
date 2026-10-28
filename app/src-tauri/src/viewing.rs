use std::collections::HashMap;
use std::path::{Component, Path, PathBuf, Prefix};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Emitter, Manager};
use tisty_core::witness::{self, Fact, channel};

use crate::answers::carrying::Left;
use crate::{Answer, Refusal, Session, held, widgets};

const OPENS: [&str; 5] = ["md", "markdown", "txt", "ics", "ical"];

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
    Diagram(crate::diagramming::Drawn),
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
    Diagram {
        paper: Paper,
        source: String,
        nth: usize,
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
    calendars: Mutex<Vec<PathBuf>>,
    unsaved: Mutex<std::collections::HashSet<String>>,
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
    let (calendars, wanted): (Vec<PathBuf>, Vec<PathBuf>) =
        wanted.into_iter().partition(|at| calendar(at));
    awaited(app, calendars);
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

pub(crate) fn calendar(at: &Path) -> bool {
    at.extension()
        .and_then(|one| one.to_str())
        .is_some_and(|one| one.eq_ignore_ascii_case("ics") || one.eq_ignore_ascii_case("ical"))
}

// A calendar becomes tasks in the main window, which may not be listening yet while Tisty starts.
fn awaited(app: &AppHandle, calendars: Vec<PathBuf>) {
    if calendars.is_empty() {
        return;
    }
    if let Ok(mut held) = app.state::<Views>().calendars.lock() {
        held.extend(calendars);
    }
    crate::tray::surface(app);
    let _ = app.emit_to("main", "calendar", ());
}

#[tauri::command]
pub fn ics_waiting(views: tauri::State<'_, Views>) -> Vec<String> {
    views
        .calendars
        .lock()
        .map(|mut held| {
            held.drain(..)
                .map(|at| at.to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
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

// As tall as a fifth less than Tisty as it stands, and wide enough for the index beside the page.
fn beside_main(app: &AppHandle) -> (f64, f64) {
    let Some(main) = app.get_webview_window("main") else {
        return SIZED;
    };
    let (Ok(size), Ok(scale)) = (main.inner_size(), main.scale_factor()) else {
        return SIZED;
    };
    let logical = size.to_logical::<f64>(scale);
    scaled(logical.height)
}

const WIDE: f64 = 1400.0;
const SIZED: (f64, f64) = (WIDE, 656.0);

fn scaled(height: f64) -> (f64, f64) {
    (WIDE, (height * 0.8).max(560.0))
}

pub(crate) fn open(app: &AppHandle, shown: Shown) {
    let views = app.state::<Views>();
    let label = format!("view-{}", views.made.fetch_add(1, Ordering::Relaxed));
    let title = match &shown {
        Shown::File(at) => named(at),
        Shown::Doc(paper) => paper.title.clone(),
        Shown::Diagram(drawn) => drawn.paper.title.clone(),
        Shown::Widget { .. } => "Tisty".into(),
    };
    if let Ok(mut held) = views.shown.lock() {
        held.insert(label.clone(), shown);
    }
    let sized = beside_main(app);
    let made = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::App(crate::refusing::page(spanish(app)).into()),
    )
    .title(title)
    .inner_size(sized.0, sized.1)
    .min_inner_size(520.0, 380.0)
    .decorations(false)
    .transparent(true)
    .shadow(true)
    .center()
    .build();
    match made {
        Ok(window) => {
            crate::tray::docked(app, true);
            let gone = app.clone();
            let named = label.clone();
            window.on_window_event(move |event| {
                if !matches!(event, tauri::WindowEvent::Destroyed) {
                    return;
                }
                let views = gone.state::<Views>();
                if let Ok(mut held) = views.shown.lock() {
                    held.remove(&named);
                }
                mark_unsaved(&views, &named, false);
                gone.state::<widgets::Lent>().release(&named);
                crate::tray::undock_unless_seen(&gone, &named);
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

pub(crate) fn shown_in(views: &Views, label: &str) -> Answer<Shown> {
    views
        .shown
        .lock()
        .ok()
        .and_then(|held| held.get(label).cloned())
        .ok_or_else(|| Refusal::of("notAllowed"))
}

pub(crate) fn reshow(views: &Views, label: &str, shown: Shown) {
    if let Ok(mut held) = views.shown.lock()
        && let Some(was) = held.get_mut(label)
    {
        *was = shown;
    }
}

pub(crate) fn showing(views: &Views, wanted: impl Fn(&Shown) -> bool) -> Option<String> {
    let held = views.shown.lock().ok()?;
    held.iter()
        .find(|(_, one)| wanted(one))
        .map(|(label, _)| label.clone())
}

pub(crate) fn mark_unsaved(views: &Views, label: &str, unsaved: bool) {
    if let Ok(mut held) = views.unsaved.lock() {
        if unsaved {
            held.insert(label.to_string());
        } else {
            held.remove(label);
        }
    }
}

pub(crate) fn first_unsaved(views: &Views) -> Option<String> {
    views.unsaved.lock().ok()?.iter().next().cloned()
}

fn session_of(app: &AppHandle) -> Answer<tauri::State<'_, Mutex<Session>>> {
    app.try_state::<Mutex<Session>>()
        .ok_or_else(|| Refusal::of("internal"))
}

// Read without being minded: the main window's guard against overwriting a changed file must stay its own.
pub(crate) fn doc_body(app: &AppHandle, file: &str) -> Answer<String> {
    let root = held(&*session_of(app)?).paths.docs();
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
        Shown::Diagram(drawn) => Showing::Diagram {
            paper: drawn.paper,
            source: drawn.was,
            nth: drawn.place.nth,
        },
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
        Shown::Diagram(drawn) => crate::answers::papers::doc_facts(session, drawn.paper.file),
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
) -> Answer<Converted> {
    let _one_at_a_time = views.importing.lock().unwrap_or_else(|e| e.into_inner());
    let (paper, left) = match shown_in(&views, window.label())? {
        Shown::Doc(paper) => (paper, Left::default()),
        Shown::File(at) => {
            let (paper, left) = imported_once(&app, &views, &at)?;
            if let Ok(mut held) = views.shown.lock() {
                held.insert(window.label().to_string(), Shown::Doc(paper.clone()));
            }
            (paper, left)
        }
        Shown::Widget { .. } | Shown::Diagram(_) => return Err(Refusal::of("notAllowed")),
    };
    if open {
        bring(&app, &paper.file);
    } else {
        let _ = app.emit_to("main", "papered", ());
    }
    Ok(Converted { paper, left })
}

#[derive(serde::Serialize)]
pub struct Converted {
    #[serde(flatten)]
    paper: Paper,
    #[serde(flatten)]
    left: Left,
}

// The same file brought in twice is the same document, as long as both are still what they were.
fn imported_once(app: &AppHandle, views: &Views, at: &Path) -> Answer<(Paper, Left)> {
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
        return Ok((known.paper, Left::default()));
    }
    let (made, id, left) = crate::answers::carrying::arrived(&session, at, None)?;
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
    Ok((paper, left))
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
