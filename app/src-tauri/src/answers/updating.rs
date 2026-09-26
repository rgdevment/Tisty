use std::sync::Mutex;

use tauri::{Emitter, Manager};

use crate::{Answer, HERE, Refusal, Session, Updating, held, shop, translated, update};

/// Which window owns the dialogs the Store raises on its own.
#[cfg(not(windows))]
fn owner(_app: &tauri::AppHandle) -> Option<isize> {
    None
}

#[cfg(windows)]
fn owner(app: &tauri::AppHandle) -> Option<isize> {
    app.get_webview_window("main")
        .or_else(|| app.webview_windows().into_values().next())
        .and_then(|window| window.hwnd().ok())
        .map(|window| window.0 as isize)
}

#[tauri::command]
pub async fn update_ready(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    now_please: Option<bool>,
) -> Answer<Option<update::Ready>> {
    let kept = update::route();
    let (last, found, the_shop_said, wants) = {
        let held = held(&session);
        (
            held.config.checked_at,
            held.config.found_version.clone(),
            held.config.found_in_the_shop,
            held.config.candidates,
        )
    };
    let now = jiff::Timestamp::now();
    let asked = now_please.unwrap_or(false);

    // A copy kept by the Store asks the Store and nobody else: a release the Store is still
    // certifying is out for everyone else, and being told of one this copy cannot take is a
    // button that does nothing. What the Store last said stands until it says otherwise.
    if kept.route == update::Route::Store {
        let last_said = || match (the_shop_said, found.as_deref()) {
            (Some(true), Some(version)) => update::from_the_shop(version, HERE),
            (Some(false), Some(version)) => {
                update::published(HERE, version, wants).map(|_| update::on_its_way(Some(version)))
            }
            _ => None,
        };
        let Some(window) = owner(&app) else {
            return Ok(last_said());
        };
        if !asked && !update::due(last, now) {
            return Ok(last_said());
        }
        let shelf = tauri::async_runtime::spawn_blocking(move || shop::asked(window, asked))
            .await
            .unwrap_or(shop::Shelf::Silent);
        let manifest = tauri::async_runtime::spawn_blocking(update::fetch)
            .await
            .ok()
            .flatten();
        let out = manifest
            .as_deref()
            .and_then(|said| update::published(HERE, said, wants));
        let waiting_for_it =
            |session: &tauri::State<'_, Mutex<Session>>| -> Answer<Option<update::Ready>> {
                held(session).keep(|c| {
                    c.checked_at = Some(now);
                    c.found_version = out.clone();
                    c.found_in_the_shop = out.as_ref().map(|_| false);
                })?;
                Ok(out
                    .as_deref()
                    .map(|version| update::on_its_way(Some(version))))
            };
        return match shelf {
            shop::Shelf::Waiting(version) => {
                let seen = update::from_the_shop(&version, HERE);
                held(&session).keep(|c| {
                    c.checked_at = Some(now);
                    c.found_version = seen.as_ref().map(|one| one.version.clone());
                    c.found_in_the_shop = seen.as_ref().map(|_| true);
                })?;
                Ok(seen)
            }
            // Already down and waiting for this window to close: there is nothing to fetch, so
            // it is announced without the button that would ask the Store for it again.
            shop::Shelf::Landed(version) => {
                let seen = update::from_the_shop(&version, HERE).map(|one| update::Ready {
                    installs: false,
                    ..one
                });
                held(&session).keep(|c| {
                    c.checked_at = Some(now);
                    c.found_version = seen.as_ref().map(|one| one.version.clone());
                    c.found_in_the_shop = seen.as_ref().map(|_| true);
                })?;
                Ok(seen)
            }
            shop::Shelf::Queued => {
                held(&session).keep(|c| {
                    c.checked_at = Some(now);
                    c.found_version = out.clone();
                    c.found_in_the_shop = Some(false);
                })?;
                Ok(Some(update::on_its_way(out.as_deref())))
            }
            shop::Shelf::Current if out.is_some() => waiting_for_it(&session),
            shop::Shelf::Current if manifest.is_some() => {
                held(&session).keep(|c| {
                    c.checked_at = Some(now);
                    c.found_version = None;
                    c.found_in_the_shop = None;
                })?;
                Ok(None)
            }
            shop::Shelf::Current => {
                held(&session).keep(|c| c.checked_at = Some(now))?;
                Ok(last_said())
            }
            // Asked and not answered is still asked: the next look waits its turn like any other,
            // and a copy the Store never signed is not asked again every few hours.
            shop::Shelf::Silent if out.is_some() => waiting_for_it(&session),
            shop::Shelf::Silent => {
                held(&session).keep(|c| c.checked_at = Some(now))?;
                if asked {
                    return Err(Refusal::of("updateUnanswered"));
                }
                Ok(last_said())
            }
        };
    }

    if !asked && !update::due(last, now) {
        return Ok(update::remembered(HERE, found.as_deref(), kept, wants));
    }

    let manifest = tauri::async_runtime::spawn_blocking(update::fetch)
        .await
        .map_err(|_| Refusal::of("internal"))?;

    // A look that never answered says nothing about whether an update is owed, so what was found
    // before stays where it is.
    let Some(manifest) = manifest else {
        return Ok(update::remembered(HERE, found.as_deref(), kept, wants));
    };

    let seen = update::newer(HERE, &manifest, kept, wants);
    let version = seen.as_ref().map(|one| one.version.clone());
    held(&session).keep(|c| {
        c.checked_at = Some(now);
        c.found_version = version;
        c.found_in_the_shop = None;
    })?;
    Ok(seen)
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Underway {
    stage: &'static str,
    far: u64,
}

