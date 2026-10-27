use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Refusal, Session, elsewhere, held};

const PATIENCE: std::time::Duration = std::time::Duration::from_secs(20);

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    slug: String,
    title: &'static str,
    section: &'static str,
    order: usize,
    twin: String,
}

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pictures {
    weighs: u64,
    missing: u64,
    kept: BTreeMap<String, String>,
}

#[tauri::command]
pub fn help_pages(spanish: bool) -> Vec<Entry> {
    let others = tisty_help::pages(!spanish);
    tisty_help::pages(spanish)
        .iter()
        .map(|one| Entry {
            slug: one.slug(),
            title: one.title(),
            section: one.section,
            order: one.order,
            twin: others
                .iter()
                .find(|other| other.page == one.twin)
                .map(tisty_help::Page::slug)
                .unwrap_or_default(),
        })
        .collect()
}

#[tauri::command]
pub fn help_page(spanish: bool, slug: String) -> Option<String> {
    tisty_help::found(spanish, &slug).map(|one| one.readable(spanish))
}

pub fn folder(cache: &Path) -> PathBuf {
    cache.join("help-pictures")
}

fn kept_at(folder: &Path, url: &str) -> PathBuf {
    let kind = url
        .rsplit_once('.')
        .map(|(_, kind)| kind.to_ascii_lowercase())
        .filter(|kind| kind.len() <= 4 && kind.chars().all(|one| one.is_ascii_alphanumeric()))
        .unwrap_or_else(|| "png".to_string());
    let named = tisty_core::attach::printed(url.as_bytes());
    folder.join(format!("{}.{kind}", &named[..16]))
}

fn standing(folder: &Path, spanish: bool) -> Pictures {
    let mut said = Pictures::default();
    for url in tisty_help::pictures(spanish) {
        let bytes = tisty_help::weight(&url).unwrap_or_default();
        said.weighs += bytes;
        let at = kept_at(folder, &url);
        if at.is_file() {
            said.kept.insert(url, at.display().to_string());
        } else {
            said.missing += bytes;
        }
    }
    said
}

fn brought(client: &reqwest::blocking::Client, url: &str, to: &Path) -> Result<(), String> {
    let asked = client.get(url).send().map_err(|why| why.to_string())?;
    if !asked.status().is_success() {
        return Err(asked.status().to_string());
    }
    let bytes = asked.bytes().map_err(|why| why.to_string())?;
    tisty_core::store::write_atomic(to, &bytes).map_err(|why| why.to_string())
}

fn bring_all(folder: &Path, spanish: bool) -> (Pictures, usize) {
    let _ = std::fs::create_dir_all(folder);
    let wanted = tisty_help::pictures(spanish);
    let Ok(client) = reqwest::blocking::Client::builder()
        .timeout(PATIENCE)
        .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
        .build()
    else {
        return (standing(folder, spanish), wanted.len());
    };
    let mut failed = 0;
    for url in wanted {
        let at = kept_at(folder, &url);
        if !at.is_file() && brought(&client, &url, &at).is_err() {
            failed += 1;
        }
    }
    (standing(folder, spanish), failed)
}

fn folder_of(session: &Mutex<Session>) -> PathBuf {
    folder(held(session).paths.cache())
}

#[tauri::command(async)]
pub fn help_pictures(session: tauri::State<'_, Mutex<Session>>, spanish: bool) -> Pictures {
    standing(&folder_of(&session), spanish)
}

#[tauri::command]
pub async fn help_pictures_fetch(
    session: tauri::State<'_, Mutex<Session>>,
    spanish: bool,
) -> Answer<Pictures> {
    let at = folder_of(&session);
    let (said, failed) = elsewhere(move || bring_all(&at, spanish)).await?;
    witness::note(
        channel::WINDOW,
        "the help's pictures were brought on asking",
        &[("failed", Fact::Count(failed))],
    );
    if failed > 0 && said.kept.is_empty() {
        return Err(Refusal::of("helpPicturesAway"));
    }
    Ok(said)
}

#[cfg(test)]
#[path = "helping_test.rs"]
mod tests;
