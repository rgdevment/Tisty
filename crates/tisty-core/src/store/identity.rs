use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::witness::{self, Fact, channel};
use crate::{Error, Result};

use super::{is_store_name, write_atomic};

/// Every key set aside is named after the file it replaces, so what they share is the mark,
/// not the name: a device key parked under the store key's mark would never be listed.
pub(crate) const DISPLACED: &str = ".was-";

/// This says it is really that store: it never leaves the machine, and without it nobody
/// can write a parcel that lands here as though it had been born here.
pub const KEEP: &str = ".store-key";

pub const MARKER: &str = ".store-id";

/// The identity says which store a parcel came from, and travels inside every one of them.
pub fn identity(store_root: impl AsRef<Path>) -> Result<String> {
    if let Some(held) = peek_identity(&store_root) {
        return Ok(held);
    }
    let at = store_root.as_ref().join(MARKER);
    let fresh = ulid::Ulid::generate().to_string();
    std::fs::create_dir_all(store_root.as_ref())?;

    match File::create_new(&at) {
        Ok(mut file) => {
            file.write_all(fresh.as_bytes())?;
            file.sync_all()?;
            let _ = crate::paths::ours_alone(&at);
            Ok(fresh)
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if let Some(held) = peek_identity(&store_root) {
                return Ok(held);
            }
            write_atomic(&at, fresh.as_bytes())?;
            Ok(peek_identity(&store_root).unwrap_or(fresh))
        }
        Err(e) => Err(Error::Io(e)),
    }
}

pub fn kept_at(paths: &crate::Paths, named: &str) -> Option<PathBuf> {
    is_store_name(named).then(|| paths.private().join(format!("{named}{KEEP}")))
}

pub fn secret_kept(paths: &crate::Paths) -> Option<[u8; 32]> {
    let named = peek_identity(paths.store())?;
    let held = std::fs::read(kept_at(paths, &named)?).ok()?;
    <[u8; 32]>::try_from(held.as_slice()).ok()
}

pub fn secret(paths: &crate::Paths) -> Option<[u8; 32]> {
    let named = identity(paths.store()).ok()?;
    let at = kept_at(paths, &named)?;
    match kept(paths, &at, "what was kept as the key is not one") {
        Kept::Good(one) => return Some(one),
        Kept::Blocked => return None,
        Kept::Gone => {}
    }

    if let Ok(inside) = std::fs::read(paths.store().join(KEEP))
        && let Ok(kept) = <[u8; 32]>::try_from(inside.as_slice())
    {
        return Some(kept);
    }

    minted(paths, &at)
}

pub(crate) enum Kept {
    Good([u8; 32]),
    Gone,
    Blocked,
}

