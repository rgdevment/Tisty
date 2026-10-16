use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{HELD, PAPERS, STORE, Trouble};

pub const NAMED: &str = "tisty.toml";
const SEEN: &str = ".shape-seen";
const KEPT_FOR: usize = 8;
const RAISED: &str = ".shape-raised";

/// What this build knows the meeting place to look like. A folder that says a higher number was
/// arranged by a build that knows something this one does not, and guessing at it writes over it.
/// Shape 3 keeps every attachment in one flat folder and every document in Tisty's own words.
pub const OURS: u32 = 3;

/// The oldest release that may write in a place this build arranges; a later change to the place raises it.
pub const NEEDS: &str = "1.31.0";

enum Told {
    Shape { shape: u32, needs: Option<String> },
    Nothing,
    Unreadable,
}

fn told(dest: &Path) -> Told {
    match std::fs::read_to_string(dest.join(NAMED)) {
        Ok(said) => match said.lines().find_map(shape_in) {
            Some(shape) => Told::Shape {
                shape,
                needs: said.lines().find_map(needs_in),
            },
            None => Told::Unreadable,
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Told::Nothing,
        Err(_) => Told::Unreadable,
    }
}

fn shape_in(line: &str) -> Option<u32> {
    let (name, said) = line.split_once('=')?;
    match name.trim() == "shape" {
        true => said.trim().parse().ok(),
        false => None,
    }
}

fn needs_in(line: &str) -> Option<String> {
    let (name, said) = line.split_once('=')?;
    (name.trim() == "needs").then(|| said.trim().trim_matches('"').to_string())
}

fn release(said: &str) -> Option<(u64, u64, u64)> {
    let mut parts = said.split(['-', '+']).next()?.split('.');
    let mut next = || parts.next()?.parse().ok();
    Some((next()?, next()?, next()?))
}

/// A release this build cannot read is taken for a later one: guessing it older would write where it must not.
pub fn needs_later(needs: &str) -> bool {
    match (release(needs), release(NEEDS)) {
        (Some(needs), Some(ours)) => needs > ours,
        _ => true,
    }
}

/// Read before anything else in the folder is. A folder that says nothing may be one from before
/// this file existed, and is adopted; one that said something to this machine and has gone quiet
/// is one where a round did not finish or a hand went through, and its silence is never read as
/// «there is nothing here».
pub(crate) fn before_reading(data: &Path, dest: &Path) -> Result<(), Trouble> {
    match told(dest) {
        Told::Shape {
            needs: Some(needs), ..
        } if needs_later(&needs) => {
            witness::warn(
                channel::SYNC,
                "the meeting place asks for a later Tisty than this one, so nothing was carried either way",
                &[
                    ("at", Fact::Path(dest.to_path_buf())),
                    ("needs", Fact::Id(needs.clone())),
                ],
            );
            Err(Trouble::Needs(needs))
        }
        Told::Shape { shape, .. } if shape > OURS => {
            witness::warn(
                channel::SYNC,
                "the meeting place is in a shape this build does not know, so nothing was carried either way",
                &[
                    ("at", Fact::Path(dest.to_path_buf())),
                    ("shape", Fact::Count(shape as usize)),
                ],
            );
            Err(Trouble::Shape(dest.display().to_string()))
        }
        Told::Shape { shape, .. } => {
            remember(data, dest, shape);
            if shape < OURS {
                flat(dest);
            }
            Ok(())
        }
        Told::Unreadable => {
            witness::warn(
                channel::SYNC,
                "the meeting place says what shape it is in and it cannot be read, so nothing was carried either way",
                &[("at", Fact::Path(dest.join(NAMED)))],
            );
            Err(Trouble::Unshaped(dest.display().to_string()))
        }
        Told::Nothing if seen(data, dest).is_some() => {
            witness::warn(
                channel::SYNC,
                "the meeting place said what shape it was in and now says nothing, so nothing was read from it",
                &[("at", Fact::Path(dest.to_path_buf()))],
            );
            Err(Trouble::Unshaped(dest.display().to_string()))
        }
        Told::Nothing => {
            flat(dest);
            Ok(())
        }
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
        Told::Shape { shape, needs } if shape == OURS => {
            match needs.is_some_and(|needs| needs != NEEDS && !needs_later(&needs)) {
                true => {
                    tell(data, dest);
                }
                false => remember(data, dest, OURS),
            }
            false
        }
        Told::Shape { shape, .. } if shape > OURS => false,
        Told::Unreadable => false,
        Told::Shape { .. } => tell(data, dest),
        Told::Nothing => {
            tell(data, dest);
            false
        }
    }
}

/// Raised by a machine as it leaves, so the builds before this one stop there even without a round of its own.
pub fn raise_place(dest: &Path) {
    let older = match told(dest) {
        Told::Shape { shape, .. } => shape < OURS,
        Told::Nothing => dest.join(STORE).is_dir(),
        Told::Unreadable => false,
    };
    if older {
        written(dest);
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

fn tell(data: &Path, dest: &Path) -> bool {
    let done = written(dest);
    if done {
        remember(data, dest, OURS);
    }
    done
}

fn written(dest: &Path) -> bool {
    let said = format!(
        "# What this folder is, and the shape it is in. Tisty reads this before anything else here.\nshape = {OURS}\nneeds = \"{NEEDS}\"\nfolders = [\"{STORE}\", \"{HELD}\", \"{PAPERS}\"]\n"
    );
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
