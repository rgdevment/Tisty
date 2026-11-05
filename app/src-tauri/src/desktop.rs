#[cfg(target_os = "macos")]
use tisty_core::witness::Fact;
#[cfg(any(windows, target_os = "macos"))]
use tisty_core::witness::{self, channel};

#[cfg(target_os = "macos")]
use crate::worded;

/// The window answers in milliseconds; this is only for the window that never does.
const WINDOW_ANSWERS_WITHIN: std::time::Duration = std::time::Duration::from_millis(1500);

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

#[cfg(windows)]
pub(crate) fn answers_for(scheme: &str) -> bool {
    use windows::Win32::UI::Shell::{ASSOCSTR_APPID, ASSOCSTR_EXECUTABLE};

    answered_by(
        associated(scheme, ASSOCSTR_EXECUTABLE).as_deref(),
        associated(scheme, ASSOCSTR_APPID).as_deref(),
    )
}

#[cfg(windows)]
#[allow(unsafe_code)]
pub(crate) fn associated(
    scheme: &str,
    asked: windows::Win32::UI::Shell::ASSOCSTR,
) -> Option<String> {
    use windows::Win32::UI::Shell::{
        ASSOCF_INIT_IGNOREUNKNOWN, ASSOCF_IS_PROTOCOL, AssocQueryStringW,
    };
    use windows::core::{HSTRING, PWSTR, w};

    let scheme = HSTRING::from(scheme);
    let flags = ASSOCF_IS_PROTOCOL | ASSOCF_INIT_IGNOREUNKNOWN;
    let mut needed = 0u32;
    unsafe { AssocQueryStringW(flags, asked, &scheme, w!("open"), None, &raw mut needed) }
        .ok()
        .ok()?;
    if needed <= 1 {
        return None;
    }
    let mut said = vec![0u16; needed as usize];
    unsafe {
        AssocQueryStringW(
            flags,
            asked,
            &scheme,
            w!("open"),
            Some(PWSTR(said.as_mut_ptr())),
            &raw mut needed,
        )
    }
    .ok()
    .ok()?;
    let end = said.iter().position(|one| *one == 0).unwrap_or(said.len());
    Some(String::from_utf16_lossy(&said[..end]))
}

#[cfg(windows)]
pub(crate) fn answered_by(executable: Option<&str>, packaged: Option<&str>) -> bool {
    let picker = executable.is_some_and(|one| {
        std::path::Path::new(one)
            .file_name()
            .and_then(|leaf| leaf.to_str())
            .is_some_and(|leaf| leaf.eq_ignore_ascii_case("OpenWith.exe"))
    });
    if picker {
        return false;
    }
    executable.is_some_and(|one| !one.is_empty()) || packaged.is_some_and(a_package)
}

#[cfg(windows)]
fn a_package(id: &str) -> bool {
    id.get(..4)
        .is_some_and(|head| head.eq_ignore_ascii_case("appx"))
        || id.contains('!')
}

/// Blocks until anything under `key` is written or made; false once it can no longer be watched.
#[cfg(windows)]
#[allow(unsafe_code)]
pub(crate) fn changed(key: &winreg::RegKey) -> bool {
    use windows::Win32::System::Registry::{
        HKEY, REG_NOTIFY_CHANGE_LAST_SET, REG_NOTIFY_CHANGE_NAME, RegNotifyChangeKeyValue,
    };

    // The handle stays open while `key` is borrowed, and a synchronous wait takes no event.
    unsafe {
        RegNotifyChangeKeyValue(
            HKEY(key.raw_handle() as _),
            true,
            REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET,
            None,
            false,
        )
    }
    .is_ok()
}

