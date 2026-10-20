use std::io::Read;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{
    Error, Result,
    witness::{self, Fact, channel},
};

pub const COPIED_UP_TO: u64 = 25 * 1024 * 1024;
pub const COPIED_IN_DOC: u64 = 650 * 1024 * 1024;
pub const HELD_UP_TO: u64 = 750 * 1024 * 1024;

const SHORTENS_TO: usize = 56;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub at: String,
    pub sha256: String,
    pub bytes: u64,
}

impl Kept {
    pub fn told(&self) -> crate::Op {
        crate::Op::AttachKept {
            d: crate::event::Held {
                at: self.at.clone(),
                sha256: self.sha256.clone(),
                bytes: self.bytes,
            },
        }
    }

    pub fn written(&self, label: &str) -> String {
        let name = spoken(label);
        let leaf = leaf_of(&self.at).unwrap_or(&self.at);
        format!("![{name}]({leaf})")
    }
}

/// In bytes: a body is measured in bytes and an accented label is two apiece.
fn written_at_most(label: &str) -> usize {
    spoken(label).len() + SHORTENS_TO + 64
}

pub fn counted(body: &str) -> usize {
    crate::refs::extract(body)
        .iter()
        .filter(|one| canonical(&one.target).is_some())
        .count()
}

/// Where the window stops calling a document a document and starts warning that it is a shelf.
pub const KEPT_IN_A_DOC: usize = 150;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoRoom {
    Crowded(usize),
    Full,
}

pub fn fits(body: &str, label: &str) -> std::result::Result<(), NoRoom> {
    let held = counted(body);
    if held >= KEPT_IN_A_DOC {
        return Err(NoRoom::Crowded(held));
    }
    if (body.len() + written_at_most(label)) as u64 > crate::docs::BODY_AT_MOST {
        return Err(NoRoom::Full);
    }
    Ok(())
}

fn spoken(label: &str) -> String {
    let flat: String = label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .map(|c| if c == '[' || c == ']' { '_' } else { c })
        .collect();
    if flat.is_empty() { "file".into() } else { flat }
}

/// One journal entry for a kept file, with where it came from. Copying reaches the folder that
/// syncs, so a wrong path has to be visible rather than silent.
pub fn journalled(kept: &Kept, label: &str, from: &Path, said: &str) -> String {
    let where_from = said.replace("{path}", &from.display().to_string());
    format!(
        "{}

{where_from}",
        kept.written(label)
    )
}

pub fn called(source: &Path, label: Option<String>) -> String {
    label.unwrap_or_else(|| {
        source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file")
            .to_string()
    })
}

/// A copy that never finished, left by a process that is gone. Nothing else looks in here for
/// them, and one can be as large as the attachment it was going to become.
pub fn swept(data: &Path) {
    let Ok(entries) = std::fs::read_dir(data.join("attachments")) else {
        return;
    };
    for at in entries.filter_map(|one| one.ok()).map(|one| one.path()) {
        let named = at.file_name().and_then(|one| one.to_str()).unwrap_or("");
        if crate::parting::spent(&at, named) && at.is_file() {
            let _ = std::fs::remove_file(&at);
        }
    }
}

pub fn keep(source: &Path, root: &Path, limit: u64) -> Result<Kept> {
    let mut file = std::fs::File::open(source)?;
    let opened = file.metadata()?;
    if !opened.is_file() {
        return Err(Error::OutsideTheStore(source.display().to_string()));
    }
    if opened.len() > limit {
        return Err(Error::AttachmentTooBig {
            bytes: opened.len(),
            limit,
        });
    }

    let shed = root.join("attachments");
    std::fs::create_dir_all(&shed)?;
    let _ = crate::paths::ours_alone(root);
    let _ = crate::paths::ours_alone(&shed);

    let part = shed.join(parting());
    let kept = through(&mut file, source, root, &shed, &part, limit);
    // One that found its place was renamed away; this reaches only a copy that did not.
    let _ = std::fs::remove_file(&part);
    kept
}

