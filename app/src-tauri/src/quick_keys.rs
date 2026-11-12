use std::sync::Mutex;

use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tisty_core::witness::{self, channel};

use crate::{Answer, Refusal, Session, held, tray};

pub(crate) const OFF: &str = "off";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Door {
    #[default]
    Task,
    Idea,
}

impl Door {
    pub(crate) fn named(self) -> &'static str {
        match self {
            Door::Task => "task",
            Door::Idea => "idea",
        }
    }

    // The two lists never share a combination, so one door can never take the other's keys.
    fn offered(self) -> Vec<(&'static str, Shortcut)> {
        let control_shift = Some(Modifiers::CONTROL | Modifiers::SHIFT);
        let control_alt = Some(Modifiers::CONTROL | Modifiers::ALT);
        match self {
            Door::Task => vec![
                (
                    "Ctrl+Shift+Space",
                    Shortcut::new(control_shift, Code::Space),
                ),
                #[cfg(not(target_os = "macos"))]
                ("Ctrl+Alt+Space", Shortcut::new(control_alt, Code::Space)),
                ("Ctrl+Shift+T", Shortcut::new(control_shift, Code::KeyT)),
            ],
            Door::Idea => vec![
                ("Ctrl+Alt+I", Shortcut::new(control_alt, Code::KeyI)),
                ("Ctrl+Alt+N", Shortcut::new(control_alt, Code::KeyN)),
                ("Ctrl+Shift+I", Shortcut::new(control_shift, Code::KeyI)),
            ],
        }
    }

    fn combo(self, said: &str) -> Option<Shortcut> {
        self.offered()
            .into_iter()
            .find(|(named, _)| *named == said)
            .map(|(_, combo)| combo)
    }
}

pub(crate) struct Bound {
    task: Mutex<Option<String>>,
    idea: Mutex<Option<String>>,
}

impl Bound {
    pub(crate) fn new(task: Option<String>, idea: Option<String>) -> Self {
        Self {
            task: Mutex::new(task),
            idea: Mutex::new(idea),
        }
    }

    pub(crate) fn held(&self, door: Door) -> Option<String> {
        self.of(door).lock().ok().and_then(|now| now.clone())
    }

    fn of(&self, door: Door) -> &Mutex<Option<String>> {
        match door {
            Door::Task => &self.task,
            Door::Idea => &self.idea,
        }
    }
}

fn take<R: tauri::Runtime>(app: &tauri::AppHandle<R>, door: Door, combo: Shortcut) -> bool {
    let handle = app.clone();
    app.global_shortcut()
        .on_shortcut(combo, move |_, _, event| {
            if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                tray::quicken(&handle, door.named());
            }
        })
        .is_ok()
}

pub(crate) fn listen_for<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    door: Door,
    chosen: Option<&str>,
) -> Option<String> {
    if chosen == Some(OFF) {
        return None;
    }
    if let Some(said) = chosen
        && let Some(wanted) = door.combo(said)
    {
        if take(app, door, wanted) {
            return Some(said.to_string());
        }
        witness::note(
            channel::WINDOW,
            "the chosen shortcut was taken, so another stands in",
            &[],
        );
    }
    for (said, combo) in door.offered() {
        if chosen != Some(said) && take(app, door, combo) {
            return Some(said.to_string());
        }
    }
    witness::warn(channel::WINDOW, "no shortcut was free", &[]);
    None
}

#[tauri::command]
pub fn shortcuts_offered(door: Option<Door>) -> Vec<&'static str> {
    door.unwrap_or_default()
        .offered()
        .into_iter()
        .map(|(said, _)| said)
        .collect()
}

#[tauri::command]
pub fn shortcut(bound: tauri::State<'_, Bound>, door: Option<Door>) -> Option<String> {
    bound.held(door.unwrap_or_default())
}

#[tauri::command]
pub fn keep_shortcut(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    bound: tauri::State<'_, Bound>,
    keys: String,
    door: Option<Door>,
) -> Answer<Option<String>> {
    let door = door.unwrap_or_default();
    let wanted = match keys.as_str() {
        OFF => None,
        said => Some(door.combo(said).ok_or_else(|| Refusal::of("keysUnknown"))?),
    };
    // The session first and then the shortcut, the order every other reader takes them in.
    let mut session = held(&session);
    let mut now = bound.of(door).lock().map_err(|_| Refusal::of("internal"))?;
    let was = now.as_deref().and_then(|said| door.combo(said));
    let back = |now: &mut Option<String>| {
        if let Some(wanted) = wanted {
            let _ = app.global_shortcut().unregister(wanted);
        }
        if was.is_none_or(|was| !take(&app, door, was)) {
            *now = None;
        }
    };
    if let Some(was) = was {
        let _ = app.global_shortcut().unregister(was);
    }
    if let Some(wanted) = wanted
        && !take(&app, door, wanted)
    {
        back(&mut now);
        return Err(Refusal::about("keysTaken", keys));
    }
    let kept = session.keep(|config| match door {
        Door::Task => config.quick_keys = Some(keys.clone()),
        Door::Idea => config.idea_keys = Some(keys.clone()),
    });
    if let Err(refused) = kept {
        back(&mut now);
        return Err(refused);
    }
    *now = wanted.map(|_| keys);
    Ok(now.clone())
}
