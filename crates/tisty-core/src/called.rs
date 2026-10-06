use crate::event::{DeviceId, Op};
use crate::{Named, State};

pub fn here() -> Option<Named> {
    let name = computer_name()?;
    let name = crate::text::plainly(name.trim());
    (!name.is_empty()).then(|| Named {
        name,
        os: Some(SYSTEM.to_string()),
    })
}

/// What to write so the others learn this machine's name, when they do not know it yet.
pub fn told(state: &State, who: &DeviceId, now: Option<Named>) -> Option<Op> {
    let now = now?;
    (state.named.get(who) != Some(&now)).then(|| Op::DeviceNamed {
        d: who.clone(),
        name: now.name,
        os: now.os,
    })
}

#[cfg(target_os = "macos")]
const SYSTEM: &str = "macOS";
#[cfg(windows)]
const SYSTEM: &str = "Windows";
#[cfg(not(any(target_os = "macos", windows)))]
const SYSTEM: &str = "Linux";

// The name the Mac shows in Sharing, not the network hostname with dashes and `.local`.
#[cfg(target_os = "macos")]
fn computer_name() -> Option<String> {
    let out = std::process::Command::new("/usr/sbin/scutil")
        .args(["--get", "ComputerName"])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(windows)]
fn computer_name() -> Option<String> {
    std::env::var("COMPUTERNAME").ok()
}

#[cfg(not(any(target_os = "macos", windows)))]
fn computer_name() -> Option<String> {
    std::fs::read_to_string("/etc/hostname").ok()
}

#[cfg(test)]
#[path = "called_test.rs"]
mod tests;
