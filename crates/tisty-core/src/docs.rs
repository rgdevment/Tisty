use std::path::{Path, PathBuf};

use std::io::Read;

use crate::{Error, Result, event::DeviceId, store::write_atomic};

mod cards;
mod carried;
mod prints;
mod text;

pub use cards::{Card, Gist, card_of, cards_of, forget_stray_cards, sighted};
use carried::kept_still;
pub use carried::{
    Carried, before_left_at, carried_at, carried_print, forget_carried, forget_what_was_carried,
    keep_carried, kept_before, print_of, read_before, read_carried,
};
pub use prints::Prints;
use text::{Fencing, as_written, bullet, quoted, quoteless, unpictured, unspanned, wordless};
pub use text::{
    Heading, ends_fenced, fencing, headings, lines_between, marked, outlined, section_lines,
    settled, spelled, titled,
};
pub(crate) use text::{fenced_spans, nameless};

const EXTENSION: &str = "md";
const DIGITS: usize = 4;
const MOST_DIGITS: u64 = 999_999_999_999;
const TITLE_AT_MOST: u64 = 4 * 1024;
pub const BODY_AT_MOST: u64 = 500 * 1024;
pub const BODY_ROOMY: u64 = 300 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Doc {
    pub id: String,
    pub title: String,
}

fn base(data: &Path) -> PathBuf {
    data.join("carried")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Nothing,
    Bring,
    Send,
    TheyDecide,
}

pub fn moved(base: Option<&str>, here: Option<&str>, there: Option<&str>) -> Move {
    match (here, there) {
        (None, None) => Move::Nothing,
        (Some(_), None) => Move::Send,
        (None, Some(_)) => Move::Bring,
        (Some(here), Some(there)) if here == there => Move::Nothing,
        (Some(here), Some(there)) => match base {
            Some(base) if base == here => Move::Bring,
            Some(base) if base == there => Move::Send,
            _ => Move::TheyDecide,
        },
    }
}

