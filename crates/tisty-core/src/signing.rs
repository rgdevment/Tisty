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
