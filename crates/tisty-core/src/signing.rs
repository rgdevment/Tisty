use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use ed25519_dalek::{SigningKey, VerifyingKey};

use crate::event::DeviceId;
use crate::witness::{self, Fact, channel};

pub const KEEP: &str = ".device-key";

pub fn kept_at(paths: &crate::Paths, device: &DeviceId) -> Option<PathBuf> {
    crate::store::is_device_name(&device.0)
        .then(|| paths.private().join(format!("{}{KEEP}", device.0)))
}

pub fn shown(key: &SigningKey) -> String {
    hexed(key.verifying_key().as_bytes())
}

pub fn read(said: &str) -> Option<VerifyingKey> {
    let bytes = unhexed(said)?;
    VerifyingKey::from_bytes(&bytes).ok()
}

pub fn mine(paths: &crate::Paths, device: &DeviceId) -> Option<SigningKey> {
    let at = kept_at(paths, device)?;
    match std::fs::read(&at) {
        Ok(held) => match <[u8; 32]>::try_from(held.as_slice()) {
            Ok(kept) => return Some(SigningKey::from_bytes(&kept)),
            Err(_) => {
                witness::error(
                    channel::STORE,
                    "what was kept as this machine's signing key is not one, so it can prove nothing it wrote",
                    &[("at", Fact::Path(at.clone()))],
                );
                return None;
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            witness::error(
                channel::STORE,
                "this machine's signing key could not be read",
                &[
                    ("at", Fact::Path(at.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            return None;
        }
    }

    let mut fresh = [0u8; 32];
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut fresh).ok()?;
    std::fs::create_dir_all(paths.private()).ok()?;
    let _ = crate::paths::ours_alone(&paths.private());
    match File::create_new(&at) {
        Ok(mut file) => {
            file.write_all(&fresh).ok()?;
            file.sync_all().ok()?;
            let _ = crate::paths::ours_alone(&at);
            Some(SigningKey::from_bytes(&fresh))
        }
        Err(_) => std::fs::read(&at)
            .ok()
            .and_then(|held| <[u8; 32]>::try_from(held.as_slice()).ok())
            .map(|kept| SigningKey::from_bytes(&kept)),
    }
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

pub fn sealed(key: &SigningKey, tip: &[u8; 32]) -> String {
    use ed25519_dalek::Signer;
    let said = Said {
        tip: hexed(tip),
        sig: hexed(&key.sign(tip).to_bytes()),
    };
    serde_json::to_string(&said).unwrap_or_default()
}

pub const SEAL: &str = "sig";

pub fn tip_in(said: &str) -> Option<[u8; 32]> {
    let said: Said = serde_json::from_str(said).ok()?;
    unhexed(&said.tip)
}

pub fn holds(by: &VerifyingKey, said: &str) -> Option<[u8; 32]> {
    use ed25519_dalek::Verifier;
    let said: Said = serde_json::from_str(said).ok()?;
    let tip = unhexed(&said.tip)?;
    let sig = ed25519_dalek::Signature::from_slice(&unhexed_long(&said.sig)?).ok()?;
    by.verify(&tip, &sig).ok().map(|()| tip)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Said {
    tip: String,
    sig: String,
}

fn unhexed_long(said: &str) -> Option<[u8; 64]> {
    if said.len() != 128 || !said.chars().all(|one| one.is_ascii_hexdigit()) {
        return None;
    }
    let mut out = [0u8; 64];
    for (at, one) in out.iter_mut().enumerate() {
        *one = u8::from_str_radix(said.get(at * 2..at * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

fn hexed(bytes: &[u8]) -> String {
    bytes.iter().map(|one| format!("{one:02x}")).collect()
}

fn unhexed(said: &str) -> Option<[u8; 32]> {
    if said.len() != 64 || !said.chars().all(|one| one.is_ascii_hexdigit()) {
        return None;
    }
    let mut out = [0u8; 32];
    for (at, one) in out.iter_mut().enumerate() {
        *one = u8::from_str_radix(said.get(at * 2..at * 2 + 2)?, 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
#[path = "signing_test.rs"]
mod tests;
