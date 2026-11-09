use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::policy;

const KEPT: &str = "folio-new-shares-today.json";
const DAY: u64 = 24 * 60 * 60;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Today {
    day: u64,
    ids: Vec<String>,
}

fn kept(home: &Path, now: u64) -> Today {
    policy::kept_at::<Today>(&home.join(KEPT))
        .filter(|one| one.day == now / DAY)
        .unwrap_or(Today {
            day: now / DAY,
            ids: Vec::new(),
        })
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
    if let Ok(bytes) = serde_json::to_vec(&today) {
        let _ = std::fs::create_dir_all(home);
        let _ = tisty_core::store::write_atomic(&home.join(KEPT), &bytes);
    }
}

#[cfg(test)]
#[path = "folio_today_test.rs"]
mod tests;
