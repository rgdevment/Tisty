use std::collections::HashMap;
use std::path::{Component, Path, PathBuf, Prefix};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use tauri::{AppHandle, Emitter, Manager};
use tisty_carrier::folio_hosts;
use tisty_core::witness::{self, Fact, channel};

use crate::answers::carrying::Left;
use crate::{Answer, Refusal, Session, held};

const OPENS: [&str; 5] = ["md", "markdown", "txt", "ics", "ical"];

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
    Widget { source: String, from: Option<Paper> },
    Diagram(crate::diagramming::Drawn),
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Showing {
    Doc { paper: Paper, body: String },
    Widget { source: String, from: Option<Paper> },
    Diagram { paper: Paper, source: String },
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
    pub(crate) waiting_pieces: Mutex<HashMap<String, Vec<String>>>,
    edits_owed: Mutex<Vec<String>>,
    links_owed: Mutex<Vec<url::Url>>,
    links_arriving: Mutex<LinksArriving>,
    link_asking: Mutex<Option<LinkAsking>>,
    link_asked: AtomicU64,
}

struct LinkAsking {
    id: u64,
    question: LinkQuestion,
    answer: std::sync::mpsc::Sender<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LinkAsked {
    id: u64,
    #[serde(flatten)]
    question: LinkQuestion,
}

const LINK_ASKS_IN: &str = "main";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LinkQuestion {
    Look,
    Keep {
        title: String,
        owner: Option<crate::answers::folio_standing::Party>,
        signer: crate::answers::folio_standing::Party,
        doubtful: bool,
    },
    Failed {
        code: String,
    },
    LanguagesLeft {
        languages: Vec<String>,
    },
}

#[derive(Default)]
struct LinksArriving {
    waiting: Vec<url::Url>,
    running: bool,
    handling: Option<url::Url>,
}

// Any page can fire a link, over and over, and each would stand a question on the desktop.
const LINKS_WAITING_AT_MOST: usize = 8;

impl Views {
    // True for whoever has to go through them: the others only leave theirs in line.
    pub(crate) fn link_arrived(&self, link: url::Url) -> bool {
        let mut links = self
            .links_arriving
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let known = links.waiting.contains(&link) || links.handling.as_ref() == Some(&link);
        if !known && links.waiting.len() < LINKS_WAITING_AT_MOST {
            links.waiting.push(link);
        }
        !std::mem::replace(&mut links.running, true)
    }

    pub(crate) fn ask_of_a_link(&self, question: LinkQuestion) -> std::sync::mpsc::Receiver<bool> {
        let (answer, answered) = std::sync::mpsc::channel();
        let id = self.link_asked.fetch_add(1, Ordering::SeqCst);
        *self.link_asking.lock().unwrap_or_else(|e| e.into_inner()) = Some(LinkAsking {
            id,
            question,
            answer,
        });
        answered
    }

    fn link_standing(&self) -> Option<LinkAsked> {
        let asking = self.link_asking.lock().unwrap_or_else(|e| e.into_inner());
        asking.as_ref().map(|one| LinkAsked {
            id: one.id,
            question: one.question.clone(),
        })
    }

    // An answer holds only for the question it was shown: the next one in line has not been read.
    fn link_answered(&self, id: u64, yes: bool) {
        let mut asking = self.link_asking.lock().unwrap_or_else(|e| e.into_inner());
        if asking.as_ref().is_some_and(|one| one.id == id)
            && let Some(one) = asking.take()
        {
            let _ = one.answer.send(yes);
        }
    }

    fn next_link(&self) -> Option<url::Url> {
        let mut links = self
            .links_arriving
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if links.waiting.is_empty() {
            links.running = false;
            links.handling = None;
            return None;
        }
        let next = links.waiting.remove(0);
        links.handling = Some(next.clone());
        Some(next)
    }

    pub(crate) fn go_through_links(&self, bring: impl Fn(&url::Url)) {
        while let Some(link) = self.next_link() {
            // One that breaks must not leave every later link waiting for a restart.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| bring(&link)));
        }
    }