fn parting() -> String {
    static TURN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    crate::parting::named(TURN.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
}

fn through(
    file: &mut std::fs::File,
    source: &Path,
    root: &Path,
    shed: &Path,
    part: &Path,
    limit: u64,
) -> Result<Kept> {
    let (sha256, bytes) = poured(file, part, limit)?;

    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .filter(|e| plain(e))
        .map(|e| format!(".{e}"))
        .unwrap_or_default();

    let rest = &sha256[2..];
    let stamp = &rest[..8];
    let folder = shed.to_path_buf();

    if let Some(at) = listed(root, &sha256) {
        match resolve(&at, root) {
            Ok(held) if holds(&held, part, bytes) => {
                return Ok(Kept { at, sha256, bytes });
            }
            // Space freed here leaves the ledger line, so the same bytes go back under their name.
            Ok(held) if !held.exists() && named_for(&at, &sha256) => {
                if let Some(parent) = held.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::rename(part, &held)?;
                let _ = crate::paths::ours_alone(&held);
                return Ok(Kept { at, sha256, bytes });
            }
            Ok(held) => witness::warn(
                channel::ATTACH,
                "what the ledger points at is not what it says it is",
                &[
                    ("at", Fact::Path(held)),
                    ("sha256", Fact::Id(sha256.clone())),
                ],
            ),
            Err(_) => witness::warn(
                channel::ATTACH,
                "the ledger names a path outside the store",
                &[("at", Fact::Id(at)), ("sha256", Fact::Id(sha256.clone()))],
            ),
        }
    }

    let name = match already(&folder, stamp, part, bytes) {
        Some(kept) => kept,
        None => {
            let mut name = named(source, stamp, &ext);
            if folder.join(&name).exists() {
                name = named(source, &rest[..16], &ext);
            }
            let target = folder.join(&name);
            std::fs::rename(part, &target)?;
            let _ = crate::paths::ours_alone(&target);
            name
        }
    };
    let kept = Kept {
        at: format!("{SHED}/{name}"),
        sha256,
        bytes,
    };
    note(root, &kept);
    Ok(kept)
}

const AT_A_TIME: usize = 64 * 1024;

/// The bytes are never held whole: at the ceiling a document allows, reading one into memory and
/// then reading what is already kept to compare against it costs twice the bytes for the file itself.
fn poured(file: &mut std::fs::File, part: &Path, limit: u64) -> Result<(String, u64)> {
    use std::io::Write;

    let mut out = std::fs::File::create(part)?;
    let _ = crate::paths::ours_alone(part);
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; AT_A_TIME];
    let mut bytes = 0u64;
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        bytes += read as u64;
        if bytes > limit {
            return Err(Error::AttachmentTooBig { bytes, limit });
        }
        hasher.update(&buf[..read]);
        out.write_all(&buf[..read])?;
    }
    out.sync_all()?;
    Ok((hexed(hasher.finalize()), bytes))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Noted {
    at: String,
    sha256: String,
    bytes: u64,
}

fn ledger(root: &Path) -> PathBuf {
    root.join("attachments.jsonl")
}

// A copy let go of when two became one keeps its line, so the one still here is named first.
fn listed(root: &Path, sha256: &str) -> Option<String> {
    let text = std::fs::read_to_string(ledger(root)).ok()?;
    let named: Vec<String> = text
        .lines()
        .filter_map(|line| serde_json::from_str::<Noted>(line).ok())
        .filter(|one| one.sha256 == sha256)
        .map(|one| canonical(&one.at).unwrap_or(one.at))
        .collect();
    named
        .iter()
        .find(|at| resolve(at, root).is_ok_and(|held| held.is_file()))
        .or(named.first())
        .cloned()
}

pub fn digests(root: &Path) -> std::collections::BTreeMap<String, (String, u64)> {
    let Ok(text) = std::fs::read_to_string(ledger(root)) else {
        return Default::default();
    };
    text.lines()
        .filter_map(|line| serde_json::from_str::<Noted>(line).ok())
        .map(|one| {
            let at = canonical(&one.at).unwrap_or(one.at);
            (at, (one.sha256, one.bytes))
        })
        .collect()
}

fn holds(at: &Path, part: &Path, bytes: u64) -> bool {
    std::fs::metadata(at).is_ok_and(|held| held.is_file() && held.len() == bytes) && alike(at, part)
}

fn alike(one: &Path, two: &Path) -> bool {
    let (Ok(mut a), Ok(mut b)) = (std::fs::File::open(one), std::fs::File::open(two)) else {
        return false;
    };
    let mut here = vec![0u8; AT_A_TIME];
    let mut there = vec![0u8; AT_A_TIME];
    loop {
        let (Ok(read), Ok(again)) = (filled(&mut a, &mut here), filled(&mut b, &mut there)) else {
            return false;
        };
        if read != again || here[..read] != there[..again] {
            return false;
        }
        if read == 0 {
            return true;
        }
    }
}

