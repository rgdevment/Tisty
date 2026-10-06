use std::io::Write;
use std::path::{Path, PathBuf};

const STORE: &str = "store";
const KEPT: [&str; 2] = ["data", "config"];
pub const FENCE_VERSION: u32 = u32::MAX;
const FENCE_OP: &str = "storeMoved";
const MOVED_NOTE: &str = "MOVED.txt";
const RETRIES: u32 = 10;
const POLL: std::time::Duration = std::time::Duration::from_millis(200);

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
    Moved,
    Failed(String),
}

pub fn settle(roots: &Roots) -> Settled {
    if roots.new.exists() {
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
    let sources: Vec<&Path> = real
        .iter()
        .chain(private.iter())
        .map(PathBuf::as_path)
        .collect();
    let Some(_quiet) = quieted(&sources) else {
        return Settled::Failed("another Tisty is still writing the old store".into());
    };
    swept(parent, &roots.new);

    let part = parent.join(format!("{}.part-{}", leaf(&roots.new), std::process::id()));
    if let Err(why) = gathered(&part, &sources) {
        let _ = std::fs::remove_dir_all(&part);
        return Settled::Failed(why.to_string());
    }
    if let Err(why) = landed(&part, &roots.new) {
        let _ = std::fs::remove_dir_all(&part);
        return match roots.new.exists() {
            true => Settled::AlreadyThere,
            false => Settled::Failed(why.to_string()),
        };
    }
    for source in &sources {
        if fenced(source, &roots.new).is_ok() {
            let _ = std::fs::write(source.join(MOVED_NOTE), moved_note(&roots.new));
        }
    }
    Settled::Moved
}

fn held(root: &Path) -> bool {
    root.join("data").join(STORE).is_dir() || root.join("config").is_dir()
}

fn leaf(at: &Path) -> String {
    at.file_name()
        .map(|one| one.to_string_lossy().into_owned())
        .unwrap_or_else(|| "root".into())
}

fn moved_note(new: &Path) -> String {
    format!(
        "Tisty keeps this machine's tasks and documents in {} now.\nThis folder is a copy from before, left as it was.\n",
        new.display()
    )
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

/// Holding every device's write lock keeps an older Tisty from writing mid-copy.
fn quieted(sources: &[&Path]) -> Option<Vec<crate::store::Alone>> {
    let mut held = Vec::new();
    for source in sources {
        let Ok(devices) = std::fs::read_dir(source.join("data").join(STORE)) else {
            continue;
        };
        for device in devices.filter_map(|one| one.ok()) {
            if device.path().is_dir() {
                held.push(crate::store::alone(&device.path())?);
            }
        }
    }
    Some(held)
}

/// The packaged app read the real folder with its own private copy laid over it, file by file.
fn gathered(part: &Path, sources: &[&Path]) -> std::io::Result<()> {
    std::fs::create_dir_all(part)?;
    for source in sources {
        for under in KEPT {
            let at = source.join(under);
            if at.is_dir() {
                copied(&at, &part.join(under), false)?;
            }
        }
    }
    Ok(())
}

fn copied(from: &Path, into: &Path, attachments: bool) -> std::io::Result<()> {
    std::fs::create_dir_all(into)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let name = entry.file_name();
        let named = name.to_string_lossy();
        let at = entry.path();
        let kind = std::fs::metadata(&at)?;
        if kind.is_dir() {
            copied(
                &at,
                &into.join(&name),
                attachments || named == "attachments",
            )?;
        } else if kind.is_file() && !passing(&named, attachments) {
            if named == "active.tisty" {
                unfenced(&at, &into.join(&name))?;
            } else {
                std::fs::copy(&at, into.join(&name))?;
            }
        }
    }
    Ok(())
}

fn passing(named: &str, attachments: bool) -> bool {
    named == ".lock"
        || crate::holes::marker(named)
        || match attachments {
            true => named.starts_with('.') && named.ends_with(".part"),
            false => named.ends_with(".part") || named.ends_with(".tmp"),
        }
}

/// A store copied again after its new home was lost must not carry the fence that retired it.
fn unfenced(from: &Path, into: &Path) -> std::io::Result<()> {
    let body = std::fs::read(from)?;
    if !body
        .windows(FENCE_OP.len())
        .any(|one| one == FENCE_OP.as_bytes())
    {
        std::fs::copy(from, into)?;
        return Ok(());
    }
    let kept: Vec<&[u8]> = body
        .split_inclusive(|byte| *byte == b'\n')
        .filter(|line| !is_fence(line))
        .collect();
    std::fs::write(into, kept.concat())
}

fn is_fence(line: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(line)
        .ok()
        .is_some_and(|said| said.get("op").and_then(|op| op.as_str()) == Some(FENCE_OP))
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
        std::thread::sleep(POLL);
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("the rename never ran")))
}

/// An older Tisty still pointed at the old store must refuse it rather than write where nobody reads.
fn fenced(root: &Path, new: &Path) -> std::io::Result<()> {
    let devices = match std::fs::read_dir(root.join("data").join(STORE)) {
        Ok(devices) => devices,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(why) => return Err(why),
    };
    let line = format!(
        "{}\n",
        serde_json::json!({
            "v": FENCE_VERSION,
            "op": FENCE_OP,
            "d": { "to": new.display().to_string() },
        })
    );
    for device in devices {
        let active = device?.path().join("active.tisty");
        if !active.is_file() {
            continue;
        }
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .append(true)
            .open(&active)?;
        let last = last_line(&mut file)?;
        if is_fence(&last) {
            continue;
        }
        let lead = if last.is_empty() || last.ends_with(b"\n") {
            ""
        } else {
            "\n"
        };
        file.write_all(format!("{lead}{line}").as_bytes())?;
        file.sync_all()?;
    }
    Ok(())
}

fn last_line(file: &mut std::fs::File) -> std::io::Result<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let len = file.metadata()?.len();
    file.seek(SeekFrom::Start(len.saturating_sub(512)))?;
    let mut tail = Vec::new();
    file.read_to_end(&mut tail)?;
    let trimmed = tail.strip_suffix(b"\n").unwrap_or(&tail);
    let start = trimmed
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |at| at + 1);
    Ok(tail[start..].to_vec())
}

#[cfg(test)]
#[path = "moving_test.rs"]
mod tests;