#[cfg(windows)]
#[allow(unsafe_code)]
pub(crate) fn known_as(at: &std::path::Path) -> Option<String> {
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Storage::FileSystem::{
        FILE_BASIC_INFO, FILE_ID_INFO, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ,
        FILE_SHARE_WRITE, FileBasicInfo, FileIdInfo, GetFileInformationByHandleEx,
    };

    // Attributes only: opening for its bytes would bring down a body a cloud left up there.
    let file = std::fs::OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES.0)
        .share_mode((FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE).0)
        .open(at)
        .ok()?;
    let mut info = FILE_BASIC_INFO::default();
    unsafe {
        GetFileInformationByHandleEx(
            HANDLE(file.as_raw_handle()),
            FileBasicInfo,
            (&raw mut info).cast(),
            std::mem::size_of::<FILE_BASIC_INFO>() as u32,
        )
    }
    .ok()?;
    // Two files made in one tick can share every date on ReFS; only the file id tells them apart.
    let mut id = FILE_ID_INFO::default();
    // FAT, exFAT and some shares give no id, and the change time alone still answers for a rewrite.
    let given = unsafe {
        GetFileInformationByHandleEx(
            HANDLE(file.as_raw_handle()),
            FileIdInfo,
            (&raw mut id).cast(),
            std::mem::size_of::<FILE_ID_INFO>() as u32,
        )
    }
    .is_ok();
    let named: String = if given {
        id.FileId
            .Identifier
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    } else {
        String::new()
    };
    let changed = if given || info.ChangeTime != 0 {
        info.ChangeTime
    } else {
        info.LastWriteTime
    };
    Some(format!("{changed}.{:x}.{named}", id.VolumeSerialNumber))
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
    let whole = MenuItem::with_id(
        app,
        "whole",
        worded(locale, "wholeDoc"),
        true,
        Some("Cmd+E"),
    )?;
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
            &PredefinedMenuItem::separator(app)?,
            &whole,
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
    let guide = MenuItem::with_id(app, "help", worded(locale, "helpGuide"), true, None::<&str>)?;
    let help = Submenu::with_items(app, worded(locale, "help"), true, &[&guide])?;
    tauri::menu::Menu::with_items(app, &[&app_menu, &edit, &window, &help])
}

pub(crate) fn parting<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use tauri::{Emitter, Manager};

    if crate::diagramming::holds_back(app) {
        return;
    }
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
        std::thread::sleep(WINDOW_ANSWERS_WITHIN);
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
pub(crate) struct Leaving(pub(crate) std::sync::atomic::AtomicBool);

#[derive(Default)]
pub(crate) struct Departed(std::sync::atomic::AtomicBool);

#[tauri::command]
pub fn parted(app: tauri::AppHandle) {
    leave(&app);
}

#[derive(Default)]
pub(crate) struct Ending(std::sync::Arc<std::sync::atomic::AtomicBool>);

#[tauri::command]
pub fn kept_for_the_end(app: tauri::AppHandle) {
    use tauri::Manager;

    app.state::<Ending>()
        .0
        .store(true, std::sync::atomic::Ordering::SeqCst);
}

/// tao ends the process inside `WM_ENDSESSION` without a word to anyone, so the question before it
/// is the only turn the window gets to keep what is still waiting to be written. Nothing leaves
/// here: another program may yet refuse the shutdown, and then Tisty simply carries on.
#[cfg(windows)]
pub(crate) fn warned_first(window: &tauri::WebviewWindow) {
    use tauri::{Emitter, Manager};

    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    let app = window.app_handle().clone();
    let answered = app.state::<Ending>().0.clone();
    // From a thread of its own: this runs inside whatever the event loop was doing, and an emit
    // from its own thread would step right into it.
    let tell = Box::new(move || {
        let app = app.clone();
        std::thread::spawn(move || {
            let _ = app.emit_to("main", "ending", ());
        });
    });
    let warning = warned::Warning::new(tell, answered, WINDOW_ANSWERS_WITHIN);
    if !warned::listens(windows::Win32::Foundation::HWND(hwnd.0 as _), warning) {
        witness::warn(
            channel::WINDOW,
            "the window will not hear when Windows is about to close it",
            &[],
        );
    }
}

#[cfg(not(windows))]
pub(crate) fn warned_first(_window: &tauri::WebviewWindow) {}

#[cfg(windows)]
mod warned {
    use std::cell::Cell;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    use std::time::{Duration, Instant};

