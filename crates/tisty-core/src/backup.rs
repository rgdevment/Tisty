use std::io::{Read, Seek};
use std::path::{Component, Path, PathBuf};

use crate::{
    Config, Error, Paths, Result, store,
    witness::{self, Fact, channel},
};

const CARRIED: [&str; 4] = ["store", "docs", "originals", "attachments"];
const AT_MOST: u64 = 8 * 1024 * 1024 * 1024;
const AT_MOST_FILES: usize = 200_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Made {
    pub files: usize,
    pub bytes: u64,
    pub store_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Restored {
    pub files: usize,
    pub devices: usize,
}

pub fn write(data: &Path, into: &Path, aside: &Path) -> Result<Made> {
    let store_id = store::identity(data.join("store"))?;
    store::read_all(data.join("store"))?;

    let named = into
        .file_name()
        .map(|one| one.to_string_lossy().into_owned())
        .unwrap_or_else(|| "backup".into());
    let named = format!("{named}.{}.part", std::process::id());

    let beside = into.with_file_name(&named);
    let part = if std::fs::File::create(&beside).is_ok() {
        beside
    } else {
        std::fs::create_dir_all(aside)?;
        aside.join(&named)
    };

    match fill(data, &part, store_id) {
        Ok(made) => {
            place(&part, into)?;
            let _ = std::fs::remove_file(&part);
            Ok(made)
        }
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            Err(e)
        }
    }
}

