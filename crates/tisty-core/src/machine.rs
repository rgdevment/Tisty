use sha2::{Digest, Sha256};

use crate::Config;
use crate::event::DeviceId;
use crate::witness::{self, Fact, channel};

/// Read from the operating system on every call, so a configuration copied to another computer
/// carries the old answer and never the new one.
pub fn here() -> Option<String> {
    machine_uid::get().ok().and_then(|raw| inst_of(&raw))
}

pub fn inst_of(raw: &str) -> Option<String> {
    let raw = raw.trim();
    (!raw.is_empty()).then(|| hexed(&digest(&["tisty.inst", raw])[..16]))
}

/// A configuration that wakes on a computer other than the one it was written on stops speaking
/// as the machine it came from. True when the configuration changed and has to be saved.
pub fn settled(config: &mut Config, here: Option<&str>) -> bool {
    let Some(here) = here else {
        return false;
    };
    match config.inst.as_deref() {
        Some(kept) if kept == here => false,
        None => {
            config.inst = Some(here.to_string());
            true
        }
        Some(_) => {
            let was = config.device_id.clone();
            config.device_id = successor(&was, here);
            config.agent_id = None;
            config.inst = Some(here.to_string());
            witness::warn(
                channel::CONFIG,
                "this configuration was written on another computer, so this one takes a name of its own",
                &[
                    ("was", Fact::Id(was.0)),
                    ("now", Fact::Id(config.device_id.0.clone())),
                ],
            );
            true
        }
    }
}

/// Derived rather than drawn, so the window and the terminal opening at once agree on it.
fn successor(was: &DeviceId, here: &str) -> DeviceId {
    DeviceId(format!(
        "dev_{}",
        &hexed(&digest(&["tisty.successor", here, &was.0]))[..8]
    ))
}

fn digest(parts: &[&str]) -> [u8; 32] {
    let mut over = Sha256::new();
    for one in parts {
        over.update(one.as_bytes());
        over.update([0u8]);
    }
    over.finalize().into()
}

fn hexed(bytes: &[u8]) -> String {
    bytes.iter().map(|one| format!("{one:02x}")).collect()
}

#[cfg(test)]
#[path = "machine_test.rs"]
mod tests;