    use tisty_core::witness::{self, Fact, channel};
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, ENDSESSION_CLOSEAPP, MSG, MsgWaitForMultipleObjectsEx, PM_QS_POSTMESSAGE,
        PM_QS_SENDMESSAGE, PM_REMOVE, PeekMessageW, PostQuitMessage, QS_POSTMESSAGE,
        QS_SENDMESSAGE, TranslateMessage, WM_QUERYENDSESSION, WM_QUIT,
    };

    const LOOKS_AGAIN: Duration = Duration::from_millis(20);

    pub struct Warning {
        tell: Box<dyn Fn()>,
        answered: Arc<AtomicBool>,
        patience: Duration,
        waiting: Cell<bool>,
    }

    impl Warning {
        pub fn new(tell: Box<dyn Fn()>, answered: Arc<AtomicBool>, patience: Duration) -> Self {
            Self {
                tell,
                answered,
                patience,
                waiting: Cell::new(false),
            }
        }

        fn heard(&self, closing: isize) {
            if self.waiting.replace(true) {
                return;
            }
            self.answered.store(false, SeqCst);
            let began = Instant::now();
            (self.tell)();
            let answered = waited(&self.answered, self.patience);
            self.waiting.set(false);

            let because = match closing as u32 & ENDSESSION_CLOSEAPP {
                0 => "the_session_ends",
                _ => "a_file_is_replaced",
            };
            let facts = [
                ("for", Fact::Word(because)),
                (
                    "waited_ms",
                    Fact::Count(began.elapsed().as_millis() as usize),
                ),
            ];
            if answered {
                witness::note(
                    channel::WINDOW,
                    "Windows is closing Tisty, and what was waiting to be written is kept",
                    &facts,
                );
            } else {
                witness::warn(
                    channel::WINDOW,
                    "Windows is closing Tisty, and the window did not say in time that it kept what was waiting",
                    &facts,
                );
            }
        }
    }

    // The answer comes back through this very thread, as WebView2 hands the window's calls over in
    // window messages. Paint and input stay queued: only the answer's messages are needed, and
    // tao's paint handling asserts that it is not entered twice.
    #[allow(unsafe_code)]
    fn waited(answered: &AtomicBool, patience: Duration) -> bool {
        let deadline = Instant::now() + patience;
        let mut message = MSG::default();
        while !answered.load(SeqCst) {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                return false;
            }
            let only_the_answers = PM_REMOVE | PM_QS_POSTMESSAGE | PM_QS_SENDMESSAGE;
            if !unsafe { PeekMessageW(&raw mut message, None, 0, 0, only_the_answers) }.as_bool() {
                let nap = left.min(LOOKS_AGAIN).as_millis() as u32;
                let woken_by = QS_POSTMESSAGE | QS_SENDMESSAGE;
                unsafe { MsgWaitForMultipleObjectsEx(None, nap, woken_by, Default::default()) };
                continue;
            }
            if message.message == WM_QUIT {
                unsafe { PostQuitMessage(message.wParam.0 as i32) };
                return answered.load(SeqCst);
            }
            unsafe {
                let _ = TranslateMessage(&raw const message);
                DispatchMessageW(&raw const message);
            }
        }
        true
    }

    #[allow(unsafe_code)]
    unsafe extern "system" fn asked(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        warning: usize,
    ) -> LRESULT {
        if message == WM_QUERYENDSESSION {
            // Windows calls this on the window's thread, with the pointer `listens` let go of.
            let warning = unsafe { &*(warning as *const Warning) };
            // A panic must not cross back into Windows, and Windows still needs its answer.
            let _ = catch_unwind(AssertUnwindSafe(|| warning.heard(lparam.0)));
        }
        unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
    }

    /// The warning stays for good: the main window is only ever hidden, so it lasts as long as the process.
    #[allow(unsafe_code)]
    pub fn listens(hwnd: HWND, warning: Warning) -> bool {
        let warning = Box::into_raw(Box::new(warning)) as usize;
        unsafe { SetWindowSubclass(hwnd, Some(asked), 1, warning) }.as_bool()
    }
}

