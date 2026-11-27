use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::Manager;
use tisty_core::witness::{self, Fact, channel};

use crate::{Fetching, HERE, Session, held, update};

const KEPT: &str = "update";
const PACKAGE: &str = "waiting.pkg";
const SAID: &str = "waiting.json";
const PART: &str = "waiting.part";

#[derive(serde::Serialize, serde::Deserialize)]
struct Said {
    version: String,
    raw: serde_json::Value,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum When {
    Opening,
    Leaving,
}

fn kept(paths: &tisty_core::Paths) -> PathBuf {
    paths.cache().join(KEPT)
}

fn said(at: &Path) -> Option<Said> {
    serde_json::from_slice(&std::fs::read(at.join(SAID)).ok()?).ok()
}

pub fn forget(paths: &tisty_core::Paths) {
    let _ = std::fs::remove_dir_all(kept(paths));
}

fn waits_for(paths: &tisty_core::Paths, version: &str) -> bool {
    let at = kept(paths);
    at.join(PACKAGE).is_file() && said(&at).is_some_and(|one| one.version == version)
}

pub async fn fetch_ahead(app: tauri::AppHandle, version: String) {
    if !crate::refusing::takes_itself_there() {
        return;
    }
    let paths = held(&app.state::<Mutex<Session>>()).paths.clone();
    if waits_for(&paths, &version) {
        return;
    }
    let fetching = app.state::<Fetching>();
    let Some(_busy) = fetching.inner().0.claim() else {
        return;
    };
    match fetched(&app, &paths, &version).await {
        Ok(()) => witness::note(
            channel::UPDATE,
            "an update is downloaded and waits for Tisty to close or open",
            &[("to", Fact::Id(version))],
        ),
        Err(why) => {
            let _ = std::fs::remove_file(kept(&paths).join(PART));
            witness::warn(
                channel::UPDATE,
                "an update could not be downloaded ahead",
                &[("to", Fact::Id(version)), ("why", Fact::Why(why))],
            );
        }
    }
}

async fn fetched(
    app: &tauri::AppHandle,
    paths: &tisty_core::Paths,
    version: &str,
) -> Result<(), String> {
    let mut update = crate::answers::updating::offered(app, version)
        .await
        .map_err(|why| why.code.to_string())?
        .ok_or_else(|| "the feed no longer offers it".to_string())?;
    if !update::ours(update.download_url.as_str()) {
        return Err("the feed names a download from elsewhere".into());
    }
    let read = update.raw_json.clone();
    let want = version.to_string();
    let vouched =
        tauri::async_runtime::spawn_blocking(move || update::channel_vouched(&read, &want))
            .await
            .map_err(|why| why.to_string())?;
    if !vouched {
        return Err("the feed's signature does not vouch for what was read".into());
    }
    update.timeout = Some(std::time::Duration::from_secs(600));
    let bytes = update
        .download(|_, _| {}, || {})
        .await
        .map_err(|why| why.to_string())?;

    let at = kept(paths);
    forget(paths);
    std::fs::create_dir_all(&at).map_err(|why| why.to_string())?;
    std::fs::write(at.join(PART), &bytes).map_err(|why| why.to_string())?;
    std::fs::rename(at.join(PART), at.join(PACKAGE)).map_err(|why| why.to_string())?;
    let said = Said {
        version: version.to_string(),
        raw: update.raw_json,
    };
    let said = serde_json::to_vec(&said).map_err(|why| why.to_string())?;
    tisty_core::store::write_atomic(&at.join(SAID), &said).map_err(|why| why.to_string())
}

pub fn on_leaving<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let Some(session) = app.try_state::<Mutex<Session>>() else {
        return;
    };
    installs(app, &mut held(&session), When::Leaving);
}

pub fn installs<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    session: &mut Session,
    when: When,
) -> bool {
    let at = kept(&session.paths);
    let _ = std::fs::remove_file(at.join(PART));
    let Some(said) = said(&at) else {
        return false;
    };
    if session.config.auto_tried.as_deref() == Some(said.version.as_str()) {
        if !update::later(HERE, &said.version) {
            forget(&session.paths);
        } else if session.config.found_version.is_none() {
            let _ = session.keep(|c| c.found_version = Some(said.version));
        }
        return false;
    }
    let wanted = session.config.auto_updates == Some(true) && crate::refusing::takes_itself_there();
    if !wanted
        || !update::still_offered(&said.version, HERE, session.config.found_version.as_deref())
    {
        forget(&session.paths);
        return false;
    }
    if !replaceable() {
        return false;
    }
    let Ok(bytes) = std::fs::read(at.join(PACKAGE)) else {
        forget(&session.paths);
        return false;
    };
    let found = session.config.found_version.clone();
    let version = said.version.clone();
    if session
        .keep(|c| {
            c.auto_tried = Some(version.clone());
            c.found_version = None;
        })
        .is_err()
    {
        return false;
    }
    match installed(app, said, &bytes, when) {
        Ok(()) => true,
        Err(why) => {
            let _ = session.keep(|c| c.found_version = found);
            forget(&session.paths);
            witness::warn(
                channel::UPDATE,
                "a downloaded update could not be installed",
                &[("to", Fact::Id(version)), ("why", Fact::Why(why))],
            );
            false
        }
    }
}

fn installed<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    said: Said,
    bytes: &[u8],
    when: When,
) -> Result<(), String> {
    let update = restored(app, &said.version, said.raw.clone(), None)?;
    let update = match update::updater_key(&update.signature) {
        None => update,
        Some(spare) => restored(app, &said.version, said.raw, Some(spare))?,
    };
    #[cfg(windows)]
    let update = update.restart_after_install(when == When::Opening);
    if when == When::Leaving {
        let _ = app.remove_tray_by_id("tisty");
    }
    update.install(bytes).map_err(|why| why.to_string())?;
    if when == When::Opening {
        app.restart();
    }
    Ok(())
}

fn restored<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    version: &str,
    raw: serde_json::Value,
    key: Option<&'static str>,
) -> Result<tauri_plugin_updater::Update, String> {
    use tauri_plugin_updater::UpdaterExt;

    let asked = version.to_string();
    let mut building = app.updater_builder();
    if let Some(platform) = update::platform(crate::desktop::translated()) {
        building = building.target(platform);
    }
    if let Some(key) = key {
        building = building.pubkey(key);
    }
    building
        .version_comparator(move |_, release| release.version.to_string() == asked)
        .build()
        .and_then(|updater| updater.restore_update(raw))
        .map_err(|why| why.to_string())?
        .ok_or_else(|| "what was kept no longer names this version".to_string())
}

#[cfg(target_os = "macos")]
fn replaceable() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|exe| crate::waking::bundle_of(&exe))
        .and_then(|bundle| bundle.parent().map(Path::to_path_buf))
        .is_some_and(|folder| {
            let probe = folder.join(format!(".tisty-{}", std::process::id()));
            let wrote = std::fs::write(&probe, b"").is_ok();
            let _ = std::fs::remove_file(&probe);
            wrote
        })
}

#[cfg(not(target_os = "macos"))]
const fn replaceable() -> bool {
    true
}

#[cfg(test)]
#[path = "awaiting_test.rs"]
mod tests;
