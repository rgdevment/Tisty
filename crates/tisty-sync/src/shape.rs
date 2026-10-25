use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{HELD, PAPERS, STORE, Trouble};

pub const NAMED: &str = "tisty.toml";
const SEEN: &str = ".shape-seen";
const KEPT_FOR: usize = 8;
const RAISED: &str = ".shape-raised";
const SEALED: &str = "sealed";
const SEALING: &str = "sealing";
const HEADING: &str =
    "# What this folder is, and the shape it is in. Tisty reads this before anything else here.";

/// What this build knows the meeting place to look like. A folder that says a higher number was
/// arranged by a build that knows something this one does not, and guessing at it writes over it.
/// Shape 3 keeps every attachment in one flat folder and every document in Tisty's own words.
pub const OURS: u32 = 3;

/// The oldest release that may write in a place this build arranges; a later change to the place raises it.
pub const NEEDS: &str = "1.34.0";

pub const SEALS: &str = "1.35.0";

pub const TURNS: &str = "1.36.0";

const COPYING_ON: &str = "copying:seal:";
const COPYING_OFF: &str = "copying:unseal:";
const CLOSED: &str = "closed:";

enum Told {
    Shape {
        shape: u32,
        needs: Option<String>,
        said: Option<Box<toml_edit::DocumentMut>>,
    },
    Nothing,
    Unreadable {
        needs: Option<String>,
    },
    Unreached,
}

fn told(dest: &Path) -> Told {
    let bytes = match std::fs::read(dest.join(NAMED)) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Told::Nothing,
        Err(_) => return Told::Unreached,
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        let needs = String::from_utf8_lossy(&bytes).lines().find_map(needs_in);
        return Told::Unreadable { needs };
    };
    let Ok(said) = text.parse::<toml_edit::DocumentMut>() else {
        let needs = text.lines().find_map(needs_in);
        return match text.lines().find_map(shape_in) {
            Some(shape) => Told::Shape {
                shape,
                needs,
                said: None,
            },
            None => Told::Unreadable { needs },
        };
    };
    let shape = said
        .get("shape")
        .and_then(toml_edit::Item::as_integer)
        .and_then(|shape| u32::try_from(shape).ok());
    let needs = said.get("needs").map(|needs| match needs.as_str() {
        Some(release) => release.to_string(),
        None => needs.to_string().trim().to_string(),
    });
    match shape {
        Some(shape) => Told::Shape {
            shape,
            needs,
            said: Some(Box::new(said)),
        },
        None => Told::Unreadable { needs },
    }
}

fn shape_in(line: &str) -> Option<u32> {
    let (name, said) = line.split_once('=')?;
    (name.trim() == "shape").then(|| said.trim().parse().ok())?
}

fn needs_in(line: &str) -> Option<String> {
    said_in(line, "needs")
}

fn said_in(line: &str, key: &str) -> Option<String> {
    let (name, said) = line.split_once('=')?;
    (name.trim() == key).then(|| said.trim().trim_matches('"').to_string())
}

/// A release this build cannot read is taken for a later one: guessing it older would write where it must not.
pub fn needs_later(needs: &str) -> bool {
    let ours = [NEEDS, SEALS, TURNS, tisty_core::VERSION]
        .into_iter()
        .filter_map(tisty_core::released)
        .max();
    match (tisty_core::released(needs), ours) {
        (Some(needs), Some(ours)) => needs > ours,
        _ => true,
    }
}

fn needs_earlier(needs: &str) -> bool {
    matches!((tisty_core::released(needs), tisty_core::released(NEEDS)), (Some(needs), Some(ours)) if needs < ours)
}