fn place(part: &Path, into: &Path) -> std::io::Result<()> {
    if std::fs::rename(part, into).is_ok() {
        return Ok(());
    }

    let beside = into.with_extension(format!("part{}", std::process::id()));
    if std::fs::copy(part, &beside).is_ok() && std::fs::rename(&beside, into).is_ok() {
        witness::warn(channel::BACKUP, "backup placed by copying beside", &[]);
        return Ok(());
    }
    let _ = std::fs::remove_file(&beside);

    witness::warn(
        channel::BACKUP,
        "backup placed by overwriting, which is not atomic",
        &[],
    );
    let done = std::fs::copy(part, into).map(|_| ());
    if let Err(e) = &done {
        witness::error(
            channel::BACKUP,
            "backup could not be placed and may be torn",
            &[
                ("at", Fact::Path(into.into())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
    }
    done
}

fn fill(data: &Path, into: &Path, store_id: String) -> Result<Made> {
    let file = std::fs::File::create(into)?;
    let _ = crate::paths::ours_alone(into);
    let mut zip = zip::ZipWriter::new(file);
    let mut made = Made {
        files: 0,
        bytes: 0,
        store_id,
    };

    for folder in CARRIED {
        let root = data.join(folder);
        for at in walk(&root) {
            let Ok(rest) = at.strip_prefix(data) else {
                continue;
            };
            let Some(parts) = named(rest) else {
                continue;
            };
            if parts
                .iter()
                .any(|one| matches!(one, std::borrow::Cow::Owned(_)))
            {
                witness::warn(
                    channel::BACKUP,
                    "a name this copy cannot spell went in spelled as close as it can be",
                    &[("at", Fact::Path(at.clone()))],
                );
            }
            if kept_out(&parts) {
                continue;
            }
            let named = rest.to_string_lossy().replace('\\', "/");
            let weighs = std::fs::metadata(&at)?.len();

            made.files += 1;
            made.bytes += weighs;
            if made.bytes > AT_MOST || made.files > AT_MOST_FILES {
                return Err(Error::TooBig);
            }

            zip.start_file(named, zip::write::SimpleFileOptions::default())
                .map_err(zipped)?;
            let mut file = std::fs::File::open(&at)?;
            std::io::copy(&mut file, &mut zip)?;
        }
    }
    zip.finish().map_err(zipped)?;
    Ok(made)
}

pub fn read(paths: &Paths, from: &Path) -> Result<Restored> {
    within(paths, from, AT_MOST)
}

pub(crate) fn within(paths: &Paths, from: &Path, at_most: u64) -> Result<Restored> {
    let data = paths.data();
    let file = std::fs::File::open(from)?;
    let mut zip = zip::ZipArchive::new(file).map_err(zipped)?;

    let root = data.join("store");
    let theirs = named_in(&mut zip)?;
    let ours = store::peek_identity(&root);
    let free = ours.is_none()
        && store::read_all(&root)
            .map(|all| all.is_empty())
            .unwrap_or(false);
    match (&ours, theirs.is_empty()) {
        (_, false) if ours.as_ref() == Some(&theirs) || free => {}
        (None, true) if free => {}
        _ => {
            return Err(Error::OtherStore {
                theirs: if theirs.is_empty() {
                    from.display().to_string()
                } else {
                    theirs
                },
            });
        }
    }

    let staged = data.join(format!(".restoring-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staged);
    let done = unpack(&mut zip, &staged, at_most).and_then(|done| {
        store::read_all(staged.join("store"))?;
        if done.files == 0 || !staged.join("store").is_dir() {
            return Err(Error::OtherStore {
                theirs: from.display().to_string(),
            });
        }
        Ok(done)
    });
    let done = match done {
        Ok(done) => {
            if let Some(first) = &done.first {
                witness::warn(
                    channel::BACKUP,
                    "a copy holds entries this version does not put back, so they were left out",
                    &[
                        ("count", Fact::Id(done.leftover.to_string())),
                        ("first", Fact::Id(first.clone())),
                    ],
                );
            }
            done
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&staged);
            return Err(e);
        }
    };

    let was = Config::load_or_init(paths)?;
    let mut config = was.clone();
    config.device_id = crate::DeviceId(crate::config::new_device_id());
    config.synced_at = None;
    config.heard_at = None;
    config.save(paths)?;

    store::kept_before_the_store_goes(paths);

    let old = data.join(format!(".replaced-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&old);
    std::fs::create_dir_all(&old)?;
    if let Err(e) = swap(data, &staged, &old) {
        if was.save(paths).is_err() {
            witness::error(channel::BACKUP, "device name not put back", &[]);
        }
        let _ = std::fs::remove_dir_all(&staged);
        return Err(e);
    }
    let _ = std::fs::remove_dir_all(&staged);
    let _ = std::fs::remove_dir_all(&old);
    let _ = std::fs::remove_dir_all(paths.cache());
    if store::secret_kept(paths).is_none() {
        witness::warn(
            channel::BACKUP,
            "a copy carries the store's name and not what proves it, so parcels this store handed out before will land as a stranger's",
            &[],
        );
    }
    crate::docs::forget_what_was_carried(data);

    Ok(Restored {
        files: done.files,
        devices: std::fs::read_dir(data.join("store"))
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .count()
            })
            .unwrap_or(0),
    })
}

fn swap(data: &Path, staged: &Path, old: &Path) -> Result<()> {
    let mut moved: Vec<&str> = Vec::new();
    for folder in CARRIED {
        let at = data.join(folder);
        if !at.exists() {
            continue;
        }
        if let Err(e) = std::fs::rename(&at, old.join(folder)) {
            undo(data, old, &moved);
            return Err(Error::Io(e));
        }
        moved.push(folder);
    }

    for folder in CARRIED {
        let fresh = staged.join(folder);
        if !fresh.exists() {
            continue;
        }
        if let Err(e) = std::fs::rename(&fresh, data.join(folder)) {
            for done in CARRIED {
                let at = data.join(done);
                if at.exists() && staged.join(done).exists() {
                    continue;
                }
                let _ = std::fs::rename(&at, staged.join(done));
            }
            undo(data, old, &moved);
            return Err(Error::Io(e));
        }
    }
    Ok(())
}

fn undo(data: &Path, old: &Path, moved: &[&str]) {
    for folder in moved {
        let at = data.join(folder);
        if at.exists() {
            let _ = std::fs::remove_dir_all(&at);
        }
        if let Err(e) = std::fs::rename(old.join(folder), &at) {
            witness::error(
                channel::BACKUP,
                "folder not put back",
                &[
                    ("at", Fact::Path(at.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
    }
}

pub fn reset(paths: &Paths, into: &Path, aside: &Path) -> Result<Made> {
    let data = paths.data();
    let made = write(data, into, aside)?;

    let mut config = Config::load_or_init(paths)?;
    config.device_id = crate::DeviceId(crate::config::new_device_id());
    config.synced_at = None;
    config.heard_at = None;
    config.save(paths)?;

    store::kept_before_the_store_goes(paths);

    let old = data.join(format!(".resetting-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&old);
    std::fs::create_dir_all(&old)?;

    let mut moved: Vec<&str> = Vec::new();
    for folder in CARRIED {
        let at = data.join(folder);
        if !at.exists() {
            continue;
        }
        if let Err(e) = std::fs::rename(&at, old.join(folder)) {
            undo(data, &old, &moved);
            let _ = std::fs::remove_dir_all(&old);
            return Err(Error::Io(e));
        }
        moved.push(folder);
    }

    let _ = std::fs::remove_dir_all(&old);
    let _ = std::fs::remove_dir_all(paths.cache());
    crate::docs::forget_what_was_carried(data);
    Ok(made)
}

fn rescued(dest: &Path) {
    let Ok(entries) = std::fs::read_dir(dest) else {
        return;
    };
    for at in entries.filter_map(|e| e.ok()).map(|e| e.path()) {
        let stale = at.is_dir()
            && at
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(".taking-over-"));
        if !stale {
            continue;
        }
        for folder in CARRIED {
            let back = dest.join(folder);
            if !back.exists()
                && at.join(folder).is_dir()
                && let Err(e) = std::fs::rename(at.join(folder), &back)
            {
                witness::error(
                    channel::BACKUP,
                    "a folder left behind by an interrupted take-over could not be put back",
                    &[
                        ("at", Fact::Path(back.clone())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
        }
        if std::fs::remove_dir(&at).is_err() {
            witness::warn(
                channel::BACKUP,
                "an interrupted take-over left things behind that could not be put back, so they were kept where they are",
                &[("at", Fact::Path(at.clone()))],
            );
        }
    }
}

fn only_one_taking_over(dest: &Path) -> Result<std::fs::File> {
    let gate = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(dest.join(".taking-over.lock"))?;
    match gate.try_lock() {
        Ok(()) => Ok(gate),
        Err(std::fs::TryLockError::WouldBlock) => Err(Error::AlreadyRunning),
        Err(std::fs::TryLockError::Error(why)) => Err(why.into()),
    }
}

pub fn take_over(dest: &Path, ours: &str, into: &Path, aside: &Path) -> Result<Made> {
    let _gate = only_one_taking_over(dest)?;
    rescued(dest);
    let made = write(dest, into, aside)?;

    let old = dest.join(format!(".taking-over-{}", std::process::id()));
    if old.exists() {
        return Err(Error::AlreadyRunning);
    }
    std::fs::create_dir_all(&old)?;

    let mut moved: Vec<&str> = Vec::new();
    for folder in CARRIED {
        let at = dest.join(folder);
        if !at.exists() {
            continue;
        }
        if let Err(e) = std::fs::rename(&at, old.join(folder)) {
            undo(dest, &old, &moved);
            let _ = std::fs::remove_dir_all(&old);
            return Err(Error::Io(e));
        }
        moved.push(folder);
    }

    let _ = std::fs::remove_dir_all(&old);

    let store = dest.join("store");
    std::fs::create_dir_all(&store)?;
    store::write_atomic(&store.join(store::MARKER), ours.as_bytes())?;
    Ok(made)
}

pub fn leftovers(data: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(data) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|at| {
            at.is_dir()
                && at.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                    n.starts_with(".restoring-")
                        || n.starts_with(".replaced-")
                        || n.starts_with(".resetting-")
                })
        })
        .collect()
}

struct Unpacked {
    files: usize,
    leftover: usize,
    first: Option<String>,
}

fn unpack<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    into: &Path,
    at_most: u64,
) -> Result<Unpacked> {
    if zip.len() > AT_MOST_FILES {
        return Err(Error::TooBig);
    }
    let mut files = 0;
    let mut leftover = 0;
    let mut first: Option<String> = None;
    let mut bytes = 0u64;

    for i in 0..zip.len() {
        let mut held = zip.by_index(i).map_err(zipped)?;
        if held.is_dir() {
            continue;
        }
        let Some(rest) = safe(held.name()) else {
            first.get_or_insert_with(|| held.name().to_string());
            leftover += 1;
            continue;
        };
        if held.size() > at_most.saturating_sub(bytes) {
            return Err(Error::TooBig);
        }

        let at = into.join(&rest);
        if let Some(parent) = at.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(&at)?;
        let _ = crate::paths::ours_alone(&at);
        let room = at_most.saturating_sub(bytes).saturating_add(1);
        let written = std::io::copy(&mut held.by_ref().take(room), &mut file)?;
        if written >= room {
            return Err(Error::TooBig);
        }
        bytes = bytes.saturating_add(written);
        files += 1;
    }
    Ok(Unpacked {
        files,
        leftover,
        first,
    })
}

fn named_in<R: Read + Seek>(zip: &mut zip::ZipArchive<R>) -> Result<String> {
    let at = format!("store/{}", store::MARKER);
    match zip.by_name(&at) {
        Ok(mut held) => {
            let mut said = String::new();
            held.read_to_string(&mut said)?;
            Ok(said.trim().to_string())
        }
        Err(_) => Ok(String::new()),
    }
}

fn named(at: &Path) -> Option<Vec<std::borrow::Cow<'_, str>>> {
    at.components()
        .map(|part| match part {
            Component::Normal(one) => Some(one.to_string_lossy()),
            _ => None,
        })
        .collect()
}

fn kept_out(parts: &[std::borrow::Cow<'_, str>]) -> bool {
    let Some(leaf) = parts.last().map(|one| one.as_ref()) else {
        return true;
    };
    let under = parts.first().map(|one| one.as_ref()).unwrap_or_default();
    leaf == store::KEEP
        || leaf == ".lock"
        || crate::icloud::marker(leaf)
        || (under == "attachments" && leaf.starts_with('.') && leaf.ends_with(".part"))
        || (under != "attachments" && (leaf.ends_with(".part") || leaf.ends_with(".tmp")))
}

fn carried(at: &Path) -> bool {
    let Some(parts) = named(at) else {
        return false;
    };
    parts
        .first()
        .is_some_and(|under| CARRIED.contains(&under.as_ref()) && parts.len() > 1)
        && !kept_out(&parts)
}

fn safe(named: &str) -> Option<PathBuf> {
    let at = Path::new(named);
    carried(at).then(|| at.to_path_buf())
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::BACKUP,
                    "folder left out of the copy",
                    &[
                        ("at", Fact::Path(root.to_path_buf())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
            return found;
        }
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let at = entry.path();
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => found.extend(walk(&at)),
            Ok(kind) if kind.is_file() => found.push(at),
            _ => {}
        }
    }
    found.sort();
    found
}

fn zipped(e: zip::result::ZipError) -> Error {
    Error::Io(std::io::Error::other(e.to_string()))
}

#[cfg(test)]
#[path = "backup_test.rs"]
mod tests;
