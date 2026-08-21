use std::sync::Mutex;

use tauri::{
    AppHandle, Manager, Runtime,
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
            "quit" => crate::parting(app),
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

pub fn repaint<R: Runtime>(app: &AppHandle<R>) {
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
        let _ = window.hide();
        return;
    }
    let _ = window.center();
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn surface<R: Runtime>(app: &AppHandle<R>) {
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
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