pub fn create(root: &Path, device: &DeviceId, body: &str) -> Result<Doc> {
    std::fs::create_dir_all(root)?;
    let _ = crate::paths::ours_alone(root);
    let mut number = next(root, device);
    loop {
        if number > MOST_DIGITS {
            return Err(Error::OutsideTheStore(format!("{}-{number}", stem(device))));
        }
        let id = format!("{}-{number:0width$}", stem(device), width = DIGITS);
        let at = resolve(root, &id)?;
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&at)
        {
            Ok(_) => {
                // `create_new` already won this name against everyone, and taking the lock here
                // would queue creations that never contend for the same body.
                if let Err(e) = written(root, &id, body) {
                    // The name was won before the body was weighed; an empty file must not outlive
                    // the refusal, and the number is spent anyway because it was handed out once.
                    let _ = std::fs::remove_file(&at);
                    spend(root, device, number);
                    return Err(e);
                }
                spend(root, device, number);
                return Ok(Doc {
                    title: titled(body),
                    id,
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => number += 1,
            Err(e) => return Err(Error::Io(e)),
        }
    }
}

/// The editor hands back what it loaded with its own line endings and without the last newline,
/// and neither is a change somebody made.
pub fn unchanged(was: &str, now: &str) -> bool {
    let plain = |said: &str| settled(&said.replace("\r\n", "\n"));
    plain(was) == plain(now)
}

/// One door for every writer: a body read by one is never written under another.
pub fn write(root: &Path, id: &str, body: &str) -> Result<()> {
    alone(root, || written(root, id, body))
}

fn written(root: &Path, id: &str, body: &str) -> Result<()> {
    let whole = settled(body);
    let bytes = whole.len() as u64;
    if bytes > BODY_AT_MOST {
        return Err(Error::DocumentTooBig {
            bytes,
            limit: BODY_AT_MOST,
        });
    }
    let at = resolve(root, id)?;
    std::fs::create_dir_all(root)?;
    let _ = crate::paths::ours_alone(root);
    write_atomic(&at, whole.as_bytes())
}

/// What happened when a book was asked to name pages at its end. One answer for every door, so
/// the window and the assistant cannot come to differ about when a line is written.
#[derive(Debug, PartialEq, Eq)]
pub enum Naming {
    Wrote { named: Vec<String>, whole: String },
    Fenced,
    WouldRename,
    Nothing,
}

pub fn name_at_end(root: &Path, data: &Path, parent: &str, which: &[&str]) -> Result<Naming> {
    alone(root, || {
        let body = read(root, parent)?;
        let told: Vec<String> = crate::refs::paper_lines(&body)
            .into_iter()
            .map(|(one, _)| one)
            .collect();
        let mut cards: Vec<String> = Vec::new();
        let mut named: Vec<String> = Vec::new();
        for one in which {
            if told.iter().any(|said| said == one) || named.iter().any(|said| said == one) {
                continue;
            }
            // The lock is held over this and all that is wanted is a name: the opening few
            // thousand bytes hold it, and a chapter can run to half a megabyte.
            let title = resolve(root, one)
                .map(|at| opening(&at))
                .unwrap_or_default();
            cards.push(crate::refs::card(one, &title));
            named.push((*one).to_string());
        }
        if cards.is_empty() {
            return Ok(Naming::Nothing);
        }
        if ends_fenced(&body) {
            return Ok(Naming::Fenced);
        }
        let whole = named_after(&body, &cards);
        if titled(&body) != titled(&whole) {
            return Ok(Naming::WouldRename);
        }
        written(root, parent, &whole)?;
        kept_still(data, parent, &body, &whole)?;
        Ok(Naming::Wrote {
            named,
            whole: settled(&whole),
        })
    })
}

fn named_after(body: &str, cards: &[String]) -> String {
    let ending = match body.contains("\r\n") {
        true => "\r\n",
        false => "\n",
    };
    let mut lines: Vec<String> = body.lines().map(str::to_string).collect();
    while lines.last().is_some_and(|one| one.trim().is_empty()) {
        lines.pop();
    }
    for card in cards {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(card.clone());
    }
    let mut out = lines.join(ending);
    if !out.ends_with('\n') {
        out.push_str(ending);
    }
    out
}

pub fn append(root: &Path, id: &str, body: &str) -> Result<String> {
    alone(root, || {
        let was = read(root, id)?;
        let ending = match was.contains("\r\n") {
            true => "\r\n",
            false => "\n",
        };
        let flat = settled(body.trim_start_matches(['\n', '\r'])).replace("\r\n", "\n");
        let added = match ending {
            "\r\n" => flat.replace('\n', "\r\n"),
            _ => flat,
        };
        let whole = match was.trim_end().is_empty() {
            true => added,
            false => format!("{}{ending}{ending}{added}", was.trim_end()),
        };
        written(root, id, &whole)?;
        Ok(whole)
    })
}

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    Made { was: String, whole: String },
    Missing,
    Twice(usize),
    TheLot,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Rewrite {
    Made { was: String, whole: String },
    Moved,
}

/// What it said before is kept first: a rewrite that cannot be undone is not written at all.
pub fn rewrite(root: &Path, data: &Path, id: &str, body: &str, print: &str) -> Result<Rewrite> {
    alone(root, || {
        let at = resolve(root, id)?;
        if print_of(&at)?.as_deref() != Some(print) {
            return Ok(Rewrite::Moved);
        }
        let was = read(root, id)?;
        kept_before(data, id, &was, body)?;
        written(root, id, body)?;
        Ok(Rewrite::Made {
            was,
            whole: settled(body),
        })
    })
}

pub fn edit(root: &Path, data: &Path, id: &str, old: &str, new: &str) -> Result<Change> {
    if old.is_empty() {
        return Ok(Change::Missing);
    }
    alone(root, || {
        let was = read(root, id)?;
        let (old, new) = as_written(&was, old, new);
        // A rewrite wearing an edit's clothes: no tool hands a document a new body.
        if was.trim() == old.trim() {
            return Ok(Change::TheLot);
        }
        match was.matches(old.as_str()).count() {
            0 => Ok(Change::Missing),
            1 => {
                let whole = was.replacen(old.as_str(), new.as_str(), 1);
                kept_before(data, id, &was, &whole)?;
                written(root, id, &whole)?;
                Ok(Change::Made { was, whole })
            }
            many => Ok(Change::Twice(many)),
        }
    })
}

/// Work out the new body under the same lock that writes it, so nothing slips in between.
pub fn amend(
    root: &Path,
    data: &Path,
    id: &str,
    make: impl FnOnce(&str) -> Option<String>,
) -> Result<Option<String>> {
    alone(root, || {
        let was = read(root, id)?;
        let Some(whole) = make(&was) else {
            return Ok(None);
        };
        kept_before(data, id, &was, &whole)?;
        written(root, id, &whole)?;
        Ok(Some(settled(&whole)))
    })
}

pub const SUMMARY_AT_MOST: usize = 2_000;
pub const NOTES_AT_MOST: usize = 4_000;

const LOCK: &str = ".lock";
/// Waiting beats refusing: the writers that queue here are a saving editor, a sync round and an
/// agent, and every one of them holds it for a write, not for a session.
const LOCK_WAIT_MS: u64 = 2_000;
const LOCK_POLL_MS: u64 = 5;

pub struct Alone(std::fs::File);

impl Drop for Alone {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

/// For a writer that would rather carry on unheld than not write at all.
pub fn hold(root: &Path) -> Option<Alone> {
    std::fs::create_dir_all(root).ok()?;

    let mut waited = 0;
    loop {
        let taken = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(root.join(LOCK))
            .ok()
            .filter(|file| file.try_lock().is_ok());
        if let Some(file) = taken {
            return Some(Alone(file));
        }
        if waited >= LOCK_WAIT_MS {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(LOCK_POLL_MS));
        waited += LOCK_POLL_MS;
    }
}

fn alone<T>(root: &Path, work: impl FnOnce() -> Result<T>) -> Result<T> {
    let held = hold(root).ok_or(Error::AlreadyRunning)?;
    let out = work();
    drop(held);
    out
}

/// What a watcher compares to tell that a body moved: a body writes no event to notice it by.
pub fn print(root: &Path) -> String {
    let Ok(entries) = std::fs::read_dir(root) else {
        return String::new();
    };
    let mut parts: Vec<String> = entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .filter_map(|one| {
            let at = one.path();
            let id = named(&at)?;
            let told = one.metadata().ok()?;
            let when = told
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?;
            Some(format!("{id}:{}:{}", told.len(), when.as_nanos()))
        })
        .collect();
    parts.sort();
    parts.join("|")
}

pub const IMPORTS: [&str; 3] = ["md", "markdown", "txt"];

pub fn importable(at: &Path) -> bool {
    match at.extension().and_then(|one| one.to_str()) {
        None => at.file_name().is_some(),
        Some(one) => IMPORTS.contains(&one.to_lowercase().as_str()),
    }
}

pub fn read_outside(at: &Path) -> Result<String> {
    if !importable(at) {
        return Err(Error::OutsideTheStore(at.display().to_string()));
    }
    let file = std::fs::File::open(at)?;
    crate::counting::opened();
    if !file.metadata()?.is_file() {
        return Err(Error::OutsideTheStore(at.display().to_string()));
    }
    let mut body = String::new();
    let read = file.take(BODY_AT_MOST + 1).read_to_string(&mut body)? as u64;
    if read > BODY_AT_MOST {
        return Err(Error::DocumentTooBig {
            bytes: read,
            limit: BODY_AT_MOST,
        });
    }
    Ok(body)
}

pub fn read(root: &Path, id: &str) -> Result<String> {
    let at = resolve(root, id)?;
    let file = std::fs::File::open(&at)?;
    crate::counting::opened();
    if !file.metadata()?.is_file() {
        return Err(Error::OutsideTheStore(id.to_string()));
    }
    let mut body = String::new();
    let read = file.take(BODY_AT_MOST + 1).read_to_string(&mut body)? as u64;
    if read > BODY_AT_MOST {
        return Err(Error::DocumentTooBig {
            bytes: read,
            limit: BODY_AT_MOST,
        });
    }
    Ok(body)
}

/// What came out, and what could not: a page missing from disk is left behind, and saying so
/// is the only way the person learns their book came out a chapter short.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Taken {
    pub files: usize,
    pub missed: usize,
    pub left: Vec<String>,
}

pub fn exported(data: &Path, id: &str, into: &Path) -> Result<Taken> {
    with_pages(data, id, &[], into, None)
}

/// The pages travel with the document: a book exported by its cover alone is not the book.
pub fn with_pages(
    data: &Path,
    id: &str,
    pages: &[String],
    into: &Path,
    also: Option<&Path>,
) -> Result<Taken> {
    laid_out_as(data, id, pages, into, None, also)
}

pub fn laid_out_as(
    data: &Path,
    id: &str,
    pages: &[String],
    into: &Path,
    called: Option<&str>,
    also: Option<&Path>,
) -> Result<Taken> {
    if into.starts_with(data) || data.starts_with(into) {
        return Err(Error::OutsideTheStore(into.display().to_string()));
    }
    let body = read(&data.join("docs"), id)?;

    let named = match called {
        Some(one) => one.to_string(),
        None => {
            let named = titled(&body);
            spelled(if named.is_empty() { id } else { &named })
        }
    };
    let folder = into.join(&named);
    std::fs::create_dir_all(into)?;
    if folder.exists() {
        return Err(Error::AlreadyTakenOut(named.clone()));
    }
    std::fs::create_dir(&folder)?;

    let wide = pages.len().to_string().len().max(2);
    let mut missed = 0;
    let mut written: Vec<(String, String, String)> = Vec::new();
    for (n, page) in pages.iter().enumerate() {
        let Ok(body) = read(&data.join("docs"), page) else {
            missed += 1;
            continue;
        };
        let title = titled(&body);
        let title = spelled(if title.is_empty() { page } else { &title });
        let at = format!("{:0wide$} {title}.{EXTENSION}", n + 1);
        written.push((page.clone(), at, body));
    }

    let beside = |body: &str| {
        written
            .iter()
            .fold(body.to_string(), |body, (file, at, _)| {
                let named = format!("{}{file}", crate::refs::DOC);
                unpictured(
                    &body
                        .replace(&format!("](<{named}>)"), &format!("](<{at}>)"))
                        .replace(&format!("]({named})"), &format!("](<{at}>)")),
                    at,
                )
            })
    };

    let mut taken = laid_out(
        data,
        &beside(&body),
        &folder,
        &format!("{named}.{EXTENSION}"),
        also,
    )?;
    for (_, at, body) in &written {
        let more = laid_out(data, &beside(body), &folder, at, also)?;
        taken.files += more.files;
        for one in more.left {
            left_behind(&mut taken.left, one);
        }
    }
    Ok(Taken {
        files: taken.files,
        missed,
        left: taken.left,
    })
}

fn left_behind(left: &mut Vec<String>, one: String) {
    if !left.contains(&one) {
        left.push(one);
    }
}

fn shelved<'a>(from: &'a Path, held: &Path, also: Option<&Path>) -> Option<&'a Path> {
    from.strip_prefix(held)
        .ok()
        .or_else(|| also.and_then(|beside| from.strip_prefix(beside.join("attachments")).ok()))
}

