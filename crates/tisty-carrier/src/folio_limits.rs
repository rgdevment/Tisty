use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::policy::{self, Refused};

pub const LIMITS_URL: &str =
    "https://raw.githubusercontent.com/rgdevment/Folio-by-Tisty/main/docs/limits.json";
pub const SIGNATURE_URL: &str =
    "https://raw.githubusercontent.com/rgdevment/Folio-by-Tisty/main/docs/limits.json.minisig";

const KEY: &str = "RWTBkOCJN2ITUeWMK6V3gXmiJGo2hT6mX+hgcjpQ9F6QBO9sWnO84Mak";
const SIGNED_AS: &str = "limits.json";
pub(crate) const KEPT: &str = "folio-limits.json";
const ASKED: &str = "folio-limits-asked";
const ASKED_EVERY: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FolioLimits {
    pub serial: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_days: Option<Vec<u64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shares_at_most: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope_bytes_at_most: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_bytes_at_most: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rest_bytes_at_most: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pictures_at_most: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub picture_bytes_at_most: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pictures_bytes_at_most: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files_at_most: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub files_bytes_at_most: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uploads_of_a_root_an_hour: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upload_waits_minutes: Option<Vec<u64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uploads_halted_until: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oldest_contract_taken: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oldest_contract_served: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terms: Option<FolioTerms>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sharing_open: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolioTerms {
    pub version: String,
    pub es: String,
    pub en: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SharingNow {
    Open,
    Closed,
    Unreached,
}

fn read_under(said: &[u8], signature: &str, key: &str) -> Result<FolioLimits, Refused> {
    policy::believed(said, signature, || {
        policy::signed_as(said, signature, key, SIGNED_AS)
    })
}

pub fn kept(home: &Path) -> Option<FolioLimits> {
    policy::kept_at(&home.join(KEPT))
}

pub fn accepted(home: &Path, said: &[u8], signature: &str) -> Result<FolioLimits, Refused> {
    let limits = read_under(said, signature, KEY)?;
    if let Some(kept) = kept(home)
        && limits.serial < kept.serial
    {
        return Err(Refused::Behind {
            kept: kept.serial,
            said: limits.serial,
        });
    }
    // These limits only warn, and Folio applies its own, so a disk that will not keep them costs nothing here.
    if let Ok(bytes) = serde_json::to_vec(&limits) {
        let _ = std::fs::create_dir_all(home);
        let _ = tisty_core::store::write_atomic(&home.join(KEPT), &bytes);
    }
    Ok(limits)
}

pub fn uploads_halted(limits: Option<&FolioLimits>, now: u64) -> bool {
    policy::pause_holds(limits.and_then(|one| one.uploads_halted_until), now)
}

pub fn heed(
    home: &Path,
    bring: impl Fn(&str) -> Option<Vec<u8>>,
) -> Option<Result<FolioLimits, Refused>> {
    if !policy::ask_now(&home.join(ASKED), ASKED_EVERY) {
        return None;
    }
    heed_now(home, bring)
}

pub fn heed_now(
    home: &Path,
    bring: impl Fn(&str) -> Option<Vec<u8>>,
) -> Option<Result<FolioLimits, Refused>> {
    // The two files are asked for apart, so a publish between them looks like a bad signature.
    match brought_and_accepted(home, &bring)? {
        Err(Refused::Unsigned) => brought_and_accepted(home, &bring),
        told => Some(told),
    }
}

pub fn sharing_now(home: &Path, bring: impl Fn(&str) -> Option<Vec<u8>>) -> SharingNow {
    let due = policy::ask_now(&home.join(ASKED), ASKED_EVERY);
    match limits_for_sharing(home, due, &bring) {
        Some(limits) if limits.sharing_open == Some(true) => SharingNow::Open,
        Some(_) => SharingNow::Closed,
        None => {
            // A look that reached nothing must not hold the next try back for an hour.
            let _ = std::fs::remove_file(home.join(ASKED));
            SharingNow::Unreached
        }
    }
}

fn limits_for_sharing(
    home: &Path,
    due: bool,
    bring: &impl Fn(&str) -> Option<Vec<u8>>,
) -> Option<FolioLimits> {
    let held = kept(home);
    if !due && held.is_some() {
        return held;
    }
    match heed_now(home, bring)? {
        Ok(limits) => Some(limits),
        Err(Refused::Behind { .. }) => held,
        Err(_) => None,
    }
}

fn brought_and_accepted(
    home: &Path,
    bring: &impl Fn(&str) -> Option<Vec<u8>>,
) -> Option<Result<FolioLimits, Refused>> {
    let said = bring(LIMITS_URL)?;
    let signed = bring(SIGNATURE_URL)?;
    Some(accepted(home, &said, &String::from_utf8_lossy(&signed)))
}

#[cfg(test)]
#[path = "folio_limits_test.rs"]
mod tests;