/// A short read is not the end of a file, and treating it as one would compare bytes against the
/// bytes after them.
fn filled(file: &mut std::fs::File, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut held = 0;
    while held < buf.len() {
        match file.read(&mut buf[held..])? {
            0 => break,
            read => held += read,
        }
    }
    Ok(held)
}

fn tailed(at: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(at) else {
        return true;
    };
    let Ok(size) = file.metadata().map(|one| one.len()) else {
        return true;
    };
    if size == 0 {
        return true;
    }
    use std::io::{Read, Seek};
    if file.seek(std::io::SeekFrom::End(-1)).is_err() {
        return true;
    }
    let mut last = [0u8; 1];
    file.read_exact(&mut last).is_ok_and(|()| last[0] == b'\n')
}

pub fn noted(root: &Path, reference: &str, sha256: &str, bytes: u64) {
    note(
        root,
        &Kept {
            at: reference.to_string(),
            sha256: sha256.to_string(),
            bytes,
        },
    );
}

/// Copies a file across without holding it in memory, hashing it on the way.
pub fn copied(from: &Path, part: &Path, limit: u64) -> Result<(String, u64)> {
    let mut file = std::fs::File::open(from)?;
    poured(&mut file, part, limit)
}

/// The same reading, without the writing: for asking a file what it is where it lies.
pub fn hashed(at: &Path) -> Result<(String, u64)> {
    let mut file = std::fs::File::open(at)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; AT_A_TIME];
    let mut bytes = 0u64;
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        bytes += read as u64;
        hasher.update(&buf[..read]);
    }
    Ok((hexed(hasher.finalize()), bytes))
}

fn note(root: &Path, kept: &Kept) {
    if listed(root, &kept.sha256).is_some() {
        return;
    }
    noted_line(root, kept);
}

/// The copy two became one in is named in the ledger, so the same bytes kept again land on it.
pub fn stays_listed(root: &Path, at: &str) -> Result<()> {
    let at = canonical(at).ok_or_else(|| Error::OutsideTheStore(at.to_string()))?;
    let held = resolve(&at, root)?;
    let kept = Kept {
        sha256: fingerprint_of(&held)?,
        bytes: std::fs::metadata(&held)?.len(),
        at,
    };
    if digests(root)
        .get(&kept.at)
        .is_none_or(|(sha, _)| *sha != kept.sha256)
    {
        noted_line(root, &kept);
    }
    Ok(())
}

fn noted_line(root: &Path, kept: &Kept) {
    let line = match serde_json::to_string(&Noted {
        at: kept.at.clone(),
        sha256: kept.sha256.clone(),
        bytes: kept.bytes,
    }) {
        Ok(line) => line,
        Err(_) => return,
    };
    let at = ledger(root);
    let whole = if tailed(&at) {
        format!("{line}\n")
    } else {
        witness::warn(
            channel::ATTACH,
            "the ledger had no newline to append after",
            &[("at", Fact::Path(at.clone()))],
        );
        format!("\n{line}\n")
    };
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&at)
        .and_then(|mut file| std::io::Write::write_all(&mut file, whole.as_bytes()));
    let _ = crate::paths::ours_alone(&at);
}

fn named(source: &Path, stamp: &str, ext: &str) -> String {
    let slug: String = source
        .file_stem()
        .and_then(|one| one.to_str())
        .map(|one| crate::text::composed(one).to_lowercase())
        .unwrap_or_default()
        .chars()
        .map(plainly)
        .collect();

    let mut parts: Vec<&str> = slug.split('-').filter(|part| !part.is_empty()).collect();
    // A source that is itself a kept attachment already carries this stamp; it is added once, never again.
    while parts.last().is_some_and(|last| carries(last, stamp)) {
        parts.pop();
    }
    let slug: String = parts.join("-").chars().take(SHORTENS_TO).collect();
    let slug = slug.trim_matches('-');

    if slug.is_empty() {
        format!("{stamp}{ext}")
    } else {
        format!("{slug}-{stamp}{ext}")
    }
}

fn carries(part: &str, stamp: &str) -> bool {
    let short = &stamp[..stamp.len().min(8)];
    (part.len() == 8 || part.len() == 16) && lower_hex(part) && part.starts_with(short)
}

