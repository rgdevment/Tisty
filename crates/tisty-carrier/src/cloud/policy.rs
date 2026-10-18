use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};

use crate::Limits;

pub const READ_AT_MOST: usize = 16 * 1024;
/// A signed word may lower every limit, but never lift one past twice what this build ships with.
pub const RAISED_AT_MOST: u64 = 2;
pub const PAUSED_AT_MOST: u64 = 7 * 24 * 60 * 60;

const KEPT: &str = "cloud-policy.json";
const PAUSED_HERE: &str = "paused-here";

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Policy {
    pub serial: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub units_a_day: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes_a_day: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub halted_until: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    TooBig,
    Unsigned,
    Unreadable,
    Behind { kept: u64, said: u64 },
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |one| one.as_secs())
}

/// Whether these bytes, exactly as they came, carry a minisign signature by this key.
pub fn signed(said: &[u8], signature: &str, key: &str) -> bool {
    let (Ok(key), Ok(signature)) = (PublicKey::from_base64(key), Signature::decode(signature))
    else {
        return false;
    };
    key.verify(said, &signature, false).is_ok()
}

// The signature is checked on the bytes as they came, before a single field is believed.
pub fn read(said: &[u8], signature: &str, key: &str) -> Result<Policy, Refused> {
    if said.len() > READ_AT_MOST || signature.len() > READ_AT_MOST {
        return Err(Refused::TooBig);
    }
    if !signed(said, signature, key) {
        return Err(Refused::Unsigned);
    }
    serde_json::from_slice(said).map_err(|_| Refused::Unreadable)
}

pub fn kept(home: &Path) -> Option<Policy> {
    std::fs::read(home.join(KEPT))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
}

/// The newest signed word is kept for good; one older than it is a replay and is refused.
pub fn accepted(home: &Path, said: &[u8], signature: &str, key: &str) -> Result<Policy, Refused> {
    let policy = read(said, signature, key)?;
    if let Some(kept) = kept(home)
        && policy.serial < kept.serial
    {
        return Err(Refused::Behind {
            kept: kept.serial,
            said: policy.serial,
        });
    }
    let bytes = serde_json::to_vec(&policy).map_err(|_| Refused::Unreadable)?;
    let _ = std::fs::create_dir_all(home);
    tisty_core::store::write_atomic(&home.join(KEPT), &bytes).map_err(|_| Refused::Unreadable)?;
    Ok(policy)
}

pub fn applied(mut limits: Limits, policy: Option<&Policy>) -> Limits {
    if let Some(policy) = policy {
        limits.units_a_day = within(limits.units_a_day, policy.units_a_day);
        limits.bytes_a_day = within(limits.bytes_a_day, policy.bytes_a_day);
    }
    limits
}

pub(super) fn within(shipped: u64, said: Option<u64>) -> u64 {
    said.map_or(shipped, |said| {
        said.min(shipped.saturating_mul(RAISED_AT_MOST))
    })
}

/// A pause from afar holds only while its date is ahead and never more than a week out, so a lost word cannot stop Tisty for good.
pub fn halted(policy: Option<&Policy>, now: u64) -> bool {
    policy
        .and_then(|one| one.halted_until)
        .is_some_and(|until| until > now && until <= now.saturating_add(PAUSED_AT_MOST))
}

pub fn paused_here(home: &Path) -> bool {
    home.join(PAUSED_HERE).exists()
}

pub fn pause_here(home: &Path, paused: bool) -> std::io::Result<()> {
    let at = home.join(PAUSED_HERE);
    match paused {
        true => {
            std::fs::create_dir_all(home)?;
            std::fs::write(at, b"")
        }
        false => match std::fs::remove_file(at) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => other,
        },
    }
}

#[cfg(test)]
#[path = "policy_test.rs"]
mod tests;
