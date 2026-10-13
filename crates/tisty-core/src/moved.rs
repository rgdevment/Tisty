use std::path::Path;

use ed25519_dalek::{Signer, Verifier};
use serde::{Deserialize, Serialize};

use crate::config::Sync;
use crate::event::DeviceId;
use crate::signing::{SigningKey, hexed, unhexed};

/// Left in a place a machine stopped syncing with, so the machines still there learn where it went.
pub const NAMED: &str = "moved.json";
/// A word the place left behind has not taken yet, kept beside the settings until it has.
pub const OWED: &str = "word-owed.json";
const READ_AT_MOST: usize = 4 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Word {
    pub by: String,
    pub to: String,
    pub at: u64,
    #[serde(default)]
    pub store: String,
    #[serde(default)]
    pub place: String,
    sig: String,
}

pub struct Leaving<'a> {
    pub by: &'a DeviceId,
    pub to: &'a str,
    pub at: u64,
    pub store: &'a str,
    pub place: &'a str,
}

pub fn place_of(sync: &Sync) -> String {
    sync.named_alike()
}

pub fn store_of(data: &Path) -> String {
    crate::store::peek_identity(data.join("store"))
        .map_or_else(String::new, |id| id.trim().to_string())
}

fn over(word: &Word) -> Vec<u8> {
    let Word {
        by,
        to,
        at,
        store,
        place,
        ..
    } = word;
    format!("tisty.moved.2\u{0}{by}\u{0}{to}\u{0}{at}\u{0}{store}\u{0}{place}").into_bytes()
}

pub fn said(key: &SigningKey, leaving: Leaving) -> String {
    let mut word = Word {
        by: leaving.by.0.clone(),
        to: leaving.to.to_string(),
        at: leaving.at,
        store: leaving.store.to_string(),
        place: leaving.place.to_string(),
        sig: String::new(),
    };
    word.sig = hexed(&key.sign(&over(&word)).to_bytes());
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
    key.verify(&over(word), &sig).is_ok()
}

/// The word another machine left that this one has to heed: never its own, never one it cannot vouch for,
/// and never one signed for another store or another place, where it could only have been carried.
pub fn heeded(said: Option<&str>, data: &Path, me: &DeviceId, place: &str) -> Option<Word> {
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
    if word.store != store_of(data) || word.place != place {
        crate::witness::warn(
            crate::witness::channel::SYNC,
            "a word that a machine moved away was signed for another store or another place, so it was not heeded",
            &[("by", crate::witness::Fact::Id(word.by.clone()))],
        );
        return None;
    }
    Some(word)
}

#[cfg(test)]
#[path = "moved_test.rs"]
mod tests;
