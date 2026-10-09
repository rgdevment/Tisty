use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{HELD, PAPERS, STORE, Trouble};

pub const NAMED: &str = "tisty.toml";
const SEEN: &str = ".shape-seen";
const KEPT_FOR: usize = 8;

/// What this build knows the meeting place to look like. A folder that says a higher number was
/// arranged by a build that knows something this one does not, and guessing at it writes over it.
pub(crate) const OURS: u32 = 1;

enum Told {
    Shape(u32),
    Nothing,
    Unreadable,
}

fn told(dest: &Path) -> Told {
    match std::fs::read_to_string(dest.join(NAMED)) {
        Ok(said) => said
            .lines()
            .find_map(shape_in)
            .map_or(Told::Unreadable, Told::Shape),
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

/// Read before anything else in the folder is. A folder that says nothing may be one from before
/// this file existed, and is adopted; one that said something to this machine and has gone quiet
/// is one where a round did not finish or a hand went through, and its silence is never read as
/// «there is nothing here».
pub(crate) fn before_reading(data: &Path, dest: &Path) -> Result<(), Trouble> {
    match told(dest) {
        Told::Shape(said) if said > OURS => {
            witness::warn(
                channel::SYNC,
                "the meeting place is in a shape this build does not know, so nothing was carried either way",
                &[
                    ("at", Fact::Path(dest.to_path_buf())),
                    ("shape", Fact::Count(said as usize)),
                ],
            );
            Err(Trouble::Shape(dest.display().to_string()))
        }
        Told::Shape(said) => {
            remember(data, dest, said);
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
        Told::Nothing => Ok(()),
    }
}

/// Written after everything else a round writes, so finding it is finding a round that finished.
pub(crate) fn stamp(data: &Path, dest: &Path) {
    let said = format!(
        "# What this folder is, and the shape it is in. Tisty reads this before anything else here.\nshape = {OURS}\nfolders = [\"{STORE}\", \"{HELD}\", \"{PAPERS}\"]\n"
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
        return;
    }
    remember(data, dest, OURS);
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
