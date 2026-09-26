use std::sync::Mutex;

pub const WRITTEN_BY: &str = "Tisty";

pub const PICTURES: &[&str] = &[
    "captura.png",
    "prioridades.png",
    "capture.png",
    "priorities.png",
    "rina.jpg",
];

pub const GUIDE_PAGES_ES: &[(&str, &str)] = &[
    (
        "tisty:code",
        include_str!("../../resources/guide/es/codigo.md"),
    ),
    (
        "tisty:page",
        include_str!("../../resources/guide/es/pagina.md"),
    ),
];

pub const GUIDE_PAGES_EN: &[(&str, &str)] = &[
    (
        "tisty:code",
        include_str!("../../resources/guide/en/code.md"),
    ),
    (
        "tisty:page",
        include_str!("../../resources/guide/en/page.md"),
    ),
];

pub const GUIDE_ES: &str = include_str!("../../resources/guide/es/guia.md");

pub const GUIDE_EN: &str = include_str!("../../resources/guide/en/guide.md");

pub const NOTICES: &str = include_str!("../../../../THIRD-PARTY-BUNDLED.md");

use tauri::{Emitter, Manager};
use tisty_core::Op;
use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Bound, HERE, Refusal, Session, held, herald, language, parting, update};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct About {
    version: String,
    sandbox: Option<String>,
    repository: &'static str,
    license: &'static str,
    store: String,
    candidates: bool,
    candidates_apply: bool,
    /// Kept by the Microsoft Store: updates come from it alone, and «none» means it has none yet.
    kept_by_the_store: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    quiet: Vec<String>,
    attach_up_to: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    theme: Option<tisty_core::config::Theme>,
    holds: tisty_core::config::Holds,
    /// Whether the choice means anything here: without a shared folder there is nowhere else.
    shares: bool,
    only_shared_above: u64,
}

#[tauri::command]
pub fn settings(session: tauri::State<'_, Mutex<Session>>) -> Answer<Settings> {
    let session = held(&session);
    Ok(as_settings(&session))
}

fn as_settings(session: &Session) -> Settings {
    Settings {
        quiet: session.config.muted().to_vec(),
        attach_up_to: session.config.copies_up_to(),
        locale: session.config.locale.clone(),
        theme: session.config.theme,
        holds: session.config.holds.unwrap_or_default(),
        shares: !session.config.backs_up(),
        only_shared_above: session.config.only_shared_above(),
    }
}

#[tauri::command]
pub fn keep_settings(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    settings: Settings,
) -> Answer<Settings> {
    let mut session = held(&session);
    let quiet = settings.quiet.clone();
    let up_to = settings.attach_up_to.clamp(
        tisty_core::attach::COPIED_LEAST,
        tisty_core::attach::COPIED_MOST,
    );
    let holds = settings.holds;
    session.keep(|config| {
        config.quiet = (!quiet.is_empty()).then_some(quiet);
        config.attach_up_to = Some(up_to);
        config.holds = Some(holds);
    })?;
    let now = as_settings(&session);
    drop(session);
    herald::respeak(&app, &now.quiet);
    Ok(now)
}

#[tauri::command]
pub fn icons() -> Vec<&'static str> {
    let mut all = tisty_core::model::mark::MARKS.to_vec();
    all.extend_from_slice(tisty_core::model::icon::ICONS);
    all
}

#[tauri::command]
pub fn families() -> Vec<(&'static str, usize)> {
    let mut all = tisty_core::model::mark::MARK_FAMILIES.to_vec();
    all.extend_from_slice(tisty_core::model::icon::FAMILIES);
    all
}

