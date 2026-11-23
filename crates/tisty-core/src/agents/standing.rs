use std::path::{Component, Path, PathBuf};

use super::{Admitted, Kind, Ledger, Refused, kept, rendered};
use crate::docs::{Move, READ_AT_MOST, moved};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub root: PathBuf,
    pub at: PathBuf,
    pub assistant: String,
    pub pieces: Vec<Admitted>,
    pub(super) held: bool,
    pub(super) unproven: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Standing {
    NoConsent,
    Off,
    Held,
    Unproven,
    Empty,
    Linked,
    Broken {
        piece: usize,
        why: Refused,
    },
    UpToDate,
    Agreed,
    Write,
    Wrote,
    Outside {
        now: Option<Vec<u8>>,
        piece: Option<usize>,
    },
    Both {
        now: Vec<u8>,
    },
    /// Over bytes Tisty never wrote, or, `changed`, bytes it wrote that the person changed or took away since.
    Foreign {
        changed: bool,
    },
    Unreadable(String),
}

impl Target {
    pub fn put(&mut self, piece: &Admitted) {
        self.pieces.push(piece.clone());
    }

    pub fn files(&self) -> impl Iterator<Item = &str> {
        self.pieces.iter().map(Admitted::file)
    }

    pub fn held(&self) -> bool {
        self.held
    }

    /// Every piece is one of Tisty's own skills, so no document stands behind the file.
    pub fn shipped(&self) -> bool {
        !self.pieces.is_empty() && self.files().all(super::builtin)
    }

    fn agents_own(&self) -> bool {
        self.pieces
            .iter()
            .any(|one| one.card().kind == Kind::Memory)
    }

    // The documents a file is made of can change while its bytes do not.
    fn stale_in(&self, ledger: &Ledger) -> bool {
        let applied: std::collections::BTreeSet<&str> = ledger.applied_at(&self.at).collect();
        applied != self.files().collect()
    }

    fn settled_in(&self, ledger: &mut Ledger, bytes: &[u8]) {
        ledger.settle(&self.at, bytes);
        ledger.applied_to(&self.at, self.files().map(str::to_string));
    }
}

fn rendered_pieces(target: &Target) -> Result<Vec<Vec<u8>>, (usize, Refused)> {
    target
        .pieces
        .iter()
        .enumerate()
        .map(|(n, one)| rendered(one.body()).map_err(|why| (n, why)))
        .collect()
}

fn separator(first: &[u8]) -> &'static [u8] {
    match first.windows(2).any(|pair| pair == b"\r\n") {
        true => b"\r\n",
        false => b"\n",
    }
}

fn ended(piece: &[u8], newline: &[u8]) -> Vec<u8> {
    let mut piece = piece.to_vec();
    if !piece.is_empty() && !piece.ends_with(b"\n") {
        piece.extend_from_slice(newline);
    }
    piece
}

fn composed(pieces: &[Vec<u8>]) -> Vec<u8> {
    let [one] = pieces else {
        let newline = separator(pieces.first().map_or(&[][..], Vec::as_slice));
        let ended: Vec<Vec<u8>> = pieces.iter().map(|one| ended(one, newline)).collect();
        return ended.join(newline);
    };
    one.clone()
}

pub fn wanted(target: &Target) -> Result<Vec<u8>, (usize, Refused)> {
    Ok(composed(&rendered_pieces(target)?))
}

fn blank(bytes: &[u8]) -> bool {
    String::from_utf8_lossy(bytes)
        .chars()
        .all(|one| one.is_whitespace() || one == '\u{feff}')
}

// A link anywhere from the assistant's folder down would send the write into whatever it points at.
fn linked(target: &Target) -> bool {
    let stepped_out = target.at.components().any(|step| {
        !matches!(
            step,
            Component::Normal(_) | Component::RootDir | Component::Prefix(_)
        )
    });
    if stepped_out || !target.root.is_absolute() || !target.at.starts_with(&target.root) {
        return true;
    }
    target
        .at
        .ancestors()
        .take_while(|step| step.starts_with(&target.root))
        .any(super::link)
}

