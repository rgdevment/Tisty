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
        if allowed(invoke.message.webview().label(), invoke.message.command()) {
            return handler(invoke);
        }
        let command = invoke.message.command().to_string();
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

// In the screen's own pixels: a logical place would be read back through another monitor's scale.
fn screens(app: &AppHandle) -> Vec<Screen> {
    app.available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|monitor| {
            let area = monitor.work_area();
            Screen {
                x: f64::from(area.position.x),
                y: f64::from(area.position.y),
                width: f64::from(area.size.width),
                height: f64::from(area.size.height),
            }
        })
        .collect()
}

fn seen_of(window: &tauri::WebviewWindow) -> Option<Seen> {
    let at = window.outer_position().ok()?;
    let size = window.inner_size().ok()?;
    Some(Seen {
        place: Place {
            x: f64::from(at.x),
            y: f64::from(at.y),
            width: f64::from(size.width),
            height: f64::from(size.height),
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
    let window = building.inner_size(WIDE, TALL).center().build()?;
    if let Some(place) = restored {
        let _ = window.set_size(tauri::PhysicalSize::new(place.width, place.height));
        let _ = window.set_position(tauri::PhysicalPosition::new(place.x, place.y));
    }
    let tracked = Arc::new(Mutex::new(
        seen_of(&window).and_then(|seen| after(restored, seen)),
    ));
    if restored.is_some_and(|place| place.maximized) {
        let _ = window.maximize();
    }
    crate::viewing::shown_late(&window);
    crate::tray::docked(app, true);
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

fn shown_or_opened(app: &AppHandle, near: Option<&str>) {
    if let Some(window) = app.get_webview_window(LABEL) {
        crate::viewing::fronted(&window);
        return;
    }
    if let Err(why) = open(app, near) {
        witness::warn(
            channel::WINDOW,
            "Brainsty's window would not open",
            &[("why", Fact::Why(why.to_string()))],
        );
    }
}

#[derive(Default)]
pub struct Opening(Mutex<()>);

// One press at a time, so two quick ones never build two windows.
#[tauri::command(async)]
pub fn brainsty_show(app: AppHandle, opening: tauri::State<'_, Opening>, near: Option<String>) {
    let _one_at_a_time = opening.0.lock().unwrap_or_else(PoisonError::into_inner);
    shown_or_opened(&app, near.as_deref());
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
