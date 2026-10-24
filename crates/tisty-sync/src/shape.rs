use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{HELD, PAPERS, STORE, Trouble};

pub const NAMED: &str = "tisty.toml";
const SEEN: &str = ".shape-seen";
const KEPT_FOR: usize = 8;
const RAISED: &str = ".shape-raised";
const HEADING: &str =
    "# What this folder is, and the shape it is in. Tisty reads this before anything else here.";

/// What this build knows the meeting place to look like. A folder that says a higher number was
/// arranged by a build that knows something this one does not, and guessing at it writes over it.
/// Shape 3 keeps every attachment in one flat folder and every document in Tisty's own words.
pub const OURS: u32 = 3;

/// The oldest release that may write in a place this build arranges; a later change to the place raises it.
pub const NEEDS: &str = "1.34.0";

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
    let (name, said) = line.split_once('=')?;
    (name.trim() == "needs").then(|| said.trim().trim_matches('"').to_string())
}

fn release(said: &str) -> Option<(u64, u64, u64, bool)> {
    let said = said.split('+').next()?;
    let (core, finished) = match said.split_once('-') {
        Some((core, _)) => (core, false),
        None => (said, true),
    };
    let mut parts = core.split('.');
    let mut next = || parts.next()?.parse().ok();
    Some((next()?, next()?, next()?, finished))
}

/// A release this build cannot read is taken for a later one: guessing it older would write where it must not.
pub fn needs_later(needs: &str) -> bool {
    let ours = [NEEDS, tisty_core::VERSION]
        .into_iter()
        .filter_map(release)
        .max();
    match (release(needs), ours) {
        (Some(needs), Some(ours)) => needs > ours,
        _ => true,
    }
}

fn needs_earlier(needs: &str) -> bool {
    matches!((release(needs), release(NEEDS)), (Some(needs), Some(ours)) if needs < ours)
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
        None => format!(
            "{HEADING}\nshape = {OURS}\nneeds = \"{NEEDS}\"\nfolders = [\"{STORE}\", \"{HELD}\", \"{PAPERS}\"]\n"
        ),
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
