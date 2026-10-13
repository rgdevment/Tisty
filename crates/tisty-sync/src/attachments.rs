use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use tisty_core::config::Holds;

use crate::{
    Deferred, HELD, NAMED_AT_MOST, Reached, Reason, Trouble, copy_held, left_behind, let_go_of,
};

pub struct Giving<'a> {
    pub data: &'a Path,
    pub buried: &'a BTreeSet<String>,
    pub avowed: &'a BTreeMap<String, (String, u64)>,
    pub again: bool,
    pub holds: Holds,
    pub saying: &'a mut dyn FnMut(Reached),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Given {
    pub sent: usize,
    pub freed: u64,
    pub let_go: Vec<String>,
    /// What could not go and stays here, so the person is told rather than left waiting elsewhere.
    pub unsent: Option<Deferred>,
}

pub struct Taking<'a> {
    pub data: &'a Path,
    pub buried: &'a BTreeSet<String>,
    pub avowed: &'a BTreeMap<String, (String, u64)>,
    pub reachable: Option<&'a BTreeSet<String>>,
    pub holds: Holds,
    pub saying: &'a mut dyn FnMut(Reached),
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Taken {
    pub brought: usize,
    pub took_in: Vec<(String, String, u64)>,
}

pub trait Attachments {
    fn give(&mut self, round: Giving) -> Result<Given, Trouble>;
    fn take(&mut self, round: Taking) -> Result<Taken, Trouble>;
    fn reaches(&self, reference: &str) -> bool;
    fn takes(&self) -> bool {
        true
    }
}

// Whatever is here and not there after a copy did not go: unreadable, not what its name says, or too big.
fn unsent_beside(data: &Path, dest: &Path, buried: &BTreeSet<String>) -> Option<Deferred> {
    let mut unsent = Vec::new();
    let Ok(shelves) = std::fs::read_dir(data.join(HELD)) else {
        return None;
    };
    for shelf in shelves.filter_map(|one| one.ok()) {
        let Some(under) = shelf.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|one| one.ok()) {
            let Some(leaf) = file.file_name().to_str().map(str::to_string) else {
                continue;
            };
            let reference = format!("{HELD}/{under}/{leaf}");
            let here = file.metadata().ok().filter(|meta| meta.is_file());
            let there = std::fs::metadata(dest.join(HELD).join(&under).join(&leaf)).ok();
            let missing = here
                .as_ref()
                .is_some_and(|here| there.is_none_or(|there| there.len() != here.len()));
            if missing
                && !leaf.ends_with(".part")
                && !tisty_core::holes::marker(&leaf)
                && tisty_core::attach::shelved(&under, &leaf)
                && !buried.contains(&reference)
                && !tisty_core::holes::a_hole(&file.path())
            {
                unsent.push(reference);
            }
        }
    }
    unsent.sort();
    (!unsent.is_empty()).then(|| Deferred {
        reason: Reason::Unsent,
        left: unsent.len(),
        retry_after: None,
        names: unsent.into_iter().take(NAMED_AT_MOST).collect(),
    })
}

pub struct Beside<'a> {
    pub dest: &'a Path,
}

impl Attachments for Beside<'_> {
    fn give(&mut self, round: Giving) -> Result<Given, Trouble> {
        let mut carried = Vec::new();
        let owed = owed_beside(
            &round.data.join(HELD),
            &self.dest.join(HELD),
            round.buried,
            round.again,
        );
        let mut copied = 0;
        if owed > 0 {
            (round.saying)(Reached::Sending {
                part: 0,
                done: 0,
                whole: owed,
                sent: 0,
                weighs: 0,
            });
        }
        let sent = copy_held(
            &round.data.join(HELD),
            &self.dest.join(HELD),
            round.buried,
            round.again,
            None,
            None,
            None,
            Some(&mut carried),
            round.avowed,
            &mut |far| {
                if let Reached::Kept { .. } = far {
                    copied += 1;
                    (round.saying)(Reached::Sending {
                        part: 0,
                        done: copied,
                        whole: owed,
                        sent: 0,
                        weighs: 0,
                    });
                }
            },
        )?;
        if owed > copied {
            (round.saying)(Reached::Sending {
                part: 0,
                done: owed,
                whole: owed,
                sent: 0,
                weighs: 0,
            });
        }
        let (freed, let_go) = match round.holds {
            Holds::Shared => let_go_of(
                round.data,
                self.dest,
                &carried,
                tisty_core::attach::COPIED_UP_TO,
            ),
            _ => (0, Vec::new()),
        };
        Ok(Given {
            sent,
            freed,
            let_go,
            unsent: unsent_beside(round.data, self.dest, round.buried),
        })
    }

    fn reaches(&self, reference: &str) -> bool {
        crate::held_in(reference, self.dest)
    }

    fn take(&mut self, round: Taking) -> Result<Taken, Trouble> {
        let mut took_in = Vec::new();
        let brought = copy_held(
            &self.dest.join(HELD),
            &round.data.join(HELD),
            round.buried,
            false,
            Some(round.data),
            left_behind(round.holds),
            round.reachable,
            Some(&mut took_in),
            round.avowed,
            round.saying,
        )?;
        Ok(Taken { brought, took_in })
    }
}

// Only what the copy would carry is told, so a round with nothing to copy says nothing.
fn owed_beside(from: &Path, into: &Path, buried: &BTreeSet<String>, again: bool) -> usize {
    let Ok(shelves) = std::fs::read_dir(from) else {
        return 0;
    };
    let len = |at: &Path| std::fs::metadata(at).map(|one| one.len()).ok();
    shelves
        .filter_map(|shelf| shelf.ok())
        .filter_map(|shelf| {
            std::fs::read_dir(shelf.path())
                .ok()
                .map(|files| (shelf, files))
        })
        .flat_map(|(shelf, files)| {
            files
                .filter_map(|file| file.ok())
                .map(move |file| (shelf.file_name(), file))
        })
        .filter(|(shelf, file)| {
            let (Some(under), Some(named)) = (
                shelf.to_str(),
                file.file_name().to_str().map(str::to_string),
            ) else {
                return false;
            };
            let at = file.path();
            at.is_file()
                && tisty_core::attach::shelved(under, &named)
                && !buried.contains(&format!("attachments/{under}/{named}"))
                && len(&at).is_some_and(|weighs| weighs <= tisty_core::attach::HELD_UP_TO)
                && !tisty_core::holes::a_hole(&at)
                && (again || len(&at) != len(&into.join(under).join(&named)))
        })
        .count()
}
