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
const KEPT: &str = "folio-limits.json";
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
}

fn read_under(said: &[u8], signature: &str, key: &str) -> Result<FolioLimits, Refused> {
    if said.len() > policy::READ_AT_MOST || signature.len() > policy::READ_AT_MOST {
        return Err(Refused::TooBig);
    }
    if !policy::signed_as(said, signature, key, SIGNED_AS) {
        return Err(Refused::Unsigned);
    }
    serde_json::from_slice(said).map_err(|_| Refused::Unreadable)
}

pub fn kept(home: &Path) -> Option<FolioLimits> {
    std::fs::read(home.join(KEPT))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
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
    let bytes = serde_json::to_vec(&limits).map_err(|_| Refused::Unreadable)?;
    let _ = std::fs::create_dir_all(home);
    tisty_core::store::write_atomic(&home.join(KEPT), &bytes).map_err(|_| Refused::Unreadable)?;
    Ok(limits)
}

pub fn uploads_halted(limits: Option<&FolioLimits>, now: u64) -> bool {
    limits
        .and_then(|one| one.uploads_halted_until)
        .is_some_and(|until| until > now && until <= now.saturating_add(policy::PAUSED_AT_MOST))
}

/// Returns nothing when it was asked within the hour, or when either file did not arrive.
pub fn heed(
    home: &Path,
    bring: impl Fn(&str) -> Option<Vec<u8>>,
) -> Option<Result<FolioLimits, Refused>> {
    let asked = home.join(ASKED);
    let recently = std::fs::metadata(&asked)
        .and_then(|one| one.modified())
        .ok()
        .and_then(|at| at.elapsed().ok())
        .is_some_and(|since| since < ASKED_EVERY);
    if recently {
        return None;
    }
    let _ = std::fs::create_dir_all(home);
    let _ = std::fs::write(&asked, b"");
    let said = bring(LIMITS_URL)?;
    let signed = bring(SIGNATURE_URL)?;
    Some(accepted(home, &said, &String::from_utf8_lossy(&signed)))
}

#[cfg(test)]
#[path = "folio_limits_test.rs"]
mod tests;
