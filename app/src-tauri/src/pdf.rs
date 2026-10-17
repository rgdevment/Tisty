use std::path::PathBuf;
use std::sync::mpsc::Sender;

// The margins the page already prints with, 22 mm above and below and 20 mm at the sides.
const TOP: f64 = 22.0 / 25.4;
const SIDE: f64 = 20.0 / 25.4;

/// The sheet in inches, as both engines take it.
pub fn leaf(paper: &str) -> (f64, f64) {
    let (named, turned) = paper
        .strip_suffix("-landscape")
        .map_or((paper, false), |one| (one, true));
    let (wide, tall) = match named {
        "letter" => (8.5, 11.0),
        "tabloid" => (11.0, 17.0),
        _ => (210.0 / 25.4, 297.0 / 25.4),
    };
    if turned { (tall, wide) } else { (wide, tall) }
}

/// The window's own engine lays the page out on paper, so the PDF is the page as it is drawn.
pub fn print(
    window: &tauri::WebviewWindow,
    to: PathBuf,
    sheet: (f64, f64),
    done: Sender<bool>,
) -> tauri::Result<()> {
    window.with_webview(move |platform| started(&platform, &to, sheet, done))
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn started(
    platform: &tauri::webview::PlatformWebview,
    to: &std::path::Path,
    (wide, high): (f64, f64),
    done: Sender<bool>,
) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2_7, ICoreWebView2Environment6,
    };
    use windows::core::{HSTRING, Interface};

    let told = done.clone();
    let asked = || -> windows::core::Result<()> {
        // SAFETY: live COM objects handed over on their own thread, each call borrowing values that outlive it.
        unsafe {
            let view: ICoreWebView2_7 = platform.controller().CoreWebView2()?.cast()?;
            let made: ICoreWebView2Environment6 = platform.environment().cast()?;
            let settings = made.CreatePrintSettings()?;
            settings.SetPageWidth(wide)?;
            settings.SetPageHeight(high)?;
            settings.SetMarginTop(TOP)?;
            settings.SetMarginBottom(TOP)?;
            settings.SetMarginLeft(SIDE)?;
            settings.SetMarginRight(SIDE)?;
            settings.SetShouldPrintBackgrounds(true)?;
            settings.SetShouldPrintHeaderAndFooter(false)?;
            let handler = webview2_com::PrintToPdfCompletedHandler::create(Box::new(
                move |result, printed| {
                    let _ = told.send(result.is_ok() && printed);
                    Ok(())
                },
            ));
            view.PrintToPdf(
                &HSTRING::from(to.to_string_lossy().as_ref()),
                &settings,
                &handler,
            )
        }
    };
    if asked().is_err() {
        let _ = done.send(false);
    }
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn started(
    platform: &tauri::webview::PlatformWebview,
    to: &std::path::Path,
    (wide, high): (f64, f64),
    done: Sender<bool>,
) {
    use objc2::msg_send;
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2_app_kit::{
        NSPrintInfo, NSPrintJobSavingURL, NSPrintOperation, NSPrintSaveJob,
        NSPrintingPaginationMode, NSWindow,
    };
    use objc2_foundation::{NSSize, NSString, NSURL};

    let info = NSPrintInfo::new();
    info.setPaperSize(NSSize::new(wide * 72.0, high * 72.0));
    info.setTopMargin(TOP * 72.0);
    info.setBottomMargin(TOP * 72.0);
    info.setLeftMargin(SIDE * 72.0);
    info.setRightMargin(SIDE * 72.0);
    info.setHorizontalPagination(NSPrintingPaginationMode::Fit);
    info.setVerticalPagination(NSPrintingPaginationMode::Automatic);
    info.setVerticallyCentered(false);
    let url = NSURL::fileURLWithPath(&NSString::from_str(&to.to_string_lossy()));
    // SAFETY: the live WKWebView and NSWindow on the main thread, and AppKit's own keys for the job.
    unsafe {
        info.setJobDisposition(NSPrintSaveJob);
        let saving: &AnyObject = &url;
        info.dictionary().insert(NSPrintJobSavingURL, saving);
        let view = &*(platform.inner() as *const AnyObject);
        let window = &*(platform.ns_window() as *const NSWindow);
        let operation: Retained<NSPrintOperation> =
            msg_send![view, printOperationWithPrintInfo: &*info];
        operation.setShowsPrintPanel(false);
        operation.setShowsProgressPanel(false);
        // WebKit prints only through the modal call; run on its own, it writes blank sheets.
        operation.runOperationModalForWindow_delegate_didRunSelector_contextInfo(
            window,
            None,
            None,
            std::ptr::null_mut(),
        );
    }
    let to = to.to_path_buf();
    std::thread::spawn(move || {
        let _ = done.send(landed(&to));
    });
}

// The modal call tells its end only to a delegate, so the file settling on disk is the answer.
#[cfg(target_os = "macos")]
fn landed(to: &std::path::Path) -> bool {
    let mut last = None;
    for _ in 0..240 {
        std::thread::sleep(std::time::Duration::from_millis(250));
        let now = std::fs::metadata(to).ok().map(|one| one.len());
        if now.is_some_and(|bytes| bytes > 0) && now == last {
            return true;
        }
        last = now;
    }
    false
}

#[cfg(not(any(windows, target_os = "macos")))]
fn started(
    _: &tauri::webview::PlatformWebview,
    _: &std::path::Path,
    _: (f64, f64),
    done: Sender<bool>,
) {
    let _ = done.send(false);
}
