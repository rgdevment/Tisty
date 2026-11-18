use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::store::write_atomic;
use crate::witness::{self, Fact, channel};
use crate::{Error, Result};

pub const FOLDER: &str = "history";
const ORIGINALS: &str = "originals";
const ORIGINALS_AT: &str = "originals-at";
const MARKER: &str = "state.json";
const EXTENSION: &str = "md";
const PRUNED: &str = ".pruned";

pub const QUIET: jiff::SignedDuration = jiff::SignedDuration::from_mins(30);
pub const DAYS: [u32; 4] = [1, 5, 15, 30];
pub const DAYS_BY_DEFAULT: u32 = 30;

/// The one rule, for the disk and for whoever lists versions from the log; `via` is none for the person.
pub fn cuts(
    before: Option<(jiff::Timestamp, Option<&str>)>,
    at: jiff::Timestamp,
    via: Option<&str>,
) -> bool {
    let Some((was_at, was_via)) = before else {
        return true;
    };
    was_via != via || at.duration_since(was_at).abs() >= QUIET
}

pub fn days_kept(asked: Option<u32>) -> u32 {
    asked
        .filter(|days| DAYS.contains(days))
        .unwrap_or(DAYS_BY_DEFAULT)
}

pub type Hand = (jiff::Timestamp, Option<String>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// What is replaced becomes the one step back.
    Back,
    /// The step back stays where it was, so going back sees there was writing since.
    Stays,
    /// Not somebody's writing: the step back follows the body it stood against.
    Follows,
}

#[derive(Debug, Clone)]
pub struct Writer<'a> {
    data: &'a Path,
    via: Option<String>,
    at: jiff::Timestamp,
    answered: bool,
}

impl<'a> Writer<'a> {
    pub fn now(data: &'a Path, via: Option<String>) -> Self {
        Self::at(data, via, jiff::Timestamp::now())
    }

    pub fn at(data: &'a Path, via: Option<String>, at: jiff::Timestamp) -> Self {
        Self {
            data,
            via,
            at,
            answered: true,
        }
    }

    /// A body no line of the log answers for: what it replaces is always kept.
    pub fn unanswered(data: &'a Path) -> Self {
        Self {
            answered: false,
            ..Self::now(data, None)
        }
    }

    /// Called under the documents' lock, before the new body is written.
    pub fn replaces(&self, id: &str, was: &[u8], left: impl AsRef<[u8]>, step: Step) -> Result<()> {
        adopting(self.data);
        let shelf = shelf(self.data, id)?;
        let print = crate::docs::print_of_bytes(was);
        let leaves = crate::docs::print_of_bytes(left.as_ref());
        if print == leaves {
            return Ok(());
        }
        let mut told = Told::read(&shelf);
        let before = told
            .last
            .as_ref()
            .filter(|one| one.print == print)
            .map(|one| (one.at, one.via.as_deref()));
        let cut = !self.answered || cuts(before, self.at, self.via.as_deref());
        let mut stepped_from = None;

        match step {
            Step::Back => {
                let spare = !cut && told.spare_as_back(&shelf, &print);
                kept(&shelf, &print, was, self.at)?;
                stepped_from = told.back.replace(Back {
                    print: print.clone(),
                    left: Some(leaves.clone()),
                    spare,
                });
            }
            Step::Stays | Step::Follows => {
                if cut {
                    kept(&shelf, &print, was, self.at)?;
                    if let Some(back) = told.back.as_mut().filter(|one| one.print == print) {
                        back.spare = false;
                    }
                }
                if step == Step::Follows
                    && let Some(back) = told
                        .back
                        .as_mut()
                        .filter(|one| one.left.as_deref() == Some(print.as_str()))
                {
                    back.left = Some(leaves.clone());
                }
            }
        }
        if cut {
            told.newest = Some(print);
        }
        // A body nobody answers for has no writer to weigh the next write against, so that one cuts too.
        told.last = self.answered.then(|| Last {
            at: self.at,
            via: self.via.clone(),
            print: leaves,
        });
        told.save_stepping(&shelf, stepped_from)
    }