fn plainly(c: char) -> char {
    match c {
        'a'..='z' | '0'..='9' => c,
        'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        _ => '-',
    }
}

fn already(folder: &Path, stamp: &str, part: &Path, bytes: u64) -> Option<String> {
    std::fs::read_dir(folder)
        .ok()?
        .filter_map(|one| one.ok())
        .find_map(|one| {
            let name = one.file_name().to_str()?.to_string();
            let stem = name.split('.').next().unwrap_or(&name);
            if stem != stamp && !stem.ends_with(&format!("-{stamp}")) {
                return None;
            }
            let held = one.metadata().ok()?;
            if held.len() != bytes {
                return None;
            }
            alike(&one.path(), part).then_some(name)
        })
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Astray {
    pub at: String,
    pub bytes: u64,
    pub when: i64,
    /// Found in the shared folder rather than in this machine's store.
    #[serde(default)]
    pub shared: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Loose {
    pub items: Vec<Astray>,
    pub bytes: u64,
}

impl Loose {
    pub fn files(&self) -> usize {
        self.items.len()
    }
}

pub fn loose(root: &Path, referenced: &[String]) -> Loose {
    let held: std::collections::BTreeSet<&str> =
        referenced.iter().filter_map(|one| leaf_of(one)).collect();

    let mut found = Loose::default();
    let at = root.join(SHED);
    let files = match std::fs::read_dir(&at) {
        Ok(files) => files,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::ATTACH,
                    "attachments unreadable",
                    &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
                );
            }
            return found;
        }
    };
    for file in files.filter_map(|e| e.ok()) {
        let Some(leaf) = file.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        // A marker is the file iCloud took away, not a stray: taking it out is taking the file.
        if crate::holes::marker(&leaf) || !stamped(&leaf) || held.contains(leaf.as_str()) {
            continue;
        }
        let told = file.metadata().ok();
        if !told.as_ref().is_some_and(std::fs::Metadata::is_file) {
            continue;
        }
        let bytes = told.as_ref().map(|m| m.len()).unwrap_or(0);
        found.items.push(Astray {
            shared: false,
            at: format!("{SHED}/{leaf}"),
            bytes,
            when: told
                .and_then(|m| m.modified().ok())
                .map(since_epoch)
                .unwrap_or(0),
        });
        found.bytes += bytes;
    }
    found
        .items
        .sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.at.cmp(&b.at)));
    found
}

pub const BIN_HOLDS_FOR: i64 = 30 * 24 * 60 * 60;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Binned {
    at: String,
    when: i64,
}

fn bin(root: &Path) -> PathBuf {
    root.join("bin")
}

fn bin_ledger(root: &Path) -> PathBuf {
    root.join("bin.jsonl")
}

pub use tisty_markdown::kept::{SHED, leaf_of, lower_hex, stamped, stamped_by};

pub fn canonical(reference: &str) -> Option<String> {
    leaf_of(reference).map(|leaf| format!("{SHED}/{leaf}"))
}

pub fn names_an_attachment(reference: &str) -> bool {
    leaf_of(reference).is_some()
}

pub fn set_aside(root: &Path, reference: &str, now: i64) -> Result<()> {
    if !names_an_attachment(reference) {
        return Err(Error::OutsideTheStore(reference.to_string()));
    }
    let from = resolve(reference, root)?;
    if !from.is_file() {
        return Err(Error::OutsideTheStore(reference.to_string()));
    }
    let Some(leaf) = leaf_of(reference) else {
        return Err(Error::OutsideTheStore(reference.to_string()));
    };
    let into = bin(root).join(leaf);
    if let Some(folder) = into.parent() {
        std::fs::create_dir_all(folder)?;
        let _ = crate::paths::ours_alone(folder);
    }

    let line = serde_json::to_string(&Binned {
        at: reference.to_string(),
        when: now,
    })
    .map_err(|e| Error::Io(std::io::Error::other(e)))?;
    let ledger = bin_ledger(root);
    let whole = if tailed(&ledger) {
        format!("{line}\n")
    } else {
        format!("\n{line}\n")
    };
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&ledger)
        .and_then(|mut file| std::io::Write::write_all(&mut file, whole.as_bytes()))?;
    let _ = crate::paths::ours_alone(&ledger);

    std::fs::rename(&from, &into)?;
    Ok(())
}

