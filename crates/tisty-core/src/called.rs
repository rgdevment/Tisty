use crate::event::{DeviceId, Op};
use crate::{Named, State};

pub const AT_MOST: usize = 64;

// Read once per run: a window asks after every round, and on a Mac that is a process each time.
pub fn here() -> Option<Named> {
    static ONCE: std::sync::OnceLock<Option<Named>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        let name = cleaned(&computer_name()?);
        (!name.is_empty()).then(|| Named {
            name,
            os: Some(SYSTEM.to_string()),
        })
    })
    .clone()
}

/// One shape for a name wherever it is read, or a long one would never match what was kept.
pub fn cleaned(said: &str) -> String {
    crate::text::plainly(said.trim())
        .chars()
        .take(AT_MOST)
        .collect::<String>()
        .trim()
        .to_string()
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

#[cfg(not(target_os = "macos"))]
fn computer_name() -> Option<String> {
    sysinfo::System::host_name()
}

#[cfg(test)]
#[path = "called_test.rs"]
mod tests;
