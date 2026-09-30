use std::path::PathBuf;

use ed25519_dalek::{SigningKey, VerifyingKey};

use crate::event::DeviceId;

pub const KEEP: &str = ".device-key";

pub fn kept_at(paths: &crate::Paths, device: &DeviceId) -> Option<PathBuf> {
    crate::store::is_device_name(&device.0)
        .then(|| paths.private().join(format!("{}{KEEP}", device.0)))
}

pub fn shown(key: &SigningKey) -> String {
    hexed(key.verifying_key().as_bytes())
}

pub fn read(said: &str) -> Option<VerifyingKey> {
    let bytes = unhexed::<32>(said)?;
    VerifyingKey::from_bytes(&bytes).ok()
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

pub const SEAL: &str = "sig";

/// A tip alone says nothing about what it is a tip of, so an old seal would answer for a segment
/// rolled back to the bytes it covered, or for another segment holding the same lines.
fn over(about: &About, tip: &[u8; 32]) -> Vec<u8> {
    let mut said =
        format!("tisty.seal\u{0}{}\u{0}{}\u{0}", about.device, about.segment).into_bytes();
    said.extend_from_slice(tip);
    said
}

pub struct About<'a> {
    pub device: &'a str,
    pub segment: &'a str,
}

pub fn sealed(key: &SigningKey, about: &About, tip: &[u8; 32]) -> String {
    use ed25519_dalek::Signer;
    let said = Said {
        tip: hexed(tip),
        sig: hexed(&key.sign(&over(about, tip)).to_bytes()),
    };
    serde_json::to_string(&said).unwrap_or_default()
}

pub fn holds(by: &VerifyingKey, about: &About, said: &str) -> Option<[u8; 32]> {
    use ed25519_dalek::Verifier;
    let said: Said = serde_json::from_str(said).ok()?;
    let tip = unhexed::<32>(&said.tip)?;
    let sig = ed25519_dalek::Signature::from_slice(&unhexed::<64>(&said.sig)?).ok()?;
    by.verify(&over(about, &tip), &sig).ok().map(|()| tip)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Said {
    tip: String,
    sig: String,
}

fn hexed(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut said, one| {
            let _ = write!(said, "{one:02x}");
            said
        })
}

fn unhexed<const N: usize>(said: &str) -> Option<[u8; N]> {
    if said.len() != N * 2 || !said.chars().all(|one| one.is_ascii_hexdigit()) {
        return None;
    }
    let mut out = [0u8; N];
    for (at, one) in out.iter_mut().enumerate() {
        *one = u8::from_str_radix(said.get(at * 2..at * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
#[path = "signing_test.rs"]
mod tests;