fn laid_out(
    data: &Path,
    body: &str,
    folder: &Path,
    named: &str,
    also: Option<&Path>,
) -> Result<Taken> {
    write_atomic(&folder.join(named), body.as_bytes())?;

    let held = data.join("attachments");
    let mut taken = Taken::default();
    for one in crate::refs::extract(body).into_iter().map(|one| one.target) {
        if !one.starts_with("attachments/") {
            continue;
        }
        let Ok(from) = crate::attach::found(&one, data, also) else {
            left_behind(&mut taken.left, one);
            continue;
        };
        let Some(rest) = shelved(&from, &held, also) else {
            continue;
        };
        if !from.is_file() {
            left_behind(&mut taken.left, one);
            continue;
        }
        let at = folder.join("attachments").join(rest);
        if let Some(under) = at.parent() {
            std::fs::create_dir_all(under)?;
        }
        if std::fs::copy(&from, &at).is_ok() {
            taken.files += 1;
        } else {
            left_behind(&mut taken.left, one);
        }
    }
    Ok(taken)
}

pub fn referenced(root: &Path) -> Vec<String> {
    all(root)
        .iter()
        .filter_map(|doc| match read(root, &doc.id) {
            Ok(body) => Some(body),
            Err(e) => {
                crate::witness::warn(
                    crate::witness::channel::ATTACH,
                    "a document could not be read while counting what is still named",
                    &[
                        ("id", crate::witness::Fact::Id(doc.id.clone())),
                        ("why", crate::witness::Fact::Why(e.to_string())),
                    ],
                );
                None
            }
        })
        .flat_map(|body| crate::refs::extract(&body))
        .map(|one| one.target)
        .collect()
}

