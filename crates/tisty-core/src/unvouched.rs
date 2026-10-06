use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::State;
use crate::attach::{hashed, names_an_attachment, resolve, vouched};
use crate::event::Held;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Unvouched {
    pub at: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vouch {
    Held { held: Held, here: bool },
    Unlike,
    Gone,
}

/// What a task or a document points at, lies here or in the folder, and no history answers for.
pub fn unvouched(state: &State, named: &[String], places: &[&Path]) -> Vec<Unvouched> {
    let wanted: BTreeSet<&str> = named
        .iter()
        .map(String::as_str)
        .filter(|at| names_an_attachment(at) && !state.kept.contains_key(*at))
        .collect();
    wanted
        .into_iter()
        .filter_map(|at| {
            let lies = found(at, places)?;
            let bytes = std::fs::metadata(&lies).ok()?.len();
            Some(Unvouched {
                at: at.to_string(),
                bytes,
            })
        })
        .collect()
}

/// Reads the whole file, so a name that only looks right is never written down as an answer.
/// `here` is whether it lies in the first place given, this machine's own store.
pub fn vouch(at: &str, places: &[&Path]) -> crate::Result<Vouch> {
    if !names_an_attachment(at) {
        return Ok(Vouch::Unlike);
    }
    let Some((lies, first)) = found_where(at, places) else {
        return Ok(Vouch::Gone);
    };
    let (sha256, bytes) = hashed(&lies)?;
    let mut parts = at.rsplit('/');
    let (Some(leaf), Some(shelf)) = (parts.next(), parts.next()) else {
        return Ok(Vouch::Unlike);
    };
    if !vouched(shelf, leaf, &sha256) {
        return Ok(Vouch::Unlike);
    }
    Ok(Vouch::Held {
        held: Held {
            at: at.to_string(),
            sha256,
            bytes,
        },
        here: first == 0,
    })
}

fn found(at: &str, places: &[&Path]) -> Option<PathBuf> {
    found_where(at, places).map(|(lies, _)| lies)
}

fn found_where(at: &str, places: &[&Path]) -> Option<(PathBuf, usize)> {
    places.iter().enumerate().find_map(|(spot, root)| {
        resolve(at, root)
            .ok()
            .filter(|one| one.is_file())
            .map(|one| (one, spot))
    })
}

#[cfg(test)]
#[path = "unvouched_test.rs"]
mod tests;
