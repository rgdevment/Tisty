use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{HELD, PAPERS, STORE, Trouble, straight};

pub(crate) fn before_carrying(dest: &Path, device: &str) -> Result<(), Trouble> {
    if !dest.is_dir() {
        return Err(Trouble::NotThere(dest.display().to_string()));
    }
    for folder in [STORE, HELD, PAPERS] {
        straight(&dest.join(folder), dest)?;
    }
    nobody_newer(dest, device)
}

fn nobody_newer(dest: &Path, device: &str) -> Result<(), Trouble> {
    let Ok(entries) = std::fs::read_dir(dest.join(STORE)) else {
        return Ok(());
    };
    for entry in entries.filter_map(|one| one.ok()) {
        if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
            continue;
        }
        let named = entry.file_name().to_string_lossy().into_owned();
        if named == device {
            continue;
        }
        let Ok(schema) = tisty_core::store::newest_schema(&entry.path()) else {
            continue;
        };
        if schema > tisty_core::event::SCHEMA_VERSION {
            witness::warn(
                channel::SYNC,
                "another machine writes a newer schema, so nothing was carried either way",
                &[
                    ("at", Fact::Id(named.clone())),
                    ("schema", Fact::Count(schema as usize)),
                ],
            );
            return Err(Trouble::Newer(named));
        }
    }
    Ok(())
}

pub(crate) fn allowed_to_write(store: &Path, device: &str) -> Result<(), Trouble> {
    let who = tisty_core::event::DeviceId(device.to_string());
    let told = tisty_core::store::ledger(store).map_err(|e| Trouble::Unreadable(e.to_string()))?;
    match told.may_write(&who) {
        true => Ok(()),
        false => Err(Trouble::NotAllowed(device.to_string())),
    }
}