fn on_disk(at: &Path) -> Result<Option<Vec<u8>>, Refused> {
    let meta = match std::fs::metadata(at) {
        Ok(meta) => meta,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    if meta.len() > READ_AT_MOST {
        return Err(Refused::TooBig);
    }
    Ok(Some(std::fs::read(at)?))
}

fn printed(bytes: &[u8]) -> String {
    crate::attach::printed(bytes)
}

pub fn standing(ledger: &Ledger, target: &Target) -> Standing {
    judged(ledger, target).0
}

type Seen = Option<String>;

// Tisty's own skills, and copies of them alone, follow their own switch; only a document stops at a paused agent.
fn allowed(ledger: &Ledger, target: &Target) -> Result<(), Refused> {
    let copy_alone = target.pieces.is_empty() && shipped_copy(ledger, &target.at);
    let (consented, paused) = match (target.shipped(), copy_alone) {
        (true, _) => (ledger.tisty_skills(), false),
        (false, true) => (ledger.named(), false),
        (false, false) => (ledger.consented(), true),
    };
    if !consented {
        return Err(Refused::NoConsent);
    }
    if paused && !ledger.syncs(&target.assistant) {
        return Err(Refused::Off);
    }
    Ok(())
}

/// A file the ledger records as written from Tisty's own skills alone, so it is never anyone's document.
pub fn shipped_copy(ledger: &Ledger, at: &Path) -> bool {
    // Compared as paths, not as the text the ledger keeps them under.
    ledger
        .written_files()
        .find(|(one, _)| *one == at)
        .is_some_and(|(_, files)| {
            !files.is_empty() && files.iter().all(|file| super::builtin(file))
        })
}

fn judged(ledger: &Ledger, target: &Target) -> (Standing, Seen) {
    match allowed(ledger, target) {
        Err(Refused::NoConsent) => return (Standing::NoConsent, None),
        Err(_) => return (Standing::Off, None),
        Ok(()) => {}
    }
    if target.held {
        return (Standing::Held, None);
    }
    let wanted = match wanted(target) {
        Ok(wanted) => wanted,
        Err((piece, why)) => return (Standing::Broken { piece, why }, None),
    };
    if linked(target) {
        return (Standing::Linked, None);
    }
    let now = match on_disk(&target.at) {
        Ok(now) => now,
        Err(why) => return (Standing::Unreadable(why.to_string()), None),
    };
    let there = now.as_deref().map(printed);
    (weighed(ledger, target, &wanted, now), there)
}

fn weighed(ledger: &Ledger, target: &Target, wanted: &[u8], now: Option<Vec<u8>>) -> Standing {
    // A file the person left with the agent is theirs: a document landing on it again asks before writing.
    let base = ledger
        .written(&target.at)
        .filter(|_| !ledger.stays(&target.at));
    if target.agents_own() {
        return match now {
            Some(now) if now != wanted && base != Some(printed(&now).as_str()) => {
                Standing::Outside {
                    now: Some(now),
                    piece: Some(0),
                }
            }
            _ => Standing::UpToDate,
        };
    }
    if blank(wanted) {
        return Standing::Empty;
    }
    let here = printed(wanted);
    if target.shipped() {
        return shipped(ledger, target, &here, now);
    }
    if target.unproven && base != Some(here.as_str()) {
        return Standing::Unproven;
    }
    let there = now.as_deref().map(printed);
    if base.is_some() && now.is_none() {
        return Standing::Outside {
            now: None,
            piece: None,
        };
    }
    match moved(base, Some(&here), there.as_deref()) {
        Move::Nothing if base == Some(here.as_str()) => Standing::UpToDate,
        Move::Nothing => Standing::Agreed,
        Move::Send => Standing::Write,
        Move::Bring => {
            let now = now.unwrap_or_default();
            let piece = rendered_pieces(target)
                .ok()
                .and_then(|pieces| attributed(&pieces, &now))
                .map(|(piece, _)| piece);
            Standing::Outside {
                now: Some(now),
                piece,
            }
        }
        Move::TheyDecide => Standing::Both {
            now: now.unwrap_or_default(),
        },
    }
}

// Tisty's own skills only go over a file Tisty wrote from them and nothing else, while it still reads as written.
fn shipped(ledger: &Ledger, target: &Target, here: &str, now: Option<Vec<u8>>) -> Standing {
    let alone = match (ledger.written(&target.at), now) {
        _ if !shipped_here(ledger, &target.at) => Standing::Foreign { changed: false },
        (None, None) => Standing::Write,
        (None, Some(_)) => Standing::Foreign { changed: false },
        (Some(base), Some(now)) if printed(&now) == base => match base == here {
            true => Standing::UpToDate,
            false => Standing::Write,
        },
        (Some(_), _) => Standing::Foreign { changed: true },
    };
    match alone {
        Standing::Write | Standing::UpToDate => beside(ledger, target).unwrap_or(alone),
        other => other,
    }
}

fn shipped_here(ledger: &Ledger, at: &Path) -> bool {
    ledger.applied_at(at).all(super::builtin) && !ledger.unproven(at)
}

const SKILL_FOLDER_AT_MOST: usize = 64;
const SKILL_FOLDER_DEPTH: usize = 4;

// A skill is one folder on disk: a file in it that is not Tisty's, or one the person changed, holds every file of it.
fn beside(ledger: &Ledger, target: &Target) -> Option<Standing> {
    let folder = skill_folder(target)?;
    // The ledger keeps paths as text, and `a/b` and `a\b` are one file on Windows.
    let recorded: Vec<&Path> = ledger
        .written_files()
        .map(|(at, _)| at)
        .filter(|at| at.starts_with(&folder))
        .collect();
    let mut left = vec![(folder, 0)];
    let mut seen = 0;
    let mut changed = false;
    while let Some((dir, depth)) = left.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return Some(Standing::Foreign { changed: false }),
        };
        for entry in entries {
            seen += 1;
            let Ok(entry) = entry else {
                return Some(Standing::Foreign { changed: false });
            };
            let at = entry.path();
            let kind = std::fs::symlink_metadata(&at).map(|meta| meta.file_type());
            match kind {
                Ok(kind) if kind.is_dir() && depth < SKILL_FOLDER_DEPTH => {
                    left.push((at, depth + 1))
                }
                Ok(kind) if kind.is_file() && seen <= SKILL_FOLDER_AT_MOST => {
                    let known = recorded.iter().find(|one| **one == at);
                    let base = known.and_then(|one| ledger.written(one));
                    match (base, known.is_some_and(|one| shipped_here(ledger, one))) {
                        (Some(base), true) => {
                            changed |= on_disk(&at).ok().flatten().map(|now| printed(&now))
                                != Some(base.to_string());
                        }
                        _ => return Some(Standing::Foreign { changed: false }),
                    }
                }
                _ => return Some(Standing::Foreign { changed: false }),
            }
        }
    }
    changed.then_some(Standing::Foreign { changed: true })
}