    fn owe_edit(&self, label: &str) {
        let mut owed = self.edits_owed.lock().unwrap_or_else(|e| e.into_inner());
        if !owed.iter().any(|one| one == label) {
            owed.push(label.to_string());
        }
    }

    fn owe_link(&self, link: url::Url) {
        let mut owed = self.links_owed.lock().unwrap_or_else(|e| e.into_inner());
        if !owed.contains(&link) {
            owed.push(link);
        }
    }

    fn links_due(&self) -> Vec<url::Url> {
        std::mem::take(&mut *self.links_owed.lock().unwrap_or_else(|e| e.into_inner()))
    }

    fn edits_due(&self) -> Vec<String> {
        std::mem::take(&mut *self.edits_owed.lock().unwrap_or_else(|e| e.into_inner()))
    }

    // Finder hands over the file a launch was for before the app has finished starting, so it waits for the start.
    pub fn started(&self, wanted: &mut Vec<PathBuf>) {
        let mut held = self.early.lock().unwrap_or_else(|e| e.into_inner());
        wanted.append(&mut held);
        self.ready.store(true, Ordering::SeqCst);
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

const PUBLIC_HOST: &str = "folio.tisty.app";

// Any page can fire this address, and it carries a key: it takes its one link from Folio's servers, and it is never logged.
pub fn asked_to_edit(said: &str, private: &str) -> Option<url::Url> {
    let link = url::Url::parse(said.trim()).ok()?;
    if link.scheme() != "tisty"
        || link.host_str() != Some("edit")
        || !matches!(link.path(), "" | "/")
        || link.fragment().is_some()
    {
        return None;
    }
    let mut parameters = link.query_pairs();
    let (name, value) = parameters.next()?;
    if name != "link" || parameters.next().is_some() {
        return None;
    }
    let web = url::Url::parse(&value).ok()?;
    let host = web.host_str()?;
    let ours = [folio_hosts::FIRST_PRIVATE, PUBLIC_HOST, private].contains(&host);
    (web.scheme() == "https" && ours).then_some(web)
}

// A first copy would make the store look used before the welcome has said where the copies go, so a link waits for it.
pub fn told_of_edits(app: &AppHandle, argv: &[String]) -> bool {
    let private = app.try_state::<Mutex<Session>>().map_or_else(
        || folio_hosts::FIRST_PRIVATE.to_owned(),
        |session| folio_hosts::private_kept(&held(&session).paths.private()),
    );
    let links: Vec<url::Url> = argv
        .iter()
        .skip(1)
        .filter_map(|one| asked_to_edit(one, &private))
        .collect();
    let any = !links.is_empty();
    let welcomed = app
        .try_state::<Mutex<Session>>()
        .is_some_and(|session| held(&session).config.sync.is_some());
    for link in links {
        if welcomed {
            crate::answers::folio_receiving::bring_apart(app, link);
        } else {
            app.state::<Views>().owe_link(link);
        }
    }
    if any && !welcomed {
        crate::tray::surface(app);
    }
    any
}

pub(crate) fn readable(at: &Path) -> bool {
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

pub(crate) fn named(at: &Path) -> String {
    at.file_name()
        .map(|one| one.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub(crate) fn place_of(at: &Path) -> String {
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
    told_of_edits(app, argv);
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
    let edits = told_of_edits(app, &said);
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
    if !any && !edits {
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
// The sheet and its panels side by side, with their gaps and the scrollbar, and nothing to spare.
const LIGHT_WIDE: f64 = 1200.0;
const SIZED: (f64, f64) = (WIDE, 656.0);

fn scaled(height: f64) -> (f64, f64) {
    (WIDE, (height * 0.8).max(560.0))
}

#[derive(Debug, PartialEq, Eq)]
struct Look {
    framed: bool,
    clear: bool,
}

// The light viewer takes the system's own title bar; the rest draw theirs.
fn look_of(shown: &Shown) -> Look {
    let light = matches!(shown, Shown::File(_));
    Look {
        framed: light,
        clear: !light,
    }
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
    let spanish = spanish(app);
    let light = matches!(shown, Shown::File(_));
    let look = look_of(&shown);
    let page = if light {
        crate::refusing::light_page(spanish)
    } else {
        crate::refusing::page(spanish)
    };
    if let Ok(mut held) = views.shown.lock() {
        held.insert(label.clone(), shown);
    }
    let sized = beside_main(app);
    let wide = if light { LIGHT_WIDE } else { sized.0 };
    let made = crate::engine::window(app, &label, tauri::WebviewUrl::App(page.into()))
        .title(title)
        .inner_size(wide, sized.1)
        .min_inner_size(520.0, 380.0)
        .decorations(look.framed)
        .transparent(look.clear)
        .shadow(true)
        .center()
        // Shown by its page once drawn, so nobody sees it empty; a transparent one would even be painted black.
        .visible(false)
        .build();
    match made {
        Ok(window) => {
            // Should the page never say so, the window is not left hidden for good.
            let late = window.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let _ = late.show();
            });
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
                if let Ok(mut held) = views.waiting_pieces.lock() {
                    held.remove(&named);
                }
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

#[tauri::command]
pub async fn view_shown(app: AppHandle, window: tauri::WebviewWindow) -> Answer<Showing> {
    let label = window.label().to_owned();
    let shown = shown_in(&app.state::<Views>(), &label)?;
    Ok(match shown {
        Shown::File(_) => return Err(Refusal::of("notAllowed")),
        Shown::Doc(paper) => Showing::Doc {
            body: doc_body(&app, &paper.file)?,
            paper,
        },
        Shown::Widget { source, from } => Showing::Widget { source, from },
        Shown::Diagram(drawn) => Showing::Diagram {
            paper: drawn.paper,
            source: drawn.was,
        },
    })
}

// Asked rather than told only by the event: the window may still be loading when a link opens Tisty.
#[tauri::command]
pub fn link_question_standing(views: tauri::State<'_, Views>) -> Option<LinkAsked> {
    views.link_standing()
}

#[tauri::command]
pub fn link_question_answered(
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
    id: u64,
    yes: bool,
) {
    if window.label() == LINK_ASKS_IN {
        views.link_answered(id, yes);
    }
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
) -> Answer<Option<Converted>> {
    let _one_at_a_time = views.importing.lock().unwrap_or_else(|e| e.into_inner());
    let Shown::File(at) = shown_in(&views, window.label())? else {
        return Err(Refusal::of("notAllowed"));
    };
    // A first copy would make the store look used before the welcome has said where the copies go.
    if held(&*session_of(&app)?).config.sync.is_none() {
        views.owe_edit(window.label());
        crate::tray::surface(&app);
        return Ok(None);
    }
    let (paper, left) = imported_once(&app, &views, &at)?;
    bring(&app, &paper.file);
    Ok(Some(Converted { paper, left }))
}

#[tauri::command(async)]
pub fn view_welcomed(app: AppHandle, views: tauri::State<'_, Views>) {
    for label in views.edits_due() {
        let _ = app.emit_to(label.as_str(), "view-edit", ());
    }
    for link in views.links_due() {
        crate::answers::folio_receiving::bring_apart(&app, link);
    }
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
    let (made, id, left) = crate::answers::carrying::arrived_as(&session, at, None, |raw| {
        tisty_markdown::file_as_tisty(&raw)
    })?;
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
pub fn view_widget(app: AppHandle, source: String, from: Option<Paper>) {
    open(&app, Shown::Widget { source, from });
}

#[tauri::command(async)]
pub fn view_doc(app: AppHandle, paper: Paper) {
    open(&app, Shown::Doc(paper));
}

#[cfg(test)]
#[path = "viewing_test.rs"]
mod tests;
