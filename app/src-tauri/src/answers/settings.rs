use std::sync::Mutex;

pub const WRITTEN_BY: &str = tisty_core::untouched::SOWN_BY;

pub const ATTACHED: &[&str] = &[
    "captura.png",
    "prioridades.png",
    "capture.png",
    "priorities.png",
    "rina.jpg",
    "roger.jpg",
    "cartilla.pdf",
    "vet-card.pdf",
    "bark.mp3",
    "walk.mp4",
    "paseos.csv",
    "walks.csv",
    "paseos.html",
    "walks.html",
    "idiomas.png",
    "languages.png",
    "diagramas.png",
    "diagrams.png",
];

pub const GUIDE_PAGES_ES: &[(&str, &str)] = &[
    (
        "tisty:sample",
        include_str!("../../resources/guide/es/muestra.md"),
    ),
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
        "tisty:sample",
        include_str!("../../resources/guide/en/sample.md"),
    ),
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
pub const LICENCES: &str = include_str!("../../../../THIRD-PARTY-LICENSES.md");

use tauri::Manager;
use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, HERE, Refusal, Session, desktop, held, herald, language, update};

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
    /// Installed where the Store installs, by other means: nobody brings this copy updates.
    not_sold: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    quiet: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    theme: Option<tisty_core::config::Theme>,
    holds: tisty_core::config::Holds,
    /// Whether the choice means anything here: without a shared folder there is nowhere else.
    shares: bool,
    only_shared_above: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    on_close: Option<tisty_core::config::Closing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    quick_keys: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    idea_keys: Option<String>,
}

#[tauri::command]
pub fn settings(session: tauri::State<'_, Mutex<Session>>) -> Answer<Settings> {
    let session = held(&session);
    Ok(as_settings(&session))
}

fn as_settings(session: &Session) -> Settings {
    Settings {
        quiet: session.config.muted().to_vec(),
        locale: session.config.locale.clone(),
        theme: session.config.theme,
        holds: session.config.holds.unwrap_or_default(),
        shares: session.config.shares(),
        only_shared_above: session.config.only_shared_above(),
        on_close: session.config.on_close,
        quick_keys: session.config.quick_keys.clone(),
        idea_keys: session.config.idea_keys.clone(),
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
    let holds = settings.holds;
    session.keep(|config| {
        config.quiet = (!quiet.is_empty()).then_some(quiet);
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
    own: Option<bool>,
) -> Answer<tisty_core::docs::Doc> {
    let mut session = held(&session);
    let code = tisty_core::model::spoken(session.locale.as_deref());
    let tongue = if code.starts_with("es") { "es" } else { "en" };
    let [english, castilian] = tisty_core::untouched::GUIDE_FOLDERS;
    let called = if tongue == "es" { castilian } else { english };
    let told = if tongue == "es" { GUIDE_ES } else { GUIDE_EN };

    let standing = session
        .config
        .guide
        .clone()
        .and_then(|kept| session.state.papers().find(|one| one.file == kept))
        .map(|one| {
            (
                one.id,
                one.file.clone(),
                one.title.clone().unwrap_or_default(),
            )
        });
    let reach = |one: &str| {
        app.path()
            .resolve(
                format!("resources/guide/{one}"),
                tauri::path::BaseDirectory::Resource,
            )
            .ok()
    };
    if let Some(found) = super::guiding::signed_guide_here(&mut session, tongue)? {
        return Ok(found);
    }
    if let Some(author) = super::guiding::signed_author(&reach) {
        return super::guiding::signed_guide(&mut session, tongue, called, &author, reach);
    }
    let found = match (standing, own) {
        (Some((id, file, title)), Some(true)) => {
            let here = guide_here_as(&session, &tisty_core::docs::titled(told))
                .or_else(|| in_their_tongue(&session, &file, tongue));
            match here {
                Some(found) => Some(found),
                // An older guide in another language gets this one beside it, as asking for it always did.
                None if title != tisty_core::docs::titled(told) => {
                    return super::guiding::sown_beside(
                        &mut session,
                        tongue,
                        id,
                        WRITTEN_BY,
                        reach,
                    );
                }
                None => Some((file, title)),
            }
        }
        (Some((_, file, title)), _) => Some((file, title)),
        (None, Some(true)) => guide_here_as(&session, &tisty_core::docs::titled(told)),
        (None, _) => guide_already_here(&session),
    };
    if let Some((file, title)) = found {
        let taken = file.clone();
        session.keep(|config| config.guide = Some(taken))?;
        return Ok(tisty_core::docs::Doc { id: file, title });
    }
    super::guiding::sown(&mut session, tongue, called, WRITTEN_BY, reach)
}

// The same guide in the person's language, when the one kept is another of its languages.
fn in_their_tongue(session: &Session, file: &str, tongue: &str) -> Option<(String, String)> {
    let cover = session.state.papers().find(|one| one.file == file)?;
    session
        .state
        .tongues_of(cover.id)
        .into_iter()
        .find(|one| one.lang.as_deref() == Some(tongue))
        .map(|one| (one.file.clone(), one.title.clone().unwrap_or_default()))
}

#[tauri::command]
pub fn notices() -> &'static str {
    NOTICES
}

#[tauri::command]
pub fn licences() -> &'static str {
    LICENCES
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
        candidates_apply: update::self_installs(update::route().route),
        kept_by_the_store: update::route().route == update::Route::Store,
        not_sold: update::route().route == update::Route::Store && !crate::shop::sold(),
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
        "hide" => Some(tisty_core::config::Closing::Hide),
        "quit" => Some(tisty_core::config::Closing::Quit),
        "ask" => None,
        _ => return Err(Refusal::of("notAClosing")),
    };
    held(&session).keep(|config| config.on_close = how)?;
    Ok(())
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
        tisty_core::config::Closing::Hide => crate::tray::withdraw(&window),
        tisty_core::config::Closing::Quit => desktop::parting(window.app_handle()),
    }
    Ok(())
}

pub(crate) fn guide_here_as(session: &Session, title: &str) -> Option<(String, String)> {
    session
        .state
        .docs
        .values()
        .filter(|one| one.page_of.is_none())
        .find(|one| one.title.as_deref() == Some(title))
        .map(|one| (one.file.clone(), title.to_string()))
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
