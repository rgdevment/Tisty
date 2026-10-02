use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::store::write_atomic;
use crate::{Error, Result};

use super::{BODY_AT_MOST, base, resolve, settled};

/// The body a write replaced, kept with the print of the body that write left behind. Not every
/// way of writing a document keeps one — the window's own save does not, nor does adding to the
/// end — so the print is what says whether this is still the step back it looks like.
pub fn kept_before(data: &Path, id: &str, body: &str, left: &str) -> Result<()> {
    let at = data.join("originals");
    std::fs::create_dir_all(&at)?;
    let _ = crate::paths::ours_alone(&at);
    let into = resolve(&at, id)?;
    write_atomic(&into, body.as_bytes())?;
    let _ = crate::paths::ours_alone(&into);

    let marked = data.join("originals-at");
    std::fs::create_dir_all(&marked)?;
    let _ = crate::paths::ours_alone(&marked);
    let into = resolve(&marked, id)?;
    // What reaches the disk is the settled body, so hashing what was handed in would leave the
    // print of a text that was never written and go back on nothing.
    write_atomic(
        &into,
        crate::attach::printed(settled(left).as_bytes()).as_bytes(),
    )?;
    let _ = crate::paths::ours_alone(&into);
    Ok(())
}

/// What the document read at when what is kept beside it was set aside.
pub fn before_left_at(data: &Path, id: &str) -> Option<String> {
    let at = resolve(&data.join("originals-at"), id).ok()?;
    std::fs::read_to_string(at).ok()
}

pub fn keep_carried(data: &Path, id: &str, body: &str) -> Result<()> {
    let at = base(data);
    std::fs::create_dir_all(&at)?;
    let _ = crate::paths::ours_alone(&at);
    let into = resolve(&at, id)?;
    write_atomic(&into, body.as_bytes())?;
    let _ = crate::paths::ours_alone(&into);
    Ok(())
}

pub fn carried_at(data: &Path, id: &str) -> Option<PathBuf> {
    resolve(&base(data), id).ok()
}

pub fn carried_print(data: &Path, id: &str) -> Option<String> {
    resolve(&base(data), id)
        .ok()
        .and_then(|at| print_of(&at).ok()?)
}

pub fn read_carried(data: &Path, id: &str) -> Option<String> {
    let at = resolve(&base(data), id).ok()?;
    std::fs::read_to_string(at).ok()
}

pub fn forget_carried(data: &Path, id: &str) {
    if let Ok(at) = resolve(&base(data), id) {
        let _ = std::fs::remove_file(at);
    }
}

pub fn read_before(data: &Path, id: &str) -> Option<String> {
    let at = resolve(&data.join("originals"), id).ok()?;
    std::fs::read_to_string(at).ok()
}

fn as_settled(mut bytes: Vec<u8>) -> Vec<u8> {
    if !bytes.is_empty() && bytes.last() != Some(&b'\n') {
        bytes.push(b'\n');
    }
    bytes
}

pub fn print_of(at: &Path) -> std::io::Result<Option<String>> {
    match std::fs::metadata(at) {
        Ok(one) if one.len() > BODY_AT_MOST => {
            return Err(std::io::Error::other("a body past the ceiling"));
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    }
    match std::fs::read(at) {
        Ok(bytes) => {
            crate::counting::opened();
            Ok(Some(crate::attach::printed(&as_settled(bytes))))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Carried {
    #[serde(default)]
    prints: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    up_to: Option<String>,
}

fn ledger(data: &Path) -> PathBuf {
    data.join("carried.json")
}

pub fn forget_what_was_carried(data: &Path) {
    let _ = std::fs::remove_file(ledger(data));
    let _ = std::fs::remove_dir_all(base(data));
}

/// A line naming a page is not the person's writing, so it does not take their one step back with
/// it: the body kept beside the document stays, and only the print of what it stands against moves.
/// A step back already spent stays spent — carrying that one forward would offer to undo whatever
/// spent it, which is somebody's writing.
pub(super) fn kept_still(data: &Path, id: &str, was: &str, left: &str) -> Result<()> {
    let stood = crate::attach::printed(settled(was).as_bytes());
    if before_left_at(data, id).as_deref() != Some(stood.as_str()) {
        return Ok(());
    }
    let Ok(into) = resolve(&data.join("originals-at"), id) else {
        return Ok(());
    };
    write_atomic(
        &into,
        crate::attach::printed(settled(left).as_bytes()).as_bytes(),
    )
}

impl Carried {
    pub fn read(data: &Path) -> Self {
        let Ok(said) = std::fs::read_to_string(ledger(data)) else {
            return Self::default();
        };
        if let Ok(prints) = serde_json::from_str(&said) {
            return Self {
                prints,
                up_to: None,
            };
        }
        serde_json::from_str(&said).unwrap_or_default()
    }

    pub fn save(&self, data: &Path) -> Result<()> {
        let said = serde_json::to_string(self).map_err(|e| Error::Io(std::io::Error::other(e)))?;
        write_atomic(&ledger(data), said.as_bytes())?;
        let _ = crate::paths::ours_alone(&ledger(data));
        Ok(())
    }

    pub fn of(&self, id: &str) -> Option<&str> {
        self.prints.get(id).map(String::as_str)
    }

    pub fn keep(&mut self, id: &str, print: &str) {
        self.prints.insert(id.to_string(), print.to_string());
    }

    pub fn forget(&mut self, id: &str) {
        self.prints.remove(id);
    }

    pub fn facing(&mut self, dest: &Path, been_here: bool) -> bool {
        if self
            .up_to
            .as_deref()
            .is_some_and(|kept| crate::paths::is_the_one(kept, dest))
        {
            return false;
        }
        if self.up_to.is_none() && been_here && !self.prints.is_empty() {
            self.up_to = Some(crate::paths::told_of(dest));
            return false;
        }
        self.prints.clear();
        self.up_to = Some(crate::paths::told_of(dest));
        true
    }
}
