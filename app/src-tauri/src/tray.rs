//! Where no tray exists nothing here runs, and closing keeps its plain meaning.

use std::sync::Mutex;

use tauri::{
    AppHandle, Manager, Runtime,
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder, TrayIconEvent},
};

/// Kept so the menu can be reworded: the terminal can change the language
/// while the window is open, and the tray would keep the startup one.
pub struct Said<R: Runtime>(pub Mutex<Vec<MenuItem<R>>>);

#[cfg(target_os = "macos")]
const MACOS: &[u8] = include_bytes!("../icons/tray/tray-macos/menubarTemplate@2x.png");
#[cfg(not(target_os = "macos"))]
const ON_DARK: &[u8] = include_bytes!("../icons/tray/tray-windows/on-dark-32.png");
#[cfg(not(target_os = "macos"))]
const ON_LIGHT: &[u8] = include_bytes!("../icons/tray/tray-windows/on-light-32.png");

pub struct Words {
    pub show: String,
    pub capture: String,
    pub quit: String,
}

/// `None` where the desktop has no tray, and Linux counts as none: the GTK
/// backend reports success whether or not anything is listening, and hiding
/// into a tray that is not there loses the app with no way back.
pub fn raise<R: Runtime>(app: &AppHandle<R>, words: &Words) -> Option<TrayIcon<R>> {
    let show = MenuItem::with_id(app, "show", &words.show, true, None::<&str>).ok()?;
    let capture = MenuItem::with_id(app, "capture", &words.capture, true, None::<&str>).ok()?;
    let quit = MenuItem::with_id(app, "quit", &words.quit, true, None::<&str>).ok()?;
    let menu = Menu::with_items(app, &[&capture, &show, &quit]).ok()?;

    let tray = TrayIconBuilder::with_id("tisty")
        .icon(art(app)?)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "capture" => quicken(app),
            "show" => surface(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                surface(tray.app_handle());
            }
        })
        .build(app)
        .ok()?;

    #[cfg(target_os = "macos")]
    // Without this the white art is painted white, which is nothing at all on
    // a light menu bar — and that never shows up while developing on Windows.
    let _ = tray.set_icon_as_template(true);

    app.manage(Said(Mutex::new(vec![capture, show, quit])));

    #[cfg(target_os = "linux")]
    return {
        let _ = tray;
        None
    };
    #[cfg(not(target_os = "linux"))]
    Some(tray)
}

pub fn reword<R: Runtime>(app: &AppHandle<R>, words: &Words) {
    let Some(items) = app.try_state::<Said<R>>() else {
        return;
    };
    let Ok(items) = items.0.lock() else {
        return;
    };
    for (item, text) in items.iter().zip([&words.capture, &words.show, &words.quit]) {
        let _ = item.set_text(text);
    }
}

/// Windows 11 switches the taskbar theme while running, so the icon has to be
/// chosen again, not only at startup.
pub fn repaint<R: Runtime>(app: &AppHandle<R>) {
    if let Some(tray) = app.tray_by_id("tisty")
        && let Some(icon) = art(app)
    {
        let _ = tray.set_icon(Some(icon));
        #[cfg(target_os = "macos")]
        let _ = tray.set_icon_as_template(true);
    }
}

#[cfg(target_os = "macos")]
fn art<R: Runtime>(_app: &AppHandle<R>) -> Option<tauri::image::Image<'static>> {
    tauri::image::Image::from_bytes(MACOS).ok()
}

#[cfg(not(target_os = "macos"))]
fn art<R: Runtime>(app: &AppHandle<R>) -> Option<tauri::image::Image<'static>> {
    let _ = app;
    // Dark taskbar takes the light-inked icon, and the other way round.
    tauri::image::Image::from_bytes(if dark_bar() { ON_LIGHT } else { ON_DARK }).ok()
}

/// The taskbar follows `SystemUsesLightTheme`, which is a different setting
/// from the one a window reports: «Windows dark, apps light» is a stock choice
/// in Windows 11, and reading the window would ink the icon to vanish.
#[cfg(windows)]
fn dark_bar() -> bool {
    winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|key| key.get_value::<u32, _>("SystemUsesLightTheme"))
        .map(|light| light == 0)
        .unwrap_or(true)
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn dark_bar() -> bool {
    true
}

/// Toggles: the same shortcut that opens it puts it away, so a capture begun
/// by mistake costs one keystroke rather than a trip to the mouse.
pub fn quicken<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("quick") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        let _ = window.hide();
        return;
    }
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
}

/// The main window only: the quick one has its own way of appearing, and
/// raising it here would put a capture field in front of every tray click.
pub fn surface<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}