pub fn remove(root: &Path, id: &str) -> Result<()> {
    let at = resolve(root, id)?;
    match std::fs::remove_file(at) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::Io(e)),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stray {
    pub file: String,
    pub title: String,
    pub bytes: u64,
    pub when: i64,
}

pub fn strayed(root: &Path, alive: &[String]) -> Vec<Stray> {
    loose(root, alive)
        .into_iter()
        .map(|one| {
            let at = resolve(root, &one.id).ok();
            let told = at.as_ref().and_then(|at| std::fs::metadata(at).ok());
            Stray {
                title: one.title,
                file: one.id,
                bytes: told.as_ref().map(|one| one.len()).unwrap_or(0),
                when: told
                    .and_then(|one| one.modified().ok())
                    .and_then(|one| one.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|one| one.as_secs() as i64)
                    .unwrap_or(0),
            }
        })
        .collect()
}

pub fn missing(root: &Path, alive: &[String]) -> Vec<String> {
    alive
        .iter()
        .filter(|id| !resolve(root, id).is_ok_and(|at| at.exists()))
        .cloned()
        .collect()
}

pub fn loose(root: &Path, alive: &[String]) -> Vec<Doc> {
    let held: std::collections::BTreeSet<&str> = alive.iter().map(String::as_str).collect();
    all(root)
        .into_iter()
        .filter(|one| !held.contains(one.id.as_str()))
        .collect()
}

