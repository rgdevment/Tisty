use std::sync::Mutex;

mod answers;
mod asked;
mod command;
mod desktop;
mod finding;
mod glimpse;
mod herald;
mod refusing;
mod report;
mod session;
mod shop;
mod summing;
mod tray;
mod update;
mod vouching;
mod waking;
mod wiring;

use tauri::{Emitter, Manager};

pub(crate) use asked::{Change, View, ahead, dated_field, recalled, repeated, tagged};
pub(crate) use refusing::{Refusal, blamed, refusal_code};
pub(crate) use refusing::{behind_buttons, behind_said, where_it_comes_from};
pub(crate) use session::Session;
pub(crate) use summing::{Coming, Counted, Habit, coming, recurring, tags_in_use, tally};

use tisty_core::{
    List, Op, Reading, State, Task,
    event::TaskPatch,
    view::{Filter, Scope, Window},
    witness::{self, Fact, channel},
};

#[derive(serde::Serialize)]
struct Snapshot {
    tasks: Vec<Task>,
    ahead: Vec<Coming>,
    routines: Vec<Habit>,
    lists: Vec<List>,
    tags: Vec<Counted>,
    refs: Vec<String>,
    counts: std::collections::BTreeMap<String, usize>,
    locale: Option<String>,
    agents: std::collections::BTreeMap<String, String>,
    agent_tag: &'static str,
    hosts: std::collections::BTreeMap<String, String>,
    machines: std::collections::BTreeMap<String, String>,
    machine_here: String,
    clients: std::collections::BTreeMap<String, String>,
}

impl From<tisty_core::capture::Rejected> for Refusal {
    fn from(rejected: tisty_core::capture::Rejected) -> Self {
        use tisty_core::capture::Rejected;
        match rejected {
            Rejected::Untitled => Refusal::of("untitled"),
            Rejected::EndedAlready => Refusal::of("pastEnd"),
            Rejected::NoSuchList(name) => Refusal::about("noSuchList", name),
            Rejected::ArchivedList(name) => Refusal::about("archivedList", name),
            Rejected::AmbiguousList(name) => Refusal::about("ambiguousList", name),
        }
    }
}

impl From<tisty_core::Error> for Refusal {
    fn from(error: tisty_core::Error) -> Self {
        blamed(channel::WINDOW, "a command could not finish", error)
    }
}

type Answer<T> = std::result::Result<T, Refusal>;

fn held<'a>(session: &'a tauri::State<'_, Mutex<Session>>) -> std::sync::MutexGuard<'a, Session> {
    session
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Disk work that grows with the store belongs on a thread of its own: a command that is an
/// `async fn` runs on the executor, and the window stops drawing until it returns.
async fn elsewhere<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Answer<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| {
            witness::error(
                channel::WINDOW,
                "a piece of work never came back",
                &[("why", Fact::Why(e.to_string()))],
            );
            Refusal::of("internal")
        })
}

fn today() -> jiff::civil::Date {
    jiff::Zoned::now().date()
}

fn zone() -> String {
    jiff::Zoned::now()
        .time_zone()
        .iana_name()
        .unwrap_or("UTC")
        .to_string()
}

#[tauri::command(async)]
fn search(
    session: tauri::State<'_, Mutex<Session>>,
    query: String,
    scope: Option<String>,
) -> Answer<Found> {
    let mut session = held(&session);
    session.reload()?;

    let scope = match scope.as_deref() {
        Some("open") => Scope::Open,
        Some("archived") => Scope::Archived,
        _ => Scope::Either,
    };
    let (hits, total) = session.state.searching(&query, scope, MOST);
    let tasks: Vec<Task> = hits.into_iter().cloned().collect();
    let listed: std::collections::BTreeMap<String, bool> = session
        .state
        .docs
        .values()
        .filter(|one| match scope {
            Scope::Open => !session.state.held_away(one),
            Scope::Archived => session.state.held_away(one),
            Scope::Either => true,
        })
        .map(|one| (one.file.clone(), session.state.held_away(one)))
        .collect();
    let root = session.paths.docs();
    let papers = session
        .corpus
        .searching(&root, &query, PAPERS_MOST, |id| listed.contains_key(id))
        .into_iter()
        .map(|one| Paper {
            archived: listed.get(&one.id).copied().unwrap_or(false),
            id: one.id,
            title: one.title,
            line: one.line,
        })
        .collect();
    Ok(Found {
        tasks,
        total,
        papers,
    })
}

