use std::path::Path;
use std::sync::Mutex;

use tauri::Manager;
use tisty_core::witness::{self, Fact, channel};

use crate::{HERE, Session, held};

const NEWS: &str = include_str!("../../src/news.json");
const SEEN: &str = "news-seen";

#[derive(Default)]
pub struct Waiting(Mutex<Vec<String>>);

#[derive(serde::Deserialize)]
struct Told {
    version: String,
}

pub fn told() -> Vec<String> {
    serde_json::from_str::<Vec<Told>>(NEWS)
        .map(|all| all.into_iter().map(|one| one.version).collect())
        .unwrap_or_default()
}

/// The versions whose news this machine has not been told yet, newest first.
pub fn decide(fresh: bool, seen: Option<&str>, running: &str, told: &[String]) -> Vec<String> {
    let Ok(now) = semver::Version::parse(running) else {
        return Vec::new();
    };
    // A first install is greeted by the welcome, which already says what Tisty is.
    if fresh {
        return Vec::new();
    }
    let since = seen.and_then(|one| semver::Version::parse(one.trim()).ok());
    if since.as_ref().is_some_and(|since| *since >= now) {
        return Vec::new();
    }
    let parsed = || {
        told.iter()
            .filter_map(|said| semver::Version::parse(said).ok().map(|one| (one, said)))
    };
    let mut fresh_news: Vec<(semver::Version, &String)> = match &since {
        Some(since) => parsed()
            .filter(|(one, _)| *one <= now && one > since)
            .collect(),
        // Told nothing before: everything up to what it runs is news to it.
        None => parsed().filter(|(one, _)| *one <= now).collect(),
    };
    if fresh_news.is_empty() {
        fresh_news = parsed().filter(|(one, _)| is_minor_of(one, &now)).collect();
    }
    fresh_news.sort_by(|a, b| b.0.cmp(&a.0));
    fresh_news.dedup_by(|a, b| a.0 == b.0);
    fresh_news
        .into_iter()
        .map(|(_, said)| said.clone())
        .collect()
}

fn is_minor_of(one: &semver::Version, patch: &semver::Version) -> bool {
    patch.patch > 0
        && patch.pre.is_empty()
        && *one == semver::Version::new(patch.major, patch.minor, 0)
}

pub fn behind(seen: Option<&str>, running: &str) -> bool {
    let Ok(now) = semver::Version::parse(running) else {
        return false;
    };
    seen.and_then(|one| semver::Version::parse(one.trim()).ok())
        .is_none_or(|since| since < now)
}

pub fn raise(app: &tauri::AppHandle, config: &Path, fresh: bool) {
    let seen = std::fs::read_to_string(config.join(SEEN)).ok();
    let versions = decide(fresh, seen.as_deref(), HERE, &told());
    if versions.is_empty() && behind(seen.as_deref(), HERE) {
        mark(config);
    }
    match app.try_state::<Waiting>() {
        Some(waiting) => {
            if let Ok(mut held) = waiting.0.lock() {
                *held = versions;
            }
        }
        None => {
            app.manage(Waiting(Mutex::new(versions)));
        }
    }
}

fn mark(config: &Path) {
    if let Err(why) = tisty_core::store::write_atomic(&config.join(SEEN), HERE.as_bytes()) {
        witness::warn(
            channel::WINDOW,
            "the news told could not be written down",
            &[("why", Fact::Why(why.to_string()))],
        );
    }
}

#[tauri::command]
pub fn news(waiting: tauri::State<'_, Waiting>) -> Vec<String> {
    waiting
        .0
        .lock()
        .map(|held| held.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn news_seen(session: tauri::State<'_, Mutex<Session>>, waiting: tauri::State<'_, Waiting>) {
    if let Ok(mut held) = waiting.0.lock() {
        held.clear();
    }
    let config = held(&session).paths.config().to_path_buf();
    mark(&config);
}

#[cfg(test)]
#[path = "news_test.rs"]
mod tests;