pub fn sweep(root: &Path, shed: &std::collections::BTreeSet<String>) -> usize {
    let mut gone = 0;
    for id in shed {
        let Ok(at) = resolve(root, id) else {
            continue;
        };
        match std::fs::remove_file(&at) {
            Ok(()) => gone += 1,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
            Err(why) => crate::witness::warn(
                crate::witness::channel::STORE,
                "a deleted document kept its file, and it is swept again at the next opening",
                &[
                    ("at", crate::witness::Fact::Id(id.clone())),
                    ("why", crate::witness::Fact::Why(why.to_string())),
                ],
            ),
        }
        forget_carried(root.parent().unwrap_or(root), id);
    }
    if gone > 0 {
        crate::witness::note(
            crate::witness::channel::STORE,
            "documents deleted elsewhere had their files taken out here",
            &[("count", crate::witness::Fact::Count(gone))],
        );
    }
    gone
}

pub fn names(root: &Path) -> std::collections::BTreeSet<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return std::collections::BTreeSet::new();
    };
    entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .filter_map(|one| named(&one.path()))
        .collect()
}

pub fn title_of(root: &Path, id: &str) -> Option<String> {
    let at = resolve(root, id).ok()?;
    at.is_file().then(|| opening(&at))
}

pub fn all(root: &Path) -> Vec<Doc> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut found: Vec<Doc> = entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .filter_map(|one| {
            let at = one.path();
            let id = named(&at)?;
            Some(Doc {
                title: opening(&at),
                id,
            })
        })
        .collect();
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

pub fn resolve(root: &Path, id: &str) -> Result<PathBuf> {
    if !well_formed(id) {
        return Err(Error::OutsideTheStore(id.to_string()));
    }
    Ok(root.join(format!("{id}.{EXTENSION}")))
}

fn well_formed(id: &str) -> bool {
    let Some((device, number)) = id.rsplit_once('-') else {
        return false;
    };
    crate::store::is_device_name(device)
        && !number.is_empty()
        && number.len() <= 12
        && number.chars().all(|c| c.is_ascii_digit())
}

pub fn a_body(leaf: &str) -> bool {
    named(Path::new(leaf)).is_some()
}

fn named(at: &Path) -> Option<String> {
    if at.extension()? != EXTENSION {
        return None;
    }
    let id = at.file_stem()?.to_str()?.to_string();
    well_formed(&id).then_some(id)
}

fn spent(root: &Path, device: &DeviceId) -> PathBuf {
    root.join(format!(".spent-{}", stem(device)))
}

fn spend(root: &Path, device: &DeviceId, number: u64) {
    let at = spent(root, device);
    if let Err(e) = write_atomic(&at, number.to_string().as_bytes()) {
        crate::witness::warn(
            crate::witness::channel::STORE,
            "the highest document name given out could not be kept, so a deleted one could come back",
            &[("why", crate::witness::Fact::Why(e.to_string()))],
        );
    }
}

