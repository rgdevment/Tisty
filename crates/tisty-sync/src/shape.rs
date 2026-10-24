use std::path::Path;

use tisty_core::witness::{self, Fact, channel};

use crate::{HELD, PAPERS, STORE, Trouble};

pub const NAMED: &str = "tisty.toml";
const SEEN: &str = ".shape-seen";
const KEPT_FOR: usize = 8;
const RAISED: &str = ".shape-raised";
const HEADING: &str =
    "# What this folder is, and the shape it is in. Tisty reads this before anything else here.";
const OWN: [&str; 3] = ["shape", "needs", "folders"];

/// What this build knows the meeting place to look like. A folder that says a higher number was
/// arranged by a build that knows something this one does not, and guessing at it writes over it.
/// Shape 3 keeps every attachment in one flat folder and every document in Tisty's own words.
pub const OURS: u32 = 3;

/// The oldest release that may write in a place this build arranges; a later change to the place raises it.
pub const NEEDS: &str = "1.31.0";

enum Told {
    Shape {
        shape: u32,
        needs: Option<String>,
        others: String,
    },
    Nothing,
    Unreadable,
    Unreached,
}

fn told(dest: &Path) -> Told {
    let bytes = match std::fs::read(dest.join(NAMED)) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Told::Nothing,
        Err(_) => return Told::Unreached,
    };
    let Ok(said) = String::from_utf8(bytes) else {
        return Told::Unreadable;
    };
    let (top, tables) = entries(&said);
    match top.iter().find_map(|entry| shape_in(entry)) {
        Some(shape) => Told::Shape {
            shape,
            needs: top.iter().find_map(|entry| needs_in(entry)),
            others: said_by_others(&top, tables),
        },
        None => Told::Unreadable,
    }
}

fn entries(said: &str) -> (Vec<String>, &str) {
    let mut top: Vec<String> = Vec::new();
    let (mut depth, mut at) = (0, 0);
    for line in said.split_inclusive('\n') {
        match top.last_mut() {
            Some(entry) if depth > 0 => entry.push_str(line),
            _ if line.trim_start().starts_with('[') => return (top, &said[at..]),
            _ => top.push(line.to_string()),
        }
        depth = (depth + opened(line)).max(0);
        at += line.len();
    }
    (top, "")
}

fn opened(line: &str) -> i32 {
    let (mut depth, mut quoted, mut escaped) = (0, false, false);
    for one in line.chars() {
        match one {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '#' if !quoted => break,
            '[' if !quoted => depth += 1,
            ']' if !quoted => depth -= 1,
            _ => {}
        }
    }
    depth
}

fn said_by_others(top: &[String], tables: &str) -> String {
    let mut kept = String::new();
    for entry in top {
        let ours = entry
            .split_once('=')
            .is_some_and(|(name, _)| OWN.contains(&name.trim()));
        if ours || entry.trim().is_empty() || entry.trim_end() == HEADING {
            continue;
        }
        kept.push_str(entry.trim_end_matches('\n'));
        kept.push('\n');
    }
    kept.push_str(tables);
    if !kept.is_empty() && !kept.ends_with('\n') {
        kept.push('\n');
    }
    kept
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
        Told::Shape { shape, needs, .. } => {
            if let Err(trouble) = barred(dest, shape, needs) {
                warned(dest, shape, &trouble);
                return Err(trouble);
            }
            remember(data, dest, shape);
            if shape < OURS {
                flat(dest);
            }
            Ok(())
        }
        Told::Unreadable | Told::Unreached => {
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

pub fn refuses(dest: &Path) -> Result<(), Trouble> {
    match told(dest) {
        Told::Shape { shape, needs, .. } => barred(dest, shape, needs),
        Told::Unreadable => Err(Trouble::Unshaped(dest.display().to_string())),
        Told::Unreached => Err(Trouble::NotThere(dest.display().to_string())),
        Told::Nothing => Ok(()),
    }
}

fn barred(dest: &Path, shape: u32, needs: Option<String>) -> Result<(), Trouble> {
    match needs.filter(|needs| needs_later(needs)) {
        Some(needs) => Err(Trouble::Needs(needs)),
        None if shape > OURS => Err(Trouble::Shape(dest.display().to_string())),
        None => Ok(()),
    }
}

fn warned(dest: &Path, shape: u32, trouble: &Trouble) {
    match trouble {
        Trouble::Needs(needs) => witness::warn(
            channel::SYNC,
            "the meeting place asks for a later Tisty than this one, so nothing was carried either way",
            &[
                ("at", Fact::Path(dest.to_path_buf())),
                ("needs", Fact::Id(needs.clone())),
            ],
        ),
        _ => witness::warn(
            channel::SYNC,
            "the meeting place is in a shape this build does not know, so nothing was carried either way",
            &[
                ("at", Fact::Path(dest.to_path_buf())),
                ("shape", Fact::Count(shape as usize)),
            ],
        ),
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
        Told::Shape {
            shape,
            needs,
            others,
        } if shape == OURS => {
            match needs.is_some_and(|needs| needs != NEEDS && !needs_later(&needs)) {
                true => {
                    tell(data, dest, &others);
                }
                false => remember(data, dest, OURS),
            }
            false
        }
        Told::Shape { shape, .. } if shape > OURS => false,
        Told::Unreadable | Told::Unreached => false,
        Told::Shape { others, .. } => tell(data, dest, &others),
        Told::Nothing => {
            tell(data, dest, "");
            false
        }
    }
}

/// Raised before anything is given, so an older build stops before it can take what it cannot read.
/// Only a place that already says its shape is raised this early: a new one is still named last.
pub(crate) fn raise_first(data: &Path, dest: &Path) -> bool {
    match told(dest) {
        Told::Shape {
            shape,
            needs,
            others,
        } if shape < OURS
            || (shape == OURS
                && needs
                    .as_deref()
                    .is_some_and(|needs| needs != NEEDS && !needs_later(needs))) =>
        {
            tell(data, dest, &others)
        }
        _ => false,
    }
}

/// Raised by a machine as it leaves, so the builds before this one stop there even without a round of its own.
pub fn raise_place(dest: &Path) {
    match told(dest) {
        Told::Shape { shape, others, .. } if shape < OURS => {
            written(dest, &others);
        }
        Told::Nothing if dest.join(STORE).is_dir() => {
            written(dest, "");
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

fn tell(data: &Path, dest: &Path, others: &str) -> bool {
    let done = written(dest, others);
    if done {
        remember(data, dest, OURS);
    }
    done
}

fn written(dest: &Path, others: &str) -> bool {
    let said = format!(
        "{HEADING}\nshape = {OURS}\nneeds = \"{NEEDS}\"\nfolders = [\"{STORE}\", \"{HELD}\", \"{PAPERS}\"]\n{others}"
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