fn skill_folder(target: &Target) -> Option<PathBuf> {
    let [piece] = target.pieces.as_slice() else {
        return None;
    };
    let name = &piece.card().name;
    let steps = name.split('/').count();
    let folder = target.at.ancestors().nth(steps.checked_sub(1)?)?;
    let skill = super::skill_of(name);
    let named = folder
        .file_name()
        .is_some_and(|it| it.to_string_lossy().eq_ignore_ascii_case(skill));
    (steps > 1 && named && folder.starts_with(&target.root) && folder != target.root)
        .then(|| folder.to_path_buf())
}

fn attributed(pieces: &[Vec<u8>], now: &[u8]) -> Option<(usize, Vec<u8>)> {
    if let [_] = pieces {
        return Some((0, now.to_vec()));
    }
    let newline = separator(pieces.first()?);
    let ended: Vec<Vec<u8>> = pieces.iter().map(|one| ended(one, newline)).collect();
    let mut found = None;
    for n in 0..ended.len() {
        let mut before = ended[..n].join(newline);
        if n > 0 {
            before.extend_from_slice(newline);
        }
        let mut after = Vec::new();
        if n + 1 < ended.len() {
            after.extend_from_slice(newline);
            after.extend(ended[n + 1..].join(newline));
        }
        if now.len() < before.len() + after.len()
            || !now.starts_with(&before)
            || !now.ends_with(&after)
        {
            continue;
        }
        let middle = now[before.len()..now.len() - after.len()].to_vec();
        if middle == ended[n] {
            continue;
        }
        if found.is_some() {
            return None;
        }
        found = Some((n, middle));
    }
    found
}

fn writable(ledger: &Ledger, target: &Target) -> Result<Vec<u8>, Refused> {
    allowed(ledger, target)?;
    if target.held {
        return Err(Refused::Held);
    }
    if target.agents_own() {
        return Err(Refused::AgentsOwn);
    }
    if linked(target) {
        return Err(Refused::Linked);
    }
    let wanted = wanted(target).map_err(|(_, why)| why)?;
    if blank(&wanted) {
        return Err(Refused::Empty);
    }
    Ok(wanted)
}