pub(crate) fn kept(paths: &crate::Paths, at: &Path, why: &str) -> Kept {
    match std::fs::read(at) {
        Ok(held) => match <[u8; 32]>::try_from(held.as_slice()) {
            Ok(one) => Kept::Good(one),
            Err(_) if set_aside(paths, at, &held, why) && std::fs::remove_file(at).is_ok() => {
                Kept::Gone
            }
            Err(_) => Kept::Blocked,
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Kept::Gone,
        Err(e) => {
            witness::error(
                channel::STORE,
                "a key could not be read, so this machine cannot prove its own writing",
                &[
                    ("at", Fact::Path(at.to_path_buf())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            Kept::Blocked
        }
    }
}

pub(crate) fn minted(paths: &crate::Paths, at: &Path) -> Option<[u8; 32]> {
    let mut fresh = [0u8; 32];
    rand_core::TryRngCore::try_fill_bytes(&mut rand_core::OsRng, &mut fresh).ok()?;
    std::fs::create_dir_all(paths.private()).ok()?;
    let _ = crate::paths::ours_alone(&paths.private());
    match made(at) {
        Ok(mut file) => {
            if file
                .write_all(&fresh)
                .and_then(|()| file.sync_all())
                .is_err()
            {
                let _ = std::fs::remove_file(at);
                return None;
            }
            let _ = crate::paths::ours_alone(at);
            Some(fresh)
        }
        Err(_) => std::fs::read(at)
            .ok()
            .and_then(|held| <[u8; 32]>::try_from(held.as_slice()).ok()),
    }
}

/// Unix narrows before the secret lands; Windows cannot, and leans on a private directory nobody
/// else's account reaches.
#[cfg(unix)]
fn made(at: &Path) -> std::io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(at)
}

#[cfg(not(unix))]
fn made(at: &Path) -> std::io::Result<File> {
    File::create_new(at)
}

pub fn kept_before_the_store_goes(paths: &crate::Paths) {
    let was = paths.store().join(KEEP);
    let Ok(held) = std::fs::read(&was) else {
        return;
    };
    if <[u8; 32]>::try_from(held.as_slice()).is_err() {
        return;
    }
    let named = peek_identity(paths.store()).unwrap_or_default();
    let at = kept_at(paths, &named).unwrap_or_else(|| paths.private().join(KEEP));
    let _ = std::fs::create_dir_all(paths.private());
    let _ = crate::paths::ours_alone(&paths.private());
    set_aside(
        paths,
        &at,
        &held,
        "the store it was kept in was about to be replaced",
    );
}

pub fn brought_home(paths: &crate::Paths) {
    let was = paths.store().join(KEEP);
    let Ok(held) = std::fs::read(&was) else {
        return;
    };
    if <[u8; 32]>::try_from(held.as_slice()).is_err() {
        witness::trace(
            channel::STORE,
            "what was kept where the key used to live is not a key, so it was left alone",
            &[("at", Fact::Path(was.clone()))],
        );
        return;
    }
    if !paths.of_one_install() {
        witness::trace(
            channel::STORE,
            "this store was named on its own, so its key was left where it is",
            &[("at", Fact::Path(was.clone()))],
        );
        return;
    }
    let Ok(named) = identity(paths.store()) else {
        witness::warn(
            channel::STORE,
            "a key sits in a store that cannot be named, so it was left where it is",
            &[("at", Fact::Path(was.clone()))],
        );
        return;
    };

    let Some(now) = kept_at(paths, &named) else {
        return;
    };
    match std::fs::read(&now) {
        Ok(there) if <[u8; 32]>::try_from(there.as_slice()).is_err() => {
            if !set_aside(paths, &now, &there, "what was kept as the key is not one") {
                return;
            }
        }
        Ok(there) if there != held => {
            if set_aside(
                paths,
                &now,
                &held,
                "an older build made a second key inside the store",
            ) {
                swept(&was);
            }
            return;
        }
        Ok(_) => {
            swept(&was);
            return;
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => {
            witness::error(
                channel::STORE,
                "the key already kept apart could not be read, so nothing was moved over it",
                &[
                    ("at", Fact::Path(now.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            return;
        }
    }

    let _ = std::fs::create_dir_all(paths.private());
    let _ = crate::paths::ours_alone(&paths.private());
    if write_atomic(&now, &held).is_err() {
        witness::warn(
            channel::STORE,
            "the key could not be moved out of the store, so it stays where a backup reaches it",
            &[("at", Fact::Path(was.clone()))],
        );
        return;
    }
    let _ = crate::paths::ours_alone(&now);
    swept(&was);
}

pub fn displaced(paths: &crate::Paths) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(paths.private()) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_name().to_string_lossy().contains(DISPLACED))
        .map(|one| one.path())
        .collect();
    found.sort();
    found
}

pub(crate) fn set_aside(paths: &crate::Paths, at: &Path, held: &[u8], why: &str) -> bool {
    if displaced(paths)
        .iter()
        .any(|one| std::fs::read(one).is_ok_and(|kept| kept == held))
    {
        return true;
    }
    let stamp = jiff::Zoned::now().strftime("%Y%m%dT%H%M%S").to_string();
    let named = at.file_name().unwrap_or_default().to_string_lossy();
    let mut aside = paths.private().join(format!("{named}{DISPLACED}{stamp}"));
    for again in 1..100 {
        match File::create_new(&aside) {
            Ok(mut file) => {
                if file.write_all(held).and_then(|()| file.sync_all()).is_err() {
                    let _ = std::fs::remove_file(&aside);
                    break;
                }
                let _ = crate::paths::ours_alone(&aside);
                witness::warn(
                    channel::STORE,
                    "a key was set aside",
                    &[
                        ("at", Fact::Path(aside)),
                        ("why", Fact::Why(why.to_string())),
                    ],
                );
                return true;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                aside = paths
                    .private()
                    .join(format!("{named}{DISPLACED}{stamp}-{again}"));
            }
            Err(_) => break,
        }
    }
    witness::error(
        channel::STORE,
        "a key had to be set aside and could not be, so nothing was changed",
        &[("why", Fact::Why(why.to_string()))],
    );
    false
}

fn swept(at: &Path) {
    if let Err(e) = std::fs::remove_file(at)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        witness::error(
            channel::STORE,
            "the key is still inside the store, where a backup reaches it",
            &[
                ("at", Fact::Path(at.to_path_buf())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
    }
}

pub fn peek_identity(store_root: impl AsRef<Path>) -> Option<String> {
    let held = std::fs::read_to_string(store_root.as_ref().join(MARKER)).ok()?;
    let held = held.trim().to_string();
    is_store_name(&held).then_some(held)
}
