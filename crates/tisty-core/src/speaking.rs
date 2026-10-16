use std::path::Path;

use crate::event::Said;
use crate::witness::{self, Fact, channel};

/// Left in the data folder once a store speaks only Tisty's own words, so it is turned once.
const SPOKEN: &str = "spoken";

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Spoke {
    /// What each turned document now says, for the log to answer for the new bytes.
    pub docs: Vec<(String, Said)>,
    pub moved: usize,
    pub left: usize,
    pub missed: usize,
}

/// What anything outside the window hands in, in Tisty's own words: an older Tisty's tags included.
pub fn turned(body: &str) -> String {
    tisty_markdown::to_tisty(body)
}

pub fn spoken(data: &Path) -> bool {
    data.join(SPOKEN).is_file()
}

/// Every attachment into the one flat folder, and every document into Tisty's own words.
pub fn speak(data: &Path, docs: &Path) -> Spoke {
    let mut done = Spoke::default();
    for shed in [crate::attach::SHED, "bin"] {
        let (moved, left) = flattened(&data.join(shed));
        done.moved += moved;
        done.left += left;
    }
    for doc in crate::docs::all(docs) {
        match turned_one(data, docs, &doc.id) {
            Ok(Some(said)) => done.docs.push((doc.id.clone(), said)),
            Ok(None) => {}
            Err(why) => {
                done.missed += 1;
                witness::warn(
                    channel::STORE,
                    "a document could not be turned into Tisty's own words, so it is tried again next time",
                    &[
                        ("id", Fact::Id(doc.id.clone())),
                        ("why", Fact::Why(why.to_string())),
                    ],
                );
            }
        }
    }
    // Only a whole turn is marked done; whatever stayed behind is tried again on the next opening.
    if done.left == 0
        && done.missed == 0
        && let Err(why) = crate::store::write_atomic(
            &data.join(SPOKEN),
            b"1
",
        )
    {
        witness::warn(
            channel::STORE,
            "the store speaks Tisty's own words but could not be marked so",
            &[("why", Fact::Why(why.to_string()))],
        );
    }
    done
}

fn turned_one(data: &Path, docs: &Path, id: &str) -> crate::Result<Option<Said>> {
    let body = crate::docs::read(docs, id)?;
    let said = tisty_markdown::to_tisty(&body);
    if said == body {
        return Ok(None);
    }
    let print = crate::docs::print_of(&crate::docs::resolve(docs, id)?)?
        .ok_or_else(|| crate::Error::OutsideTheStore(id.to_string()))?;
    match crate::docs::rewrite(docs, data, id, &said, &print)? {
        crate::docs::Rewrite::Made { .. } => Ok(Some(Said::of(&said))),
        crate::docs::Rewrite::Moved => Err(crate::Error::OutsideTheStore(id.to_string())),
    }
}

// A twin already flat is the same bytes by its stamp; anything else in the way stays and is said.
pub fn flattened(shed: &Path) -> (usize, usize) {
    let (mut moved, mut left) = (0, 0);
    let Ok(shelves) = std::fs::read_dir(shed) else {
        return (moved, left);
    };
    for shelf in shelves.filter_map(|one| one.ok()) {
        let named = shelf.file_name();
        let is_shelf = named
            .to_str()
            .is_some_and(|one| one.len() == 2 && crate::attach::lower_hex(one));
        if !is_shelf || !shelf.path().is_dir() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|one| one.ok()) {
            let from = file.path();
            let into = shed.join(file.file_name());
            let same = std::fs::metadata(&into)
                .ok()
                .zip(std::fs::metadata(&from).ok())
                .is_some_and(|(there, here)| there.is_file() && there.len() == here.len());
            let gone = match (into.exists(), same) {
                (false, _) => std::fs::rename(&from, &into).is_ok(),
                (true, true) => std::fs::remove_file(&from).is_ok(),
                (true, false) => false,
            };
            if gone {
                moved += 1;
            } else {
                left += 1;
                witness::warn(
                    channel::ATTACH,
                    "an attachment could not leave its shelf for the one flat folder",
                    &[("at", Fact::Path(from))],
                );
            }
        }
        let _ = std::fs::remove_dir(shelf.path());
    }
    (moved, left)
}

#[cfg(test)]
#[path = "speaking_test.rs"]
mod tests;