fn written_over(data: &Path, at: &Path, bytes: &[u8], seen: Option<&Seen>) -> Result<(), Refused> {
    let was = on_disk(at)?;
    if seen.is_some_and(|seen| *seen != was.as_deref().map(printed)) {
        return Err(Refused::Changed);
    }
    if let Some(was) = was
        && was != bytes
    {
        kept::keep(data, at, &was)?;
    }
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent)?;
    }
    crate::store::write_atomic(at, bytes)?;
    Ok(())
}

pub fn overwrite(data: &Path, ledger: &mut Ledger, target: &Target) -> Result<(), Refused> {
    if target.shipped() {
        return Err(Refused::Foreign);
    }
    written(data, ledger, target, None)?;
    kept_ledger(data, ledger);
    Ok(())
}

fn written(
    data: &Path,
    ledger: &mut Ledger,
    target: &Target,
    seen: Option<&Seen>,
) -> Result<(), Refused> {
    let wanted = writable(ledger, target)?;
    written_over(data, &target.at, &wanted, seen)?;
    target.settled_in(ledger, &wanted);
    Ok(())
}

// A file written but not yet in a kept ledger reads as agreed on the next start, so losing it costs nothing.
fn kept_ledger(data: &Path, ledger: &Ledger) {
    if ledger.keep(data).is_err() {
        crate::witness::warn(
            crate::witness::channel::AGENT,
            "agent files were written but the ledger could not be kept",
            &[],
        );
    }
}

pub fn round(data: &Path, ledger: &mut Ledger, targets: &[Target]) -> Vec<Standing> {
    let mut settled = false;
    let told = targets
        .iter()
        .map(|target| match judged(ledger, target) {
            (Standing::Write, seen) => match written(data, ledger, target, Some(&seen)) {
                Ok(()) => {
                    settled = true;
                    Standing::Wrote
                }
                Err(Refused::Changed) => standing(ledger, target),
                Err(why) => Standing::Unreadable(why.to_string()),
            },
            (Standing::UpToDate, _) if target.unproven || target.stale_in(ledger) => {
                ledger.applied_to(&target.at, target.files().map(str::to_string));
                settled = true;
                Standing::UpToDate
            }
            (Standing::Agreed, _) => {
                if let Ok(wanted) = wanted(target) {
                    target.settled_in(ledger, &wanted);
                    settled = true;
                }
                Standing::UpToDate
            }
            (other, _) => other,
        })
        .collect();
    if settled {
        kept_ledger(data, ledger);
    }
    told
}

// A file just brought in stands for itself, so what joins it later is written over it with a backup, not asked about.
pub fn adopted(ledger: &mut Ledger, target: &Target, brought: &str) -> bool {
    // Bringing alone touches nothing, the ledger included: only a file Tisty writes is ever recorded as written.
    if !ledger.consented()
        || !ledger.syncs(&target.assistant)
        || ledger.written(&target.at).is_some()
        || linked(target)
    {
        return false;
    }
    let Some(piece) = target.pieces.iter().find(|one| one.file() == brought) else {
        return false;
    };
    let Ok(itself) = rendered(piece.body()) else {
        return false;
    };
    if !matches!(on_disk(&target.at), Ok(Some(now)) if now == itself) {
        return false;
    }
    ledger.settle(&target.at, &itself);
    ledger.applied_to(&target.at, [brought.to_string()]);
    true
}

// No document places this file any more; the person chose to take it away, so it leaves a backup first.
pub fn removed(data: &Path, ledger: &mut Ledger, target: &Target) -> Result<(), Refused> {
    allowed(ledger, target)?;
    if !target.pieces.is_empty() || target.held {
        return Err(Refused::Held);
    }
    if linked(target) {
        return Err(Refused::Linked);
    }
    if let Some(was) = on_disk(&target.at)? {
        kept::keep(data, &target.at, &was)?;
        std::fs::remove_file(&target.at)?;
        emptied_up(target);
    }
    ledger.forget(&target.at);
    kept_ledger(data, ledger);
    Ok(())
}

