use std::path::Path;

use ed25519_dalek::{Signer, Verifier};
use serde::{Deserialize, Serialize};

use crate::event::DeviceId;
use crate::signing::{SigningKey, hexed, unhexed};

/// Left in a place a machine stopped syncing with, so the machines still there learn where it went.
pub const NAMED: &str = "moved.json";
const READ_AT_MOST: usize = 4 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Word {
    pub by: String,
    pub to: String,
    pub at: u64,
    sig: String,
}

fn over(by: &str, to: &str, at: u64) -> Vec<u8> {
    format!("tisty.moved\u{0}{by}\u{0}{to}\u{0}{at}").into_bytes()
}

pub fn said(key: &SigningKey, by: &DeviceId, to: &str, at: u64) -> String {
    let word = Word {
        by: by.0.clone(),
        to: to.to_string(),
        at,
        sig: hexed(&key.sign(&over(&by.0, to, at)).to_bytes()),
    };
    serde_json::to_string(&word).unwrap_or_default()
}

pub fn read(said: &str) -> Option<Word> {
    if said.len() > READ_AT_MOST {
        return None;
    }
    serde_json::from_str(said).ok()
}

// Only a key the person confirmed here may stop this machine, or anyone who can write in the folder could.
pub fn vouched(word: &Word, data: &Path) -> bool {
    let Some(confirmed) = crate::vouched::confirmed(data, &DeviceId(word.by.clone())) else {
        return false;
    };
    let Some(key) = crate::signing::read(&confirmed.key) else {
        return false;
    };
    let Some(sig) = unhexed::<64>(&word.sig) else {
        return false;
    };
    let Ok(sig) = ed25519_dalek::Signature::from_slice(&sig) else {
        return false;
    };
    key.verify(&over(&word.by, &word.to, word.at), &sig).is_ok()
}

/// The word another machine left that this one has to heed: never its own, never one it cannot vouch for.
pub fn heeded(said: Option<&str>, data: &Path, me: &DeviceId) -> Option<Word> {
    let word = read(said?)?;
    if word.by == me.0 {
        return None;
    }
    if !vouched(&word, data) {
        crate::witness::warn(
            crate::witness::channel::SYNC,
            "a word that a machine moved away was left here without a signature this machine can vouch for, so it was not heeded",
            &[("by", crate::witness::Fact::Id(word.by.clone()))],
        );
        return None;
    }
    Some(word)
}

#[cfg(test)]
#[path = "moved_test.rs"]
mod tests;
