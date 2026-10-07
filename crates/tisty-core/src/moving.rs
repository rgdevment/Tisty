use std::io::Write;
use std::path::{Path, PathBuf};

const STORE: &str = "store";
const KEPT: [&str; 2] = ["data", "config"];
pub const FENCE_VERSION: u32 = u32::MAX;
const FENCE_OP: &str = "storeMoved";
const MOVED_NOTE: &str = "MOVED.txt";
const MOVED_FROM: &str = ".moved-from";
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

/// How far a move has come: bytes copied and bytes it will copy in all.
pub type Telling<'a> = &'a mut dyn FnMut(u64, u64);

struct Tally<'a> {
    done: u64,
    whole: u64,
    telling: Telling<'a>,
}

impl Tally<'_> {
    fn add(&mut self, bytes: u64) {
        self.done = (self.done + bytes).min(self.whole);
        (self.telling)(self.done, self.whole);
    }
}

pub fn moves(roots: &Roots) -> bool {
    !roots.new.exists() && (held(&roots.real) || roots.private.as_deref().is_some_and(held))
}

pub fn settle(roots: &Roots) -> Settled {
    settle_telling(roots, &mut |_, _| {})
}

pub fn settle_telling(roots: &Roots, telling: Telling) -> Settled {
    if roots.new.exists() {
        fence_left(&roots.new);
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

    let needs: u64 = sources.iter().map(|source| weighed(source)).sum();
    if let Ok(free) = fs4::available_space(parent)
        && free < needs
    {
        return Settled::Failed(format!(
            "there is not enough room to move it: it needs {needs} bytes and {free} are free"
        ));
    }
    let part = parent.join(format!("{}.part-{}", leaf(&roots.new), std::process::id()));
    let mut tally = Tally {
        done: 0,
        whole: needs,
        telling,
    };
    tally.add(0);
    let made = gathered(&part, &sources, &mut tally).and_then(|()| {
        let said: Vec<String> = sources
            .iter()
            .map(|one| one.display().to_string())
            .collect();
        std::fs::write(
            part.join(MOVED_FROM),
            said.join(
                "
",
            ) + "
",
        )
    });
    if let Err(why) = made {
        let _ = std::fs::remove_dir_all(&part);
        return Settled::Failed(why.to_string());
    }
    // What was weighed includes files a move leaves behind, so the end is said outright.
    let whole = tally.whole;
    tally.add(whole);
    if let Err(why) = landed(&part, &roots.new) {
        let _ = std::fs::remove_dir_all(&part);
        return match roots.new.exists() {
            true => Settled::AlreadyThere,
            false => Settled::Failed(why.to_string()),
        };
    }
    fenced_all(&sources, &roots.new);
    Settled::Moved
}

/// A fence that could not be written is retried at every start until it holds.
fn fence_left(new: &Path) {
    let Ok(from) = std::fs::read_to_string(new.join(MOVED_FROM)) else {
        return;
    };
    let left: Vec<PathBuf> = from
        .lines()
        .filter(|one| !one.trim().is_empty())
        .map(PathBuf::from)
        .filter(|root| held(root) && !root.join(MOVED_NOTE).exists())
        .collect();
    if left.is_empty() {
        return;
    }
    let sources: Vec<&Path> = left.iter().map(PathBuf::as_path).collect();
    if let Some(_quiet) = quieted(&sources) {
        fenced_all(&sources, new);
    }
}

fn fenced_all(sources: &[&Path], new: &Path) {
    for source in sources {
        match fenced(source, new) {
            Ok(()) => {
                let _ = std::fs::write(source.join(MOVED_NOTE), moved_note(new));
            }
            Err(why) => crate::witness::warn(
                crate::witness::channel::STORE,
                "the old store could not be fenced yet, so the next start tries again",
                &[
                    ("at", crate::witness::Fact::Path(source.to_path_buf())),
                    ("why", crate::witness::Fact::Why(why.to_string())),
                ],
            ),
        }
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

fn moved_note(new: &Path) -> String {
    format!(
        "Tisty keeps this machine's tasks and documents in {} now.\nThis folder is a copy from before, left as it was.\n",
        new.display()
    )
}

fn weighed(root: &Path) -> u64 {
    fn walked(at: &Path, seen: &mut std::collections::HashSet<PathBuf>) -> u64 {
        let Ok(settled) = at.canonicalize() else {
            return 0;
        };
        if !seen.insert(settled) {
            return 0;
        }
        let Ok(entries) = std::fs::read_dir(at) else {
            return 0;
        };
        entries
            .filter_map(|one| one.ok())
            .map(|one| match std::fs::metadata(one.path()) {
                Ok(meta) if meta.is_dir() => walked(&one.path(), seen),
                Ok(meta) => meta.len(),
                Err(_) => 0,
            })
            .sum()
    }
    let mut seen = std::collections::HashSet::new();
    KEPT.iter()
        .map(|under| walked(&root.join(under), &mut seen))
        .sum()
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
fn gathered(part: &Path, sources: &[&Path], tally: &mut Tally) -> std::io::Result<()> {
    std::fs::create_dir_all(part)?;
    for source in sources {
        for under in KEPT {
            let at = source.join(under);
            if at.is_dir() {
                let mut seen = std::collections::HashSet::new();
                copied(&at, &part.join(under), false, &mut seen, tally)?;
            }
        }
    }
    Ok(())
}

fn copied(
    from: &Path,
    into: &Path,
    attachments: bool,
    seen: &mut std::collections::HashSet<PathBuf>,
    tally: &mut Tally,
) -> std::io::Result<()> {
    if !seen.insert(from.canonicalize()?) {
        return Ok(());
    }
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
                seen,
                tally,
            )?;
        } else if kind.is_file() && !passing(&named, attachments) {
            if named == "active.tisty" {
                unfenced(&at, &into.join(&name))?;
            } else {
                std::fs::copy(&at, into.join(&name))?;
            }
            tally.add(kind.len());
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
