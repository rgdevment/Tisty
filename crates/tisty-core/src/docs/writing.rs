use std::path::Path;

use crate::Result;
use crate::history::{Step, Writer};

use super::{
    alone, as_written, print_of, print_of_bytes, read, resolve, settled, whole_of, written,
    written_over,
};

/// One door for every writer: a body read by one is never written under another.
pub fn write(root: &Path, id: &str, body: &str) -> Result<()> {
    alone(root, || written(root, id, body))
}

pub fn write_by(root: &Path, by: &Writer, id: &str, body: &str) -> Result<()> {
    alone(root, || {
        if let Some(was) = whole_of(&resolve(root, id)?)? {
            by.passes(id, &was, body, Step::Stays);
        }
        written(root, id, body)
    })
}

/// The window's save lands only on the body its words were typed over, judged under the same lock.
pub fn write_over(root: &Path, by: &Writer, id: &str, body: &str, print: &str) -> Result<bool> {
    alone(root, || {
        // A body gone from the disk under the window is written back, as any save would.
        let Some(was) = whole_of(&resolve(root, id)?)? else {
            written(root, id, body)?;
            return Ok(true);
        };
        if print_of_bytes(&was) != print {
            return Ok(false);
        }
        by.passes(id, &was, body, Step::Stays);
        written_over(root, id, body, print)
    })
}

pub fn append(root: &Path, by: &Writer, id: &str, body: &str) -> Result<String> {
    alone(root, || {
        let was = read(root, id)?;
        let ending = match was.contains("\r\n") {
            true => "\r\n",
            false => "\n",
        };
        let flat = settled(body.trim_start_matches(['\n', '\r'])).replace("\r\n", "\n");
        let added = match ending {
            "\r\n" => flat.replace('\n', "\r\n"),
            _ => flat,
        };
        let whole = match was.trim_end().is_empty() {
            true => added,
            false => format!("{}{ending}{ending}{added}", was.trim_end()),
        };
        by.passes(id, was.as_bytes(), &whole, Step::Stays);
        written(root, id, &whole)?;
        Ok(whole)
    })
}

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    Made { was: String, whole: String },
    Missing,
    Twice(usize),
    TheLot,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Rewrite {
    Made { was: String, whole: String },
    Moved,
}

/// What it said before is kept first: a rewrite that cannot be undone is not written at all.
pub fn rewrite(root: &Path, by: &Writer, id: &str, body: &str, print: &str) -> Result<Rewrite> {
    alone(root, || {
        let at = resolve(root, id)?;
        if print_of(&at)?.as_deref() != Some(print) {
            return Ok(Rewrite::Moved);
        }
        let was = read(root, id)?;
        by.replaces(id, was.as_bytes(), body, Step::Back)?;
        if !written_over(root, id, body, print)? {
            return Ok(Rewrite::Moved);
        }
        Ok(Rewrite::Made {
            was,
            whole: settled(body),
        })
    })
}

/// Written over the body it was read from, with no step back kept: going back would name what is gone.
pub fn rewrite_for_good(
    root: &Path,
    by: &Writer,
    id: &str,
    body: &str,
    print: &str,
) -> Result<Rewrite> {
    alone(root, || {
        let at = resolve(root, id)?;
        if print_of(&at)?.as_deref() != Some(print) {
            return Ok(Rewrite::Moved);
        }
        let was = read(root, id)?;
        by.passes(id, was.as_bytes(), body, Step::Stays);
        written(root, id, body)?;
        Ok(Rewrite::Made {
            was,
            whole: settled(body),
        })
    })
}

pub fn edit(root: &Path, by: &Writer, id: &str, old: &str, new: &str) -> Result<Change> {
    if old.is_empty() {
        return Ok(Change::Missing);
    }
    alone(root, || {
        let was = read(root, id)?;
        let (old, new) = as_written(&was, old, new);
        // A rewrite wearing an edit's clothes: no tool hands a document a new body.
        if was.trim() == old.trim() {
            return Ok(Change::TheLot);
        }
        match was.matches(old.as_str()).count() {
            0 => Ok(Change::Missing),
            1 => {
                let whole = was.replacen(old.as_str(), new.as_str(), 1);
                by.replaces(id, was.as_bytes(), &whole, Step::Back)?;
                written(root, id, &whole)?;
                Ok(Change::Made { was, whole })
            }
            many => Ok(Change::Twice(many)),
        }
    })
}

/// Work out the new body under the same lock that writes it, so nothing slips in between.
pub fn amend(
    root: &Path,
    by: &Writer,
    id: &str,
    make: impl FnOnce(&str) -> Option<String>,
) -> Result<Option<String>> {
    amended(root, by, Step::Back, id, make)
}

/// No step back is kept, so going back never takes this write off unseen.
pub fn amend_unkept(
    root: &Path,
    by: &Writer,
    id: &str,
    make: impl FnOnce(&str) -> Option<String>,
) -> Result<Option<String>> {
    amended(root, by, Step::Stays, id, make)
}

fn amended(
    root: &Path,
    by: &Writer,
    step: Step,
    id: &str,
    make: impl FnOnce(&str) -> Option<String>,
) -> Result<Option<String>> {
    alone(root, || {
        let was = read(root, id)?;
        let Some(whole) = make(&was) else {
            return Ok(None);
        };
        match step {
            Step::Back => by.replaces(id, was.as_bytes(), &whole, step)?,
            _ => by.passes(id, was.as_bytes(), &whole, step),
        }
        written(root, id, &whole)?;
        Ok(Some(settled(&whole)))
    })
}