#[tauri::command]
pub async fn update_install(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, Updating>,
) -> Answer<()> {
    use tauri_plugin_updater::UpdaterExt;

    let _busy = alone
        .inner()
        .0
        .claim()
        .ok_or_else(|| Refusal::of("updateBusy"))?;

    let kept = update::route();
    if kept.route == update::Route::Store {
        return take_from_the_shop(app).await;
    }
    if !update::self_installs(kept.route) || update::from_a_mount() {
        return Err(Refusal::of("updateNotHere"));
    }

    let (found, wants) = {
        let held = held(&session);
        (held.config.found_version.clone(), held.config.candidates)
    };
    let Some(want) = update::remembered(HERE, found.as_deref(), kept, wants).map(|one| one.version)
    else {
        return Err(moved_on(&session, kept, wants).await);
    };

    let asked = want.clone();
    let mut building = app.updater_builder();
    if let Some(platform) = update::platform(translated()) {
        building = building.target(platform);
    }
    let update = building
        .endpoints(
            update::feeds_for(&want)
                .into_iter()
                .map(|one| one.parse())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| Refusal::of("internal"))?,
        )
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?
        // Pinned to what the person was shown, so a feed that moves in between cannot quietly
        // hand them a different version than the one they agreed to.
        .version_comparator(move |_, release| release.version.to_string() == asked)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?
        .check()
        .await
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?;

    let Some(mut update) = update else {
        return Err(moved_on(&session, kept, wants).await);
    };

    // The feed names the address the installer comes from, so it is checked against where our
    // releases actually live before a single byte is asked for.
    if !update::ours(update.download_url.as_str()) {
        return Err(Refusal::of("updateElsewhere"));
    }
    // The plugin builds the download with no deadline of its own, and a server that dribbles
    // bytes forever would otherwise be waited on forever.
    update.timeout = Some(std::time::Duration::from_secs(600));

    let telling = app.clone();
    let done = app.clone();
    let mut carried: u64 = 0;
    let mut said = 0;
    update
        .download_and_install(
            move |chunk, whole| {
                // The callback hands over the length of one chunk, not how much has arrived.
                carried += chunk as u64;
                let far = whole.map_or(0, |all| carried * 100 / all.max(1));
                if far != said {
                    said = far;
                    let _ = telling.emit(
                        "updating",
                        Underway {
                            stage: "getting",
                            far,
                        },
                    );
                }
            },
            // The last thing anyone sees on Windows: the installer takes the process with it and
            // nothing after the await ever runs.
            move || {
                let _ = done.emit(
                    "updating",
                    Underway {
                        stage: "installing",
                        far: 100,
                    },
                );
            },
        )
        .await
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?;

    // Only macOS gets this far, and restarting has to happen where the app loop lives.
    let handle = app.clone();
    app.run_on_main_thread(move || handle.restart())
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?;
    Ok(())
}

/// The version the person was shown is not on the feed any more — a copy left closed for weeks
/// remembers an offer the feed has moved past. Looking again on the spot keeps what is offered
/// now and says so, rather than leaving them with a button that only ever fails.
async fn moved_on(
    session: &tauri::State<'_, Mutex<Session>>,
    kept: update::Kept,
    wants: Option<bool>,
) -> Refusal {
    let manifest = tauri::async_runtime::spawn_blocking(update::fetch)
        .await
        .ok()
        .flatten();
    let seen = manifest.and_then(|manifest| update::newer(HERE, &manifest, kept, wants));
    let version = seen.map(|one| one.version);
    let _ = held(session).keep(|c| {
        c.found_version = version.clone();
        c.found_in_the_shop = None;
    });
    match version {
        Some(version) => Refusal::about("updateMoved", version),
        None => Refusal::of("updateGone"),
    }
}

/// Windows ends the process to put the new package in place, so the progress left behind is the
/// last thing anyone sees.
async fn take_from_the_shop(app: tauri::AppHandle) -> Answer<()> {
    let window = owner(&app).ok_or_else(|| Refusal::of("updateNotHere"))?;
    let telling = app.clone();
    let taken = tauri::async_runtime::spawn_blocking(move || {
        let mut said: (&'static str, u64) = ("", 0);
        shop::take(window, move |stage, far| {
            if said != (stage, far) {
                said = (stage, far);
                let _ = telling.emit("updating", Underway { stage, far });
            }
        })
    })
    .await
    .map_err(|_| Refusal::of("internal"))?;

    match taken {
        Ok(()) => Ok(()),
        Err(shop::Trouble::Gone) => Err(Refusal::of("updateGone")),
        Err(shop::Trouble::Stopped) => Err(Refusal::of("updateStopped")),
        Err(shop::Trouble::Failed(why)) => Err(Refusal::about("updateFailed", why)),
    }
}

/// A copy only ever reaches the candidates' track from here. Turning it off does not walk it back:
/// a candidate already installed stays one until a stable release passes it.
#[tauri::command]
pub fn update_candidates(session: tauri::State<'_, Mutex<Session>>, wants: bool) -> Answer<()> {
    held(&session).keep(|c| {
        c.candidates = Some(wants);
        // What was found under the old answer says nothing about the new one.
        c.checked_at = None;
        c.found_version = None;
        c.found_in_the_shop = None;
    })?;
    Ok(())
}
