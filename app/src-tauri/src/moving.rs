use tauri::{Emitter, Manager};
use tisty_core::witness::{self, Fact, channel};

const BETWEEN: std::time::Duration = std::time::Duration::from_millis(150);

#[derive(Clone, serde::Serialize)]
pub struct Along {
    done: u64,
    whole: u64,
}

/// The first start after the store's home changed copies it all, so the window says how far it is.
pub fn begin(app: tauri::AppHandle) {
    let shown = tauri::WebviewWindowBuilder::new(
        &app,
        "moving",
        tauri::WebviewUrl::App("index.html".into()),
    )
    .title("Tisty")
    .inner_size(440.0, 170.0)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .center()
    .build();
    if let Err(why) = shown {
        witness::warn(
            channel::WINDOW,
            "the window that shows the store moving would not open",
            &[("why", Fact::Why(why.to_string()))],
        );
    }
    std::thread::spawn(move || {
        let mut last: Option<std::time::Instant> = None;
        tisty_core::paths::settle_home_telling(&mut |done, whole| {
            if done < whole && last.is_some_and(|at| at.elapsed() < BETWEEN) {
                return;
            }
            last = Some(std::time::Instant::now());
            let _ = app.emit_to("moving", "moving", Along { done, whole });
        });
        let back = app.clone();
        let _ = app.run_on_main_thread(move || settled_in(&back));
    });
}

fn settled_in(app: &tauri::AppHandle) {
    if let Err(why) = crate::opened(app) {
        witness::error(
            channel::WINDOW,
            "the window could not open after the store moved",
            &[("why", Fact::Why(why.to_string()))],
        );
        app.exit(1);
        return;
    }
    // They loaded while nothing was there to answer them, so they start over now that everything is.
    for label in ["main", "quick"] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.reload();
        }
    }
    if let Some(moving) = app.get_webview_window("moving") {
        let _ = moving.close();
    }
}
