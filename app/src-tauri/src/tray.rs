use std::sync::Mutex;

use tauri::{
    AppHandle, Emitter, Manager, Runtime,
    menu::{Menu, MenuItem},
    tray::{TrayIcon, TrayIconBuilder, TrayIconEvent},
};
use tisty_core::witness::{self, Fact, channel};

fn kept<T, E: std::fmt::Display>(what: Result<T, E>, said: &'static str) -> Option<T> {
    match what {
        Ok(one) => Some(one),
        Err(why) => {
            witness::warn(
                channel::WINDOW,
                said,
                &[("why", Fact::Why(why.to_string()))],
            );
            None
        }
    }
}

pub struct Said<R: Runtime>(pub Mutex<Vec<MenuItem<R>>>);

#[cfg(target_os = "macos")]
const MACOS: &[u8] = include_bytes!("../icons/tray/tray-macos/menubarTemplate@2x.png");
#[cfg(not(target_os = "macos"))]
const ON_DARK: &[u8] = include_bytes!("../icons/tray/tray-windows/on-dark-32.png");
#[cfg(not(target_os = "macos"))]
const ON_LIGHT: &[u8] = include_bytes!("../icons/tray/tray-windows/on-light-32.png");
#[cfg(windows)]
const WINDOW_ON_DARK: &[u8] = include_bytes!("../icons/taskbar/on-dark.png");
#[cfg(windows)]
const WINDOW_ON_LIGHT: &[u8] = include_bytes!("../icons/taskbar/on-light.png");

pub struct Words {
    pub show: String,
    pub capture: String,
    pub quit: String,
}

pub fn raise<R: Runtime>(app: &AppHandle<R>, words: &Words) -> Option<TrayIcon<R>> {
    let item = "a tray menu item was refused";
    let show = kept(
        MenuItem::with_id(app, "show", &words.show, true, None::<&str>),
        item,
    )?;
    let capture = kept(
        MenuItem::with_id(app, "capture", &words.capture, true, None::<&str>),
        item,
    )?;
    let quit = kept(
        MenuItem::with_id(app, "quit", &words.quit, true, None::<&str>),
        item,
    )?;
    let menu = kept(
        Menu::with_items(app, &[&capture, &show, &quit]),
        "the tray menu was refused",
    )?;

    let building = TrayIconBuilder::with_id("tisty")
        .icon(art(app)?)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "capture" => quicken(app),
            "show" => surface(app),
            "quit" => crate::desktop::parting(app),
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
        });
    let tray = kept(building.build(app), "the tray would not build")?;

    #[cfg(target_os = "macos")]
    kept(
        tray.set_icon_as_template(true),
        "the tray icon would not be a template",
    );

    app.manage(Said(Mutex::new(vec![capture, show, quit])));

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

pub fn repaint<R: Runtime>(app: &AppHandle<R>) {
    dress(app);
    if let Some(tray) = app.tray_by_id("tisty")
        && let Some(icon) = art(app)
    {
        let _ = tray.set_icon(Some(icon));
        #[cfg(target_os = "macos")]
        kept(
            tray.set_icon_as_template(true),
            "the tray icon would not be a template",
        );
    }
}

#[cfg(target_os = "macos")]
fn art<R: Runtime>(_app: &AppHandle<R>) -> Option<tauri::image::Image<'static>> {
    tauri::image::Image::from_bytes(MACOS).ok()
}

#[cfg(not(target_os = "macos"))]
fn art<R: Runtime>(app: &AppHandle<R>) -> Option<tauri::image::Image<'static>> {
    let _ = app;
    tauri::image::Image::from_bytes(for_bar(dark_bar())).ok()
}

#[cfg(not(target_os = "macos"))]
fn for_bar(dark: bool) -> &'static [u8] {
    if dark { ON_DARK } else { ON_LIGHT }
}

// The installer's .ico is one drawing; the taskbar shows the window's icon, which can follow the bar.
#[cfg(windows)]
pub fn dress<R: Runtime>(app: &AppHandle<R>) {
    let worn = if dark_bar() {
        WINDOW_ON_DARK
    } else {
        WINDOW_ON_LIGHT
    };
    if let Some(window) = app.get_webview_window("main")
        && let Ok(icon) = tauri::image::Image::from_bytes(worn)
    {
        kept(window.set_icon(icon), "the window icon was refused");
    }
}