pub fn empty_the_bin(root: &Path, now: i64) -> usize {
    let Ok(text) = std::fs::read_to_string(bin_ledger(root)) else {
        return 0;
    };
    let all: Vec<Binned> = text
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    let (stale, held): (Vec<Binned>, Vec<Binned>) = all
        .into_iter()
        .partition(|one| now - one.when >= BIN_HOLDS_FOR);
    if stale.is_empty() {
        return 0;
    }

    let mut gone = 0;
    for one in &stale {
        // A bin filled before the shelves went keeps them, and the last step finds either.
        let rest = leaf_of(&one.at).unwrap_or(one.at.trim_start_matches("attachments/"));
        let Ok(at) = resolve(&format!("bin/{rest}"), root) else {
            continue;
        };
        match std::fs::remove_file(&at) {
            Ok(()) => gone += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => witness::warn(
                channel::ATTACH,
                "the bin could not be emptied",
                &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
            ),
        }
    }

    let left: String = held
        .iter()
        .filter_map(|one| serde_json::to_string(one).ok())
        .map(|line| format!("{line}\n"))
        .collect();
    if let Err(why) = crate::store::write_atomic(&bin_ledger(root), left.as_bytes()) {
        witness::warn(
            channel::ATTACH,
            "the bin was emptied but its ledger was not written, so what went may be named still",
            &[("why", Fact::Why(why.to_string()))],
        );
    }
    gone
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Twins {
    pub bytes: u64,
    pub at: Vec<String>,
}

#[cfg(unix)]
fn told_apart(told: &std::fs::Metadata, _at: &Path, _one: &str) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((told.dev(), told.ino()))
}

#[cfg(windows)]
fn told_apart(_told: &std::fs::Metadata, at: &Path, one: &str) -> Option<same_file::Handle> {
    same_file::Handle::from_path(at)
        .inspect_err(|_| {
            witness::warn(
                channel::ATTACH,
                "an attachment could not be opened while looking for twins",
                &[("at", Fact::Id(one.to_owned()))],
            );
        })
        .ok()
}

pub fn twins(root: &Path) -> Vec<Twins> {
    let mut alike: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    let Ok(files) = std::fs::read_dir(root.join(SHED)) else {
        return Vec::new();
    };
    for file in files.filter_map(|one| one.ok()) {
        let Some(leaf) = file.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !file.file_type().is_ok_and(|kind| kind.is_file()) || !stamped(&leaf) {
            continue;
        }
        let stamp: String = stamped_by(&leaf).chars().take(8).collect();
        alike
            .entry(stamp)
            .or_default()
            .push(format!("{SHED}/{leaf}"));
    }

    let written_down = digests(root);
    let mut found = Vec::new();
    for named in alike.into_values().filter(|named| named.len() > 1) {
        let mut weighed: std::collections::BTreeMap<u64, Vec<String>> = Default::default();
        let mut standing = std::collections::HashSet::new();
        for one in named {
            let Ok(shown) = resolve(&one, root) else {
                continue;
            };
            let Ok(told) = std::fs::metadata(&shown) else {
                continue;
            };
            if told.len() > HELD_UP_TO {
                continue;
            }
            let Some(who) = told_apart(&told, &shown, &one) else {
                continue;
            };
            if !standing.insert(who) {
                continue;
            }
            weighed.entry(told.len()).or_default().push(one);
        }

        for (bytes, named) in weighed.into_iter().filter(|(_, named)| named.len() > 1) {
            let mut same: std::collections::BTreeMap<String, Vec<String>> = Default::default();
            for one in named {
                let said = match written_down.get(&one) {
                    Some((sha, held)) if *held == bytes => sha.clone(),
                    _ => {
                        let Ok(shown) = resolve(&one, root) else {
                            continue;
                        };
                        let Ok(said) = fingerprint_of(&shown) else {
                            witness::warn(
                                channel::ATTACH,
                                "an attachment could not be read while looking for twins",
                                &[("at", Fact::Id(one.clone()))],
                            );
                            continue;
                        };
                        said
                    }
                };
                same.entry(said).or_default().push(one);
            }
            for at in same.into_values().filter(|at| at.len() > 1) {
                found.push(Twins {
                    bytes: bytes * (at.len() as u64 - 1),
                    at,
                });
            }
        }
    }
    found.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.at.cmp(&b.at)));
    found
}

pub fn as_kept(
    written_down: &std::collections::BTreeMap<String, (String, u64)>,
    reference: &str,
    sha256: &str,
) -> bool {
    match written_down.get(reference) {
        Some((sha, _)) => sha == sha256,
        None => true,
    }
}