#[cfg(all(test, windows))]
#[allow(unsafe_code)]
mod warned_tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
    use std::time::{Duration, Instant};

    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::Graphics::Gdi::{RDW_INTERNALPAINT, RedrawWindow};
    use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DispatchMessageW, ENDSESSION_CLOSEAPP, GetMessageW, MSG, PostMessageW,
        SMTO_NORMAL, SendMessageTimeoutW, WM_APP, WM_PAINT, WM_QUERYENDSESSION, WS_EX_NOACTIVATE,
        WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
    };
    use windows::core::w;

    use super::warned::{Warning, listens};

    const ANSWER: u32 = WM_APP + 1;
    const PLENTY: Duration = Duration::from_secs(20);

    #[derive(Default)]
    struct Seen {
        answered: Arc<AtomicBool>,
        told: AtomicUsize,
        painted: AtomicUsize,
        painted_by_the_answer: AtomicUsize,
    }

    unsafe extern "system" fn watching(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _id: usize,
        seen: usize,
    ) -> LRESULT {
        let seen = unsafe { &*(seen as *const Seen) };
        match message {
            ANSWER => {
                let painted = seen.painted.load(SeqCst);
                seen.painted_by_the_answer.store(painted, SeqCst);
                seen.answered.store(true, SeqCst);
            }
            WM_PAINT => {
                seen.painted.fetch_add(1, SeqCst);
            }
            _ => {}
        }
        unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
    }

    // A window on a thread of its own, as the event loop's is, so the asking can block on it.
    fn a_window(
        seen: &Arc<Seen>,
        patience: Duration,
        tell: impl Fn(isize) + Send + 'static,
    ) -> isize {
        let (hand, got) = std::sync::mpsc::channel();
        let seen = Arc::clone(seen);
        std::thread::spawn(move || unsafe {
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                w!("STATIC"),
                w!(""),
                WS_POPUP | WS_VISIBLE,
                0,
                0,
                0,
                0,
                None,
                None,
                None,
                None,
            )
            .expect("a window");
            assert!(
                SetWindowSubclass(hwnd, Some(watching), 2, Arc::as_ptr(&seen) as usize).as_bool()
            );
            let at = hwnd.0 as isize;
            let warning = Warning::new(
                Box::new(move || tell(at)),
                Arc::clone(&seen.answered),
                patience,
            );
            assert!(listens(hwnd, warning));
            hand.send(at).expect("the test is waiting");
            let mut message = MSG::default();
            while GetMessageW(&raw mut message, None, 0, 0).as_bool() {
                DispatchMessageW(&raw const message);
            }
        });
        got.recv().expect("the window")
    }

    fn asked(at: isize) -> (usize, Duration) {
        let began = Instant::now();
        let mut said = 0;
        let sent = unsafe {
            SendMessageTimeoutW(
                HWND(at as _),
                WM_QUERYENDSESSION,
                WPARAM(0),
                LPARAM(ENDSESSION_CLOSEAPP as isize),
                SMTO_NORMAL,
                30_000,
                Some(&raw mut said),
            )
        };
        assert_ne!(sent.0, 0, "the window answered Windows");
        (said, began.elapsed())
    }

    fn answer(at: isize) {
        unsafe { PostMessageW(Some(HWND(at as _)), ANSWER, WPARAM(0), LPARAM(0)) }.expect("posted");
    }

    #[test]
    fn the_answer_reaches_the_window_through_the_wait_itself() {
        let seen = Arc::new(Seen::default());
        let at = a_window(&seen, PLENTY, answer);

        let (said, took) = asked(at);

        assert!(seen.answered.load(SeqCst));
        assert!(took < PLENTY, "{took:?}");
        assert_ne!(said, 0, "Windows may go ahead");
    }

    #[test]
    fn a_window_that_never_answers_does_not_hold_windows_back() {
        let seen = Arc::new(Seen::default());
        let patience = Duration::from_millis(150);
        let at = a_window(&seen, patience, |_| {});

        let (said, took) = asked(at);

        assert!(took >= patience, "{took:?}");
        assert!(took < PLENTY, "{took:?}");
        assert_ne!(said, 0, "Windows may go ahead");
    }

    #[test]
    fn a_shutdown_called_off_is_asked_about_again_the_next_time() {
        let seen = Arc::new(Seen::default());
        let counted = Arc::clone(&seen);
        let at = a_window(&seen, PLENTY, move |at| {
            counted.told.fetch_add(1, SeqCst);
            answer(at);
        });

        asked(at);
        asked(at);

        assert_eq!(seen.told.load(SeqCst), 2);
    }

    #[test]
    fn paint_waits_until_the_event_loop_has_the_thread_back() {
        let seen = Arc::new(Seen::default());
        let at = a_window(&seen, PLENTY, |at| {
            unsafe {
                let _ = RedrawWindow(Some(HWND(at as _)), None, None, RDW_INTERNALPAINT);
            }
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(100));
                answer(at);
            });
        });

        asked(at);

        assert_eq!(seen.painted_by_the_answer.load(SeqCst), 0);
        let until = Instant::now() + PLENTY;
        while seen.painted.load(SeqCst) == 0 {
            assert!(Instant::now() < until, "the paint never came");
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
