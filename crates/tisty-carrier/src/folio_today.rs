use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::policy;

const KEPT: &str = "folio-new-shares-today.json";
const DAY: u64 = 24 * 60 * 60;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Today {
    day: u64,
    ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    full_until: Option<u64>,
}

fn kept(home: &Path, now: u64) -> Today {
    let read = policy::kept_at::<Today>(&home.join(KEPT)).unwrap_or_default();
    let full_until = read.full_until.filter(|until| *until > now);
    if read.day == now / DAY {
        return Today { full_until, ..read };
    }
    Today {
        day: now / DAY,
        ids: Vec::new(),
        full_until,
    }
}

fn kept_again(home: &Path, today: &Today) {
    if let Ok(bytes) = serde_json::to_vec(today) {
        let _ = std::fs::create_dir_all(home);
        let _ = tisty_core::store::write_atomic(&home.join(KEPT), &bytes);
    }
}

// Folio counts by the UTC day, so a day here starts when it does there.
pub fn shared_today(home: &Path, now: u64) -> u64 {
    kept(home, now).ids.len() as u64
}

pub fn counted(home: &Path, id: &str, now: u64) {
    let mut today = kept(home, now);
    if today.ids.iter().any(|one| one == id) {
        return;
    }
    today.ids.push(id.to_owned());
    kept_again(home, &today);
}

// Another computer of the same person may have filled the day, and only Folio's refusal says so.
pub fn full_until(home: &Path, until: u64, now: u64) {
    let mut today = kept(home, now);
    today.full_until = today.full_until.max(Some(until));
    kept_again(home, &today);
}

pub fn full_now(home: &Path, now: u64) -> bool {
    kept(home, now).full_until.is_some()
}

#[cfg(test)]
#[path = "folio_today_test.rs"]
mod tests;
