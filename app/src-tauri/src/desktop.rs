#[cfg(target_os = "macos")]
use tisty_core::witness::{self, Fact, channel};

#[cfg(target_os = "macos")]
use crate::worded;

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
pub(crate) fn proofread(window: &tauri::WebviewWindow) {
    let done = window.with_webview(|webview| {
        use objc2::runtime::AnyObject;
        use objc2::{msg_send, sel};

        let wk = webview.inner().cast::<AnyObject>();
        if wk.is_null() {
            return;
        }
        // Private selectors: asked before told, since a missing one raises an ObjC exception Rust cannot catch.
        unsafe {
            let spelling: bool =
                msg_send![wk, respondsToSelector: sel!(setContinuousSpellCheckingEnabled:)];
            if spelling {
                let _: () = msg_send![wk, setContinuousSpellCheckingEnabled: true];
            }
            let grammar: bool = msg_send![wk, respondsToSelector: sel!(setGrammarCheckingEnabled:)];
            if grammar {
                let _: () = msg_send![wk, setGrammarCheckingEnabled: true];
            }
        }
    });
    if let Err(e) = done {
        witness::warn(
            channel::WINDOW,
            "spell checking stayed off",
            &[("why", Fact::Why(e.to_string()))],
        );
    }
}

/// Whether this process runs translated by Rosetta. Only an Apple Silicon Mac carries the key, so
/// an Intel one answers with an error, which reads as no.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
pub(crate) fn translated() -> bool {
    unsafe extern "C" {
        fn sysctlbyname(
            name: *const std::ffi::c_char,
            oldp: *mut std::ffi::c_void,
            oldlenp: *mut usize,
            newp: *mut std::ffi::c_void,
            newlen: usize,
        ) -> std::ffi::c_int;
    }
    let mut yes: std::ffi::c_int = 0;
    let mut len = size_of::<std::ffi::c_int>();
    let rc = unsafe {
        sysctlbyname(
            c"sysctl.proc_translated".as_ptr(),
            (&raw mut yes).cast(),
            &raw mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    rc == 0 && yes == 1
}

/// Whether the shell would hand a link of this scheme to an application. Without a buffer the call
/// only measures, and ignoring the unknown keeps the "Open with" picker from counting as one.
#[cfg(windows)]
#[allow(unsafe_code)]
pub(crate) fn answers_for(scheme: &str) -> bool {
    use windows::Win32::UI::Shell::{
        ASSOCF_INIT_IGNOREUNKNOWN, ASSOCF_IS_PROTOCOL, ASSOCSTR_EXECUTABLE, AssocQueryStringW,
    };
    use windows::core::{HSTRING, w};

    let mut needed = 0u32;
    let said = unsafe {
        AssocQueryStringW(
            ASSOCF_IS_PROTOCOL | ASSOCF_INIT_IGNOREUNKNOWN,
            ASSOCSTR_EXECUTABLE,
            &HSTRING::from(scheme),
            w!("open"),
            None,
            &raw mut needed,
        )
    };
    said.is_ok() && needed > 1
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn translated() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn proofread(_window: &tauri::WebviewWindow) {}

pub(crate) fn fitted(window: &tauri::WebviewWindow) {
    let (Ok(Some(screen)), Ok(asked)) = (window.current_monitor(), window.outer_size()) else {
        return;
    };
    let room = screen.work_area().size;
    if asked.width > room.width || asked.height > room.height {
        let _ = window.maximize();
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn menued(
    app: &tauri::AppHandle,
    locale: &Option<String>,
) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{AboutMetadata, MenuItem, PredefinedMenuItem, Submenu};

    let leave = MenuItem::with_id(app, "leave", worded(locale, "quit"), true, Some("Cmd+Q"))?;
    let app_menu = Submenu::with_items(
        app,
        "Tisty",
        true,
        &[
            &PredefinedMenuItem::about(app, None, Some(AboutMetadata::default()))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::show_all(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &leave,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        worded(locale, "edit"),
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let window = Submenu::with_items(
        app,
        worded(locale, "windowMenu"),
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;
    tauri::menu::Menu::with_items(app, &[&app_menu, &edit, &window])
}

pub(crate) fn parting<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use tauri::{Emitter, Manager};

    if app
        .state::<Leaving>()
        .0
        .swap(true, std::sync::atomic::Ordering::SeqCst)
    {
        return;
    }
    let _ = app.emit("parting", ());

    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        leave(&handle);
    });
}

/// The window answers in milliseconds and the timer above is only there for the window that
/// never answers, so the second of the two reaches an event loop that is already gone — and
/// tao panics rather than ignore it.
pub(crate) fn leave<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use tauri::Manager;

    if app
        .state::<Departed>()
        .0
        .swap(true, std::sync::atomic::Ordering::SeqCst)
    {
        return;
    }
    app.exit(0);
}

#[derive(Default)]
pub(crate) struct Leaving(std::sync::atomic::AtomicBool);

#[derive(Default)]
pub(crate) struct Departed(std::sync::atomic::AtomicBool);

#[tauri::command]
pub fn parted(app: tauri::AppHandle) {
    leave(&app);
}
