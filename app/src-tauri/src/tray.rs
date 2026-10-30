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
/// The window's icon at the sizes a taskbar asks for from 100 % to 200 %: side, on a dark bar, on a light one.
#[cfg(windows)]
const WINDOW: [(f64, &[u8], &[u8]); 4] = [
    (
        32.0,
        include_bytes!("../icons/taskbar/on-dark-32.png"),
        include_bytes!("../icons/taskbar/on-light-32.png"),
    ),
    (
        40.0,
        include_bytes!("../icons/taskbar/on-dark-40.png"),
        include_bytes!("../icons/taskbar/on-light-40.png"),
    ),
    (
        48.0,
        include_bytes!("../icons/taskbar/on-dark-48.png"),
        include_bytes!("../icons/taskbar/on-light-48.png"),
    ),
    (
        64.0,
        include_bytes!("../icons/taskbar/on-dark-64.png"),
        include_bytes!("../icons/taskbar/on-light-64.png"),
    ),
];

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
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let wanted = 32.0 * window.scale_factor().unwrap_or(1.0);
    let worn = for_taskbar(wanted, dark_bar());
    if let Ok(icon) = tauri::image::Image::from_bytes(worn) {
        kept(window.set_icon(icon), "the window icon was refused");
    }
}

// Windows shrinks one image to every size it shows, so the closest one at or above the wanted side is given.
#[cfg(windows)]
fn for_taskbar(wanted: f64, dark: bool) -> &'static [u8] {
    let (_, on_dark, on_light) = WINDOW
        .iter()
        .find(|(side, ..)| *side >= wanted)
        .unwrap_or(&WINDOW[WINDOW.len() - 1]);
    if dark { on_dark } else { on_light }
}

#[cfg(not(windows))]
pub fn dress<R: Runtime>(_app: &AppHandle<R>) {}

#[cfg(windows)]
const THEMES: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes";
#[cfg(windows)]
const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";

// A taskbar-only theme change raises no ThemeChanged; Themes is watched since Personalize may not exist yet.
#[cfg(windows)]
pub fn follow_bar<R: Runtime>(app: AppHandle<R>) {
    std::thread::spawn(move || {
        let Ok(key) = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).open_subkey(THEMES)
        else {
            return;
        };
        while crate::desktop::changed(&key) {
            // Windows writes a theme as a burst of values, so the icons are read once it has settled.
            std::thread::sleep(std::time::Duration::from_millis(250));
            let painted = app.clone();
            let _ = app.run_on_main_thread(move || repaint(&painted));
        }
    });
}

#[cfg(not(windows))]
pub fn follow_bar<R: Runtime>(_app: AppHandle<R>) {}

#[cfg(windows)]
fn dark_bar() -> bool {
    winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
        .open_subkey(PERSONALIZE)
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

    #[cfg(windows)]
    #[test]
    fn the_window_icon_stands_out_from_the_taskbar_behind_it() {
        for (side, on_dark, on_light) in super::WINDOW {
            assert!(
                ink(on_dark) > 160,
                "a dark taskbar needs the light drawing at {side}"
            );
            assert!(
                ink(on_light) < 96,
                "a light taskbar needs the dark drawing at {side}"
            );
        }
    }

    #[cfg(windows)]
    #[test]
    fn the_window_icon_is_never_stretched_up() {
        use super::for_taskbar;
        assert_eq!(for_taskbar(32.0, true), super::WINDOW[0].1);
        assert_eq!(for_taskbar(36.0, false), super::WINDOW[1].2);
        assert_eq!(for_taskbar(48.0, true), super::WINDOW[2].1);
        assert_eq!(for_taskbar(96.0, true), super::WINDOW[3].1);
    }
}