/// A skill folder whose every file is one this release ships, with the very bytes and nothing in the ledger for any,
/// is Tisty's copy again: the ledger was lost or began anew on this computer. One file off, and none is claimed.
pub fn reclaimed(ledger: &mut Ledger, targets: &[Target]) -> bool {
    if !ledger.tisty_skills() {
        return false;
    }
    let mut skills: std::collections::BTreeMap<PathBuf, Vec<&Target>> = Default::default();
    for target in targets.iter().filter(|one| one.shipped()) {
        if let Some(folder) = skill_folder(target) {
            skills.entry(folder).or_default().push(target);
        }
    }
    let mut claimed = false;
    for (folder, files) in skills {
        let Some(bytes) = files
            .iter()
            .map(|target| as_shipped(ledger, target))
            .collect::<Option<Vec<Vec<u8>>>>()
        else {
            continue;
        };
        if !holds_only(&folder, &files) {
            continue;
        }
        for (target, bytes) in files.iter().zip(bytes) {
            target.settled_in(ledger, &bytes);
        }
        claimed = true;
    }
    claimed
}

// The file is there as this release ships it, with no record of anyone's, and no link on the way.
fn as_shipped(ledger: &Ledger, target: &Target) -> Option<Vec<u8>> {
    let unrecorded =
        ledger.written(&target.at).is_none() && ledger.applied_at(&target.at).next().is_none();
    if !unrecorded || linked(target) {
        return None;
    }
    let wanted = wanted(target).ok()?;
    matches!(on_disk(&target.at), Ok(Some(now)) if now == wanted).then_some(wanted)
}

// Nothing in the folder but the files the release ships: no file of the person's, no link, no deeper folder.
fn holds_only(folder: &Path, files: &[&Target]) -> bool {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return false;
    };
    let mut seen = 0;
    for entry in entries {
        let Ok(entry) = entry else {
            return false;
        };
        seen += 1;
        let at = entry.path();
        let plain = std::fs::symlink_metadata(&at).is_ok_and(|meta| meta.is_file());
        if !plain || seen > SKILL_FOLDER_AT_MOST || !files.iter().any(|one| one.at == at) {
            return false;
        }
    }
    true
}

// Taken away with no file shown to the person, so a copy changed since Tisty wrote it stays for them to see.
pub fn removed_as_written(
    data: &Path,
    ledger: &mut Ledger,
    target: &Target,
) -> Result<bool, Refused> {
    allowed(ledger, target)?;
    if linked(target) {
        return Err(Refused::Linked);
    }
    let now = on_disk(&target.at)?;
    if now.is_some_and(|now| ledger.written(&target.at) != Some(printed(&now).as_str())) {
        return Ok(false);
    }
    removed(data, ledger, target).map(|()| true)
}

// With Tisty's own skills turned off, a copy still as Tisty wrote it goes with a backup; one changed since stays, loose, for the person.
pub fn unshipped(data: &Path, ledger: &mut Ledger, target: &Target) -> Result<bool, Refused> {
    if !target.pieces.is_empty() || !shipped_copy(ledger, &target.at) {
        return Err(Refused::Held);
    }
    allowed(ledger, target)?;
    if linked(target) {
        return Err(Refused::Linked);
    }
    let now = on_disk(&target.at)?;
    let gone = match now {
        Some(now) if ledger.written(&target.at) != Some(printed(&now).as_str()) => {
            return Ok(false);
        }
        Some(now) => {
            kept::keep(data, &target.at, &now)?;
            std::fs::remove_file(&target.at)?;
            emptied_up(target);
            true
        }
        None => false,
    };
    ledger.forget(&target.at);
    kept_ledger(data, ledger);
    Ok(gone)
}

// A skill's folder left with nothing in it would still be listed; the assistant's own folders, such as `rules`, stay.
// Removing a link to a folder takes the link away whatever the folder holds, so a link stops the climb.
fn emptied_up(target: &Target) {
    if !target.root.is_absolute() {
        return;
    }
    for up in target.at.ancestors().skip(1) {
        let own = up == target.root || up.parent().is_none_or(|parent| parent == target.root);
        if own || !up.starts_with(&target.root) || super::link(up) {
            return;
        }
        if std::fs::remove_dir(up).is_err() {
            return;
        }
    }
}

