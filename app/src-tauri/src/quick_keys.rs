use std::sync::Mutex;

use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tisty_core::witness::{self, channel};

use crate::{Answer, Refusal, Session, held, tray};

pub(crate) const OFF: &str = "off";

pub(crate) struct Bound(pub(crate) Mutex<Option<String>>);

fn offered() -> Vec<(&'static str, Shortcut)> {
    vec![
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
    ]
}

fn combo(said: &str) -> Option<Shortcut> {
    offered()
        .into_iter()
        .find(|(named, _)| *named == said)
        .map(|(_, combo)| combo)
}

fn take<R: tauri::Runtime>(app: &tauri::AppHandle<R>, combo: Shortcut) -> bool {
    let handle = app.clone();
    app.global_shortcut()
        .on_shortcut(combo, move |_, _, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                tray::quicken(&handle);
            }
        })
        .is_ok()
}

/// The person's choice first; with none, the first combination no other program holds.
pub(crate) fn listen_for<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    chosen: Option<&str>,
) -> Option<String> {
    if chosen == Some(OFF) {
        return None;
    }
    if let Some(said) = chosen
        && let Some(wanted) = combo(said)
        && take(app, wanted)
    {
        return Some(said.to_string());
    }
    for (said, combo) in offered() {
        if take(app, combo) {
            return Some(said.to_string());
        }
    }
    witness::warn(channel::WINDOW, "no shortcut was free", &[]);
    None
}

#[tauri::command]
pub fn shortcut(bound: tauri::State<'_, Bound>) -> Option<String> {
    bound.0.lock().ok().and_then(|now| now.clone())
}

#[tauri::command]
pub fn keep_shortcut(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    bound: tauri::State<'_, Bound>,
    keys: String,
) -> Answer<Option<String>> {
    let wanted = match keys.as_str() {
        OFF => None,
        said => Some(combo(said).ok_or_else(|| Refusal::of("keysUnknown"))?),
    };
    let mut now = bound.0.lock().map_err(|_| Refusal::of("internal"))?;
    let was = now.as_deref().and_then(combo);
    if let Some(was) = was {
        let _ = app.global_shortcut().unregister(was);
    }
    if let Some(wanted) = wanted
        && !take(&app, wanted)
    {
        if let Some(was) = was {
            take(&app, was);
        }
        return Err(Refusal::about("keysTaken", keys));
    }
    *now = wanted.map(|_| keys.clone());
    held(&session).keep(|config| config.quick_keys = Some(keys))?;
    Ok(now.clone())
}
