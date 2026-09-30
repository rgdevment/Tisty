use std::path::Path;

use tisty_core::config::Holds;
use tisty_core::witness::{self, Fact, channel};

use crate::{HELD, LetGo, Trouble, beside, plainly, sweep};

/// Deletes a local copy only after the one up there is found to hash the same. `told` hears each
/// one as it goes and answers whether to carry on.
/// What a round just put up there, checked by its size where it landed: the bytes were hashed on
/// the way and the name was renamed into place, so reading it back would only ask the cloud for
/// what we wrote a second ago.
pub(crate) fn let_go_of(data: &Path, dest: &Path, carried: &[(String, u64)], above: u64) -> u64 {
    let mut freed = 0;
    let told = tisty_core::attach::digests(data);
    for (reference, bytes) in carried {
        if *bytes <= above {
            continue;
        }
        let (Ok(here), Ok(there)) = (
            tisty_core::attach::resolve(reference, data),
            tisty_core::attach::resolve(reference, dest),
        ) else {
            continue;
        };
        if !landed_whole(&there, told.get(reference), *bytes, reference) {
            witness::warn(
                channel::ATTACH,
                "the copy up there is not one we can answer for, so the one here is kept",
                &[("at", Fact::Id(reference.clone()))],
            );
            continue;
        }
        if std::fs::remove_file(&here).is_ok() {
            freed += bytes;
        }
    }
    freed
}

pub(crate) fn landed_whole(
    there: &Path,
    told: Option<&(String, u64)>,
    bytes: u64,
    reference: &str,
) -> bool {
    if !std::fs::metadata(there).is_ok_and(|one| one.is_file() && one.len() == bytes) {
        return false;
    }
    if tisty_core::holes::a_hole(there) {
        return false;
    }
    let Ok((sha256, _)) = tisty_core::attach::hashed(there) else {
        return false;
    };
    match told {
        Some((ours, _)) => ours.eq_ignore_ascii_case(&sha256),
        None => shelved_as(reference)
            .is_some_and(|(under, named)| tisty_core::attach::vouched(under, named, &sha256)),
    }
}

pub(crate) fn shelved_as(reference: &str) -> Option<(&str, &str)> {
    let rest = reference.strip_prefix("attachments/")?;
    rest.split_once('/')
}

pub fn let_go_telling(
    data: &Path,
    dest: &Path,
    above: u64,
    told: &mut dyn FnMut(&LetGo) -> bool,
) -> Result<LetGo, Trouble> {
    let mut done = LetGo::default();
    let shed = data.join(HELD);
    let Ok(shelves) = std::fs::read_dir(&shed) else {
        return Ok(done);
    };
    for shelf in shelves.filter_map(|one| one.ok()) {
        if !shelf.path().is_dir() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|one| one.ok()) {
            let at = file.path();
            let Ok(about) = std::fs::metadata(&at) else {
                continue;
            };
            let weighs = about.len();
            if !about.is_file() || weighs <= above {
                continue;
            }
            let under = shelf.file_name();
            let (Some(under), Some(named)) =
                (under.to_str(), at.file_name().and_then(|n| n.to_str()))
            else {
                continue;
            };
            let reference = format!("attachments/{under}/{named}");
            match twinned(
                &dest.join(HELD).join(under).join(named),
                weighs,
                under,
                named,
            ) {
                true => {
                    if std::fs::remove_file(&at).is_ok() {
                        done.gone += 1;
                        done.freed += weighs;
                    }
                }
                false => done.kept.push(reference),
            }
            if !told(&done) {
                return Ok(done);
            }
        }
    }
    Ok(done)
}

pub(crate) fn twinned(there: &Path, weighs: u64, under: &str, named: &str) -> bool {
    if !std::fs::metadata(there).is_ok_and(|told| told.is_file() && told.len() == weighs) {
        return false;
    }
    if tisty_core::holes::a_hole(there) {
        return false;
    }
    tisty_core::attach::hashed(there)
        .is_ok_and(|(sha256, _)| tisty_core::attach::vouched(under, named, &sha256))
}