#[cfg(not(windows))]
pub fn dress<R: Runtime>(_app: &AppHandle<R>) {}

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

pub fn quicken<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("quick") else {
        return;
    };
    if window.is_visible().unwrap_or(false) {
        tuck(app);
        return;
    }
    bring(&window);
}

pub fn sow<R: Runtime>(app: &AppHandle<R>, priority: Option<String>) {
    let Some(window) = app.get_webview_window("quick") else {
        return;
    };
    bring(&window);
    let _ = window.emit("sow", priority);
}

fn bring<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    let _ = window.as_ref().show();
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
}

/// Hiding the window alone leaves WebView2 drawing for nobody; the webview itself has to be told.
pub fn tuck<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("quick") else {
        return;
    };
    let _ = window.hide();
    let _ = window.as_ref().hide();
}

/// The activation policy, not tao's Dock hiding, which drops Cmd+H and skips a hide right after a show.
pub fn docked<R: Runtime>(app: &AppHandle<R>, shown: bool) {
    #[cfg(target_os = "macos")]
    kept(
        app.set_activation_policy(match shown {
            true => tauri::ActivationPolicy::Regular,
            false => tauri::ActivationPolicy::Accessory,
        }),
        "the Dock icon would not change",
    );
    #[cfg(not(target_os = "macos"))]
    let _ = (app, shown);
}

/// A window on its way out may still be listed as visible, so it is named and left out.
pub fn undock_unless_seen<R: Runtime>(app: &AppHandle<R>, leaving: &str) {
    #[cfg(target_os = "macos")]
    {
        // Quitting tears the windows down, and a message after the event loop is gone makes tao panic.
        if app
            .try_state::<crate::desktop::Leaving>()
            .is_some_and(|it| it.0.load(std::sync::atomic::Ordering::SeqCst))
        {
            return;
        }
        let seen = app.webview_windows().iter().any(|(label, window)| {
            label != leaving
                && label != "quick"
                && (window.is_visible().unwrap_or(false) || window.is_minimized().unwrap_or(false))
        });
        if !seen {
            docked(app, false);
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, leaving);
}

pub fn withdraw<R: Runtime>(window: &tauri::Window<R>) {
    let _ = window.emit("withdrawn", ());
    let _ = window.hide();
    undock_unless_seen(window.app_handle(), window.label());
}

pub fn surface<R: Runtime>(app: &AppHandle<R>) {
    // While the store is still moving no session stands behind the main window, so the move shows.
    let label = match app.try_state::<crate::Perched>() {
        Some(_) => "main",
        None => "moving",
    };
    let Some(window) = app.get_webview_window(label) else {
        return;
    };
    docked(app, true);
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::{ON_DARK, ON_LIGHT, for_bar};

    fn ink(png: &[u8]) -> u32 {
        let art = tauri::image::Image::from_bytes(png).expect("tray icon is a png");
        let pixels = art.rgba();
        let (sum, seen) = pixels
            .as_chunks::<4>()
            .0
            .iter()
            .fold((0u64, 0u64), |(sum, seen), px| {
                if px[3] < 128 {
                    return (sum, seen);
                }
                let grey = (px[0] as u64 * 299 + px[1] as u64 * 587 + px[2] as u64 * 114) / 1000;
                (sum + grey, seen + 1)
            });
        assert!(seen > 0, "the icon is fully transparent");
        (sum / seen) as u32
    }

    #[test]
    fn each_tray_icon_is_inked_against_the_bar_it_sits_on() {
        assert!(ink(ON_DARK) > 160, "on-dark must be pale: {}", ink(ON_DARK));
        assert!(
            ink(ON_LIGHT) < 96,
            "on-light must be dark: {}",
            ink(ON_LIGHT)
        );
    }

    #[test]
    fn a_light_taskbar_gets_the_dark_icon() {
        assert!(ink(for_bar(false)) < 96, "a light bar needs dark ink");
        assert!(ink(for_bar(true)) > 160, "a dark bar needs pale ink");
    }
}