const MOST: usize = 200;
const PAPERS_MOST: usize = 40;

#[derive(serde::Serialize)]
struct Paper {
    id: String,
    title: String,
    line: String,
    archived: bool,
}

#[derive(serde::Serialize)]
struct Found {
    tasks: Vec<Task>,
    papers: Vec<Paper>,
    total: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Carrying {
    chosen: Option<String>,
    keeper: Option<String>,
    kept_by: Option<String>,
    asked: bool,
    last: Option<String>,
    heard: Option<String>,
    loose: usize,
    open: usize,
    archived: usize,
    lists: usize,
    attachments: usize,
    weight: u64,
    carries: u64,
    shared_was: Option<String>,
    backed_up_at: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Logs {
    at: String,
    bytes: u64,
    lines: Vec<String>,
}

fn language<R: tauri::Runtime>(app: &tauri::AppHandle<R>, locale: &Option<String>) {
    tray::reword(
        app,
        &tray::Words {
            show: worded(locale, "show"),
            capture: worded(locale, "capture"),
            quit: worded(locale, "quit"),
        },
    );
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Settling {
    ran: bool,
    brought: bool,
    agrees: bool,
    was: Option<String>,
    stuck: Option<Refusal>,
}

const HERE: &str = env!("CARGO_PKG_VERSION");

/// A document only learns its tags where its body is read, and a body is read when it is saved.
/// Everything written before this existed is caught up here, in one pass over the folder.
#[tauri::command(async)]
fn read_tags(session: tauri::State<'_, Mutex<Session>>) -> Answer<usize> {
    let root = held(&session).paths.docs();
    let owing: Vec<(tisty_core::model::DocId, String)> = held(&session)
        .state
        .docs
        .values()
        .map(|one| (one.id, one.file.clone()))
        .collect();

    let mut ops: Vec<Op> = Vec::new();
    for (id, file) in owing {
        let Ok(body) = tisty_core::docs::read(&root, &file) else {
            continue;
        };
        let session = held(&session);
        let said = tisty_core::event::Said::of(&body).by(None);
        let Some(kept) = session.state.docs.get(&id) else {
            continue;
        };
        if said.news_for(kept) {
            ops.push(Op::DocSaid { id, d: said });
        }
    }

    let told = ops.len();
    if !ops.is_empty() {
        held(&session).commit_all(ops)?;
    }
    Ok(told)
}

fn stop(found: Option<Option<usize>>) -> Option<usize> {
    found.flatten()
}

fn signing(state: &tisty_core::State) -> Option<String> {
    state.signed.alias.clone()
}

// The MSIX package ships the executable alone, so the guide travels inside the binary.
fn stale(mine: Option<&str>, now: Option<&str>) -> bool {
    matches!((mine, now), (Some(mine), Some(now)) if mine != now)
}

/// Changing a folder the archive holds, or filing anything into it, is refused everywhere.
pub(crate) fn folder_open(
    state: &State,
    at: tisty_core::model::FolderId,
    holds: bool,
) -> Answer<()> {
    match state.folder_away(at) {
        true => Err(Refusal::of(match holds {
            true => "folderAwayHolds",
            false => "folderAway",
        })),
        false => Ok(()),
    }
}

fn doc_out(state: &State, id: tisty_core::model::DocId) -> Answer<()> {
    let Some(kept) = state.docs.get(&id) else {
        return Ok(());
    };
    match (state.held_by_another(kept), kept.page_of.is_some()) {
        (true, true) => Err(Refusal::of("pageIsAway")),
        (true, false) => Err(Refusal::of("folderIsAway")),
        (false, _) => Ok(()),
    }
}

const SETTLED_IN: jiff::SignedDuration = jiff::SignedDuration::from_hours(24 * 14);
const CLOSED_ENOUGH: usize = 10;
const PAPERS_ENOUGH: usize = 10;

#[derive(Debug, PartialEq, Eq)]
enum Asking {
    Start,
    Wait,
    Now,
}

fn asking(
    asked: Option<bool>,
    since: Option<jiff::Timestamp>,
    now: jiff::Timestamp,
    papers: usize,
    closed: impl FnOnce() -> usize,
) -> Asking {
    if asked.unwrap_or(false) {
        return Asking::Wait;
    }
    let Some(since) = since.filter(|at| *at <= now) else {
        return Asking::Start;
    };
    if now.duration_since(since) < SETTLED_IN {
        return Asking::Wait;
    }
    if papers >= PAPERS_ENOUGH {
        return Asking::Now;
    }
    if closed() < CLOSED_ENOUGH {
        return Asking::Wait;
    }
    Asking::Now
}

fn offering(seen: usize, wired: usize) -> bool {
    seen > 0 && wired == 0
}

#[tauri::command]
fn reachable() -> command::Reach {
    command::reach()
}

#[tauri::command]
fn take_out_of_reach() -> Answer<command::Reach> {
    command::out_of_reach().map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    Ok(command::reach())
}

/// A folder that already holds a store is the meeting place itself; anywhere else we hang ours
/// inside, so pointing at Documents does not scatter the store through it.
fn room(at: &std::path::Path) -> std::path::PathBuf {
    if tisty_sync::theirs(at).is_some() || at.join(tisty_sync::STORE).is_dir() {
        return at.to_path_buf();
    }
    tisty_core::keepers::suggested(at)
}

type Placing = (Option<ulid::Ulid>, Option<ulid::Ulid>, String);

fn placed(beside: Option<Placing>, fresh: &str) -> Placing {
    match beside {
        Some((folder, page_of, order)) => (folder, page_of, tisty_core::order::after(&order)),
        None => (None, None, fresh.to_string()),
    }
}

fn said(trouble: tisty_sync::Trouble) -> Refusal {
    match trouble {
        tisty_sync::Trouble::NotThere(at) => Refusal::about("noMeetingPlace", at),
        tisty_sync::Trouble::OtherStore { theirs } => Refusal::about("otherStore", theirs),
        tisty_sync::Trouble::Newer(who) => Refusal::about("syncNewer", who),
        tisty_sync::Trouble::Unreadable(why) => Refusal::about("syncUnreadable", why),
        tisty_sync::Trouble::Refused(why) => Refusal::about("syncRefused", why),
        tisty_sync::Trouble::Broke(why) => Refusal::about("syncBroke", why),
        tisty_sync::Trouble::WouldReset { theirs } => Refusal::about("wouldReset", theirs),
        tisty_sync::Trouble::NotAllowed(who) => Refusal::about("notAllowed", who),
        tisty_sync::Trouble::Emptied(at) => Refusal::about("emptiedPlace", at),
        tisty_sync::Trouble::SameName(who) => {
            Refusal::about("sameName", tisty_core::config::nicknamed(&who))
        }
    }
}

fn within(at: &std::path::Path, ours: &[std::path::PathBuf]) -> bool {
    let Ok(real) = at.canonicalize() else {
        return false;
    };
    ours.iter()
        .filter_map(|one| one.canonicalize().ok())
        .any(|one| real.starts_with(&one))
}

fn show(at: &std::path::Path, said: &str) -> Answer<()> {
    tauri_plugin_opener::reveal_item_in_dir(at)
        .map_err(|_| Refusal::about("cannotOpen", said.to_string()))
}

fn weighed(bytes: u64) -> String {
    let units = ["B", "kB", "MB", "GB"];
    let mut step = 0;
    let mut left = bytes as f64;
    while left >= 1000.0 && step < units.len() - 1 {
        left /= 1000.0;
        step += 1;
    }
    if step == 0 {
        format!("{left:.0} {}", units[step])
    } else {
        format!("{left:.1} {}", units[step])
    }
}

// Erasing has no undo, so it is judged against the store as it is now — an agent or the
// terminal may have written since the window last looked.
fn erasing(session: &mut Session, id: tisty_core::TaskId) -> Answer<()> {
    session.reload()?;
    if !session.state.tasks.contains_key(&id) {
        return Err(Refusal::of("notATaskId"));
    }
    match session.state.erasable(id) {
        Ok(()) => {}
        Err(tisty_core::model::Stays::Open) => return Err(Refusal::of("onlyArchivedGoes")),
        Err(tisty_core::model::Stays::Story) => return Err(Refusal::of("storyStays")),
        Err(tisty_core::model::Stays::Routine) => return Err(Refusal::of("routineStays")),
    }
    session.commit(Op::TaskDelete { id })?;
    Ok(())
}

/// Only an open task the person wrote takes the permission: a closed one is history to an
/// assistant, and what an agent filed is the agents' already.
fn opening_to_agents(session: &mut Session, id: tisty_core::TaskId, open: bool) -> Answer<Task> {
    session.reload()?;
    let task = session
        .state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    if !task.is_open() {
        return Err(Refusal::of("onlyOpenOpens"));
    }
    if session.state.filed_by_agents(task) {
        return Err(Refusal::of("alreadyTheirs"));
    }
    session.commit(Op::TaskUpdate {
        id,
        d: TaskPatch {
            open_to_agents: Some(open),
            ..Default::default()
        },
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

fn reading_as(session: &mut Session, id: tisty_core::TaskId, how: Reading) -> Answer<Task> {
    session.reload()?;
    let task = session
        .state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    if task.is_open() {
        return Err(Refusal::of("onlyClosedConverts"));
    }
    if task.reading() == Reading::Routine {
        return Err(Refusal::of("routineReadsAsRoutine"));
    }
    session.commit(Op::TaskUpdate {
        id,
        d: TaskPatch {
            read_as: Some(Some(how)),
            ..Default::default()
        },
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

struct Perched(bool);

struct Bound(Option<String>);

fn listen_for<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Option<String> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let tries = [
        (
            "Ctrl+Shift+Space",
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space),
        ),
        #[cfg(not(target_os = "macos"))]
        (
            "Ctrl+Alt+Space",
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space),
        ),
        (
            "Ctrl+Shift+T",
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyT),
        ),
    ];

    for (said, combo) in tries {
        let handle = app.clone();
        let taken = app
            .global_shortcut()
            .on_shortcut(combo, move |_, _, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    tray::quicken(&handle);
                }
            });
        if taken.is_ok() {
            return Some(said.to_string());
        }
    }
    witness::warn(channel::WINDOW, "no shortcut was free", &[]);
    None
}

fn worded(locale: &Option<String>, key: &str) -> String {
    let spanish = locale
        .as_deref()
        .or(sys_locale::get_locale().as_deref())
        .is_some_and(|code| code.to_lowercase().starts_with("es"));

    match (key, spanish) {
        ("quit", true) => "Salir de Tisty".into(),
        ("quit", false) => "Quit Tisty".into(),
        ("edit", true) => "Edición".into(),
        ("edit", false) => "Edit".into(),
        ("windowMenu", true) => "Ventana".into(),
        ("windowMenu", false) => "Window".into(),
        ("copy", true) => " (copia)".into(),
        ("copy", false) => " (copy)".into(),
        ("show", true) => "Abrir Tisty".into(),
        ("show", false) => "Open Tisty".into(),
        ("capture", true) => "Capturar…".into(),
        ("capture", false) => "Capture…".into(),
        ("due", true) => "Recordatorio".into(),
        ("due", false) => "Reminder".into(),
        ("missed", true) => "{n} recordatorios mientras no estabas".into(),
        ("missed", false) => "{n} reminders while you were away".into(),
        (_, true) => "Salir de Tisty".into(),
        (_, false) => "Quit Tisty".into(),
    }
}

#[derive(Default)]
struct Updating(OneAtATime);

#[derive(Default)]
struct Packing(OneAtATime);

impl Packing {
    fn taken(&self) -> Answer<Releasing<'_>> {
        self.0.claim().ok_or_else(|| Refusal::of("stillPacking"))
    }
}

struct Releasing<'a>(&'a std::sync::atomic::AtomicBool);

impl Drop for Releasing<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}

