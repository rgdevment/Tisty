use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::Config;
use crate::event::DeviceId;
use crate::signing::hexed;
use crate::witness::{self, Fact, channel};

/// What another computer this configuration has woken on was called there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Home {
    pub device: DeviceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<DeviceId>,
}

// Never stored as read: the configuration travels, so only what this computer says now can tell.
pub fn here() -> Option<String> {
    machine_uid::get().ok().and_then(|raw| inst_of(&raw))
}

pub fn inst_of(raw: &str) -> Option<String> {
    let raw = raw.trim();
    (!raw.is_empty()).then(|| hexed(&digest(&["tisty.inst", raw])[..16]))
}

pub fn settled(config: &mut Config, here: Option<&str>) -> bool {
    let Some(here) = here else {
        return false;
    };
    let Some(was) = config.inst.replace(here.to_string()) else {
        return true;
    };
    if was == here {
        return false;
    }
    let leaving = Home {
        device: config.device_id.clone(),
        agent: config.agent_id.clone(),
    };
    config.homes.insert(was, leaving.clone());
    let home = config.homes.remove(here).unwrap_or_else(|| Home {
        device: successor(&leaving.device, here),
        agent: None,
    });
    config.device_id = home.device;
    config.agent_id = home.agent;
    config.synced_at = None;
    config.heard_at = None;
    witness::warn(
        channel::CONFIG,
        "this configuration was last used on another computer, so this one writes under its own name",
        &[
            ("was", Fact::Id(leaving.device.0)),
            ("now", Fact::Id(config.device_id.0.clone())),
        ],
    );
    true
}

// Derived, not drawn, so the window and the terminal waking at once agree on it.
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

#[cfg(test)]
#[path = "machine_test.rs"]
mod tests;