#[tauri::command]
pub fn guide(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
) -> Answer<tisty_core::docs::Doc> {
    let tongue = {
        let session = held(&session);
        let code = tisty_core::model::spoken(session.locale.as_deref());
        if code.starts_with("es") { "es" } else { "en" }
    };
    let called = if tongue == "es" { "Guía" } else { "Guide" };
    let told = if tongue == "es" { GUIDE_ES } else { GUIDE_EN };
    let leaves = if tongue == "es" {
        GUIDE_PAGES_ES
    } else {
        GUIDE_PAGES_EN
    };

    let from = app
        .path()
        .resolve(
            format!("resources/guide/{tongue}"),
            tauri::path::BaseDirectory::Resource,
        )
        .ok();

    let mut session = held(&session);

    if let Some(kept) = session.config.guide.clone() {
        let root = session.paths.docs();
        let standing = session.state.docs.values().any(|one| one.file == kept);
        if standing && let Ok(body) = tisty_core::docs::read(&root, &kept) {
            return Ok(tisty_core::docs::Doc {
                id: kept,
                title: tisty_core::docs::titled(&body),
            });
        }
    }

    if let Some((file, title)) = guide_already_here(&session) {
        let taken = file.clone();
        session.keep(|config| config.guide = Some(taken))?;
        return Ok(tisty_core::docs::Doc { id: file, title });
    }

    let data = session.paths.data().to_path_buf();

    let mut body = told.to_string();
    let mut pages: Vec<(&str, String)> = leaves
        .iter()
        .map(|(named, one)| (*named, one.to_string()))
        .collect();
    for shot in PICTURES {
        let Some(at) = from
            .as_ref()
            .map(|dir| dir.join(shot))
            .filter(|at| at.is_file())
        else {
            continue;
        };
        let kept = tisty_core::attach::keep(&at, &data, tisty_core::attach::COPIED_IN_DOC)
            .map_err(|e| Refusal::about("cannotRead", e.to_string()))?;
        let named = format!("](<{}>)", kept.at);
        body = body.replace(&format!("]({shot})"), &named);
        for (_, one) in pages.iter_mut() {
            *one = one.replace(&format!("]({shot})"), &named);
        }
    }

    let folder = ulid::Ulid::generate();
    let order = tisty_core::order::last_of(
        session
            .state
            .under(None)
            .iter()
            .map(|one| one.order.as_str()),
    );
    session.commit(Op::FolderAdd {
        id: folder,
        d: tisty_core::event::FolderAdd {
            name: called.to_string(),
            order,
            parent: None,
            icon: None,
            color: None,
        },
    })?;

    let root = session.paths.docs();
    let device = session.store.device().clone();
    let mut leafed = Vec::new();
    for (marker, one) in &pages {
        let made = tisty_core::docs::create(&root, &device, one)
            .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
        body = body.replace(
            &format!("]({marker})"),
            &format!("]({}{})", tisty_core::refs::DOC, made.id),
        );
        leafed.push(made.id);
    }
    let made = tisty_core::docs::create(&root, &device, &body)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;

    let sorted = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.folder == Some(folder))
            .map(|one| one.order.as_str()),
    );
    session.commit(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: true,
            made: None,
            by: Some(WRITTEN_BY.into()),
            file: made.id.clone(),
            order: sorted,
            said: Some(tisty_core::event::Said {
                title: made.title.clone(),
                bytes: None,
                tags: Some(Vec::new()),
                by: None,
            }),
            folder: Some(folder),
            page_of: None,
        },
    })?;
    let held = session
        .state
        .docs
        .values()
        .find(|one| one.file == made.id)
        .map(|one| one.id);
    if let Some(up) = held {
        let mut order = tisty_core::order::first();
        let shelf = session.paths.docs();
        for file in &leafed {
            let said = tisty_core::docs::read(&shelf, file)
                .ok()
                .map(|body| tisty_core::event::Said::of(&body));
            session.commit(Op::DocAdd {
                id: ulid::Ulid::generate(),
                d: tisty_core::event::DocAdd {
                    wrote: None,
                    guest: true,
                    made: None,
                    by: Some(WRITTEN_BY.into()),
                    file: file.clone(),
                    order: order.clone(),
                    said,
                    folder: Some(folder),
                    page_of: Some(up),
                },
            })?;
            order = tisty_core::order::after(&order);
        }
    }
    let written = made.id.clone();
    session.keep(|c| c.guide = Some(written))?;

    Ok(made)
}

#[tauri::command]
pub fn notices() -> &'static str {
    NOTICES
}

