use crate::{Answer, Refusal};

// The page goes through a window nobody sees, whose engine lays it on paper the way it draws it.
#[tauri::command]
pub async fn page_pdf(app: tauri::AppHandle, html: String, paper: String) -> Answer<Vec<u8>> {
    let room = super::carrying::Room(
        std::env::temp_dir().join(format!("tisty-pdf-{}", ulid::Ulid::generate())),
    );
    let page = room.0.join("page.html");
    let to = room.0.join("page.pdf");
    let cannot = || Refusal::about("cannotWrite", to.display().to_string());
    let (folder, written) = (room.0.clone(), page.clone());
    let laid = tauri::async_runtime::spawn_blocking(move || {
        swept(&std::env::temp_dir());
        std::fs::create_dir_all(&folder)?;
        let _ = tisty_core::paths::ours_alone(&folder);
        std::fs::write(&written, html)
    })
    .await;
    if !matches!(laid, Ok(Ok(()))) {
        return Err(cannot());
    }
    let url = url::Url::from_file_path(&page).map_err(|()| cannot())?;
    let (told, heard) = std::sync::mpsc::channel();
    let told = std::sync::Mutex::new(Some(told));
    let sheet = crate::pdf::leaf(&paper);
    let printing = to.clone();
    let label = format!(
        "paper-{}",
        ulid::Ulid::generate().to_string().to_lowercase()
    );
    let window = tauri::WebviewWindowBuilder::new(&app, label, tauri::WebviewUrl::External(url))
        .visible(false)
        .on_page_load(move |window, loaded| {
            if loaded.event() != tauri::webview::PageLoadEvent::Finished {
                return;
            }
            let Some(once) = told.lock().ok().and_then(|mut one| one.take()) else {
                return;
            };
            let back = once.clone();
            if crate::pdf::print(&window, printing.clone(), sheet, once).is_err() {
                let _ = back.send(false);
            }
        })
        .build()
        .map_err(|_| cannot())?;
    let made = to.clone();
    let printed = tauri::async_runtime::spawn_blocking(move || {
        let done = heard
            .recv_timeout(std::time::Duration::from_secs(60))
            .unwrap_or(false);
        done.then(|| std::fs::read(&made).ok()).flatten()
    })
    .await
    .ok()
    .flatten();
    let _ = window.destroy();
    tauri::async_runtime::spawn_blocking(move || drop(room));
    printed.ok_or_else(cannot)
}

// A sheet may carry the identity's phrase, so what a crash left behind is not left for the system to clear.
fn swept(temp: &std::path::Path) {
    let Some(old) =
        std::time::SystemTime::now().checked_sub(std::time::Duration::from_secs(5 * 60))
    else {
        return;
    };
    let Ok(all) = std::fs::read_dir(temp) else {
        return;
    };
    for one in all.flatten() {
        let ours = one.file_name().to_string_lossy().starts_with("tisty-pdf-");
        let stale = one
            .metadata()
            .and_then(|meta| meta.modified())
            .is_ok_and(|at| at < old);
        if ours && stale {
            let _ = std::fs::remove_dir_all(one.path());
        }
    }
}
