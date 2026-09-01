use std::io::Read;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{
    Error, Result,
    witness::{self, Fact, channel},
};

pub const COPIED_UP_TO: u64 = 50 * 1024 * 1024;
pub const COPIED_AT_FIRST: u64 = 5 * 1024 * 1024;
pub const COPIED_IN_DOC: u64 = 500 * 1024 * 1024;

const SHORTENS_TO: usize = 56;

pub const COPIED_LEAST: u64 = 64 * 1024;
pub const COPIED_MOST: u64 = COPIED_UP_TO;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub at: String,
    pub sha256: String,
}

impl Kept {
    pub fn written(&self, label: &str) -> String {
        let name = spoken(label);
        let target = self.at.clone();
        format!("![{name}](<{target}>)")
    }
}

/// In bytes: a body is measured in bytes and an accented label is two apiece.
fn written_at_most(label: &str) -> usize {
    spoken(label).len() + SHORTENS_TO + 64
}

/// How many files a body already carries, by the shape `written` gives them.
pub fn counted(body: &str) -> usize {
    body.matches("](<attachments/").count() + body.matches("](attachments/").count()
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
    let turn = TURN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!(".{}.{turn}.part", std::process::id())
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

    let (shelf, rest) = sha256.split_at(2);
    let stamp = &rest[..8];
    let folder = shed.join(shelf);
    std::fs::create_dir_all(&folder)?;
    let _ = crate::paths::ours_alone(&folder);

    if let Some(at) = listed(root, &sha256) {
        match resolve(&at, root) {
            Ok(held) if holds(&held, part, bytes) => return Ok(Kept { at, sha256 }),
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
        at: format!("attachments/{shelf}/{name}"),
        sha256,
    };
    note(root, &kept, bytes);
    Ok(kept)
}

const AT_A_TIME: usize = 64 * 1024;

/// The bytes are never held whole: at the ceiling a document allows, reading one into memory and
/// then reading what is already kept to compare against it costs a gigabyte for a 500 MB file.
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

fn listed(root: &Path, sha256: &str) -> Option<String> {
    let text = std::fs::read_to_string(ledger(root)).ok()?;
    text.lines()
        .filter_map(|line| serde_json::from_str::<Noted>(line).ok())
        .find(|one| one.sha256 == sha256)
        .map(|one| one.at)
}

pub fn digests(root: &Path) -> std::collections::BTreeMap<String, (String, u64)> {
    let Ok(text) = std::fs::read_to_string(ledger(root)) else {
        return Default::default();
    };
    text.lines()
        .filter_map(|line| serde_json::from_str::<Noted>(line).ok())
        .map(|one| (one.at, (one.sha256, one.bytes)))
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
        },
        bytes,
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

fn note(root: &Path, kept: &Kept, bytes: u64) {
    if listed(root, &kept.sha256).is_some() {
        return;
    }
    let line = match serde_json::to_string(&Noted {
        at: kept.at.clone(),
        sha256: kept.sha256.clone(),
        bytes,
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

    let slug: String = slug
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(SHORTENS_TO)
        .collect();
    let slug = slug.trim_matches('-');

    if slug.is_empty() {
        format!("{stamp}{ext}")
    } else {
        format!("{slug}-{stamp}{ext}")
    }
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
    let held: std::collections::BTreeSet<&str> = referenced
        .iter()
        .map(|one| one.trim_start_matches("attachments/"))
        .collect();

    let mut found = Loose::default();
    let at = root.join("attachments");
    let shelves = match std::fs::read_dir(&at) {
        Ok(shelves) => shelves,
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
    for shelf in shelves.filter_map(|e| e.ok()) {
        let Some(name) = shelf.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|e| e.ok()) {
            let Some(leaf) = file.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            // A marker is the file iCloud took away, not a stray: taking it out is taking the file.
            if crate::icloud::marker(&leaf) || !shelved(&name, &leaf) {
                continue;
            }
            if held.contains(format!("{name}/{leaf}").as_str()) {
                continue;
            }
            let told = file.metadata().ok();
            let bytes = told.as_ref().map(|m| m.len()).unwrap_or(0);
            found.items.push(Astray {
                shared: false,
                at: format!("attachments/{name}/{leaf}"),
                bytes,
                when: told
                    .and_then(|m| m.modified().ok())
                    .map(since_epoch)
                    .unwrap_or(0),
            });
            found.bytes += bytes;
        }
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

pub fn names_an_attachment(reference: &str) -> bool {
    reference.starts_with("attachments/")
}

pub fn set_aside(root: &Path, reference: &str, now: i64) -> Result<()> {
    if !names_an_attachment(reference) {
        return Err(Error::OutsideTheStore(reference.to_string()));
    }
    let from = resolve(reference, root)?;
    if !from.is_file() {
        return Err(Error::OutsideTheStore(reference.to_string()));
    }
    let rest = reference.trim_start_matches("attachments/");
    let into = bin(root).join(rest);
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
        let rest = one.at.trim_start_matches("attachments/");
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
    let _ = std::fs::write(bin_ledger(root), left);
    gone
}

fn lower_hex(said: &str) -> bool {
    said.bytes()
        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

pub fn stamped_by(leaf: &str) -> &str {
    let stem = leaf.split('.').next().unwrap_or_default();
    stem.rsplit('-').next().unwrap_or_default()
}

pub fn shelved(shelf: &str, leaf: &str) -> bool {
    if shelf.len() != 2 || !lower_hex(shelf) {
        return false;
    }
    let stamp = stamped_by(leaf);
    (8..=16).contains(&stamp.len())
        && lower_hex(stamp)
        && leaf.len() <= 255
        && !leaf.contains('/')
        && !leaf.contains('\\')
        && leaf.chars().all(|c| !c.is_control())
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
    let at = root.join("attachments");
    let Ok(shelves) = std::fs::read_dir(&at) else {
        return Vec::new();
    };
    for shelf in shelves.filter_map(|one| one.ok()) {
        let Some(under) = shelf.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|one| one.ok()) {
            let Some(leaf) = file.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if !file.file_type().is_ok_and(|kind| kind.is_file()) || !shelved(&under, &leaf) {
                continue;
            }
            let stamp: String = stamped_by(&leaf).chars().take(8).collect();
            alike
                .entry(format!("{under}/{stamp}"))
                .or_default()
                .push(format!("attachments/{under}/{leaf}"));
        }
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
            if told.len() > COPIED_IN_DOC {
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
                        let Ok(body) = std::fs::read(&shown) else {
                            witness::warn(
                                channel::ATTACH,
                                "an attachment could not be read while looking for twins",
                                &[("at", Fact::Id(one.clone()))],
                            );
                            continue;
                        };
                        fingerprint(&body)
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

pub fn vouched(shelf: &str, leaf: &str, sha256: &str) -> bool {
    if !shelved(shelf, leaf) {
        return false;
    }
    sha256.starts_with(shelf) && sha256[shelf.len()..].starts_with(stamped_by(leaf))
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
            Ok(()) => {
                gone += 1;
                witness::note(
                    channel::ATTACH,
                    "a retired attachment was taken out",
                    &[("at", Fact::Path(at))],
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => witness::warn(
                channel::ATTACH,
                "a retired attachment could not be taken out",
                &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
            ),
        }
    }
    gone
}

fn since_epoch(when: std::time::SystemTime) -> i64 {
    when.duration_since(std::time::UNIX_EPOCH)
        .map(|gone| gone.as_secs() as i64)
        .unwrap_or(0)
}

fn decoded(said: &str) -> Option<String> {
    let bytes = said.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;

    while at < bytes.len() {
        if bytes[at] == b'%' {
            out.push(u8::from_str_radix(said.get(at + 1..at + 3)?, 16).ok()?);
            at += 3;
            continue;
        }
        out.push(bytes[at]);
        at += 1;
    }
    String::from_utf8(out).ok()
}

pub fn resolve(reference: &str, root: &Path) -> Result<PathBuf> {
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
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
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

fn hexed(sum: impl AsRef<[u8]>) -> String {
    sum.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {

    #[test]
    fn a_retirement_that_names_the_log_never_deletes_it() {
        let root = tempfile::tempdir().unwrap();
        let device = root.path().join("store").join("dev_a");
        std::fs::create_dir_all(&device).unwrap();
        let segment = device.join("000001.tisty");
        std::fs::write(&segment, b"lo que nadie puede borrar").unwrap();
        let papers = root.path().join("docs");
        std::fs::create_dir_all(&papers).unwrap();
        let paper = papers.join("dev_a-0001.md");
        std::fs::write(&paper, b"# Algo").unwrap();

        let retired: std::collections::BTreeSet<String> = [
            "store/dev_a/000001.tisty".to_string(),
            "docs/dev_a-0001.md".to_string(),
            "attachments.jsonl".to_string(),
        ]
        .into();
        let gone = sweep(root.path(), &retired, &Default::default());

        assert_eq!(gone, 0, "borro algo que no era un adjunto");
        assert!(segment.is_file(), "se llevo un segmento del log");
        assert!(paper.is_file(), "se llevo un documento");
    }

    #[test]
    fn a_retirement_that_names_a_real_attachment_still_takes_it_out() {
        let root = tempfile::tempdir().unwrap();
        let shelf = root.path().join("attachments").join("ab");
        std::fs::create_dir_all(&shelf).unwrap();
        let at = shelf.join("foto-a1b2c3d4.png");
        std::fs::write(&at, b"unos bytes").unwrap();

        let retired: std::collections::BTreeSet<String> =
            ["attachments/ab/foto-a1b2c3d4.png".to_string()].into();
        let gone = sweep(root.path(), &retired, &Default::default());

        assert_eq!(gone, 1);
        assert!(!at.exists());
    }
    use super::*;

    fn dropped(name: &str, bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(name);
        std::fs::write(&file, bytes).unwrap();
        (dir, file)
    }

    #[test]
    fn a_small_file_is_copied_in_and_named_by_its_contents() {
        let (_src, file) = dropped("shot.PNG", b"pretend this is a screenshot");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, sha256 } = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert!(at.starts_with("attachments/"), "{at}");
        assert!(at.ends_with(".png"), "the extension is lowercased: {at}");
        assert!(
            at.contains(&sha256[..2]),
            "the shelf is the first two: {at}"
        );
        assert!(root.path().join(&at).exists());
    }

    #[test]
    fn a_label_of_accents_is_measured_in_the_bytes_it_will_take() {
        let brim = "x".repeat(crate::docs::BODY_AT_MOST as usize - 400);

        assert!(fits(&brim, "plano").is_ok());
        assert_eq!(
            fits(&brim, &"á".repeat(200)),
            Err(NoRoom::Full),
            "two bytes apiece is what the line will cost, not one"
        );
    }

    #[test]
    fn a_document_already_carrying_its_fill_takes_no_more() {
        let many = "![uno](<attachments/ab/uno.png>)
"
        .repeat(KEPT_IN_A_DOC);

        assert_eq!(fits(&many, "plano"), Err(NoRoom::Crowded(KEPT_IN_A_DOC)));
        assert!(fits(&many[..many.len() / 2], "plano").is_ok());
    }

    fn many_chunks(size: usize, seed: u8) -> Vec<u8> {
        (0..size).map(|at| (at as u8) ^ seed).collect()
    }

    #[test]
    fn a_file_longer_than_one_read_arrives_byte_for_byte() {
        let bytes = many_chunks(AT_A_TIME * 3 + 977, 0x5b);
        let (_src, file) = dropped("charla.mp4", &bytes);
        let root = tempfile::tempdir().unwrap();

        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(std::fs::read(root.path().join(&kept.at)).unwrap(), bytes);
        assert_eq!(kept.sha256, fingerprint(&bytes), "hashed as it was copied");
    }

    #[test]
    fn two_long_files_that_differ_at_the_very_end_are_two_files() {
        let bytes = many_chunks(AT_A_TIME * 2, 0x11);
        let mut other = bytes.clone();
        *other.last_mut().unwrap() ^= 1;
        let (_one, first) = dropped("a.mp4", &bytes);
        let (_two, second) = dropped("b.mp4", &other);
        let root = tempfile::tempdir().unwrap();

        let one = keep(&first, root.path(), COPIED_UP_TO).unwrap();
        let two = keep(&second, root.path(), COPIED_UP_TO).unwrap();

        assert_ne!(one.at, two.at, "a last byte apart is not the same file");
        assert_eq!(std::fs::read(root.path().join(&two.at)).unwrap(), other);
    }

    #[test]
    fn a_copy_that_could_not_be_filed_leaves_nothing_half_written() {
        let bytes = many_chunks(AT_A_TIME + 5, 0x22);
        let (_src, file) = dropped("charla.mp4", &bytes);
        let root = tempfile::tempdir().unwrap();
        let shed = root.path().join("attachments");
        std::fs::create_dir_all(&shed).unwrap();
        let shelf = fingerprint(&bytes)[..2].to_string();
        std::fs::write(shed.join(&shelf), b"a file where the shelf goes").unwrap();

        assert!(keep(&file, root.path(), COPIED_UP_TO).is_err());

        let left: Vec<_> = std::fs::read_dir(&shed)
            .unwrap()
            .filter_map(|one| one.ok())
            .map(|one| one.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(left, [shelf], "the copy it could not file was taken away");
    }

    #[test]
    fn what_was_copied_leaves_no_part_file_beside_it() {
        let bytes = many_chunks(AT_A_TIME + 1, 0x33);
        let (_src, file) = dropped("charla.mp4", &bytes);
        let root = tempfile::tempdir().unwrap();

        keep(&file, root.path(), COPIED_UP_TO).unwrap();
        keep(&file, root.path(), COPIED_UP_TO).unwrap();

        let left: Vec<_> = std::fs::read_dir(root.path().join("attachments"))
            .unwrap()
            .filter_map(|one| one.ok())
            .map(|one| one.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".part"))
            .collect();
        assert!(left.is_empty(), "{left:?}");
    }

    #[test]
    fn the_same_file_twice_is_one_file() {
        let (_src, file) = dropped("shot.png", b"the very same bytes");
        let (_other, again) = dropped("renamed.png", b"the very same bytes");
        let root = tempfile::tempdir().unwrap();

        let first = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        let second = keep(&again, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(
            first, second,
            "the name comes from the contents, not the name"
        );
        let shelves = std::fs::read_dir(root.path().join("attachments")).unwrap();
        assert_eq!(shelves.count(), 1);
    }

    #[test]
    fn a_stamp_worn_by_unlike_bytes_does_not_hand_back_the_wrong_file() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("mine.bin", b"the bytes that are mine");
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        let stem = kept.at.rsplit('/').next().unwrap().to_string();
        let stamp = stem
            .split('.')
            .next()
            .unwrap()
            .rsplit('-')
            .next()
            .unwrap()
            .to_string();
        let shelf = kept.at.split('/').nth(1).unwrap().to_string();
        let impostor = root
            .path()
            .join("attachments")
            .join(&shelf)
            .join(format!("impostor-{stamp}.bin"));
        std::fs::write(&impostor, b"entirely other bytes, same stamp").unwrap();
        std::fs::remove_file(root.path().join("attachments.jsonl")).unwrap();

        let again = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(again.at, kept.at, "the bytes decide, never the name");
        assert_eq!(
            std::fs::read(root.path().join(&again.at)).unwrap(),
            b"the bytes that are mine"
        );
    }

    #[test]
    fn the_digest_is_written_down_whole_so_it_can_be_asked_for() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("mine.bin", b"some bytes worth keeping");

        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        let noted = std::fs::read_to_string(root.path().join("attachments.jsonl")).unwrap();
        assert!(
            noted.contains(&kept.sha256),
            "the whole digest, not ten characters"
        );
        assert_eq!(kept.sha256.len(), 64);
        assert!(noted.contains(&kept.at));
        assert!(noted.contains("24"), "the weight travels with it");
    }

    #[test]
    fn what_is_written_down_saves_the_search_but_never_the_checking() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("mine.bin", b"kept once");
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        let (_other, again) = dropped("elsewhere.bin", b"kept once");
        let second = keep(&again, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(second.at, kept.at);
        assert_eq!(
            std::fs::read(root.path().join(&second.at)).unwrap(),
            b"kept once"
        );
    }

    #[test]
    fn a_kept_file_changed_underneath_is_written_again_rather_than_handed_back() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("mine.bin", b"the only copy of my report");
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        std::fs::write(root.path().join(&kept.at), b"junk").unwrap();
        let again = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(
            std::fs::read(root.path().join(&again.at)).unwrap(),
            b"the only copy of my report",
            "the ledger said it was kept, but the bytes said otherwise"
        );
    }

    #[test]
    fn a_ledger_that_lost_its_last_newline_does_not_swallow_the_entry_after_it() {
        let root = tempfile::tempdir().unwrap();
        let (_src, first) = dropped("uno.bin", b"the first one");
        let one = keep(&first, root.path(), COPIED_UP_TO).unwrap();

        let ledger = root.path().join("attachments.jsonl");
        let held = std::fs::read_to_string(&ledger).unwrap();
        std::fs::write(&ledger, held.trim_end()).unwrap();

        let (_other, second) = dropped("dos.bin", b"the second one");
        let two = keep(&second, root.path(), COPIED_UP_TO).unwrap();

        let written = std::fs::read_to_string(&ledger).unwrap();
        let lines: Vec<&str> = written.lines().filter(|one| !one.is_empty()).collect();
        assert_eq!(lines.len(), 2, "one line ate the other: {lines:?}");
        for line in lines {
            serde_json::from_str::<Noted>(line).expect("still readable");
        }
        assert_ne!(one.at, two.at);
    }

    #[test]
    fn a_heavy_file_is_refused_and_never_copied() {
        let (_src, file) = dropped("recording.mkv", b"pretend this is fifteen gigabytes");
        let root = tempfile::tempdir().unwrap();

        let refused = keep(&file, root.path(), 4).unwrap_err();

        assert!(
            matches!(
                refused,
                crate::Error::AttachmentTooBig { bytes, limit } if bytes > 4 && limit == 4
            ),
            "{refused:?}"
        );
        assert!(
            !root.path().join("attachments").exists(),
            "nothing was copied in"
        );
    }

    #[test]
    fn everything_that_is_kept_comes_in_as_a_card() {
        let held = |at: &str| Kept {
            at: at.into(),
            sha256: "ab".into(),
        };

        assert_eq!(
            held("attachments/ab/cd.png").written("the screen"),
            "![the screen](<attachments/ab/cd.png>)"
        );
        assert_eq!(
            held("attachments/ab/cd.pdf").written("the invoice"),
            "![the invoice](<attachments/ab/cd.pdf>)",
            "what cannot be drawn still deserves a card, not a bare link"
        );
    }

    #[test]
    fn a_path_with_spaces_is_still_held_together() {
        let kept = Kept {
            at: "attachments/ab/clip (1).mkv".into(),
            sha256: "ab".into(),
        };
        let written = kept.written("clip (1).mkv");
        assert!(written.starts_with("![clip (1).mkv](<"), "{written}");
        assert!(written.ends_with(">)"), "{written}");
    }

    #[test]
    fn a_name_that_would_break_the_link_is_flattened() {
        let one = Kept {
            at: "attachments/ab/cd.png".into(),
            sha256: "ab".into(),
        };
        assert_eq!(
            one.written("shot](javascript:alert(1))["),
            "![shot_(javascript:alert(1))_](<attachments/ab/cd.png>)"
        );
        assert_eq!(
            one.written("two\nlines"),
            "![two lines](<attachments/ab/cd.png>)"
        );
        assert_eq!(one.written("   "), "![file](<attachments/ab/cd.png>)");
    }

    #[test]
    fn an_extension_that_hides_a_stream_is_dropped() {
        let (_src, file) = dropped("carrier.txt-evil", b"hidden");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, .. } = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        assert!(!at.contains(':'), "a stream reached the store: {at}");
        assert!(
            at.ends_with("-evil") || !at.contains('.'),
            "odd extension kept: {at}"
        );
    }

    #[test]
    fn a_reserved_name_is_a_device_and_not_a_file() {
        let root = Path::new("/data");
        for device in ["NUL", "CON", "COM1", "LPT9", "nul.png", "attachments/CON"] {
            assert!(resolve(device, root).is_err(), "«{device}» got through");
        }
    }

    #[test]
    fn a_drive_letter_without_a_root_is_still_a_way_out() {
        let root = Path::new("/data");
        for climbing in ["C:foo", "attachments/ab/cd.png:hidden", "//server/share"] {
            assert!(resolve(climbing, root).is_err(), "«{climbing}» got through");
        }
    }

    #[test]
    fn a_backslash_is_refused_wherever_it_is_read() {
        let root = Path::new("/data");

        for climbing in [
            r"..\..\.ssh\id_rsa",
            r"attachments\..\..\secrets",
            r"attachments\ab\cd.png",
        ] {
            assert!(
                resolve(climbing, root).is_err(),
                "«{climbing}» is read as a climb on Windows and as a name elsewhere"
            );
        }
    }

    #[test]
    fn two_dots_inside_a_name_are_not_a_climb() {
        let root = Path::new("/data");
        assert!(resolve("attachments/ab/my..file.png", root).is_ok());
    }

    #[test]
    fn nothing_reaches_outside_the_data_root() {
        let root = Path::new("/data");

        for climbing in [
            "../../.ssh/id_rsa",
            "attachments/../../secrets",
            "/etc/passwd",
            "C:/Windows/System32/config",
            "",
        ] {
            assert!(
                resolve(climbing, root).is_err(),
                "«{climbing}» got out of the store"
            );
        }
        assert!(resolve("attachments/ab/cd.png", root).is_ok());
        assert!(resolve("docs/notes.md", root).is_ok());
    }

    #[test]
    fn a_name_the_window_wrote_with_percents_finds_the_file_it_names() {
        let root = tempfile::tempdir().unwrap();
        let root = root.path();

        assert_eq!(
            resolve("attachments/ab/mi%20foto.png", root).unwrap(),
            root.join("attachments").join("ab").join("mi foto.png")
        );
        assert_eq!(
            resolve("docs/nota%20larga.md", root).unwrap(),
            root.join("docs").join("nota larga.md")
        );
        for hidden in [
            "attachments%2F..%2F..%2Fsecret.png",
            "%2E%2E/secret.png",
            "attachments%5Cab%5Ccd.png",
            "attachments/%00/cd.png",
            "attachments/ab/%zz.png",
            "attachments/ab/cd.png%",
        ] {
            assert!(
                resolve(hidden, root).is_err(),
                "«{hidden}» got out of the store"
            );
        }
    }

    #[test]
    fn a_file_the_exact_size_of_the_limit_is_copied() {
        let (_src, file) = dropped("shot.png", b"1234");
        let root = tempfile::tempdir().unwrap();

        assert!(keep(&file, root.path(), 4).is_ok());
    }

    #[test]
    fn a_file_without_an_extension_keeps_its_name_and_its_stamp() {
        let (_src, file) = dropped("README", b"no extension here");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, sha256 } = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert!(at.ends_with(&format!("readme-{}", &sha256[2..10])), "{at}");
    }

    fn stored(name: &str, bytes: &[u8]) -> String {
        let (_src, file) = dropped(name, bytes);
        let root = tempfile::tempdir().unwrap();
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        kept.at.rsplit('/').next().unwrap().to_string()
    }

    #[test]
    fn the_name_on_disk_is_the_one_a_person_would_recognise() {
        assert!(stored("Informe final.pdf", b"a").starts_with("informe-final-"));
        assert!(stored("captura de pantalla.png", b"b").starts_with("captura-de-pantalla-"));
    }

    #[test]
    fn nothing_a_system_argues_about_survives_in_the_name() {
        let kept = stored("Diseño Técnico: v2 <final>.PDF", b"c");

        assert!(kept.starts_with("diseno-tecnico-v2-final-"), "{kept}");
        assert!(kept.ends_with(".pdf"), "{kept}");
        assert!(
            kept.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.'),
            "{kept}"
        );
    }

    #[test]
    fn a_name_windows_reserves_stops_being_reserved() {
        let kept = stored("CON.txt", b"d");

        assert!(kept.starts_with("con-"), "{kept}");
        assert_ne!(kept, "con.txt");
    }

    #[test]
    fn a_name_nobody_could_shorten_falls_back_to_the_stamp() {
        let kept = stored("привет мир.txt", b"e");

        assert!(kept.ends_with(".txt"), "{kept}");
        assert_eq!(kept.len(), "12345678.txt".len(), "{kept}");
    }

    #[test]
    fn a_very_long_name_is_cut_without_losing_the_stamp() {
        let (_src, file) = dropped(&format!("{}.pdf", "nombre-larguisimo-".repeat(10)), b"f");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, sha256 } = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        let kept = at.rsplit('/').next().unwrap();

        assert!(kept.len() < 80, "{} chars: {kept}", kept.len());
        assert!(
            kept.ends_with(&format!("-{}.pdf", &sha256[2..10])),
            "the stamp was cut off with the name: {kept}"
        );
        assert!(kept.starts_with("nombre-larguisimo-"), "{kept}");
    }

    #[test]
    fn what_a_real_name_looks_like_on_disk() {
        for (given, wanted) in [
            ("Informe Técnico Final v2.pdf", "informe-tecnico-final-v2"),
            (
                "Captura de pantalla 2026-08-13.png",
                "captura-de-pantalla-2026-08-13",
            ),
            ("presupuesto (copia).xlsx", "presupuesto-copia"),
            ("Diseño & Maquetación.sketch", "diseno-maquetacion"),
        ] {
            let kept = stored(given, given.as_bytes());
            assert!(kept.starts_with(wanted), "«{given}» quedó como «{kept}»");
        }
    }

    #[test]
    fn the_same_bytes_under_two_names_are_still_one_file() {
        let root = tempfile::tempdir().unwrap();
        let (_a, first) = dropped("informe.pdf", b"same bytes");
        let (_b, second) = dropped("copia del informe.pdf", b"same bytes");

        let one = keep(&first, root.path(), COPIED_UP_TO).unwrap();
        let two = keep(&second, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(one.at, two.at, "kept twice");
        assert_eq!(one.sha256, two.sha256);
    }

    #[test]
    fn a_directory_is_not_a_file_and_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();

        assert!(keep(dir.path(), root.path(), COPIED_UP_TO).is_err());
    }

    #[test]
    fn what_no_prose_names_any_more_is_counted() {
        let (_src, one) = dropped("kept.png", b"still referenced");
        let (_other, two) = dropped("gone.png", b"nobody points here");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        keep(&two, root.path(), COPIED_UP_TO).unwrap();

        let counted = loose(root.path(), &[at]);
        assert_eq!(counted.files(), 1);
        assert_eq!(counted.bytes, b"nobody points here".len() as u64);

        assert_eq!(loose(root.path(), &[]).files(), 2);
    }

    #[test]
    fn two_names_holding_the_same_bytes_are_shown_together() {
        let (_a, one) = dropped("charla.mp4", b"the very same bytes");
        let root = tempfile::tempdir().unwrap();
        let first = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mine = root.path().join(&first.at);
        let other = mine.with_file_name(format!(
            "video-{}",
            mine.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .rsplit('-')
                .next()
                .unwrap()
        ));
        std::fs::copy(&mine, &other).unwrap();

        let said = twins(root.path());

        assert_eq!(said.len(), 1, "{said:?}");
        assert_eq!(said[0].at.len(), 2);
        assert_eq!(
            said[0].bytes,
            b"the very same bytes".len() as u64,
            "it showed what one weighs, not what letting one go would give back"
        );
    }

    #[test]
    fn what_the_ledger_already_knows_is_not_read_from_disk_again() {
        let (_a, one) = dropped("charla.mp4", b"the very same bytes");
        let root = tempfile::tempdir().unwrap();
        let first = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mine = root.path().join(&first.at);
        let other = mine.with_file_name(format!("video-{}.mp4", &first.sha256[2..10]));
        std::fs::copy(&mine, &other).unwrap();

        let said = twins(root.path());
        assert_eq!(said.len(), 1, "{said:?}");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut how = std::fs::metadata(&mine).unwrap().permissions();
            how.set_mode(0o000);
            std::fs::set_permissions(&mine, how).unwrap();

            let blind = twins(root.path());

            assert_eq!(
                blind.len(),
                1,
                "it went to the disk for a digest it had already written down: {blind:?} against {:?}",
                digests(root.path())
            );
        }
    }

    #[test]
    fn a_stamp_in_capitals_is_not_a_stamp() {
        assert!(!shelved("ab", "charla-A3F9BB01.mp4"));
        assert!(!shelved("AB", "charla-a3f9bb01.mp4"));
        assert!(!vouched("ab", "charla-A3F9BB01.mp4", &printed(b"whatever")));
    }

    #[test]
    fn what_we_already_kept_is_measured_against_the_whole_digest() {
        let mine = "attachments/ab/charla-a3f9bb01.mp4".to_string();
        let written_down = std::collections::BTreeMap::from([(
            mine.clone(),
            (fingerprint(b"the bytes we trusted"), 20),
        )]);

        assert!(as_kept(
            &written_down,
            &mine,
            &printed(b"the bytes we trusted")
        ));
        assert!(
            !as_kept(
                &written_down,
                &mine,
                &printed(b"bytes that passed the name")
            ),
            "a swap under a trusted name went through on forty bits"
        );
    }

    #[test]
    fn what_we_never_kept_has_nothing_to_be_measured_against() {
        let written_down = std::collections::BTreeMap::from([(
            "attachments/ab/otra-a3f9bb01.mp4".to_string(),
            (fingerprint(b"something else"), 14),
        )]);

        assert!(
            as_kept(
                &written_down,
                "attachments/ab/nueva-a3f9bb01.mp4",
                &printed(b"brand new")
            ),
            "an attachment arriving for the first time was refused for being unknown"
        );
    }

    #[test]
    fn a_name_with_no_stamp_vouches_for_nothing() {
        assert!(!vouched("ab", "", &printed(b"anything at all")));
        assert!(!vouched("", "", &printed(b"anything at all")));
        assert!(!vouched("ab", "charla-.mp4", &printed(b"anything at all")));
        assert!(!vouched("ab", "charla.mp4", &printed(b"anything at all")));
    }

    #[test]
    fn three_copies_show_what_two_of_them_are_costing() {
        let (_a, one) = dropped("charla.mp4", b"ten bytes!");
        let root = tempfile::tempdir().unwrap();
        let first = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mine = root.path().join(&first.at);
        for slug in ["video", "copia"] {
            let other = mine.with_file_name(format!("{slug}-{}.mp4", &first.sha256[2..10]));
            std::fs::copy(&mine, &other).unwrap();
        }

        let said = twins(root.path());

        assert_eq!(said.len(), 1);
        assert_eq!(said[0].at.len(), 3);
        assert_eq!(
            said[0].bytes, 20,
            "it showed one copy, not the room to win back"
        );
    }

    #[test]
    fn a_long_stamp_and_a_short_one_are_still_the_same_file() {
        let (_a, one) = dropped("charla.mp4", b"the very same bytes");
        let root = tempfile::tempdir().unwrap();
        let first = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mine = root.path().join(&first.at);
        let other = mine.with_file_name(format!("video-{}.mp4", &first.sha256[2..18]));
        std::fs::copy(&mine, &other).unwrap();

        let said = twins(root.path());

        assert_eq!(
            said.len(),
            1,
            "the one case this exists for went unseen: {said:?}"
        );
    }

    #[test]
    fn a_second_name_for_the_same_bytes_is_not_a_second_copy() {
        let (_a, one) = dropped("charla.mp4", b"the very same bytes");
        let root = tempfile::tempdir().unwrap();
        let first = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mine = root.path().join(&first.at);
        let other = mine.with_file_name(format!("video-{}.mp4", &first.sha256[2..10]));
        std::fs::hard_link(&mine, &other).unwrap();

        assert!(
            twins(root.path()).is_empty(),
            "a second name was counted as room to win back, and letting it go frees nothing"
        );
    }

    #[cfg(windows)]
    #[test]
    fn a_name_that_cannot_be_told_apart_is_not_room_to_win_back() {
        use std::io::Write;
        use std::os::windows::fs::OpenOptionsExt;
        let body = b"the very same bytes";
        let (_a, one) = dropped("charla.mp4", body);
        let root = tempfile::tempdir().unwrap();
        let first = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mine = root.path().join(&first.at);
        let other = mine.with_file_name(format!("video-{}.mp4", &first.sha256[2..10]));
        std::fs::copy(&mine, &other).unwrap();

        let leaf = other.file_name().unwrap().to_str().unwrap();
        let noted = format!(
            "{{\"at\":\"attachments/{}/{leaf}\",\"sha256\":\"{}\",\"bytes\":{}}}\n",
            &first.sha256[..2],
            first.sha256,
            body.len()
        );
        std::fs::OpenOptions::new()
            .append(true)
            .open(root.path().join("attachments.jsonl"))
            .unwrap()
            .write_all(noted.as_bytes())
            .unwrap();

        let _shut = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&other)
            .unwrap();

        assert!(
            twins(root.path()).is_empty(),
            "it promised room back for a name it could not even open to identify"
        );
    }

    #[test]
    fn a_store_where_every_name_holds_its_own_bytes_shows_no_twins() {
        let (_a, one) = dropped("charla.mp4", b"one thing");
        let (_b, two) = dropped("notas.pdf", b"another thing entirely");
        let root = tempfile::tempdir().unwrap();
        keep(&one, root.path(), COPIED_UP_TO).unwrap();
        keep(&two, root.path(), COPIED_UP_TO).unwrap();

        assert!(twins(root.path()).is_empty());
    }

    #[test]
    fn a_shared_stamp_is_not_taken_for_shared_bytes() {
        let (_a, one) = dropped("charla.mp4", b"the very same bytes");
        let root = tempfile::tempdir().unwrap();
        let first = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mine = root.path().join(&first.at);
        let other = mine.with_file_name(format!(
            "impostor-{}",
            mine.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .rsplit('-')
                .next()
                .unwrap()
        ));
        std::fs::write(&other, b"different bytes wearing the same stamp").unwrap();

        assert!(
            twins(root.path()).is_empty(),
            "it called them twins on the strength of the name alone"
        );
    }

    #[test]
    fn the_same_bytes_under_another_name_are_kept_once() {
        let (_a, one) = dropped("charla.mp4", b"the very same bytes");
        let (_b, two) = dropped("video-de-la-charla.mp4", b"the very same bytes");
        let root = tempfile::tempdir().unwrap();

        let first = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let again = keep(&two, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(
            first.at, again.at,
            "a second copy was written under a new name"
        );
        let shelf = root.path().join(&first.at).parent().unwrap().to_path_buf();
        assert_eq!(std::fs::read_dir(shelf).unwrap().count(), 1);
    }

    #[test]
    fn the_same_bytes_are_recognised_even_with_no_ledger_to_ask() {
        let (_a, one) = dropped("charla.mp4", b"the very same bytes");
        let (_b, two) = dropped("otro-nombre.mp4", b"the very same bytes");
        let root = tempfile::tempdir().unwrap();
        let first = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        std::fs::remove_file(root.path().join("attachments.jsonl")).unwrap();

        let again = keep(&two, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(
            first.at, again.at,
            "without the ledger it stopped seeing what the folder already held"
        );
    }

    #[test]
    fn a_name_vouches_for_the_bytes_it_was_given() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mut parts = at.split('/');
        parts.next();
        let shelf = parts.next().unwrap();
        let leaf = parts.next().unwrap();

        assert!(vouched(shelf, leaf, &printed(b"the bytes of a talk")));
    }

    #[test]
    fn a_name_refuses_to_vouch_for_bytes_that_are_not_the_ones() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mut parts = at.split('/');
        parts.next();
        let shelf = parts.next().unwrap();
        let leaf = parts.next().unwrap();

        assert!(!vouched(
            shelf,
            leaf,
            &printed(b"someone else's bytes entirely")
        ));
        assert!(!vouched(shelf, leaf, &printed(b"")));
        assert!(
            !vouched("00", leaf, &printed(b"the bytes of a talk")),
            "the shelf is not read"
        );
    }

    #[test]
    fn the_right_shelf_alone_vouches_for_nothing() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let shelf = at.split('/').nth(1).unwrap();

        assert!(
            !vouched(
                shelf,
                "charla-00000000.mp4",
                &printed(b"the bytes of a talk")
            ),
            "the shelf matched and the stamp went unread"
        );
        assert!(!vouched(
            shelf,
            "charla-ffffffff.mp4",
            &printed(b"the bytes of a talk")
        ));
    }

    #[test]
    fn a_longer_stamp_is_checked_to_its_full_length() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, sha256 } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let mut parts = at.split('/');
        parts.next();
        let shelf = parts.next().unwrap();
        let long = format!("charla-{}.mp4", &sha256[2..18]);

        assert!(vouched(shelf, &long, &printed(b"the bytes of a talk")));
        assert!(!vouched(shelf, &long, &printed(b"other bytes")));
    }

    #[test]
    fn what_is_let_go_waits_in_the_bin_instead_of_vanishing() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();

        set_aside(root.path(), &at, 1_000).unwrap();

        assert!(!root.path().join(&at).exists(), "it is still in the way");
        let rest = at.trim_start_matches("attachments/");
        assert_eq!(
            std::fs::read(root.path().join("bin").join(rest)).unwrap(),
            b"the bytes of a talk",
            "letting go threw the bytes away"
        );
    }

    #[test]
    fn attaching_again_what_was_retired_does_not_hand_it_to_the_sweeper() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        set_aside(root.path(), &at, 1_000).unwrap();
        let retired: std::collections::BTreeSet<String> = [at.clone()].into();
        sweep(root.path(), &retired, &Default::default());

        let again = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        sweep(root.path(), &retired, &[again.at.as_str()].into());

        assert!(
            root.path().join(&again.at).exists(),
            "attaching it again put it straight back in front of the sweeper"
        );
    }

    #[test]
    fn a_retirement_never_outranks_a_reference_that_exists_now() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();

        let gone = sweep(root.path(), &[at.clone()].into(), &[at.as_str()].into());

        assert_eq!(gone, 0);
        assert!(
            root.path().join(&at).exists(),
            "a tombstone took something that is in use right now"
        );
    }

    #[test]
    fn what_the_bin_holds_is_not_taken_twice_when_it_is_retired_again() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        set_aside(root.path(), &at, 1_000).unwrap();

        let why = set_aside(root.path(), &at, 2_000);

        assert!(why.is_err(), "it wrote a second word about the same file");
        let said = std::fs::read_to_string(root.path().join("bin.jsonl")).unwrap();
        assert_eq!(
            said.matches("charla").count(),
            1,
            "the bin ledger grew a duplicate that empties nothing"
        );
    }

    #[test]
    fn nothing_lands_in_the_bin_without_the_ledger_knowing() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        set_aside(root.path(), &at, 1_000).unwrap();

        let said = std::fs::read_to_string(root.path().join("bin.jsonl")).unwrap();

        assert!(
            said.contains(&at),
            "a file in the bin that nothing names is a file nothing will ever empty"
        );
    }

    #[test]
    fn the_bin_is_not_counted_as_something_adrift() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();

        set_aside(root.path(), &at, 1_000).unwrap();

        assert_eq!(
            loose(root.path(), &[]),
            Loose::default(),
            "the bin was read as attachments"
        );
    }

    #[test]
    fn the_bin_holds_for_thirty_days_and_not_a_day_less() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        set_aside(root.path(), &at, 1_000).unwrap();
        let rest = at.trim_start_matches("attachments/").to_string();
        let held = root.path().join("bin").join(&rest);

        assert_eq!(empty_the_bin(root.path(), 1_000 + BIN_HOLDS_FOR - 1), 0);
        assert!(held.exists(), "it emptied a day early");

        assert_eq!(empty_the_bin(root.path(), 1_000 + BIN_HOLDS_FOR), 1);
        assert!(!held.exists(), "it never emptied");
    }

    #[test]
    fn emptying_the_bin_leaves_what_is_still_within_its_time() {
        let (_a, one) = dropped("charla.mp4", b"the bytes of a talk");
        let (_b, two) = dropped("notas.pdf", b"other bytes entirely");
        let root = tempfile::tempdir().unwrap();
        let Kept { at: old, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let Kept { at: fresh, .. } = keep(&two, root.path(), COPIED_UP_TO).unwrap();
        set_aside(root.path(), &old, 1_000).unwrap();
        set_aside(root.path(), &fresh, 1_000 + BIN_HOLDS_FOR).unwrap();

        let gone = empty_the_bin(root.path(), 1_000 + BIN_HOLDS_FOR);

        assert_eq!(gone, 1);
        let rest = fresh.trim_start_matches("attachments/");
        assert!(
            root.path().join("bin").join(rest).exists(),
            "it took what was still waiting"
        );

        let said = std::fs::read_to_string(root.path().join("bin.jsonl")).unwrap();
        assert!(!said.contains(&old), "the ledger still names what is gone");
        assert!(said.contains(&fresh), "the ledger forgot what is waiting");
    }

    #[test]
    fn emptying_the_bin_twice_says_nothing_the_second_time() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        set_aside(root.path(), &at, 1_000).unwrap();

        assert_eq!(empty_the_bin(root.path(), 1_000 + BIN_HOLDS_FOR), 1);
        assert_eq!(empty_the_bin(root.path(), 1_000 + BIN_HOLDS_FOR), 0);
    }

    #[test]
    fn what_was_retired_is_taken_out_wherever_it_is_read() {
        let (_src, one) = dropped("charla.mp4", b"the bytes of a talk");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();

        let gone = sweep(root.path(), &[at.clone()].into(), &Default::default());

        assert_eq!(gone, 1);
        assert!(!root.path().join(&at).exists());
    }

    #[test]
    fn a_retirement_that_already_happened_here_is_not_an_error() {
        let root = tempfile::tempdir().unwrap();

        let gone = sweep(
            root.path(),
            &["attachments/ab/nada-a3f9.pdf".to_string()].into(),
            &Default::default(),
        );

        assert_eq!(gone, 0, "it counted what was not there");
    }

    #[test]
    fn a_retirement_can_never_name_its_way_out_of_the_store() {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join("attachments").join("ab")).unwrap();
        let outside = root.path().join("mine.txt");
        std::fs::write(&outside, b"not yours to take").unwrap();
        let far = root.path().join("far.txt");
        std::fs::write(&far, b"nor this one").unwrap();

        let gone = sweep(
            root.path(),
            &[
                "attachments/../mine.txt".to_string(),
                "attachments/ab/../../mine.txt".to_string(),
                far.display().to_string(),
            ]
            .into(),
            &Default::default(),
        );

        assert_eq!(gone, 0, "a retirement reached outside the store");
        assert!(outside.exists(), "climbing out of attachments worked");
        assert!(far.exists(), "an absolute path was followed");
    }

    #[test]
    fn what_is_retired_leaves_the_rest_alone() {
        let (_a, one) = dropped("charla.mp4", b"the bytes of a talk");
        let (_b, two) = dropped("notas.pdf", b"other bytes entirely");
        let root = tempfile::tempdir().unwrap();
        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        let Kept { at: kept, .. } = keep(&two, root.path(), COPIED_UP_TO).unwrap();

        sweep(root.path(), &[at].into(), &Default::default());

        assert!(root.path().join(&kept).exists(), "it took the wrong one");
    }

    #[test]
    fn what_is_loose_says_where_it_is_and_what_it_weighs() {
        let (_src, one) = dropped("minuta.pdf", b"nobody points here at all");
        let root = tempfile::tempdir().unwrap();

        keep(&one, root.path(), COPIED_UP_TO).unwrap();

        let counted = loose(root.path(), &[]);
        let astray = counted.items.first().expect("the loose one is listed");
        assert!(
            astray.at.starts_with("attachments/") && astray.at.ends_with(".pdf"),
            "a person has to be able to find it by hand: {}",
            astray.at
        );
        assert_eq!(astray.bytes, b"nobody points here at all".len() as u64);
        assert!(astray.when > 0, "without a date there is nothing to judge");
    }

    #[test]
    fn the_heaviest_is_shown_first_because_that_is_what_is_asked() {
        let (_a, small) = dropped("small.bin", b"tiny");
        let (_b, big) = dropped("big.bin", &[b'x'; 400]);
        let (_c, mid) = dropped("mid.bin", &[b'y'; 40]);
        let root = tempfile::tempdir().unwrap();

        keep(&small, root.path(), COPIED_UP_TO).unwrap();
        keep(&big, root.path(), COPIED_UP_TO).unwrap();
        keep(&mid, root.path(), COPIED_UP_TO).unwrap();

        let weights: Vec<u64> = loose(root.path(), &[])
            .items
            .iter()
            .map(|one| one.bytes)
            .collect();

        assert_eq!(weights, vec![400, 40, 4]);
    }

    #[test]
    fn the_ledger_is_never_mistaken_for_something_adrift() {
        let (_src, one) = dropped("held.png", b"referenced by someone");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();

        assert!(root.path().join("attachments.jsonl").exists());
        assert_eq!(loose(root.path(), &[at]), Loose::default());
    }

    #[test]
    fn a_store_without_attachments_has_nothing_loose() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(loose(root.path(), &[]), Loose::default());
    }

    #[test]
    fn what_follows_a_question_mark_is_not_the_file() {
        let root = Path::new("/data");
        assert_eq!(
            resolve("attachments/ab/cd.png?v=2", root).unwrap(),
            root.join("attachments/ab/cd.png")
        );
    }

    #[test]
    fn a_line_that_arrived_half_written_is_skipped_not_fatal() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("attachments.jsonl"),
            "{\"at\":\"attachments/ab/cut-off\",\"sha256\":\"dead\n{\"at\":\"attachments/ab/real.bin\",\"sha256\":\"beef\",\"bytes\":3}\n",
        )
        .unwrap();

        assert_eq!(
            listed(root.path(), "beef"),
            Some("attachments/ab/real.bin".to_string())
        );
        assert_eq!(listed(root.path(), "dead"), None);
    }

    #[test]
    fn a_line_that_is_not_json_at_all_does_not_hide_the_line_beside_it() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("attachments.jsonl"),
            "not json at all\n{\"at\":\"attachments/ab/real.bin\",\"sha256\":\"beef\",\"bytes\":3}\n",
        )
        .unwrap();

        assert_eq!(
            listed(root.path(), "beef"),
            Some("attachments/ab/real.bin".to_string())
        );
    }

    #[test]
    fn an_empty_ledger_file_is_read_as_if_nothing_had_been_noted() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("attachments.jsonl"), "").unwrap();

        assert_eq!(listed(root.path(), "anything"), None);
    }

    #[cfg(unix)]
    #[test]
    fn a_ledger_the_process_cannot_read_is_treated_as_unwritten() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let ledger_at = root.path().join("attachments.jsonl");
        std::fs::write(
            &ledger_at,
            "{\"at\":\"attachments/ab/real.bin\",\"sha256\":\"beef\",\"bytes\":3}\n",
        )
        .unwrap();
        std::fs::set_permissions(&ledger_at, std::fs::Permissions::from_mode(0o000)).unwrap();

        let found = listed(root.path(), "beef");

        std::fs::set_permissions(&ledger_at, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            found, None,
            "unreadable, so nothing was found rather than trusted"
        );
    }

    #[test]
    fn a_ledger_entry_whose_file_is_gone_is_not_trusted_blindly() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("informe.pdf", b"the only copy of the report");

        let first = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        std::fs::remove_file(root.path().join(&first.at)).unwrap();

        let second = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert!(
            root.path().join(&second.at).is_file(),
            "keep() handed back a path with nothing behind it: {}",
            second.at
        );
        assert_eq!(
            std::fs::read(root.path().join(&second.at)).unwrap(),
            b"the only copy of the report"
        );
    }

    #[test]
    fn when_the_ledger_repeats_a_hash_the_first_line_written_is_the_one_trusted() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("attachments.jsonl"),
            "{\"at\":\"attachments/ab/first.bin\",\"sha256\":\"dupe\",\"bytes\":1}\n{\"at\":\"attachments/ab/second.bin\",\"sha256\":\"dupe\",\"bytes\":1}\n",
        )
        .unwrap();

        assert_eq!(
            listed(root.path(), "dupe"),
            Some("attachments/ab/first.bin".to_string())
        );
    }

    #[test]
    fn a_name_already_taken_by_other_content_gets_a_longer_stamp_instead_of_being_overwritten() {
        let root = tempfile::tempdir().unwrap();
        let bytes = b"the real bytes of the real file";
        let sha256 = fingerprint(bytes);
        let (shelf, rest) = sha256.split_at(2);
        let stamp = &rest[..8];
        let folder = root.path().join("attachments").join(shelf);
        std::fs::create_dir_all(&folder).unwrap();
        let squatted = folder.join(format!("clash-{stamp}.bin"));
        std::fs::write(&squatted, b"unrelated content squatting the name").unwrap();

        let (_src, file) = dropped("clash.bin", bytes);
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert_ne!(
            kept.at.rsplit('/').next().unwrap(),
            format!("clash-{stamp}.bin"),
            "the real file was written under the squatter's name"
        );
        assert_eq!(
            std::fs::read(&squatted).unwrap(),
            b"unrelated content squatting the name",
            "the squatter was clobbered"
        );
        assert_eq!(std::fs::read(root.path().join(&kept.at)).unwrap(), bytes);
    }

    #[test]
    fn a_file_one_byte_over_the_limit_is_refused() {
        let (_src, file) = dropped("shot.png", b"12345");
        let root = tempfile::tempdir().unwrap();

        let refused = keep(&file, root.path(), 4).unwrap_err();

        assert!(
            matches!(refused, Error::AttachmentTooBig { bytes: 5, limit: 4 }),
            "{refused:?}"
        );
    }

    #[test]
    fn a_name_made_only_of_dots_still_lands_as_a_normal_file() {
        let kept = stored("...png", b"g");

        assert!(!kept.starts_with('.'), "{kept}");
        assert!(kept.ends_with(".png"), "{kept}");
    }

    #[test]
    fn a_name_of_pure_emoji_falls_back_to_the_stamp_instead_of_writing_pictographs_to_disk() {
        let kept = stored("😀😀.png", b"h");

        assert!(kept.is_ascii(), "{kept}");
        assert!(kept.ends_with(".png"), "{kept}");
    }

    #[test]
    fn a_name_three_hundred_characters_long_is_still_cut_to_the_limit() {
        let source = Path::new("").join(format!("{}.txt", "x".repeat(300)));

        let named = named(&source, "12345678", ".txt");

        assert_eq!(named, format!("{}-12345678.txt", "x".repeat(SHORTENS_TO)));
    }

    #[test]
    fn a_ledger_entry_is_not_trusted_when_it_climbs_out_of_the_store() {
        let outside = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("informe.pdf", b"the only copy of my report");

        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        std::fs::remove_file(root.path().join(&kept.at)).unwrap();

        let planted = outside.path().join("not-an-attachment.pdf");
        std::fs::write(&planted, b"something else entirely").unwrap();
        let climbing = format!(
            "../{}/not-an-attachment.pdf",
            outside.path().file_name().unwrap().to_str().unwrap()
        );
        std::fs::write(
            root.path().join("attachments.jsonl"),
            format!(
                "{{\"at\":\"{climbing}\",\"sha256\":\"{}\",\"bytes\":7}}\n",
                kept.sha256
            ),
        )
        .unwrap();

        let again = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert!(
            !again.at.contains(".."),
            "a corrupted ledger line handed back a path that climbs out of the store: {}",
            again.at
        );
    }
}