#[derive(Default)]
struct OneAtATime(std::sync::atomic::AtomicBool);

impl OneAtATime {
    fn claim(&self) -> Option<Releasing<'_>> {
        use std::sync::atomic::Ordering;
        self.0
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            .then(|| Releasing(&self.0))
    }

    fn taken(&self) -> Answer<Releasing<'_>> {
        self.claim().ok_or_else(|| Refusal::of("stillCarrying"))
    }
}

pub fn unreach() -> std::io::Result<bool> {
    let _ = waking::wake(false);
    let reached = command::out_of_reach();
    if let Ok(paths) = tisty_core::Paths::resolve() {
        for at in paths.swept_on_leaving() {
            let _ = match at.is_dir() {
                true => std::fs::remove_dir_all(&at),
                false => std::fs::remove_file(&at),
            };
        }
    }
    for at in tisty_core::Paths::shims() {
        let _ = std::fs::remove_file(&at);
    }
    reached
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn settled(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let session = match Session::open() {
        Ok(session) => session,
        Err(why) => {
            use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
            witness::error(channel::WINDOW, "the session would not open", &why.told());
            for window in app.webview_windows().values() {
                let _ = window.close();
            }
            let behind = matches!(why, tisty_core::Error::UnsupportedVersion(_));
            let said = app
                .dialog()
                .message(if behind {
                    behind_said()
                } else {
                    why.to_string()
                })
                .kind(MessageDialogKind::Error)
                .title("Tisty");
            if behind {
                let (yes, no) = behind_buttons();
                if said
                    .buttons(MessageDialogButtons::OkCancelCustom(yes.into(), no.into()))
                    .blocking_show()
                {
                    let _ = tauri_plugin_opener::open_url(where_it_comes_from(), None::<&str>);
                }
            } else {
                said.blocking_show();
            }
            std::process::exit(1);
        }
    };

    for at in tisty_core::backup::leftovers(session.paths.data()) {
        if let Err(why) = std::fs::remove_dir_all(&at) {
            witness::warn(
                channel::BACKUP,
                "what a restore left behind could not be swept up",
                &[("at", Fact::Path(at)), ("why", Fact::Why(why.to_string()))],
            );
        }
    }

    let attachments = session.paths.attachments();
    if let Err(why) = std::fs::create_dir_all(&attachments) {
        witness::error(
            channel::ATTACH,
            "the attachments folder could not be made",
            &[
                ("at", Fact::Path(attachments.clone())),
                ("why", Fact::Why(why.to_string())),
            ],
        );
    }
    app.handle()
        .asset_protocol_scope()
        .allow_directory(&attachments, true)?;
    let words = tray::Words {
        show: worded(&session.locale, "show"),
        capture: worded(&session.locale, "capture"),
        quit: worded(&session.locale, "quit"),
    };
    let telling = herald::Words {
        due: worded(&session.locale, "due"),
        missed: worded(&session.locale, "missed"),
    };
    let watched = session.paths.clone();
    let quiet = session.config.muted().to_vec();
    // An update relaunches with the arguments it was started with, so a copy that opened
    // with the session comes back hidden — looking, to whoever pressed the button, like it
    // never came back at all.
    let came_back = session.config.found_version.as_deref() == Some(HERE);
    answers::settings::appearance(app.handle(), session.config.theme);
    app.manage(Mutex::new(session));
    app.manage(herald::Speaking::new(app.handle(), telling, &quiet));
    herald::watch(app.handle().clone(), watched);

    app.manage(answers::storing::Stopping::default());
    let perched = tray::raise(app.handle(), &words).is_some();
    app.manage(Perched(perched));
    app.manage(Bound(listen_for(app.handle())));

    {
        let held = app.state::<Mutex<Session>>();
        let held = crate::held(&held);
        let seen = app.asset_protocol_scope();
        // Its attachments and no more of it: the rest of that folder is not ours to read.
        let shared = match &held.config.sync {
            Some(tisty_core::config::Sync::Folder(dest)) => vec![dest.join("attachments")],
            _ => Vec::new(),
        };
        for at in [held.paths.attachments(), held.paths.docs()]
            .into_iter()
            .chain(shared)
        {
            if let Err(e) = seen.allow_directory(&at, true) {
                witness::warn(
                    channel::WINDOW,
                    "attachments will not show",
                    &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let locale = held(&app.state::<Mutex<Session>>()).locale.clone();
        match desktop::menued(app.handle(), &locale) {
            Ok(menu) => {
                let _ = app.set_menu(menu);
                app.on_menu_event(|app, event| {
                    if event.id() == "leave" {
                        desktop::parting(app);
                    }
                });
            }
            Err(e) => witness::warn(
                channel::WINDOW,
                "the menu would not build",
                &[("why", Fact::Why(e.to_string()))],
            ),
        }
    }

    if let Some(window) = app.get_webview_window("main") {
        desktop::proofread(&window);
        if came_back || !waking::hushed() {
            desktop::fitted(&window);
            let _ = window.show();
        }
    }
    Ok(())
}

pub fn run() {
    let mut building = tauri::Builder::default();

    if tisty_core::paths::profile().is_none() {
        building = building.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tray::surface(app);
        }));
    }

    building
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(settled)
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let tauri::WindowEvent::ThemeChanged(_) = event {
                tray::repaint(window.app_handle());
                return;
            }

            let tauri::WindowEvent::CloseRequested { api, .. } = event else {
                return;
            };

            let app = window.app_handle();
            if !app.state::<Perched>().0 {
                api.prevent_close();
                desktop::parting(app);
                return;
            }

            let asked = held(&app.state::<Mutex<Session>>()).config.on_close;
            match asked {
                Some(tisty_core::config::Closing::Quit) => {
                    api.prevent_close();
                    desktop::parting(app);
                }
                Some(tisty_core::config::Closing::Hide) => {
                    api.prevent_close();
                    let _ = window.emit("withdrawn", ());
                    let _ = window.hide();
                }
                None => {
                    api.prevent_close();
                    let _ = window.emit("closing", ());
                }
            }
        })
        .manage(OneAtATime::default())
        .manage(Packing::default())
        .manage(Updating::default())
        .manage(desktop::Leaving::default())
        .manage(desktop::Departed::default())
        .invoke_handler(tauri::generate_handler![
            answers::tasks::snapshot,
            answers::storing::keepers,
            answers::storing::keeper_of,
            answers::storing::strays_at,
            answers::storing::make_room,
            answers::storing::glimpse_kept,
            answers::storing::glimpse_fetch,
            answers::shelves::sow_lists,
            answers::tasks::task_story,
            answers::tasks::task_series,
            answers::tasks::task_left,
            answers::tasks::routines,
            answers::agents::agent,
            answers::agents::agent_turn,
            answers::tasks::archive_shape,
            answers::settings::close_window,
            answers::settings::shortcut,
            answers::storing::settle_in,
            reachable,
            take_out_of_reach,
            answers::storing::free_up,
            answers::storing::stop_freeing,
            answers::wired::wiring,
            answers::agents::assistants,
            answers::wired::wire,
            answers::wired::unwire,
            answers::wired::waking,
            answers::wired::wake_for,
            answers::settings::keep_locale,
            answers::settings::keep_closing,
            answers::settings::keep_theme,
            answers::tasks::erase,
            answers::papers::read_as,
            answers::tasks::open_to_agents,
            answers::settings::guide,
            answers::tasks::capture,
            answers::tasks::read,
            search,
            answers::attaching::complete,
            answers::attaching::owed,
            answers::tasks::reopen,
            answers::tasks::patch,
            answers::tasks::write_step,
            answers::tasks::mark_step,
            answers::tasks::drop_step,
            answers::tasks::write_log,
            answers::tasks::fold,
            answers::tasks::still_open,
            answers::tasks::discard,
            answers::attaching::attach,
            answers::attaching::served,
            answers::attaching::attached,
            answers::attaching::attach_export,
            answers::attaching::weighs,
            answers::storing::roomy,
            answers::attaching::opened,
            answers::storing::revealed,
            answers::storing::sync_state,
            answers::storing::choose_sync,
            answers::storing::sync_now,
            answers::storing::back_up,
            answers::storing::restore,
            answers::storing::join_them,
            answers::storing::take_over,
            answers::storing::merge_stores,
            answers::storing::sync_kin,
            answers::storing::joining,
            answers::storing::folder_astir,
            answers::storing::remove_machine,
            answers::storing::retire_attachment,
            answers::papers::settle_paper,
            answers::storing::paper_rifts,
            answers::storing::weave_paper,
            answers::papers::convert_paper,
            answers::reporting::checked,
            answers::reporting::twinned,
            answers::reporting::rebuild,
            answers::settings::about,
            answers::settings::notices,
            answers::settings::settings,
            answers::settings::keep_settings,
            answers::reporting::facts,
            answers::reporting::keep_report,
            answers::tasks::note_trouble,
            answers::tasks::note_break,
            answers::updating::update_ready,
            answers::updating::update_install,
            answers::updating::update_candidates,
            answers::tasks::star_due,
            answers::tasks::star_done,
            answers::tasks::door_due,
            answers::tasks::door_done,
            answers::tasks::logs,
            answers::settings::icons,
            answers::settings::families,
            answers::shelves::list_add,
            answers::shelves::list_look,
            answers::shelves::list_rename,
            answers::shelves::list_drop,
            answers::papers::docs,
            answers::papers::docs_catch_up,
            read_tags,
            answers::shelves::folder_add,
            answers::shelves::folder_rename,
            answers::shelves::folder_look,
            answers::shelves::folder_drop,
            answers::papers::doc_file,
            answers::papers::doc_read,
            answers::papers::doc_facts,
            answers::papers::keep_pdf,
            answers::papers::doc_back,
            answers::papers::doc_backable,
            answers::papers::doc_write,
            answers::papers::doc_order,
            answers::papers::doc_new,
            answers::papers::doc_page,
            answers::papers::doc_drop,
            answers::carrying::doc_import,
            answers::carrying::doc_export,
            answers::papers::signed,
            answers::papers::sign,
            answers::papers::sign_the_rest,
            answers::papers::spelled,
            answers::carrying::docs_pack,
            answers::carrying::docs_take_out,
            answers::carrying::docs_unpack,
            answers::carrying::doc_copy,
            answers::papers::doc_adopt,
            answers::papers::doc_let_go,
            answers::storing::retire_attachments,
            answers::papers::doc_away,
            answers::papers::doc_unflag,
            answers::shelves::folder_away,
            answers::papers::doc_lock,
            desktop::parted,
            answers::shelves::sow,
            answers::shelves::folder_file
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, _event| {
            #[cfg(target_os = "macos")]
            if matches!(_event, tauri::RunEvent::Reopen { .. }) {
                tray::surface(_app);
            }
        });
}

#[cfg(test)]
#[path = "lib_copying.rs"]
mod copying;

#[cfg(test)]
#[path = "lib_hanging_again.rs"]
mod hanging_again;

#[cfg(test)]
#[path = "lib_going_back.rs"]
mod going_back;

#[cfg(test)]
#[path = "lib_deleting.rs"]
mod deleting;

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;

#[cfg(test)]
#[path = "lib_behind_tests.rs"]
mod behind_tests;

#[cfg(test)]
#[path = "lib_hands_of.rs"]
mod hands_of;

#[cfg(test)]
#[path = "lib_starring.rs"]
mod starring;

#[cfg(test)]
#[path = "lib_doors.rs"]
mod doors;

#[cfg(test)]
#[path = "lib_hosting.rs"]
mod hosting;

#[cfg(test)]
#[path = "lib_letting_go.rs"]
mod letting_go;

#[cfg(test)]
#[path = "lib_ordering.rs"]
mod ordering;

#[cfg(test)]
#[path = "lib_restored.rs"]
mod restored;