pub(crate) fn left_behind(holds: Holds) -> Option<u64> {
    match holds {
        Holds::Everywhere => None,
        Holds::Mine | Holds::Shared => Some(tisty_core::attach::COPIED_UP_TO),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn copy_held(
    from: &Path,
    into: &Path,
    buried: &std::collections::BTreeSet<String>,
    again: bool,
    ledger: Option<&Path>,
    above: Option<u64>,
    reachable: Option<&std::collections::BTreeSet<String>>,
    carried: Option<&mut Vec<(String, u64)>>,
) -> Result<usize, Trouble> {
    let mut done = 0;
    let mut left = 0;
    let mut carried = carried;
    let written_down = ledger.map(tisty_core::attach::digests).unwrap_or_default();
    let shelves = match std::fs::read_dir(from) {
        Ok(shelves) => shelves,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::SYNC,
                    "attachments unreadable",
                    &[
                        ("at", Fact::Path(from.to_path_buf())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
            return Ok(0);
        }
    };
    for shelf in shelves.filter_map(|e| e.ok()) {
        if !shelf.path().is_dir() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        let onto = into.join(shelf.file_name());
        plainly(&onto)?;
        sweep(&onto);
        for file in files.filter_map(|e| e.ok()) {
            let at = file.path();
            if !at.is_file() {
                continue;
            }
            let named = at
                .file_name()
                .and_then(|one| one.to_str())
                .unwrap_or_default();
            let under = shelf.file_name();
            let under = under.to_str().unwrap_or_default();
            // What iCloud left in place of a file is not litter, and saying so would bury the log.
            if tisty_core::holes::marker(named) {
                continue;
            }
            if !tisty_core::attach::shelved(under, named) {
                witness::warn(
                    channel::SYNC,
                    "something in the shared folder is not shaped like an attachment",
                    &[("at", Fact::Id(format!("attachments/{under}/{named}")))],
                );
                continue;
            }
            let reference = format!("attachments/{under}/{named}");
            if buried.contains(&reference) {
                witness::note(
                    channel::SYNC,
                    "a retired attachment was left where it was instead of coming back",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            }
            if reachable.is_some_and(|named| !named.contains(&reference)) {
                left += 1;
                continue;
            }
            let weighs = std::fs::metadata(&at).map(|m| m.len()).unwrap_or(0);
            if above.is_some_and(|most| weighs > most) {
                continue;
            }
            if weighs > tisty_core::attach::COPIED_IN_DOC {
                witness::warn(
                    channel::SYNC,
                    "something in the shared folder is past what any attachment may weigh",
                    &[("at", Fact::Id(format!("attachments/{under}/{named}")))],
                );
                continue;
            }
            let Some(rest) = at.strip_prefix(from).ok() else {
                continue;
            };
            let target = into.join(rest);
            if !again
                && std::fs::metadata(&at).map(|m| m.len()).ok()
                    == std::fs::metadata(&target).map(|m| m.len()).ok()
            {
                continue;
            }
            let reference = format!("attachments/{under}/{named}");
            let part = beside(&target);
            let ferried = tisty_core::attach::copied(&at, &part, tisty_core::attach::COPIED_IN_DOC);
            let Ok((sha256, bytes)) = ferried else {
                let _ = std::fs::remove_file(&part);
                witness::warn(
                    channel::SYNC,
                    "an attachment could not be carried",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            };
            if !tisty_core::attach::vouched(under, named, &sha256) {
                let _ = std::fs::remove_file(&part);
                witness::warn(
                    channel::SYNC,
                    "an attachment does not hold the bytes its name vouches for",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            }
            if !tisty_core::attach::as_kept(&written_down, &reference, &sha256) {
                let _ = std::fs::remove_file(&part);
                witness::warn(
                    channel::SYNC,
                    "an attachment we already kept came back holding other bytes",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            }
            if std::fs::rename(&part, &target).is_err() {
                let _ = std::fs::remove_file(&part);
                witness::warn(
                    channel::SYNC,
                    "file not carried",
                    &[("at", Fact::Path(target.clone()))],
                );
                continue;
            }
            if let Some(ledger) = ledger {
                tisty_core::attach::noted(ledger, &reference, &sha256, bytes);
            }
            if let Some(carried) = carried.as_deref_mut() {
                carried.push((reference, bytes));
            }
            done += 1;
        }
    }
    if left > 0 {
        witness::note(
            channel::SYNC,
            "this folder's history does not account for its documents, so files nothing names were left up there",
            &[("left", Fact::Count(left))],
        );
    }
    Ok(done)
}
