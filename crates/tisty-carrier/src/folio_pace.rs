use std::collections::BTreeMap;
use std::path::Path;

use crate::folio_limits::FolioLimits;
use crate::policy;

const KEPT: &str = "folio-sync-presses.json";
const QUIET_STARTS_OVER: u64 = 30 * 60;
const HOUR: u64 = 60 * 60;
const MINUTE: u64 = 60;

fn free_at(presses: &[u64], now: u64, waits: &[u64], an_hour: Option<u64>) -> Option<u64> {
    let last = *presses.last()?;
    let mut streak = 1;
    for pair in presses.windows(2).rev() {
        if pair[1].saturating_sub(pair[0]) >= QUIET_STARTS_OVER {
            break;
        }
        streak += 1;
    }
    let stands = now.saturating_sub(last) < QUIET_STARTS_OVER;
    let waited = stands
        .then(|| waits.get(streak - 1).or(waits.last()).copied())
        .flatten()
        .map(|minutes| last + minutes * MINUTE)
        .filter(|free| *free > now);
    let in_the_hour: Vec<u64> = presses
        .iter()
        .copied()
        .filter(|at| now.saturating_sub(*at) < HOUR)
        .collect();
    let spent = an_hour
        .filter(|most| in_the_hour.len() as u64 >= *most)
        .and_then(|_| in_the_hour.first().map(|oldest| oldest + HOUR));
    waited.into_iter().chain(spent).max()
}

fn kept(home: &Path) -> BTreeMap<String, Vec<u64>> {
    policy::kept_at(&home.join(KEPT)).unwrap_or_default()
}

pub fn press(home: &Path, id: &str, now: u64, limits: Option<&FolioLimits>) -> Result<(), u64> {
    let mut all = kept(home);
    let presses = all.entry(id.to_owned()).or_default();
    let waits = limits
        .and_then(|one| one.upload_waits_minutes.as_deref())
        .unwrap_or_default();
    let an_hour = limits.and_then(|one| one.uploads_of_a_root_an_hour);
    if let Some(free) = free_at(presses, now, waits, an_hour) {
        return Err(free);
    }
    presses.retain(|at| now.saturating_sub(*at) < HOUR + QUIET_STARTS_OVER);
    presses.push(now);
    // The server holds to its own numbers, so a disk that will not keep these costs nothing but a wait.
    if let Ok(bytes) = serde_json::to_vec(&all) {
        let _ = std::fs::create_dir_all(home);
        let _ = tisty_core::store::write_atomic(&home.join(KEPT), &bytes);
    }
    Ok(())
}

#[cfg(test)]
#[path = "folio_pace_test.rs"]
mod tests;
