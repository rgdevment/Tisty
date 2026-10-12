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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shown {
    File(PathBuf),
    Doc { file: String, title: String },
    Widget(Lent),
}

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Showing {
    File { name: String, body: String },
    Doc { file: String, title: String },
    Widget { lent: Lent },
}

#[derive(Default)]
pub struct Views {
    shown: Mutex<HashMap<String, Shown>>,
    made: AtomicU64,
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

// Windows wedges a window built from inside the event loop, so the windows are built from a thread of their own.
pub fn heard(app: &AppHandle, argv: &[String], cwd: Option<&Path>) -> bool {
    let wanted: Vec<PathBuf> = argv
        .iter()
        .skip(1)
        .filter_map(|one| asked(one, cwd))
        .collect();
    if wanted.is_empty() {
        return false;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        for at in wanted {
            open(&app, Shown::File(at));
        }
    });
    true
}

#[cfg(target_os = "macos")]
pub fn heard_links(app: &AppHandle, links: &[url::Url]) {
    let said: Vec<String> = std::iter::once(String::new())
        .chain(links.iter().map(|one| one.to_string()))
        .collect();
    if !heard(app, &said, None) {
        witness::warn(
            channel::WINDOW,
            "a link Tisty was asked to open was not one it opens",
            &[],
        );
    }
}

pub fn open(app: &AppHandle, shown: Shown) {
    let views = app.state::<Views>();
    let label = format!("view-{}", views.made.fetch_add(1, Ordering::Relaxed));
    let title = match &shown {
        Shown::File(at) => named(at),
        Shown::Doc { title, .. } => title.clone(),
        Shown::Widget(_) => "Tisty".into(),
    };
    if let Ok(mut held) = views.shown.lock() {
        held.insert(label.clone(), shown);
    }
    let made = tauri::WebviewWindowBuilder::new(
        app,
        &label,
        tauri::WebviewUrl::App(
            match crate::refusing::speaks_spanish() {
                true => "index.html?locale=es",
                false => "index.html?locale=en",
            }
            .into(),
        ),
    )
    .title(title)
    .inner_size(960.0, 780.0)
    .min_inner_size(480.0, 360.0)
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
                if matches!(event, tauri::WindowEvent::Destroyed)
                    && let Ok(mut held) = gone.state::<Views>().shown.lock()
                {
                    held.remove(&named);
                }
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

#[tauri::command(async)]
pub fn view_shown(window: tauri::WebviewWindow, views: tauri::State<'_, Views>) -> Answer<Showing> {
    Ok(match shown_in(&views, window.label())? {
        Shown::File(at) => Showing::File {
            name: named(&at),
            body: crate::answers::carrying::read_outside(&at)?,
        },
        Shown::Doc { file, title } => Showing::Doc { file, title },
        Shown::Widget(lent) => Showing::Widget { lent },
    })
}

#[tauri::command(async)]
pub fn view_import(
    app: AppHandle,
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
) -> Answer<String> {
    let Shown::File(at) = shown_in(&views, window.label())? else {
        return Err(Refusal::of("notAllowed"));
    };
    let session = app
        .try_state::<Mutex<Session>>()
        .ok_or_else(|| Refusal::of("internal"))?;
    let body = crate::answers::carrying::read_outside(&at)?;
    let made = crate::answers::carrying::imported(&mut held(&session), &body, None)?;
    let _ = app.emit_to("main", "showdoc", made.id.clone());
    crate::tray::surface(&app);
    Ok(made.id)
}

#[tauri::command(async)]
pub fn view_widget(app: AppHandle, lent: Lent) -> Answer<()> {
    match &lent {
        Lent::Source(source) if source.len() > widgets::LENT_AT_MOST => {
            return Err(Refusal::about(
                "widgetTooBig",
                crate::weighed(widgets::LENT_AT_MOST as u64),
            ));
        }
        Lent::Reference(reference) if !widgets::a_page(reference) => {
            return Err(Refusal::of("notAllowed"));
        }
        _ => {}
    }
    open(&app, Shown::Widget(lent));
    Ok(())
}

#[tauri::command(async)]
pub fn view_doc(app: AppHandle, file: String, title: String) -> Answer<()> {
    open(&app, Shown::Doc { file, title });
    Ok(())
}

#[cfg(test)]
#[path = "viewing_test.rs"]
mod tests;