fn next(root: &Path, device: &DeviceId) -> u64 {
    let mine = format!("{}-", stem(device));
    let on_disk = all(root)
        .iter()
        .filter_map(|doc| doc.id.strip_prefix(&mine))
        .filter_map(|number| number.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    let given = std::fs::read_to_string(spent(root, device))
        .ok()
        .and_then(|said| said.trim().parse::<u64>().ok())
        .unwrap_or(0);
    on_disk.max(given) + 1
}

fn stem(device: &DeviceId) -> String {
    let plain = device.0.strip_prefix("dev_").unwrap_or(&device.0);
    let kept: String = plain
        .chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(48)
        .collect();
    if kept.is_empty() {
        "device".to_string()
    } else {
        kept
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sighting {
    pub id: String,
    pub title: String,
    pub line: String,
}

pub const SAID_AT_MOST: usize = 160;

fn skipped(chars: &mut std::iter::Peekable<std::str::Chars>, opens: char, shuts: char) {
    let mut depth = 1;
    for one in chars.by_ref() {
        if one == opens {
            depth += 1;
        } else if one == shuts {
            depth -= 1;
            if depth == 0 {
                return;
            }
        }
    }
}

pub fn bare(line: &str) -> String {
    let line = &unspanned(line);
    let flat = line.trim();
    let quoted = flat.starts_with('>');
    let said = flat
        .trim_start_matches(['>', '#', ' '])
        .trim_start()
        .trim_start_matches(['-', '*', '+'])
        .trim_start();
    let said = said
        .strip_prefix("[ ] ")
        .or(said.strip_prefix("[x] "))
        .unwrap_or(said);
    let said = match quoted {
        true => crate::refs::alerted(said).unwrap_or(said),
        false => said,
    };

    let mut out = String::with_capacity(said.len());
    let mut chars = said.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.extend(chars.next()),
            '`' | '*' => {}
            '~' if chars.peek() == Some(&'~') => {
                chars.next();
            }
            '!' if chars.peek() == Some(&'[') => {
                chars.next();
            }
            '[' => {}
            ']' => match chars.peek() {
                Some('(') => {
                    chars.next();
                    skipped(&mut chars, '(', ')');
                }
                Some('[') => {
                    chars.next();
                    skipped(&mut chars, '[', ']');
                }
                _ => {}
            },
            '|' => out.push(' '),
            _ => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn bared(body: &str) -> String {
    body.lines().map(bare).collect::<Vec<_>>().join("\n")
}

fn cut(said: String) -> String {
    said.chars().take(SAID_AT_MOST).collect()
}

fn shown_around(body: &str, terms: &[String]) -> Option<String> {
    let mut best: Option<(usize, String)> = None;
    let mut backup: Option<(usize, String)> = None;
    let mut fence = Fencing::default();

    for line in body.lines() {
        let drawn = fence.inside(line);
        let said = bare(line);
        let flat = crate::text::folded(&said);
        let held = terms.iter().filter(|term| flat.contains(*term)).count();
        if held == 0 {
            continue;
        }
        let slot = if drawn || wordless(&said) {
            &mut backup
        } else {
            &mut best
        };
        if slot.as_ref().is_none_or(|(had, _)| held > *had) {
            *slot = Some((held, said));
        }
        if best.as_ref().is_some_and(|(had, _)| *had == terms.len()) {
            break;
        }
    }
    best.or(backup).map(|(_, said)| cut(said))
}

pub const CORPUS_AT_MOST: usize = 64 * 1024 * 1024;

struct Held {
    stamp: (u64, u64),
    flat: String,
}

pub struct Corpus {
    kept: std::collections::HashMap<String, Held>,
    bytes: usize,
    room: usize,
}

impl Default for Corpus {
    fn default() -> Self {
        Self::holding(CORPUS_AT_MOST)
    }
}

fn stamped(at: &Path) -> Option<(u64, u64)> {
    let told = std::fs::metadata(at).ok()?;
    let when = told
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?;
    Some((told.len(), when.as_nanos() as u64))
}

impl Corpus {
    pub fn holding(room: usize) -> Self {
        Self {
            kept: std::collections::HashMap::new(),
            bytes: 0,
            room,
        }
    }

    pub fn forget(&mut self, id: &str) {
        if let Some(gone) = self.kept.remove(id) {
            self.bytes -= gone.flat.len();
        }
    }

    pub fn held(&self) -> usize {
        self.bytes
    }

    fn flattened(&mut self, root: &Path, id: &str) -> Option<&str> {
        let at = resolve(root, id).ok()?;
        let stamp = stamped(&at)?;
        if !self.kept.get(id).is_some_and(|one| one.stamp == stamp) {
            self.forget(id);
            let flat = crate::text::folded(&bared(&read(root, id).ok()?));
            if self.bytes + flat.len() > self.room {
                return None;
            }
            self.bytes += flat.len();
            self.kept.insert(id.to_string(), Held { stamp, flat });
        }
        self.kept.get(id).map(|one| one.flat.as_str())
    }

    pub fn searching(
        &mut self,
        root: &Path,
        query: &str,
        most: usize,
        wanted: impl Fn(&str) -> bool,
    ) -> Vec<Sighting> {
        let terms = crate::text::terms(query);
        if terms.is_empty() {
            return Vec::new();
        }

        let mut found = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for doc in all(root) {
            seen.insert(doc.id.clone());
            if found.len() >= most || !wanted(&doc.id) {
                continue;
            }
            let title = doc.title.clone();
            let flat = crate::text::folded(&title);
            let missing: Vec<&String> = terms
                .iter()
                .filter(|term| !flat.contains(term.as_str()))
                .collect();
            if missing.is_empty() {
                found.push(Sighting {
                    id: doc.id,
                    title,
                    line: String::new(),
                });
                continue;
            }
            let sighted = match self.flattened(root, &doc.id) {
                Some(flat) => missing.iter().all(|term| flat.contains(term.as_str())),
                None => read(root, &doc.id)
                    .map(|body| {
                        let flat = crate::text::folded(&bared(&body));
                        missing.iter().all(|term| flat.contains(term.as_str()))
                    })
                    .unwrap_or(false),
            };
            if !sighted {
                continue;
            }
            let Ok(body) = read(root, &doc.id) else {
                continue;
            };
            if let Some(line) = shown_around(&body, &terms) {
                found.push(Sighting {
                    id: doc.id,
                    title,
                    line,
                });
            }
        }
        self.kept.retain(|id, _| seen.contains(id));
        self.bytes = self.kept.values().map(|one| one.flat.len()).sum();
        found
    }
}

fn opening(at: &Path) -> String {
    let Ok(file) = std::fs::File::open(at) else {
        return String::new();
    };
    let mut head = Vec::new();
    let _ = file.take(TITLE_AT_MOST).read_to_end(&mut head);
    titled(&String::from_utf8_lossy(&head))
}

#[cfg(test)]
#[path = "docs_naming.rs"]
mod naming;

#[cfg(test)]
#[path = "docs_test.rs"]
mod tests;

/// What the window's editor destroys the first time somebody opens a document. It rewrites the
/// whole file, so anything it cannot represent is gone on the first keystroke.
fn ruled(rest: &str) -> bool {
    let Some(mark) = rest.chars().find(|c| !c.is_whitespace()) else {
        return false;
    };
    if !matches!(mark, '*' | '_' | '-') {
        return false;
    }
    let mut many = 0;
    for one in rest.chars() {
        if one == mark {
            many += 1;
        } else if one != ' ' && one != '\t' {
            return false;
        }
    }
    many >= 3
}

fn dashed(line: &str) -> bool {
    let said = quoteless(line);
    said.starts_with('|')
        && said.contains('-')
        && said.chars().all(|one| matches!(one, '|' | '-' | ':' | ' '))
}

fn blocked(line: &str, next: &str) -> bool {
    let (_, wide, said) = quoted(line);
    if wide >= 4 || ruled(said) {
        return false;
    }
    let Some(after) = bullet(said) else {
        return false;
    };
    let mark = said[..after].trim_end().len();
    let gap: usize = said[mark..after]
        .chars()
        .map(|one| if one == '\t' { 4 } else { 1 })
        .sum();
    if gap >= 5 {
        return true;
    }
    let rest = &said[after..];
    if rest.starts_with('>')
        || rest.starts_with("```")
        || rest.starts_with("~~~")
        || rest.starts_with("![")
        || bullet(rest).is_some()
        || ruled(rest)
    {
        return true;
    }
    let hashed = rest.chars().take_while(|c| *c == '#').count();
    let after_hash = &rest[hashed..];
    if (1..=6).contains(&hashed) && (after_hash.is_empty() || after_hash.starts_with([' ', '\t'])) {
        return true;
    }
    rest.starts_with('|') && dashed(next)
}

pub fn survives(body: &str) -> std::result::Result<(), &'static str> {
    let body = body.trim_start_matches('\u{feff}');
    if fronted(body) {
        return Err("YAML frontmatter");
    }
    let mut fence = Fencing::default();

    let said: Vec<&str> = body.lines().collect();
    for (at, line) in said.iter().enumerate() {
        let coded = fence.open.is_some();
        let within = fence.inside(line);
        if fence.told {
            return Err("what a fence says after its language");
        }
        if !coded && blocked(line, said.get(at + 1).unwrap_or(&"")) {
            return Err("a list item that opens on a block");
        }
        if within {
            continue;
        }
        if line.starts_with("    ") || line.starts_with('\t') {
            continue;
        }

        let plain = outside_code_spans(line);
        if let Some(why) = markup(&plain) {
            return Err(why);
        }
        let flat = plain.trim();
        if dollared(flat) {
            return Err("maths written between dollars");
        }
        if noted(flat) {
            return Err("footnotes");
        }
        if linked(flat, said.get(at + 1).unwrap_or(&"")) {
            return Err("reference links");
        }
    }
    Ok(())
}

fn fronted(body: &str) -> bool {
    let mut lines = body.trim_start_matches('\u{feff}').lines();
    if lines.next().map(str::trim) != Some("---") {
        return false;
    }
    lines.any(|line| line.trim() == "---")
}

fn outside_code_spans(line: &str) -> String {
    let bytes = line.as_bytes();
    let ticks = |from: usize| bytes[from..].iter().take_while(|c| **c == b'`').count();
    let mut out = String::with_capacity(line.len());
    let mut at = 0;

    while at < bytes.len() {
        if bytes[at] != b'`' {
            let one = line[at..].chars().next().unwrap_or('\0');
            out.push(one);
            at += one.len_utf8();
            continue;
        }
        let open = ticks(at);
        let mut scan = at + open;
        let mut shut = None;
        while scan < bytes.len() {
            if bytes[scan] != b'`' {
                scan += 1;
                continue;
            }
            let many = ticks(scan);
            if many == open {
                shut = Some(scan + many);
                break;
            }
            scan += many;
        }
        match shut {
            Some(end) => at = end,
            None => {
                out.push_str(&line[at..at + open]);
                at += open;
            }
        }
    }
    out
}

fn dollared(line: &str) -> bool {
    let said = line.trim();
    if said.starts_with("$$") {
        return true;
    }
    let bytes = said.as_bytes();
    let mut at = 0;
    while let Some(found) = said[at..].find("$$") {
        let start = at + found;
        if start == 0 || bytes[start - 1] != b'\\' {
            return true;
        }
        at = start + 2;
    }
    false
}

fn noted(line: &str) -> bool {
    let bytes = line.as_bytes();
    let mut at = 0;
    while let Some(found) = line[at..].find("[^") {
        let start = at + found;
        at = start + 2;
        if start > 0 && bytes[start - 1] == b'\\' {
            continue;
        }
        let Some(end) = line[at..].find(']') else {
            return false;
        };
        if end > 0 && !matches!(bytes.get(at + end + 1), Some(b'(') | Some(b'[')) {
            return true;
        }
        at += end + 1;
    }
    false
}

fn labelled(rest: &str) -> Option<(&str, &str)> {
    let bytes = rest.as_bytes();
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'\\' => at += 2,
            b']' if bytes.get(at + 1) == Some(&b':') => {
                return Some((&rest[..at], &rest[at + 2..]));
            }
            b']' => return None,
            _ => at += 1,
        }
    }
    None
}

fn linked(line: &str, next: &str) -> bool {
    let Some(rest) = line.strip_prefix('[') else {
        return false;
    };
    let Some((label, told)) = labelled(rest) else {
        return false;
    };
    !label.is_empty()
        && !label.starts_with('^')
        && (!told.trim().is_empty() || !next.trim().is_empty())
}

/// A `<` that opens a tag, wherever it sits on the line. An autolink is markdown and comes
/// back untouched, so it is not markup.
pub(crate) fn markup(line: &str) -> Option<&'static str> {
    let bytes = line.as_bytes();
    let mut shut = 0;
    for at in 0..bytes.len() {
        if bytes[at] == b'&' && entity(&line[at..]) {
            return Some("HTML entities");
        }
        if bytes[at] != b'<' {
            continue;
        }
        let rest = &line[at + 1..];
        if rest.starts_with("!--") {
            return Some("HTML comments");
        }
        if !rest.starts_with(|c: char| c.is_ascii_alphabetic() || c == '/' || c == '!' || c == '?')
        {
            continue;
        }
        if shut <= at {
            shut = match rest.find('>') {
                Some(end) => at + 1 + end,
                None => line.len(),
            };
        }
        let inner = &line[at + 1..shut];
        if inner.contains("://") || (inner.contains('@') && !inner.contains(' ')) {
            continue;
        }
        // `](<…>)` is where markdown puts a target that has spaces or brackets in it, and it is
        // what an attachment is written as.
        if line[..at].ends_with("](") && !inner.contains('<') && anchored(&line[..at - 2]) {
            continue;
        }
        if kept(inner) {
            continue;
        }
        return Some("HTML");
    }
    None
}

