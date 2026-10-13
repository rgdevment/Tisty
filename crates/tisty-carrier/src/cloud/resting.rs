use std::path::Path;
use std::time::Duration;

const FILE: &str = "resting.json";
// A provider that asks for a day still hears from the person within the hour.
const RESTED_AT_MOST: Duration = Duration::from_secs(60 * 60);

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Rest {
    until_ms: u64,
}

/// How much longer the provider asked to be left alone, if it still is.
pub fn left(home: &Path) -> Option<Duration> {
    let rest: Rest = std::fs::read(home.join(FILE))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())?;
    let now = super::now_ms();
    let left = Duration::from_millis(rest.until_ms.checked_sub(now)?);
    // A clock set back would otherwise keep the cloud silent for longer than anyone asked.
    (!left.is_zero() && left <= RESTED_AT_MOST).then_some(left)
}

pub fn rest(home: &Path, wait: Duration) {
    let until_ms = super::now_ms()
        .saturating_add(u64::try_from(wait.min(RESTED_AT_MOST).as_millis()).unwrap_or(u64::MAX));
    if let Ok(bytes) = serde_json::to_vec(&Rest { until_ms }) {
        let _ = std::fs::create_dir_all(home);
        let _ = tisty_core::store::write_atomic(&home.join(FILE), &bytes);
    }
}

pub fn rested(home: &Path) {
    let _ = std::fs::remove_file(home.join(FILE));
}

#[cfg(test)]
#[path = "resting_test.rs"]
mod tests;
