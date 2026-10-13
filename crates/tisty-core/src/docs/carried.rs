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

fn as_settled(bytes: &[u8]) -> std::borrow::Cow<'_, [u8]> {
    if bytes.is_empty() || bytes.last() == Some(&b'\n') {
        return std::borrow::Cow::Borrowed(bytes);
    }
    let mut settled = bytes.to_vec();
    settled.push(b'\n');
    std::borrow::Cow::Owned(settled)
}

fn past_the_ceiling() -> std::io::Error {
    std::io::Error::other("a body past the ceiling")
}

fn read_within(from: impl std::io::Read, ceiling: u64) -> std::io::Result<Vec<u8>> {
    use std::io::Read;

    let mut bytes = Vec::new();
    from.take(ceiling + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > ceiling {
        return Err(past_the_ceiling());
    }
    Ok(bytes)
}

fn whole_of(at: &Path) -> std::io::Result<Option<Vec<u8>>> {
    let file = match std::fs::File::open(at) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    if file.metadata()?.len() > BODY_AT_MOST {
        return Err(past_the_ceiling());
    }
    crate::counting::opened();
    read_within(file, BODY_AT_MOST).map(Some)
}

fn printed_body(bytes: &[u8]) -> String {
    crate::attach::printed(&as_settled(bytes))
}

pub fn print_of(at: &Path) -> std::io::Result<Option<String>> {
    Ok(whole_of(at)?.map(|bytes| printed_body(&bytes)))
}

pub fn read_as_printed(at: &Path, print: &str) -> std::io::Result<Option<Vec<u8>>> {
    Ok(whole_of(at)?.filter(|bytes| printed_body(bytes) == print))
}

type Prints = std::collections::BTreeMap<String, String>;

// A move carries to the folder and to the cloud in turns, so each keeps what the two last agreed on.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Elsewhere {
    up_to: String,
    #[serde(default)]
    prints: Prints,
}

const ELSEWHERE_AT_MOST: usize = 3;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Carried {
    #[serde(default)]
    prints: Prints,
    #[serde(default)]
    up_to: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    elsewhere: Vec<Elsewhere>,
}

fn ledger(data: &Path) -> PathBuf {
    data.join("carried.json")
}

fn bases_elsewhere(data: &Path) -> PathBuf {
    data.join("carried-elsewhere")
}

fn bases_of(data: &Path, up_to: &str) -> PathBuf {
    let named = crate::attach::printed(up_to.as_bytes());
    bases_elsewhere(data).join(&named[..named.len().min(16)])
}

pub fn forget_what_was_carried(data: &Path) {
    let _ = std::fs::remove_file(ledger(data));
    let _ = std::fs::remove_dir_all(base(data));
    let _ = std::fs::remove_dir_all(bases_elsewhere(data));
}

fn moved_whole(from: &Path, into: &Path) -> bool {
    let _ = std::fs::remove_dir_all(into);
    if !from.exists() {
        return true;
    }
    if let Some(parent) = into.parent() {
        let _ = std::fs::create_dir_all(parent);
        let _ = crate::paths::ours_alone(parent);
    }
    std::fs::rename(from, into).is_ok()
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
                ..Self::default()
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

    pub fn facing(
        &mut self,
        data: &Path,
        dest: &Path,
        been_here: bool,
        holds_our_store: bool,
    ) -> bool {
        if let Some(kept) = self.up_to.as_deref() {
            if crate::paths::is_the_one(kept, dest) {
                return false;
            }
            if crate::paths::same_folder(kept, dest, holds_our_store) {
                self.up_to = Some(crate::paths::told_of(dest));
                return false;
            }
        }
        if self.up_to.is_none() && been_here && !self.prints.is_empty() {
            self.up_to = Some(crate::paths::told_of(dest));
            return false;
        }
        let back = self
            .elsewhere
            .iter()
            .position(|one| crate::paths::same_folder(&one.up_to, dest, holds_our_store))
            .map(|at| self.elsewhere.remove(at));
        self.set_aside(data);
        if let Some(one) = back {
            match moved_whole(&bases_of(data, &one.up_to), &base(data)) {
                true => self.prints = one.prints,
                false => self.elsewhere.push(one),
            }
        }
        self.up_to = Some(crate::paths::told_of(dest));
        // The bases already moved on disk; a round cut before its end must not find the ledger telling the old place.
        let _ = self.save(data);
        true
    }

    fn set_aside(&mut self, data: &Path) {
        let prints = std::mem::take(&mut self.prints);
        let up_to = self.up_to.take();
        let Some(up_to) = up_to.filter(|_| !prints.is_empty()) else {
            if !prints.is_empty() {
                crate::witness::note(
                    crate::witness::channel::SYNC,
                    "another folder is carried to now, so what was carried before is forgotten",
                    &[("count", crate::witness::Fact::Count(prints.len()))],
                );
            }
            let _ = std::fs::remove_dir_all(base(data));
            return;
        };
        self.elsewhere.retain(|one| one.up_to != up_to);
        if !moved_whole(&base(data), &bases_of(data, &up_to)) {
            let _ = std::fs::remove_dir_all(base(data));
            return;
        }
        self.elsewhere.push(Elsewhere { up_to, prints });
        while self.elsewhere.len() > ELSEWHERE_AT_MOST {
            let gone = self.elsewhere.remove(0);
            let _ = std::fs::remove_dir_all(bases_of(data, &gone.up_to));
        }
    }
}

#[cfg(test)]
#[path = "carried_test.rs"]
mod tests;