fn quotedly<'a>(said: &'a str, name: &str, plain: fn(char) -> bool) -> Option<&'a str> {
    let rest = said.strip_prefix(name)?.strip_prefix("=\"")?;
    let (value, after) = rest.split_once('"')?;
    if value.is_empty() || !value.chars().all(plain) {
        return None;
    }
    Some(after.trim_start())
}

fn named_or_marked(said: &str) -> Option<&str> {
    let rest = said.strip_prefix("data-ico")?.strip_prefix("=\"")?;
    let (value, after) = rest.split_once('"')?;
    if value.is_empty() {
        return None;
    }
    let plain = value
        .chars()
        .all(|one| one.is_ascii_alphanumeric() || one == '-');
    (plain || crate::model::icon::a_mark(value)).then(|| after.trim_start())
}

fn iconed(inner: &str) -> bool {
    let Some(rest) = inner
        .get(..5)
        .filter(|one| one.eq_ignore_ascii_case("span "))
        .map(|_| inner[5..].trim())
    else {
        return false;
    };
    let Some(after) = named_or_marked(rest) else {
        return false;
    };
    if after.is_empty() {
        return true;
    }
    quotedly(after, "data-hue", |one| one.is_ascii_alphabetic()).is_some_and(str::is_empty)
}

pub(crate) fn kept(inner: &str) -> bool {
    for one in ["u", "/u", "mark", "/mark", "/span"] {
        if inner.eq_ignore_ascii_case(one) {
            return true;
        }
    }
    if iconed(inner) {
        return true;
    }
    let Some(pen) = inner
        .get(..5)
        .filter(|one| one.eq_ignore_ascii_case("mark "))
        .map(|_| inner[5..].trim())
    else {
        return false;
    };
    let Some(said) = pen
        .strip_prefix("data-pen=\"")
        .and_then(|one| one.strip_suffix('"'))
    else {
        return false;
    };
    !said.is_empty() && said.chars().all(|one| one.is_ascii_alphabetic())
}

fn anchored(said: &str) -> bool {
    match said.rfind('[') {
        Some(open) => !said[open + 1..].contains(']'),
        None => false,
    }
}

fn entity(from: &str) -> bool {
    let Some(shut) = from.find(';') else {
        return false;
    };
    let name = &from[1..shut];
    name.len() > 1 && name.len() < 12 && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '#')
}

#[cfg(test)]
#[path = "docs_survival.rs"]
mod survival;