/// Read before anything else in the folder is. A folder that says nothing may be one from before
/// this file existed, and is adopted; one that said something to this machine and has gone quiet
/// is one where a round did not finish or a hand went through, and its silence is never read as
/// «there is nothing here».
pub(crate) fn before_reading(data: &Path, dest: &Path) -> Result<(), Trouble> {
    let told = told(dest);
    let judged = match &told {
        Told::Nothing if seen(data, dest).is_some() => {
            witness::warn(
                channel::SYNC,
                "the meeting place said what shape it was in and now says nothing, so nothing was read from it",
                &[("at", Fact::Path(dest.to_path_buf()))],
            );
            return Err(Trouble::Unshaped(dest.display().to_string()));
        }
        Told::Unreached => Err(Trouble::Unshaped(dest.display().to_string())),
        _ => verdict(dest, &told),
    };
    if let Err(trouble) = judged {
        warned(dest, &told, &trouble);
        return Err(trouble);
    }
    match told {
        Told::Shape { shape, .. } => {
            remember(data, dest, shape);
            if shape < OURS {
                flat(dest);
            }
        }
        _ => flat(dest),
    }
    Ok(())
}

pub fn refuses(dest: &Path) -> Result<(), Trouble> {
    let told = told(dest);
    let judged = verdict(dest, &told);
    if let Err(trouble) = &judged {
        warned(dest, &told, trouble);
    }
    judged
}

