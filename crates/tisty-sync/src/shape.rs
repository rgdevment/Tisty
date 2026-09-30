use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{HELD, PAPERS, STORE, Trouble};

pub(crate) const NAMED: &str = "tisty.toml";
const SEEN: &str = ".shape-seen";

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
    (name.trim() == "shape").then(|| said.trim().parse().ok())?
}

/// Read before anything else in the folder is. A folder that says nothing may be one from before
/// this file existed, and is adopted; one that said something and has gone quiet is one where a
/// round did not finish or a hand went through, and its silence is never read as «there is
/// nothing here».
pub(crate) fn before_reading(data: &Path, dest: &Path) -> Result<(), Trouble> {
    match told(dest) {
        Told::Shape(said) if said > OURS => {
            witness::warn(
                channel::SYNC,
                "the meeting place is in a shape this build does not know, so nothing was carried either way",
                &[("shape", Fact::Count(said as usize))],
            );
            Err(Trouble::Shape(said.to_string()))
        }
        Told::Shape(said) => {
            remember(data, said);
            Ok(())
        }
        Told::Unreadable => {
            witness::warn(
                channel::SYNC,
                "the meeting place says what shape it is in and it cannot be read, so nothing was carried either way",
                &[("at", Fact::Path(dest.join(NAMED)))],
            );
            Err(Trouble::Unshaped(NAMED.to_string()))
        }
        Told::Nothing if seen(data).is_some() && holds_anything(dest) => {
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
    remember(data, OURS);
}

fn holds_anything(dest: &Path) -> bool {
    [STORE, HELD, PAPERS].iter().any(|folder| {
        std::fs::read_dir(dest.join(folder)).is_ok_and(|mut held| held.next().is_some())
    })
}

fn seen(data: &Path) -> Option<u32> {
    std::fs::read_to_string(data.join(SEEN))
        .ok()?
        .trim()
        .parse()
        .ok()
}

fn remember(data: &Path, said: u32) {
    if seen(data) == Some(said) {
        return;
    }
    let _ = tisty_core::store::write_atomic(&data.join(SEEN), said.to_string().as_bytes());
}

#[cfg(test)]
#[path = "shape_test.rs"]
mod tests;