pub fn bring(ledger: &Ledger, target: &Target, piece: usize) -> Result<String, Refused> {
    if !ledger.consented() {
        return Err(Refused::NoConsent);
    }
    if linked(target) {
        return Err(Refused::Linked);
    }
    let now = on_disk(&target.at)?.ok_or(Refused::Unattributed)?;
    let body = target.pieces.get(piece).ok_or(Refused::Unattributed)?;
    let pieces = rendered_pieces(target).map_err(|(_, why)| why)?;
    let middle = match attributed(&pieces, &now) {
        Some((n, middle)) if n == piece => middle,
        _ => return Err(Refused::Unattributed),
    };
    if middle.len() as u64 > crate::docs::BODY_AT_MOST {
        return Err(Refused::TooBig);
    }
    if let Some(told) = crate::agent::secret_in(&middle) {
        return Err(Refused::Secret(told.said()));
    }
    super::brought(body.body(), &middle)
}

pub fn restore(data: &Path, ledger: &Ledger, target: &Target, n: u64) -> Result<(), Refused> {
    if !ledger.consented() {
        return Err(Refused::NoConsent);
    }
    if !ledger.syncs(&target.assistant) {
        return Err(Refused::Off);
    }
    if target.agents_own() {
        return Err(Refused::AgentsOwn);
    }
    if linked(target) {
        return Err(Refused::Linked);
    }
    let bytes = kept::read_kept(data, &target.at, n)?;
    if blank(&bytes) {
        return Err(Refused::Empty);
    }
    written_over(data, &target.at, &bytes, None)
}

// The agent owns the rest of this file, so Tisty only ever adds its one line and never takes anything out.
pub fn lead_in(data: &Path, ledger: &Ledger, target: &Target, line: &str) -> Result<bool, Refused> {
    if !ledger.consented() {
        return Err(Refused::NoConsent);
    }
    if !ledger.syncs(&target.assistant) {
        return Err(Refused::Off);
    }
    if !line.starts_with('@') || line.contains(['\n', '\r']) || line.trim() != line {
        return Err(Refused::Name);
    }
    if linked(target) {
        return Err(Refused::Linked);
    }
    led_over(data, target, line, on_disk(&target.at)?)
}

// The agent may write its memory between the read and the write, so the write checks it still reads the same.
fn led_over(
    data: &Path,
    target: &Target,
    line: &str,
    read: Option<Vec<u8>>,
) -> Result<bool, Refused> {
    let seen = read.as_deref().map(printed);
    let was = read.unwrap_or_default();
    let text = std::str::from_utf8(&was).map_err(|_| Refused::NotText)?;
    if text.lines().any(|one| one.trim() == line) {
        return Ok(false);
    }
    let newline = separator(&was);
    let mut now = was.clone();
    if !now.is_empty() && !now.ends_with(b"\n") {
        now.extend_from_slice(newline);
    }
    now.extend_from_slice(line.as_bytes());
    now.extend_from_slice(newline);
    written_over(data, &target.at, &now, Some(&seen))?;
    Ok(true)
}

// A file Tisty cannot read is never taken for one the person emptied of the line.
pub fn lacks(target: &Target, line: &str) -> bool {
    if linked(target) {
        return false;
    }
    match on_disk(&target.at) {
        Ok(None) => true,
        Ok(Some(was)) => {
            std::str::from_utf8(&was).is_ok_and(|text| !text.lines().any(|one| one.trim() == line))
        }
        Err(_) => false,
    }
}

// Only the line Tisty put there leaves; whatever the agent wrote around it stays.
pub fn lead_out(
    data: &Path,
    ledger: &Ledger,
    target: &Target,
    line: &str,
) -> Result<bool, Refused> {
    if !ledger.consented() {
        return Err(Refused::NoConsent);
    }
    if linked(target) {
        return Err(Refused::Linked);
    }
    let Some(was) = on_disk(&target.at)? else {
        return Ok(false);
    };
    unled_over(data, target, line, was)
}

fn unled_over(data: &Path, target: &Target, line: &str, was: Vec<u8>) -> Result<bool, Refused> {
    let text = std::str::from_utf8(&was).map_err(|_| Refused::NotText)?;
    let left: String = text
        .split_inclusive('\n')
        .filter(|one| one.trim() != line)
        .collect();
    if left.len() == text.len() {
        return Ok(false);
    }
    written_over(
        data,
        &target.at,
        left.as_bytes(),
        Some(&Some(printed(&was))),
    )?;
    Ok(true)
}

#[cfg(test)]
#[path = "standing_test.rs"]
mod tests;
