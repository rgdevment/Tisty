use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use tauri::{AppHandle, Emitter, Manager, Wry, ipc::Invoke};
use tisty_core::witness::{self, Fact, channel};

use crate::{Refusal, Session, held};

pub(crate) const LABEL: &str = "brainsty";
const PLACE_FILE: &str = "brainsty-window.json";
const OPENED: &str = "brainsty-open";
const SPOKEN: &str = "spoken";
const WIDE: f64 = 1280.0;
const TALL: f64 = 820.0;
const LEAST_WIDE: f64 = 640.0;
const LEAST_TALL: f64 = 480.0;
const GRIP_WIDE: f64 = 120.0;
const GRIP_TALL: f64 = 40.0;

const ASKED: [&str; 10] = [
    "brain_network",
    "brainsty_groups",
    "brainsty_open",
    "doc_glance",
    "doc_drawn",
    "cards_drawn",
    "served",
    "draws_heic",
    "note_break",
    "note_trouble",
];

pub(crate) fn allowed(label: &str, command: &str) -> bool {
    label != LABEL || ASKED.contains(&command)
}

pub fn kept_apart<H>(handler: H) -> impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static
where
    H: Fn(Invoke<Wry>) -> bool + Send + Sync + 'static,
{
    move |invoke: Invoke<Wry>| {
        let label = invoke.message.webview().label().to_string();
        let command = invoke.message.command().to_string();
        if allowed(&label, &command) {
            return handler(invoke);
        }
        witness::warn(
            channel::WINDOW,
            "Brainsty's window asked for a command it does not use",
            &[("command", Fact::Id(command))],
        );
        invoke.resolver.reject(Refusal::of("notAllowed"));
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Place {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub maximized: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Screen {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Seen {
    pub place: Place,
    pub minimized: bool,
    pub maximized: bool,
}

fn gripped(place: &Place, screen: &Screen) -> bool {
    let wide = (place.x + place.width).min(screen.x + screen.width) - place.x.max(screen.x);
    wide >= GRIP_WIDE && place.y >= screen.y && place.y + GRIP_TALL <= screen.y + screen.height
}

// A spot on a screen since unplugged is let go, so the window never opens where nobody sees it.
pub(crate) fn kept_place(saved: Option<Place>, screens: &[Screen]) -> Option<Place> {
    saved.filter(|place| {
        [place.x, place.y, place.width, place.height]
            .iter()
            .all(|one| one.is_finite())
            && place.width > 0.0
            && place.height > 0.0
            && screens.iter().any(|screen| gripped(place, screen))
    })
}

// Minimized, Windows puts the window far off the screens; maximized, the size is the screen's.
pub(crate) fn after(was: Option<Place>, seen: Seen) -> Option<Place> {
    if seen.minimized {
        return was;
    }
    if seen.maximized {
        return Some(Place {
            maximized: true,
            ..was.unwrap_or(seen.place)
        });
    }
    Some(Place {
        maximized: false,
        ..seen.place
    })
}

pub(crate) fn address(spanish: bool, near: Option<&str>) -> String {
    let page = crate::refusing::page(spanish);
    match near.filter(|one| !one.is_empty()) {
        Some(one) => format!("{page}&near={}", utf8_percent_encode(one, NON_ALPHANUMERIC)),
        None => page.to_string(),
    }
}

pub(crate) fn read_place(at: &Path) -> Option<Place> {
    serde_json::from_slice(&std::fs::read(at).ok()?).ok()
}

pub(crate) fn keep_place(at: &Path, place: Place) {
    let written = serde_json::to_vec(&place)
        .map_err(|e| e.to_string())
        .and_then(|bytes| std::fs::write(at, bytes).map_err(|e| e.to_string()));
    if let Err(why) = written {
        witness::warn(
            channel::WINDOW,
            "Brainsty's window would not keep its place",
            &[("why", Fact::Why(why))],
        );
    }
}

fn screens(app: &AppHandle) -> Vec<Screen> {
    app.available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|monitor| {
            let area = monitor.work_area();
            let scale = monitor.scale_factor();
            let at = area.position.to_logical::<f64>(scale);
            let size = area.size.to_logical::<f64>(scale);
            Screen {
                x: at.x,
                y: at.y,
                width: size.width,
                height: size.height,
            }
        })
        .collect()
}

fn seen_of(window: &tauri::WebviewWindow) -> Option<Seen> {
    let scale = window.scale_factor().ok()?;
    let at = window.outer_position().ok()?.to_logical::<f64>(scale);
    let size = window.inner_size().ok()?.to_logical::<f64>(scale);
    Some(Seen {
        place: Place {
            x: at.x,
            y: at.y,
            width: size.width,
            height: size.height,
            maximized: false,
        },
        minimized: window.is_minimized().unwrap_or(false),
        maximized: window.is_maximized().unwrap_or(false),
    })
}

fn place_file(app: &AppHandle) -> Option<PathBuf> {
    let session = app.try_state::<Mutex<Session>>()?;
    Some(held(&session).paths.cache().join(PLACE_FILE))
}

fn theme(app: &AppHandle) -> Option<tauri::Theme> {
    let chosen = app
        .try_state::<Mutex<Session>>()
        .and_then(|session| held(&session).config.theme);
    crate::answers::settings::window_theme(chosen)
}

fn open(app: &AppHandle, near: Option<&str>) -> tauri::Result<()> {
    let kept_at = place_file(app);
    let restored = kept_place(kept_at.as_deref().and_then(read_place), &screens(app));
    let page = address(crate::viewing::spanish(app), near);
    let building = crate::engine::window(app, LABEL, tauri::WebviewUrl::App(page.into()))
        .title("Brainsty")
        .min_inner_size(LEAST_WIDE, LEAST_TALL)
        .theme(theme(app))
        .visible(false);
    let window = match restored {
        Some(place) => building
            .inner_size(place.width, place.height)
            .position(place.x, place.y),
        None => building.inner_size(WIDE, TALL).center(),
    }
    .build()?;
    if restored.is_some_and(|place| place.maximized) {
        let _ = window.maximize();
    }
    let late = window.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(2));
        if !late.is_visible().unwrap_or(false) {
            crate::viewing::fronted(&late);
        }
    });
    crate::tray::docked(app, true);
    let tracked = Arc::new(Mutex::new(
        seen_of(&window).and_then(|seen| after(restored, seen)),
    ));
    let gone = app.clone();
    window.on_window_event(move |event| match event {
        tauri::WindowEvent::Moved(_) | tauri::WindowEvent::Resized(_) => {
            if let Some(seen) = gone.get_webview_window(LABEL).and_then(|one| seen_of(&one)) {
                let mut place = tracked.lock().unwrap_or_else(PoisonError::into_inner);
                *place = after(*place, seen);
            }
        }
        tauri::WindowEvent::Destroyed => {
            let place = *tracked.lock().unwrap_or_else(PoisonError::into_inner);
            if let (Some(at), Some(place)) = (&kept_at, place) {
                keep_place(at, place);
            }
            crate::tray::undock_unless_seen(&gone, LABEL);
        }
        _ => {}
    });
    Ok(())
}

#[tauri::command(async)]
pub fn brainsty_show(app: AppHandle, near: Option<String>) {
    if let Some(window) = app.get_webview_window(LABEL) {
        crate::viewing::fronted(&window);
        return;
    }
    if let Err(why) = open(&app, near.as_deref()) {
        witness::warn(
            channel::WINDOW,
            "Brainsty's window would not open",
            &[("why", Fact::Why(why.to_string()))],
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Kind {
    Doc,
    Page,
    Task,
    Tag,
    File,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Neuron {
    pub kind: Kind,
    pub id: String,
    pub title: String,
}

pub(crate) fn opens(neuron: &Neuron) -> bool {
    neuron.kind != Kind::File
}

#[tauri::command(async)]
pub fn brainsty_open(app: AppHandle, neuron: Neuron) {
    if !opens(&neuron) {
        return;
    }
    let _ = app.emit_to("main", OPENED, neuron);
    crate::tray::surface(&app);
}

pub(crate) fn respoken(app: &AppHandle, locale: &Option<String>) {
    if app.get_webview_window(LABEL).is_some() {
        let _ = app.emit_to(LABEL, SPOKEN, locale);
    }
}

#[cfg(test)]
#[path = "brainsty_window_test.rs"]
mod tests;