    /// For a write that keeps no step back: one whose version cannot be kept is still written.
    pub fn passes(&self, id: &str, was: &[u8], left: impl AsRef<[u8]>, step: Step) {
        if let Err(e) = self.replaces(id, was, left, step) {
            witness::warn(
                channel::STORE,
                "what a document said could not be kept as a version before it was written over",
                &[
                    ("at", Fact::Id(id.to_string())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
    }

    /// A new body replaces nothing: it only says who wrote it and when, for the next write to weigh.
    pub fn begins(&self, id: &str, body: &str) {
        let Ok(shelf) = shelf(self.data, id) else {
            return;
        };
        let mut told = Told::read(&shelf);
        told.last = Some(Last {
            at: self.at,
            via: self.via.clone(),
            print: crate::docs::print_of_text(body),
        });
        if let Err(e) = told.save(&shelf) {
            witness::warn(
                channel::STORE,
                "who began a document could not be noted, so its first change keeps a version",
                &[
                    ("at", Fact::Id(id.to_string())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Last {
    at: jiff::Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    via: Option<String>,
    print: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Back {
    print: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    left: Option<String>,
    /// Kept only as the step back: the cut did not make it a version, so it goes when the step moves on.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    spare: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct Told {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last: Option<Last>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    back: Option<Back>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    newest: Option<String>,
}

impl Told {
    fn read(shelf: &Path) -> Self {
        std::fs::read(shelf.join(MARKER))
            .ok()
            .and_then(|said| serde_json::from_slice(&said).ok())
            .unwrap_or_default()
    }

    fn save(&self, shelf: &Path) -> Result<()> {
        raised(shelf)?;
        let said = serde_json::to_vec(self).map_err(|e| Error::Io(std::io::Error::other(e)))?;
        let at = shelf.join(MARKER);
        write_atomic(&at, &said)?;
        let _ = crate::paths::ours_alone(&at);
        Ok(())
    }

    fn spare_as_back(&self, shelf: &Path, print: &str) -> bool {
        match &self.back {
            Some(one) if one.print == print => one.spare,
            _ => !stands(shelf, print),
        }
    }

    // The step back it leaves goes only once the marker names the new one: a failed save loses neither.
    fn save_stepping(&self, shelf: &Path, from: Option<Back>) -> Result<()> {
        self.save(shelf)?;
        let still_named = |print: &str| self.back.as_ref().is_some_and(|now| now.print == print);
        if let Some(old) = from.filter(|old| old.spare && !still_named(&old.print))
            && let Ok(at) = leaf(shelf, &old.print)
        {
            let _ = std::fs::remove_file(at);
        }
        Ok(())
    }
}

fn shelf(data: &Path, id: &str) -> Result<PathBuf> {
    Ok(crate::docs::resolve(&data.join(FOLDER), id)?.with_extension(""))
}

fn raised(shelf: &Path) -> Result<()> {
    std::fs::create_dir_all(shelf)?;
    if let Some(root) = shelf.parent() {
        let _ = crate::paths::ours_alone(root);
    }
    let _ = crate::paths::ours_alone(shelf);
    Ok(())
}

fn a_print(said: &str) -> bool {
    said.len() == 64 && crate::attach::lower_hex(said)
}

fn leaf(shelf: &Path, print: &str) -> Result<PathBuf> {
    match a_print(print) {
        true => Ok(shelf.join(format!("{print}.{EXTENSION}"))),
        false => Err(Error::OutsideTheStore(print.to_string())),
    }
}

fn stands(shelf: &Path, print: &str) -> bool {
    leaf(shelf, print).is_ok_and(|at| at.is_file())
}

fn kept(shelf: &Path, print: &str, body: &[u8], replaced: jiff::Timestamp) -> Result<()> {
    raised(shelf)?;
    let at = leaf(shelf, print)?;
    write_atomic(&at, body)?;
    let _ = crate::paths::ours_alone(&at);
    dated(&at, replaced);
    Ok(())
}

// The limit counts from the day a version was replaced, which for a body that arrives is not today.
fn dated(at: &Path, replaced: jiff::Timestamp) {
    let _ = std::fs::OpenOptions::new()
        .write(true)
        .open(at)
        .and_then(|file| file.set_modified(replaced.into()));
}

fn as_kept(shelf: &Path, print: &str) -> Option<Vec<u8>> {
    crate::docs::read_as_printed(&leaf(shelf, print).ok()?, print)
        .ok()
        .flatten()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Before {
    pub body: String,
    pub left: Option<String>,
}

pub fn back(data: &Path, id: &str) -> Option<Before> {
    adopted(data);
    let shelf = shelf(data, id).ok()?;
    let back = Told::read(&shelf).back?;
    Some(Before {
        body: String::from_utf8(as_kept(&shelf, &back.print)?).ok()?,
        left: back.left,
    })
}

/// For a step back no write just replaced; called with nobody else writing, like `replaces`.
pub fn keep_back(data: &Path, id: &str, body: &str, left: &str) -> Result<()> {
    adopted(data);
    let shelf = shelf(data, id)?;
    let mut told = Told::read(&shelf);
    let print = crate::docs::print_of_text(body);
    let spare = told.spare_as_back(&shelf, &print);
    if !stands(&shelf, &print) {
        kept(&shelf, &print, body.as_bytes(), jiff::Timestamp::now())?;
    }
    let from = told.back.replace(Back {
        print,
        left: Some(crate::docs::print_of_text(left)),
        spare,
    });
    told.save_stepping(&shelf, from)
}

#[derive(Debug, Clone)]
pub struct RestorePoint {
    back: Back,
    body: Vec<u8>,
}

pub fn restore_point(data: &Path, id: &str) -> Option<RestorePoint> {
    adopted(data);
    let shelf = shelf(data, id).ok()?;
    let back = Told::read(&shelf).back?;
    let body = as_kept(&shelf, &back.print)?;
    Some(RestorePoint { back, body })
}

pub fn set_restore_point(data: &Path, id: &str, point: &RestorePoint) -> Result<()> {
    let shelf = shelf(data, id)?;
    let mut told = Told::read(&shelf);
    if !stands(&shelf, &point.back.print) {
        kept(
            &shelf,
            &point.back.print,
            &point.body,
            jiff::Timestamp::now(),
        )?;
    }
    let from = told.back.replace(point.back.clone());
    told.save_stepping(&shelf, from)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub print: String,
    pub replaced: jiff::Timestamp,
    pub bytes: u64,
    /// The step back alone, not a version the cut kept.
    pub spare: bool,
}

/// Newest first.
pub fn versions(data: &Path, id: &str) -> Vec<Version> {
    adopted(data);
    match shelf(data, id) {
        Ok(shelf) => listed(&shelf, &Told::read(&shelf)),
        Err(_) => Vec::new(),
    }
}

pub fn read(data: &Path, id: &str, print: &str) -> Option<String> {
    adopted(data);
    String::from_utf8(as_kept(&shelf(data, id).ok()?, print)?).ok()
}

fn listed(shelf: &Path, told: &Told) -> Vec<Version> {
    let Ok(entries) = std::fs::read_dir(shelf) else {
        return Vec::new();
    };
    let mut all: Vec<Version> = entries
        .filter_map(|one| one.ok())
        .filter_map(|one| {
            let at = one.path();
            let print = at.file_stem()?.to_str()?.to_string();
            if at.extension()? != EXTENSION || !a_print(&print) {
                return None;
            }
            let said = one.metadata().ok()?;
            Some(Version {
                spare: told
                    .back
                    .as_ref()
                    .is_some_and(|back| back.spare && back.print == print),
                replaced: jiff::Timestamp::try_from(said.modified().ok()?).ok()?,
                bytes: said.len(),
                print,
            })
        })
        .collect();
    all.sort_by(|a, b| b.replaced.cmp(&a.replaced).then(a.print.cmp(&b.print)));
    all
}

/// The last version the cut kept and the step back outlive any limit; it takes the documents' lock itself.
pub fn pruned(data: &Path, days: u32, now: jiff::Timestamp) -> usize {
    adopted(data);
    let Ok(shelves) = std::fs::read_dir(data.join(FOLDER)) else {
        return 0;
    };
    let limit = jiff::SignedDuration::from_hours(24 * i64::from(days));
    let mut gone = 0;
    for shelf in shelves.filter_map(|one| one.ok()).map(|one| one.path()) {
        let past = |one: &Version| now.duration_since(one.replaced) > limit;
        if !shelf.is_dir() || !listed(&shelf, &Told::default()).iter().any(past) {
            continue;
        }
        let Some(_held) = crate::docs::hold(&data.join("docs")) else {
            continue;
        };
        gone += pruned_in(&shelf, limit, now);
    }
    if gone > 0 {
        witness::note(
            channel::STORE,
            "versions of documents past the limit were taken out",
            &[("count", Fact::Count(gone))],
        );
    }
    gone
}

/// A limit counted in days needs no more than a walk a day, and the walk is not free at every opening.
pub fn pruned_when_due(data: &Path, days: u32, now: jiff::Timestamp) -> usize {
    adopted(data);
    let root = data.join(FOLDER);
    if !root.is_dir() {
        return 0;
    }
    let stamp = root.join(PRUNED);
    let last = std::fs::read_to_string(&stamp)
        .ok()
        .and_then(|said| said.trim().parse::<jiff::Timestamp>().ok());
    if last
        .is_some_and(|last| now.duration_since(last).abs() < jiff::SignedDuration::from_hours(24))
    {
        return 0;
    }
    let gone = pruned(data, days, now);
    let _ = write_atomic(&stamp, now.to_string().as_bytes());
    gone
}

fn pruned_in(shelf: &Path, limit: jiff::SignedDuration, now: jiff::Timestamp) -> usize {
    let told = Told::read(shelf);
    let all = listed(shelf, &told);
    let named = told
        .newest
        .as_deref()
        .filter(|print| all.iter().any(|one| one.print == *print));
    let last = named.or_else(|| {
        all.iter()
            .find(|one| !one.spare)
            .map(|one| one.print.as_str())
    });
    let back = told.back.as_ref().map(|one| one.print.as_str());
    all.iter()
        .filter(|one| Some(one.print.as_str()) != last && Some(one.print.as_str()) != back)
        .filter(|one| now.duration_since(one.replaced) > limit)
        .filter(|one| leaf(shelf, &one.print).is_ok_and(|at| std::fs::remove_file(at).is_ok()))
        .count()
}

/// Who wrote last stays, so the next write is weighed as ever; it takes the documents' lock itself.
pub fn empty(data: &Path, id: &str) -> Result<usize> {
    adopted(data);
    let shelf = shelf(data, id)?;
    let _held = crate::docs::hold(&data.join("docs")).ok_or(Error::AlreadyRunning)?;
    let mut told = Told::read(&shelf);
    let mut gone = 0;
    for one in listed(&shelf, &told) {
        std::fs::remove_file(leaf(&shelf, &one.print)?)?;
        gone += 1;
    }
    if told.back.is_some() || told.newest.is_some() {
        told.back = None;
        told.newest = None;
        told.save(&shelf)?;
    }
    Ok(gone)
}

pub fn forget(data: &Path, id: &str) {
    let Ok(shelf) = shelf(data, id) else {
        return;
    };
    match std::fs::remove_dir_all(&shelf) {
        Ok(()) => {}
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => witness::warn(
            channel::STORE,
            "a deleted document kept its history",
            &[
                ("at", Fact::Id(id.to_string())),
                ("why", Fact::Why(why.to_string())),
            ],
        ),
    }
}

fn adopted(data: &Path) {
    adopt_originals(data);
}

/// For a caller outside the documents' lock: a marker changes only under it, here as in a write.
pub fn adopt_originals(data: &Path) {
    if !data.join(ORIGINALS).is_dir() {
        return;
    }
    if let Some(_held) = crate::docs::hold(&data.join("docs")) {
        adopting(data);
    }
}

/// The one copy each document kept in `originals/` becomes its first version; a deleted document's is left.
fn adopting(data: &Path) {
    let from = data.join(ORIGINALS);
    let Ok(entries) = std::fs::read_dir(&from) else {
        return;
    };
    let mut adopted = 0;
    let mut unread = 0;
    for at in entries.filter_map(|one| one.ok()).map(|one| one.path()) {
        let named = at
            .file_name()
            .and_then(|one| one.to_str())
            .filter(|one| crate::docs::a_body(one));
        let Some(id) = named.and_then(|_| at.file_stem()?.to_str()) else {
            continue;
        };
        let alive = crate::docs::resolve(&data.join("docs"), id).is_ok_and(|body| body.is_file());
        if !alive {
            continue;
        }
        match adopt_one(data, &at, id) {
            Ok(true) => adopted += 1,
            Ok(false) => {}
            Err(e) => {
                unread += 1;
                witness::warn(
                    channel::STORE,
                    "what a document said before could not be moved into its history",
                    &[
                        ("at", Fact::Id(id.to_string())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
        }
    }
    if unread > 0 {
        return;
    }
    let _ = std::fs::remove_dir_all(&from);
    let _ = std::fs::remove_dir_all(data.join(ORIGINALS_AT));
    witness::note(
        channel::STORE,
        "what each document said before moved into its history",
        &[("count", Fact::Count(adopted))],
    );
}

fn adopt_one(data: &Path, from: &Path, id: &str) -> Result<bool> {
    let shelf = shelf(data, id)?;
    let mut told = Told::read(&shelf);
    if told.back.is_some() {
        return Ok(false);
    }
    let body = std::fs::read(from)?;
    let print = crate::docs::print_of_bytes(&body);
    let left = crate::docs::resolve(&data.join(ORIGINALS_AT), id)
        .ok()
        .and_then(|at| std::fs::read_to_string(at).ok())
        .map(|said| said.trim().to_string())
        .filter(|said| a_print(said));
    let replaced = std::fs::metadata(from)
        .and_then(|said| said.modified())
        .ok()
        .and_then(|when| jiff::Timestamp::try_from(when).ok())
        .unwrap_or_else(jiff::Timestamp::now);
    kept(&shelf, &print, &body, replaced)?;
    told.back = Some(Back {
        print: print.clone(),
        left,
        spare: false,
    });
    told.newest.get_or_insert(print);
    told.save(&shelf)?;
    Ok(true)
}

#[cfg(test)]
#[path = "history_test.rs"]
mod tests;
