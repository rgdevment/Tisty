use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const STORE: &str = "store";
pub const FENCE_VERSION: u32 = u32::MAX;
const FENCE_OP: &str = "storeMoved";
const MOVED_NOTE: &str = "MOVED.txt";
const RETRIES: u32 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roots {
    pub new: PathBuf,
    pub real: PathBuf,
    pub private: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Settled {
    Fresh,
    AlreadyThere,
    Moved { aside: Option<PathBuf> },
    Failed(String),
}

pub fn settle(roots: &Roots) -> Settled {
    if roots.new.exists() {
        fence_behind(roots);
        return Settled::AlreadyThere;
    }
    let real = held(&roots.real).then(|| roots.real.clone());
    let private = roots.private.clone().filter(|at| held(at));
    if real.is_none() && private.is_none() {
        return Settled::Fresh;
    }
    let Some(parent) = roots.new.parent() else {
        return Settled::Failed("the new root has no parent".into());
    };
    let gate = match gated(parent, &roots.new) {
        Ok(gate) => gate,
        Err(why) => return Settled::Failed(why.to_string()),
    };
    if roots.new.exists() {
        drop(gate);
        fence_behind(roots);
        return Settled::AlreadyThere;
    }
    swept(parent, &roots.new);

    let part = parent.join(format!("{}.part-{}", leaf(&roots.new), std::process::id()));
    let made = gathered(&part, real.as_deref(), private.as_deref());
    let aside = match made {
        Ok(aside) => aside,
        Err(why) => {
            let _ = std::fs::remove_dir_all(&part);
            return Settled::Failed(why.to_string());
        }
    };
    if let Err(why) = landed(&part, &roots.new) {
        let _ = std::fs::remove_dir_all(&part);
        return match roots.new.exists() {
            true => Settled::AlreadyThere,
            false => Settled::Failed(why.to_string()),
        };
    }
    drop(gate);
    fence_behind(roots);
    Settled::Moved {
        aside: aside.map(|one| roots.new.join(one)),
    }
}

fn held(root: &Path) -> bool {
    root.join("data").join(STORE).is_dir() || root.join("config").is_dir()
}

fn leaf(at: &Path) -> String {
    at.file_name()
        .map(|one| one.to_string_lossy().into_owned())
        .unwrap_or_else(|| "root".into())
}

fn gated(parent: &Path, new: &Path) -> std::io::Result<std::fs::File> {
    std::fs::create_dir_all(parent)?;
    let gate = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(parent.join(format!("{}.migrating.lock", leaf(new))))?;
    gate.lock()?;
    Ok(gate)
}

fn swept(parent: &Path, new: &Path) {
    let prefix = format!("{}.part-", leaf(new));
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    for entry in entries.filter_map(|one| one.ok()) {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// The packaged app read the real folder with its own private copy laid over it, file by file.
fn gathered(
    part: &Path,
    real: Option<&Path>,
    private: Option<&Path>,
) -> std::io::Result<Option<PathBuf>> {
    std::fs::create_dir_all(part)?;
    let stamp = jiff::Zoned::now().strftime("%Y-%m-%d").to_string();
    let private_leads = match (real, private) {
        (_, None) => false,
        (None, Some(_)) => true,
        (Some(real), Some(private)) => newest(private) >= newest(real),
    };
    if let Some(real) = real {
        copied(real, part)?;
    }
    let mut aside = None;
    match (private_leads, real, private) {
        (true, real, Some(private)) => {
            copied(private, part)?;
            if let Some(real) = real {
                let at = PathBuf::from("aside").join(format!("localappdata-{stamp}"));
                copied(real, &part.join(&at))?;
                aside = Some(at);
            }
        }
        (false, Some(_), Some(private)) => {
            let at = PathBuf::from("aside").join(format!("store-package-{stamp}"));
            copied(private, &part.join(&at))?;
            aside = Some(at);
        }
        _ => {}
    }
    Ok(aside)
}

fn newest(root: &Path) -> SystemTime {
    let Ok(devices) = std::fs::read_dir(root.join("data").join(STORE)) else {
        return SystemTime::UNIX_EPOCH;
    };
    devices
        .filter_map(|one| one.ok())
        .filter_map(|one| std::fs::metadata(one.path().join("active.tisty")).ok())
        .filter_map(|meta| meta.modified().ok())
        .max()
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

fn copied(from: &Path, into: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(into)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        let named = name.to_string_lossy();
        if passing(&named) {
            continue;
        }
        let at = entry.path();
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copied(&at, &into.join(&name))?;
        } else if kind.is_file() {
            std::fs::copy(&at, into.join(&name))?;
        }
    }
    Ok(())
}

fn passing(named: &str) -> bool {
    named == ".lock"
        || named.ends_with(".part")
        || named.ends_with(".tmp")
        || named.ends_with(".migrating.lock")
        || crate::holes::marker(named)
}

fn landed(part: &Path, new: &Path) -> std::io::Result<()> {
    let mut last = None;
    for _ in 0..RETRIES {
        if new.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "taken",
            ));
        }
        match std::fs::rename(part, new) {
            Ok(()) => return Ok(()),
            Err(why) => last = Some(why),
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("the rename never ran")))
}

/// An older Tisty must refuse the folder it left rather than keep writing a second history.
pub fn fence_behind(roots: &Roots) {
    for root in std::iter::once(&roots.real).chain(roots.private.iter()) {
        if !held(root) || root.join(MOVED_NOTE).exists() {
            continue;
        }
        if fenced(root, &roots.new).is_ok() {
            let _ = std::fs::write(
                root.join(MOVED_NOTE),
                format!(
                    "Tisty keeps this machine's tasks and documents in {} now.\nThis folder is a copy from before, left untouched.\n",
                    roots.new.display()
                ),
            );
        }
    }
}

fn fenced(root: &Path, new: &Path) -> std::io::Result<()> {
    let Ok(devices) = std::fs::read_dir(root.join("data").join(STORE)) else {
        return Ok(());
    };
    let line = serde_json::json!({
        "v": FENCE_VERSION,
        "op": FENCE_OP,
        "d": { "to": new.display().to_string() },
    });
    for device in devices.filter_map(|one| one.ok()) {
        let active = device.path().join("active.tisty");
        if !active.is_file() {
            continue;
        }
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(&active)?;
        if !ends_clean(&mut file)? {
            file.write_all(b"\n")?;
        }
        writeln!(file, "{line}")?;
    }
    Ok(())
}

fn ends_clean(file: &mut std::fs::File) -> std::io::Result<bool> {
    use std::io::{Read, Seek, SeekFrom};
    if file.metadata()?.len() == 0 {
        return Ok(true);
    }
    file.seek(SeekFrom::End(-1))?;
    let mut last = [0u8; 1];
    file.read_exact(&mut last)?;
    Ok(last[0] == b'\n')
}

pub fn told(settled: Option<&Settled>) {
    use crate::witness::{self, Fact, channel};
    match settled {
        Some(Settled::Moved { aside }) => witness::note(
            channel::STORE,
            "this machine's store moved out of AppData, where the Store would delete it",
            &[(
                "aside",
                Fact::Word(if aside.is_some() { "kept" } else { "none" }),
            )],
        ),
        Some(Settled::Failed(why)) => witness::warn(
            channel::STORE,
            "the store could not move out of AppData, so this run keeps the old place",
            &[("why", Fact::Why(why.clone()))],
        ),
        _ => {}
    }
}

#[cfg(test)]
#[path = "moving_test.rs"]
mod tests;
