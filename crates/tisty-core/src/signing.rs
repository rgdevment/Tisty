use std::path::PathBuf;

pub use ed25519_dalek::SigningKey;
use ed25519_dalek::VerifyingKey;

use crate::event::DeviceId;

pub const KEEP: &str = ".device-key";

pub fn kept_at(paths: &crate::Paths, device: &DeviceId) -> Option<PathBuf> {
    crate::store::is_device_name(&device.0)
        .then(|| paths.private().join(format!("{}{KEEP}", device.0)))
}

pub fn shown(key: &SigningKey) -> String {
    shown_of(&key.verifying_key())
}

pub fn shown_of(key: &VerifyingKey) -> String {
    hexed(key.as_bytes())
}

/// Only a machine's own word for its own key counts, and the first it publishes stands. False
/// when one already stood and differs, which is the door a rekey would otherwise walk through.
pub fn published(
    into: &mut std::collections::BTreeMap<DeviceId, String>,
    by: &DeviceId,
    whose: &DeviceId,
    said: &str,
) -> bool {
    if by != whose {
        return true;
    }
    let Some(key) = read(said) else {
        return true;
    };
    let shown = shown_of(&key);
    *into.entry(whose.clone()).or_insert(shown.clone()) == shown
}

pub fn read(said: &str) -> Option<VerifyingKey> {
    let bytes = unhexed::<32>(said)?;
    VerifyingKey::from_bytes(&bytes).ok()
}

/// What this machine already signs with, without minting one for the asking: a report that reads
/// the key must not be the thing that creates it.
pub fn shown_kept(paths: &crate::Paths, device: &DeviceId) -> Option<String> {
    let held = std::fs::read(kept_at(paths, device)?).ok()?;
    let one = <[u8; 32]>::try_from(held.as_slice()).ok()?;
    Some(shown(&SigningKey::from_bytes(&one)))
}

pub fn mine(paths: &crate::Paths, device: &DeviceId) -> Option<SigningKey> {
    use crate::store::identity::{Kept, kept, minted};

    let at = kept_at(paths, device)?;
    let one = match kept(
        paths,
        &at,
        "what was kept as this machine's signing key is not one",
    ) {
        Kept::Good(one) => Some(one),
        Kept::Gone => minted(paths, &at),
        Kept::Blocked => None,
    }?;
    Some(SigningKey::from_bytes(&one))
}

pub const NOTHING_BEFORE: [u8; 32] = [0u8; 32];

pub fn tip_of(before: [u8; 32], said: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut tip = before;
    for line in said.split_inclusive(|one| *one == b'\n') {
        let mut over = Sha256::new();
        over.update(tip);
        over.update(line);
        tip = over.finalize().into();
    }
    tip
}

pub const SIG: &str = "sig";

/// A tip alone says nothing about what it is a tip of, so an old signature would answer for a
/// segment rolled back to the bytes it covered, or for another segment holding the same lines.
fn over(about: &About, covers: &Covers) -> Vec<u8> {
    let mut said = format!(
        "tisty.sig\u{0}{}\u{0}{}\u{0}{}\u{0}",
        about.device, about.segment, covers.at
    )
    .into_bytes();
    said.extend_from_slice(&covers.tip);
    said
}

pub struct About<'a> {
    pub device: &'a str,
    pub segment: &'a str,
}

/// The count is signed with the tip, or a reader resuming from it could be told any number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Covers {
    pub tip: [u8; 32],
    pub at: u64,
}

pub fn signed(key: &SigningKey, about: &About, covers: &Covers) -> String {
    use ed25519_dalek::Signer;
    let said = Said {
        tip: hexed(&covers.tip),
        at: covers.at,
        sig: hexed(&key.sign(&over(about, covers)).to_bytes()),
    };
    serde_json::to_string(&said).unwrap_or_default()
}

/// A signature that will not parse may yet be a round that has not finished; one that parses and
/// does not answer is a hand. Telling them apart is what lets the first heal and the second not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Holds {
    Covers(Covers),
    Refused,
    Unreadable,
}

impl Holds {
    pub fn covers(self) -> Option<Covers> {
        match self {
            Holds::Covers(covers) => Some(covers),
            Holds::Refused | Holds::Unreadable => None,
        }
    }
}

pub fn holds(by: &VerifyingKey, about: &About, said: &str) -> Holds {
    use ed25519_dalek::Verifier;
    let read = || {
        let said: Said = serde_json::from_str(said).ok()?;
        let covers = Covers {
            tip: unhexed::<32>(&said.tip)?,
            at: said.at,
        };
        let sig = ed25519_dalek::Signature::from_slice(&unhexed::<64>(&said.sig)?).ok()?;
        Some((covers, sig))
    };
    let Some((covers, sig)) = read() else {
        return Holds::Unreadable;
    };
    match by.verify(&over(about, &covers), &sig) {
        Ok(()) => Holds::Covers(covers),
        Err(_) => Holds::Refused,
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Said {
    tip: String,
    at: u64,
    sig: String,
}

pub(crate) fn hexed(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut said, one| {
            let _ = write!(said, "{one:02x}");
            said
        })
}

pub(crate) fn unhexed<const N: usize>(said: &str) -> Option<[u8; N]> {
    if said.len() != N * 2 || !said.chars().all(|one| one.is_ascii_hexdigit()) {
        return None;
    }
    let mut out = [0u8; N];
    for (at, one) in out.iter_mut().enumerate() {
        *one = u8::from_str_radix(said.get(at * 2..at * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

/// About 66 bits of the key, too many for anyone to mint another key that reads the same.
pub fn spoken(said: &str) -> Option<String> {
    use sha2::{Digest, Sha256};
    let key = read(said)?;
    let mut over = Sha256::new();
    over.update(b"tisty-code-v1\0");
    over.update(key.as_bytes());
    let digest: [u8; 32] = over.finalize().into();
    let mut wide = [0u8; 16];
    wide[7..].copy_from_slice(&digest[..9]);
    let digits = format!(
        "{:020}",
        u128::from_be_bytes(wide) % 100_000_000_000_000_000_000
    );
    Some(
        digits
            .as_bytes()
            .chunks(5)
            .map(|five| std::str::from_utf8(five).unwrap_or_default())
            .collect::<Vec<_>>()
            .join(" "),
    )
}

#[cfg(test)]
#[path = "signing_test.rs"]
mod tests;