#[tauri::command]
pub fn about(session: tauri::State<'_, Mutex<Session>>) -> Answer<About> {
    let session = held(&session);
    Ok(About {
        version: env!("CARGO_PKG_VERSION").to_string(),
        sandbox: tisty_core::paths::profile(),
        repository: "https://github.com/rgdevment/Tisty",
        license: "AGPL-3.0-only",
        store: session.paths.store().display().to_string(),
        candidates: update::tracking(HERE, session.config.candidates),
        // The Store keeps its own tracks, and offers no candidates at all: a box here would be a
        // switch wired to nothing.
        candidates_apply: update::takes_candidates(update::route().route),
        kept_by_the_store: update::route().route == update::Route::Store,
    })
}

#[tauri::command]
pub fn keep_locale(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    locale: Option<String>,
) -> Answer<Option<String>> {
    let mut session = held(&session);
    let wanted = locale.filter(|one| !one.trim().is_empty());
    session.keep(|config| config.locale = wanted.clone())?;
    session.locale = wanted.clone();
    language(&app, &wanted);
    Ok(wanted)
}

#[tauri::command]
pub fn keep_theme(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    theme: Option<String>,
) -> Answer<Option<tisty_core::config::Theme>> {
    let wanted = match theme
        .as_deref()
        .map(str::trim)
        .filter(|one| !one.is_empty())
    {
        None => None,
        Some(said) => Some(
            said.parse::<tisty_core::config::Theme>()
                .map_err(|_| Refusal::of("notATheme"))?,
        ),
    };
    held(&session).keep(|config| config.theme = wanted)?;
    appearance(&app, wanted);
    Ok(wanted)
}

/// The window's own theme is what the webview reads `prefers-color-scheme` from, so the
/// page repaints by itself; absent, the window goes back to following the computer.
pub fn appearance<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    theme: Option<tisty_core::config::Theme>,
) {
    let wanted = theme.map(|one| match one {
        tisty_core::config::Theme::Light => tauri::Theme::Light,
        tisty_core::config::Theme::Dark => tauri::Theme::Dark,
    });
    for window in app.webview_windows().values() {
        if let Err(why) = window.set_theme(wanted) {
            witness::warn(
                channel::WINDOW,
                "the window would not take the theme",
                &[("why", Fact::Why(why.to_string()))],
            );
        }
    }
}

#[tauri::command]
pub fn keep_closing(session: tauri::State<'_, Mutex<Session>>, how: String) -> Answer<()> {
    let how = match how.as_str() {
        "hide" => tisty_core::config::Closing::Hide,
        "quit" => tisty_core::config::Closing::Quit,
        _ => return Err(Refusal::of("notAClosing")),
    };
    held(&session).keep(|config| config.on_close = Some(how))?;
    Ok(())
}

#[tauri::command]
pub fn shortcut(bound: tauri::State<'_, Bound>) -> Option<String> {
    bound.0.clone()
}

#[tauri::command]
pub fn close_window(
    window: tauri::Window,
    session: tauri::State<'_, Mutex<Session>>,
    how: Option<String>,
    remember: Option<bool>,
) -> Answer<()> {
    let how = match how.as_deref() {
        Some("hide") => tisty_core::config::Closing::Hide,
        Some("quit") => tisty_core::config::Closing::Quit,
        _ => return Ok(()),
    };

    if remember == Some(true) {
        held(&session).keep(|c| c.on_close = Some(how))?;
    }
    match how {
        tisty_core::config::Closing::Hide => {
            let _ = window.emit("withdrawn", ());
            let _ = window.hide();
        }
        tisty_core::config::Closing::Quit => parting(window.app_handle()),
    }
    Ok(())
}

/// The guide another machine planted arrives through the shared folder like any other document,
/// and planting a second one would leave a copy per machine.
pub fn guide_already_here(session: &Session) -> Option<(String, String)> {
    let named: Vec<String> = [GUIDE_ES, GUIDE_EN]
        .iter()
        .map(|one| tisty_core::docs::titled(one))
        .collect();
    session
        .state
        .docs
        .values()
        .filter(|one| one.page_of.is_none())
        .find(|one| {
            one.title
                .as_ref()
                .is_some_and(|title| named.iter().any(|one| one == title))
        })
        .map(|one| (one.file.clone(), one.title.clone().unwrap_or_default()))
}