pub fn spoiled(dest: &Path) -> bool {
    matches!(verdict(dest, &told(dest)), Err(Trouble::Unshaped(_)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sealing {
    Unsaid,
    Plain,
    Copying(Turn),
    Closed,
    Sealed(String),
}

pub fn sealing_of(dest: &Path) -> Sealing {
    match std::fs::read_to_string(dest.join(NAMED)) {
        Ok(text) => sealing_in(&text),
        Err(_) => Sealing::Unsaid,
    }
}

// A copy whose way this build cannot tell stops everyone, as a closed place does.
pub fn sealing_in(text: &str) -> Sealing {
    let said = text.parse::<toml_edit::DocumentMut>().ok();
    let word = |key: &str| match &said {
        Some(said) => said
            .get(key)
            .and_then(toml_edit::Item::as_str)
            .map(str::to_string),
        None => text.lines().find_map(|line| said_in(line, key)),
    };
    match (word(SEALING), word(SEALED)) {
        (Some(phase), _) if phase.starts_with(COPYING_ON) => Sealing::Copying(Turn::On),
        (Some(phase), _) if phase.starts_with(COPYING_OFF) => Sealing::Copying(Turn::Off),
        (Some(_), _) => Sealing::Closed,
        (None, Some(cloud)) => Sealing::Sealed(cloud),
        (None, None) => Sealing::Plain,
    }
}

pub fn sealed_against(dest: &Path, reading: Option<&str>) -> Option<Trouble> {
    let at = || dest.display().to_string();
    match (sealing_of(dest), reading) {
        (Sealing::Closed, _) => Some(Trouble::Sealing(at())),
        (Sealing::Copying(Turn::On), Some(_)) | (Sealing::Copying(Turn::Off), None) => {
            Some(Trouble::Sealing(at()))
        }
        (Sealing::Unsaid | Sealing::Copying(_), _) | (Sealing::Plain, None) => None,
        (Sealing::Sealed(cloud), Some(ours)) if cloud == ours => None,
        (Sealing::Sealed(_), _) => Some(Trouble::Sealed(at())),
        (Sealing::Plain, Some(_)) => Some(Trouble::Unsealed(at())),
    }
}

pub fn seal_word(dest: &Path, cloud: &str) -> std::io::Result<()> {
    let mut said = match told(dest) {
        Told::Shape {
            said: Some(said), ..
        } => *said,
        Told::Shape {
            said: None, needs, ..
        } => {
            if sealing_of(dest) != Sealing::Plain {
                return Ok(());
            }
            let mut said = template()
                .parse::<toml_edit::DocumentMut>()
                .map_err(std::io::Error::other)?;
            if let Some(needs) = needs {
                said["needs"] = toml_edit::value(needs);
            }
            said
        }
        _ => return Ok(()),
    };
    if said.contains_key(SEALED) || said.contains_key(SEALING) {
        return Ok(());
    }
    said[SEALED] = toml_edit::value(cloud);
    raise_needs(&mut said, SEALS);
    tisty_core::store::write_atomic(&dest.join(NAMED), said.to_string().as_bytes())
        .map_err(std::io::Error::other)
}

pub fn phase(turn: Turn, device: &str, nonce: &str) -> String {
    let way = match turn {
        Turn::On => COPYING_ON,
        Turn::Off => COPYING_OFF,
    };
    format!("{way}{device}:{nonce}")
}

fn phase_of(said: &toml_edit::DocumentMut) -> Option<&str> {
    said.get(SEALING).and_then(toml_edit::Item::as_str)
}

fn claim_of(phase: &str) -> Option<&str> {
    [COPYING_ON, COPYING_OFF, CLOSED]
        .into_iter()
        .find_map(|way| phase.strip_prefix(way))
}

fn ours(said: &toml_edit::DocumentMut, claim: &str) -> bool {
    phase_of(said)
        .and_then(claim_of)
        .is_some_and(|held| held == claim)
}

pub fn claim_in(text: &str) -> Option<String> {
    let said = text.parse::<toml_edit::DocumentMut>().ok()?;
    phase_of(&said).and_then(claim_of).map(str::to_string)
}

pub fn claimed_by(text: &str, claim: &str) -> bool {
    text.parse::<toml_edit::DocumentMut>()
        .is_ok_and(|said| ours(&said, claim))
}

// `needs` rises with the claim, so a build that knows no way of turning stops before the copy starts.
pub fn claimed(text: &str, phase: &str) -> Option<String> {
    let mut said = text.parse::<toml_edit::DocumentMut>().ok()?;
    if phase_of(&said).is_some() || claim_of(phase).is_none() {
        return None;
    }
    said[SEALING] = toml_edit::value(phase);
    raise_needs(&mut said, TURNS);
    Some(said.to_string())
}

pub fn closed(text: &str, claim: &str) -> Option<String> {
    let mut said = text.parse::<toml_edit::DocumentMut>().ok()?;
    if !ours(&said, claim) {
        return None;
    }
    said[SEALING] = toml_edit::value(format!("{CLOSED}{claim}"));
    Some(said.to_string())
}

pub fn switched(text: &str, claim: &str, sealed: Option<&str>) -> Option<String> {
    let mut said = text.parse::<toml_edit::DocumentMut>().ok()?;
    if !ours(&said, claim) || !phase_of(&said).is_some_and(|phase| phase.starts_with(CLOSED)) {
        return None;
    }
    said.remove(SEALING);
    match sealed {
        Some(cloud) => said[SEALED] = toml_edit::value(cloud),
        None => {
            said.remove(SEALED);
        }
    }
    Some(said.to_string())
}

pub fn dropped(text: &str, claim: &str) -> Option<String> {
    let mut said = text.parse::<toml_edit::DocumentMut>().ok()?;
    if !ours(&said, claim) {
        return None;
    }
    said.remove(SEALING);
    Some(said.to_string())
}

fn raise_needs(said: &mut toml_edit::DocumentMut, to: &str) {
    let held = said
        .get("needs")
        .and_then(toml_edit::Item::as_str)
        .is_some_and(|needs| !matches!((tisty_core::released(needs), tisty_core::released(to)), (Some(needs), Some(to)) if needs < to));
    if !held {
        said["needs"] = toml_edit::value(to);
    }
}

pub fn later(dest: &Path) -> Option<Trouble> {
    verdict(dest, &told(dest))
        .err()
        .filter(|trouble| matches!(trouble, Trouble::Needs(_) | Trouble::Shape(_)))
}

fn verdict(dest: &Path, told: &Told) -> Result<(), Trouble> {
    let (shape, needs) = match told {
        Told::Shape { shape, needs, .. } => (Some(*shape), needs),
        Told::Unreadable { needs } => (None, needs),
        Told::Unreached => return Err(Trouble::NotThere(dest.display().to_string())),
        Told::Nothing => return Ok(()),
    };
    if let Some(needs) = needs.as_ref().filter(|needs| needs_later(needs)) {
        return Err(Trouble::Needs(needs.clone()));
    }
    match shape {
        Some(shape) if shape > OURS => Err(Trouble::Shape(dest.display().to_string())),
        Some(_) => Ok(()),
        None => Err(Trouble::Unshaped(dest.display().to_string())),
    }
}

fn warned(dest: &Path, told: &Told, trouble: &Trouble) {
    let at = ("at", Fact::Path(dest.to_path_buf()));
    match (trouble, told) {
        (Trouble::Needs(needs), _) => witness::warn(
            channel::SYNC,
            "the meeting place asks for a later Tisty than this one, so nothing was carried either way",
            &[at, ("needs", Fact::Id(needs.clone()))],
        ),
        (Trouble::Shape(_), Told::Shape { shape, .. }) => witness::warn(
            channel::SYNC,
            "the meeting place is in a shape this build does not know, so nothing was carried either way",
            &[at, ("shape", Fact::Count(*shape as usize))],
        ),
        (Trouble::NotThere(_), _) => witness::warn(
            channel::SYNC,
            "the meeting place's word on its shape could not be opened just now, so nothing was done there",
            &[at],
        ),
        (Trouble::Unshaped(_), _) => witness::warn(
            channel::SYNC,
            "the meeting place says what shape it is in and it cannot be read, so nothing was carried either way",
            &[("at", Fact::Path(dest.join(NAMED)))],
        ),
        _ => {}
    }
}

// Before this round reads it, so the files an older build left on shelves are where it looks.
fn flat(dest: &Path) {
    let (moved, left) = tisty_core::speaking::flattened(&dest.join(HELD));
    if moved + left > 0 {
        witness::note(
            channel::SYNC,
            "the meeting place's attachments left their shelves for one flat folder",
            &[("moved", Fact::Count(moved)), ("left", Fact::Count(left))],
        );
    }
}

/// Written after everything else a round writes, so finding it is finding a round that finished.
/// Says whether it raised, and wrote, a place older builds had arranged.
pub(crate) fn stamp(data: &Path, dest: &Path) -> bool {
    match told(dest) {
        Told::Shape { shape, needs, said } if shape == OURS => {
            match needs.as_deref().is_some_and(needs_earlier) {
                true => {
                    tell(data, dest, said.map(|said| *said));
                }
                false => remember(data, dest, OURS),
            }
            false
        }
        Told::Shape { shape, .. } if shape > OURS => false,
        Told::Unreadable { .. } | Told::Unreached => false,
        Told::Shape { said, .. } => tell(data, dest, said.map(|said| *said)),
        Told::Nothing => {
            tell(data, dest, None);
            false
        }
    }
}

/// Raised before anything is given, so an older build stops before it can take what it cannot read.
/// Only a place that already says its shape is raised this early: a new one is still named last.
pub(crate) fn raise_first(data: &Path, dest: &Path) -> bool {
    match told(dest) {
        Told::Shape { shape, needs, said }
            if shape < OURS || (shape == OURS && needs.as_deref().is_some_and(needs_earlier)) =>
        {
            tell(data, dest, said.map(|said| *said))
        }
        _ => false,
    }
}

/// Raised by a machine as it leaves, so the builds before this one stop there even without a round of its own.
pub fn raise_place(dest: &Path) {
    match told(dest) {
        Told::Shape { shape, said, .. } if shape < OURS => {
            written(dest, said.map(|said| *said));
        }
        Told::Nothing if dest.join(STORE).is_dir() => {
            written(dest, None);
        }
        _ => {}
    }
}

/// Only another machine's history there means someone is left on an older build by raising it.
pub(crate) fn shared_with_others(dest: &Path, device: &str) -> bool {
    std::fs::read_dir(dest.join(STORE)).is_ok_and(|entries| {
        entries
            .filter_map(Result::ok)
            .any(|one| one.file_type().is_ok_and(|kind| kind.is_dir()) && one.file_name() != device)
    })
}

pub(crate) fn mark_raised(data: &Path) {
    let _ = tisty_core::store::write_atomic(&data.join(RAISED), b"");
}

fn tell(data: &Path, dest: &Path, said: Option<toml_edit::DocumentMut>) -> bool {
    let done = written(dest, said);
    if done {
        remember(data, dest, OURS);
    }
    done
}

fn template() -> String {
    format!(
        "{HEADING}\nshape = {OURS}\nneeds = \"{NEEDS}\"\nfolders = [\"{STORE}\", \"{HELD}\", \"{PAPERS}\"]\n"
    )
}

fn written(dest: &Path, before: Option<toml_edit::DocumentMut>) -> bool {
    let said = match before {
        Some(mut said) => {
            said["shape"] = toml_edit::value(i64::from(OURS));
            let kept = said
                .get("needs")
                .is_some_and(|needs| needs.as_str().is_none_or(|needs| !needs_earlier(needs)));
            if !kept {
                said["needs"] = toml_edit::value(NEEDS);
            }
            said["folders"] = toml_edit::value(toml_edit::Array::from_iter([STORE, HELD, PAPERS]));
            said.to_string()
        }
        None => template(),
    };
    if let Err(e) = tisty_core::store::write_atomic(&dest.join(NAMED), said.as_bytes()) {
        witness::warn(
            channel::SYNC,
            "the meeting place could not be told what shape it is in",
            &[
                ("at", Fact::Path(dest.join(NAMED))),
                ("why", Fact::Why(e.to_string())),
            ],
        );
        return false;
    }
    true
}

/// Left by the round that raised a place older builds had arranged, until the window has said so once.
pub fn raised(data: &Path) -> bool {
    data.join(RAISED).is_file()
}

pub fn raised_told(data: &Path) {
    let _ = std::fs::remove_file(data.join(RAISED));
}

/// Remembered against the folder it was read in, never against the machine: one that has met a
/// folder which says its shape must still be able to take up a folder from before this file
/// existed, or pointing it at an older one would lock it out with nothing to undo it.
fn seen(data: &Path, dest: &Path) -> Option<u32> {
    let looking = named(dest);
    std::fs::read_to_string(data.join(SEEN))
        .ok()?
        .lines()
        .find_map(|line| {
            let (said, whose) = line.split_once('\t')?;
            (whose == looking).then(|| said.parse().ok())?
        })
}

fn remember(data: &Path, dest: &Path, said: u32) {
    if seen(data, dest) == Some(said) {
        return;
    }
    let looking = named(dest);
    let mut held = vec![format!("{said}\t{looking}")];
    if let Ok(kept) = std::fs::read_to_string(data.join(SEEN)) {
        held.extend(
            kept.lines()
                .filter(|line| {
                    line.split_once('\t')
                        .is_some_and(|(_, whose)| whose != looking)
                })
                .take(KEPT_FOR)
                .map(str::to_string),
        );
    }
    let _ = tisty_core::store::write_atomic(&data.join(SEEN), held.join("\n").as_bytes());
}

pub fn unseen(data: &Path, dest: &Path) {
    let looking = named(dest);
    let Ok(kept) = std::fs::read_to_string(data.join(SEEN)) else {
        return;
    };
    let held: Vec<&str> = kept
        .lines()
        .filter(|line| {
            line.split_once('\t')
                .is_none_or(|(_, whose)| whose != looking)
        })
        .collect();
    if held.len() != kept.lines().count() {
        let _ = tisty_core::store::write_atomic(&data.join(SEEN), held.join("\n").as_bytes());
    }
}

fn named(dest: &Path) -> String {
    std::fs::canonicalize(dest)
        .unwrap_or_else(|_| dest.to_path_buf())
        .display()
        .to_string()
        .replace(['\n', '\t'], " ")
}

#[cfg(test)]
#[path = "shape_test.rs"]
mod tests;