fn named_for(reference: &str, sha256: &str) -> bool {
    leaf_of(reference).is_some_and(|leaf| vouched(leaf, sha256))
}

// The stamp is the sha256 after its first two characters, which named a shelf when there were any.
pub fn vouched(leaf: &str, sha256: &str) -> bool {
    stamped(leaf)
        && sha256
            .get(2..)
            .is_some_and(|rest| rest.starts_with(stamped_by(leaf)))
}

pub fn sweep(
    root: &Path,
    retired: &std::collections::BTreeSet<String>,
    held: &std::collections::BTreeSet<&str>,
) -> usize {
    let mut gone = 0;
    for one in retired {
        if held.contains(one.as_str()) {
            continue;
        }
        if !names_an_attachment(one) {
            witness::warn(
                channel::ATTACH,
                "a retirement named something that is not an attachment at all",
                &[("at", Fact::Id(one.clone()))],
            );
            continue;
        }
        let Ok(at) = resolve(one, root) else {
            witness::warn(
                channel::ATTACH,
                "a retirement named something outside the store",
                &[("at", Fact::Id(one.clone()))],
            );
            continue;
        };
        match std::fs::remove_file(&at) {
            Ok(()) => gone += 1,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => witness::warn(
                channel::ATTACH,
                "a retired attachment could not be taken out",
                &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
            ),
        }
    }
    if gone > 0 {
        witness::note(
            channel::ATTACH,
            "retired attachments were taken out",
            &[("count", Fact::Count(gone))],
        );
    }
    gone
}

fn since_epoch(when: std::time::SystemTime) -> i64 {
    when.duration_since(std::time::UNIX_EPOCH)
        .map(|gone| gone.as_secs() as i64)
        .unwrap_or(0)
}

pub fn decoded(said: &str) -> Option<String> {
    let bytes = said.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;

    while at < bytes.len() {
        let pair = (bytes[at] == b'%')
            .then(|| said.get(at + 1..at + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match pair {
            Some(byte) => {
                out.push(byte);
                at += 3;
            }
            None => {
                out.push(bytes[at]);
                at += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

pub fn found(reference: &str, root: &Path, also: Option<&Path>) -> Result<PathBuf> {
    let here = resolve(reference, root)?;
    if here.is_file() {
        return Ok(here);
    }
    match also.map(|beside| resolve(reference, beside)) {
        Some(Ok(there)) if there.is_file() => Ok(there),
        _ => Ok(here),
    }
}

pub fn resolve(reference: &str, root: &Path) -> Result<PathBuf> {
    let flat = canonical(reference);
    let reference = flat.as_deref().unwrap_or(reference);
    let cleaned = reference.split(['?', '#']).next().unwrap_or("");
    let refused = || Err(Error::OutsideTheStore(reference.to_string()));
    if cleaned.is_empty() {
        return refused();
    }

    let Some(cleaned) = decoded(cleaned) else {
        return refused();
    };
    if cleaned.contains('\\') || cleaned.chars().any(char::is_control) {
        return refused();
    }

    let mut walked = root.to_path_buf();
    let mut steps = 0;
    for part in Path::new(&cleaned).components() {
        let Component::Normal(name) = part else {
            return refused();
        };
        let Some(name) = name.to_str() else {
            return refused();
        };
        if name.contains(':') || reserved(name) {
            return refused();
        }
        walked.push(name);
        steps += 1;
    }
    if steps == 0 {
        return refused();
    }
    Ok(walked)
}

pub fn reserved(name: &str) -> bool {
    let stem = name
        .split('.')
        .next()
        .unwrap_or(name)
        .trim()
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit())
}

fn plain(ext: &str) -> bool {
    !ext.is_empty() && ext.len() <= 16 && ext.chars().all(|c| c.is_ascii_alphanumeric())
}

pub fn printed(bytes: &[u8]) -> String {
    fingerprint(bytes)
}
fn fingerprint(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hexed(hasher.finalize())
}

/// A file this app admits reaches the size of a film, and the answer wanted is 32 bytes long.
fn fingerprint_of(at: &Path) -> Result<String> {
    let mut file = std::fs::File::open(at)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; AT_A_TIME];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(hexed(hasher.finalize()))
}

fn hexed(sum: impl AsRef<[u8]>) -> String {
    sum.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
#[path = "attach_test.rs"]
mod tests;
