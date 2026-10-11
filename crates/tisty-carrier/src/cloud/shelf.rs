use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tisty_core::attach::{self, COPIED_IN_DOC};
use tisty_core::witness::{self, Fact, channel};
use tisty_sync::{Attachments, Given, Giving, HELD, LetGo, Reached, Stage, Taken, Taking, Trouble};

use super::index::Index;
use super::judge::{Judged, judge};
use super::mirror::resembles;
use crate::{Deferred, Elsewhere, Expect, Hitch, Remote, Seen, Told};

static TURN: AtomicU64 = AtomicU64::new(0);

pub struct Shelf<'a> {
    remote: &'a dyn Remote,
    index: &'a mut Index,
    paused: Option<Deferred>,
    sent: usize,
    fetching: bool,
}

struct Local {
    reference: String,
    under: String,
    leaf: String,
    at: PathBuf,
    len: u64,
}

impl<'a> Shelf<'a> {
    pub fn over(remote: &'a dyn Remote, index: &'a mut Index) -> Self {
        Self {
            remote,
            index,
            paused: None,
            sent: 0,
            fetching: false,
        }
    }

    pub fn fetching_all(remote: &'a dyn Remote, index: &'a mut Index) -> Self {
        Self {
            fetching: true,
            ..Self::over(remote, index)
        }
    }

    pub fn paused(&self) -> Option<Deferred> {
        self.paused
    }

    pub fn sent(&self) -> usize {
        self.sent
    }

    fn owed(&self, one: &Local, round: &Giving) -> bool {
        !round.buried.contains(&one.reference)
            && one.len <= COPIED_IN_DOC
            && attach::shelved(&one.under, &one.leaf)
            && (round.again
                || !self
                    .index
                    .shelf
                    .get(&one.reference)
                    .is_some_and(|up| up.bytes == one.len))
    }

    fn refuse(&mut self, hitch: Hitch, reference: &str, remaining: usize) -> Result<(), Trouble> {
        if matches!(
            hitch,
            Hitch::Broke(_) | Hitch::Missing(_) | Hitch::Changed(_)
        ) {
            witness::warn(
                channel::CLOUD,
                "an attachment could not be carried to or from the cloud",
                &[
                    ("at", Fact::Id(reference.to_string())),
                    ("why", Fact::Why(format!("{hitch:?}"))),
                ],
            );
            return Ok(());
        }
        match judge(hitch, self.remote.limits().provider) {
            Judged::Pause(reason, wait) => {
                self.paused = Some(Deferred {
                    reason,
                    left: remaining,
                    retry_after: wait,
                });
                Ok(())
            }
            Judged::Stop(trouble) => Err(trouble),
        }
    }

    fn send(&self, one: &Local) -> Result<Seen, Hitch> {
        let (seen, ours) = match self.remote.put(&one.reference, &one.at, Expect::Absent) {
            Ok(seen) => (seen, true),
            Err(Hitch::Changed(_)) => (
                self.remote
                    .about(&one.reference)?
                    .ok_or_else(|| Hitch::Missing(one.reference.clone()))?,
                false,
            ),
            Err(other) => return Err(other),
        };
        let same = seen.bytes == one.len
            && match seen.hash.is_empty() {
                true => ours,
                false => self.remote.hash_of(&one.at)? == seen.hash,
            };
        match same {
            true => Ok(seen),
            false => Err(Hitch::Broke(format!(
                "{} is up there with other bytes",
                one.reference
            ))),
        }
    }

    fn fetch(&self, reference: &str, target: &Path) -> Result<PathBuf, Hitch> {
        fetched_part(self.remote, reference, target)
    }
}

fn fetched_part(remote: &dyn Remote, reference: &str, target: &Path) -> Result<PathBuf, Hitch> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let part = tisty_core::parting::beside(target, TURN.fetch_add(1, Ordering::Relaxed));
    let fetched = std::fs::File::create(&part)
        .map_err(Hitch::from)
        .and_then(|mut file| {
            let got = remote.fetch(reference, 0, &mut file)?;
            file.sync_all()?;
            Ok(got)
        })
        .and_then(|got| match std::fs::metadata(&part)?.len() == got.bytes {
            true if got.bytes <= COPIED_IN_DOC => Ok(()),
            true => Err(Hitch::Broke(format!(
                "{reference} is past what an attachment may weigh"
            ))),
            false => Err(Hitch::Broke(format!("{reference} came short"))),
        });
    match fetched {
        Ok(()) => Ok(part),
        Err(hitch) => {
            let _ = std::fs::remove_file(&part);
            Err(hitch)
        }
    }
}

pub fn named_well(reference: &str) -> bool {
    placed(reference).is_some_and(|(under, leaf)| attach::shelved(under, leaf))
}

pub fn brought(
    remote: &dyn Remote,
    data: &Path,
    reference: &str,
    logged: &str,
) -> Result<Option<PathBuf>, Hitch> {
    let Some((under, leaf)) = placed(reference).filter(|_| named_well(reference)) else {
        return Ok(None);
    };
    let target = attach::resolve(reference, data).map_err(|e| Hitch::Broke(e.to_string()))?;
    let holds = |at: &Path| {
        attach::hashed(at).ok().filter(|(sha256, _)| {
            attach::vouched(under, leaf, sha256) && sha256.eq_ignore_ascii_case(logged)
        })
    };
    if target.is_file() && holds(&target).is_some() {
        return Ok(Some(target));
    }
    let part = match fetched_part(remote, reference, &target) {
        Ok(part) => part,
        Err(Hitch::Missing(_)) => return Ok(None),
        Err(hitch) => return Err(hitch),
    };
    let Some((sha256, bytes)) = holds(&part) else {
        let _ = std::fs::remove_file(&part);
        return Err(Hitch::Broke(format!(
            "{reference} does not hold the bytes its name and the log vouch for"
        )));
    };
    if let Err(e) = std::fs::rename(&part, &target) {
        let _ = std::fs::remove_file(&part);
        return Err(e.into());
    }
    attach::noted(data, reference, &sha256, bytes);
    Ok(Some(target))
}

impl Attachments for Shelf<'_> {
    fn give(&mut self, round: Giving) -> Result<Given, Trouble> {
        let mut given = Given::default();
        let (all, mut asked_for) = locals(round.data, round.buried);
        for (at, one) in all.iter().enumerate() {
            if self.paused.is_some() {
                break;
            }
            if tisty_core::holes::a_hole(&one.at) {
                asked_for.push(one.at.clone());
                continue;
            }
            if !self.owed(one, &round) {
                continue;
            }
            let (sha256, _) = match attach::hashed(&one.at) {
                Ok(read) => read,
                Err(e) => {
                    witness::warn(
                        channel::CLOUD,
                        "an attachment could not be read to be sent",
                        &[
                            ("at", Fact::Id(one.reference.clone())),
                            ("why", Fact::Why(e.to_string())),
                        ],
                    );
                    continue;
                }
            };
            if !attach::vouched(&one.under, &one.leaf, &sha256)
                || !attach::as_kept(round.avowed, &one.reference, &sha256)
            {
                witness::warn(
                    channel::CLOUD,
                    "an attachment does not hold what its name or the log say, so it was not sent",
                    &[("at", Fact::Id(one.reference.clone()))],
                );
                continue;
            }
            match self.send(one) {
                Ok(seen) => {
                    self.index.shelf.insert(one.reference.clone(), seen);
                    given.sent += 1;
                    self.sent += 1;
                }
                Err(hitch) => {
                    let remaining = all[at..]
                        .iter()
                        .filter(|rest| self.owed(rest, &round))
                        .count();
                    self.refuse(hitch, &one.reference, remaining)?;
                }
            }
        }
        tisty_core::holes::ask_for(asked_for);
        Ok(given)
    }

    fn reaches(&self, reference: &str) -> bool {
        self.index.shelf.contains_key(reference)
    }

    fn takes(&self) -> bool {
        self.fetching
    }

    fn take(&mut self, round: Taking) -> Result<Taken, Trouble> {
        let mut taken = Taken::default();
        let index = &mut *self.index;
        index.spoiled.retain(|reference, revision| {
            index
                .shelf
                .get(reference)
                .is_some_and(|seen| &seen.revision == revision)
        });
        let mut written_down = attach::digests(round.data);
        for (at, one) in round.avowed {
            if written_down
                .insert(at.clone(), one.clone())
                .is_some_and(|was| was.0 != one.0)
            {
                witness::warn(
                    channel::ATTACH,
                    "the book kept here and the log disagree about what an attachment holds",
                    &[("at", Fact::Id(at.clone()))],
                );
            }
        }
        let up: Vec<(String, Seen)> = self
            .index
            .shelf
            .iter()
            .map(|(reference, seen)| (reference.clone(), seen.clone()))
            .collect();
        let whole = up.len();
        for (done, (reference, seen)) in up.iter().enumerate() {
            if self.paused.is_some() {
                break;
            }
            (round.saying)(Reached::Along {
                stage: Stage::Attachments,
                done,
                whole,
            });
            if !wanted(&round, self.index, reference, seen) {
                continue;
            }
            let Some((under, leaf)) = placed(reference) else {
                continue;
            };
            let Ok(target) = attach::resolve(reference, round.data) else {
                continue;
            };
            let part = match self.fetch(reference, &target) {
                Ok(part) => part,
                Err(hitch) => {
                    let remaining = up[done..]
                        .iter()
                        .filter(|(rest, said)| wanted(&round, self.index, rest, said))
                        .count();
                    self.refuse(hitch, reference, remaining)?;
                    continue;
                }
            };
            let kept = attach::hashed(&part).is_ok_and(|(sha256, bytes)| {
                let fits = attach::vouched(under, leaf, &sha256)
                    && attach::as_kept(&written_down, reference, &sha256)
                    && std::fs::rename(&part, &target).is_ok();
                if fits {
                    attach::noted(round.data, reference, &sha256, bytes);
                    (round.saying)(Reached::Kept {
                        at: reference.clone(),
                        sha256: sha256.clone(),
                        bytes,
                    });
                    taken.took_in.push((reference.clone(), sha256, bytes));
                }
                fits
            });
            if kept {
                taken.brought += 1;
            } else {
                let _ = std::fs::remove_file(&part);
                self.index
                    .spoiled
                    .insert(reference.clone(), seen.revision.clone());
                witness::warn(
                    channel::CLOUD,
                    "an attachment from the cloud does not hold the bytes its name vouches for",
                    &[("at", Fact::Id(reference.clone()))],
                );
            }
        }
        (round.saying)(Reached::Along {
            stage: Stage::Attachments,
            done: whole,
            whole,
        });
        Ok(taken)
    }
}

fn placed(reference: &str) -> Option<(&str, &str)> {
    reference
        .strip_prefix("attachments/")
        .and_then(|rest| rest.split_once('/'))
}

fn wanted(round: &Taking, index: &Index, reference: &str, seen: &Seen) -> bool {
    let Some((under, leaf)) = placed(reference) else {
        return false;
    };
    let Ok(target) = attach::resolve(reference, round.data) else {
        return false;
    };
    attach::shelved(under, leaf)
        && !round.buried.contains(reference)
        && round
            .reachable
            .is_none_or(|named| named.contains(reference))
        && !index.spoiled(reference)
        && seen.bytes <= COPIED_IN_DOC
        && !std::fs::metadata(&target).is_ok_and(|one| one.is_file() && one.len() == seen.bytes)
}

pub fn sweep(data: &Path) {
    let Ok(shelves) = std::fs::read_dir(data.join(HELD)) else {
        return;
    };
    for shelf in shelves.filter_map(|one| one.ok()) {
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|one| one.ok()) {
            let spent = file
                .file_name()
                .to_str()
                .is_some_and(|named| tisty_core::parting::spent(&file.path(), named));
            if spent {
                let _ = std::fs::remove_file(file.path());
            }
        }
    }
}

fn locals(data: &Path, buried: &BTreeSet<String>) -> (Vec<Local>, Vec<PathBuf>) {
    let mut all = Vec::new();
    let mut asked = Vec::new();
    let Ok(shelves) = std::fs::read_dir(data.join(HELD)) else {
        return (all, asked);
    };
    for shelf in shelves.filter_map(|one| one.ok()) {
        let Some(under) = shelf.file_name().to_str().map(str::to_string) else {
            continue;
        };
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|one| one.ok()) {
            let (Some(leaf), Ok(meta)) = (
                file.file_name().to_str().map(str::to_string),
                file.metadata(),
            ) else {
                continue;
            };
            if let Some(real) = tisty_core::holes::marker(&leaf)
                .then(|| tisty_core::holes::named_away(&leaf))
                .flatten()
            {
                let reference = format!("{HELD}/{under}/{real}");
                if attach::shelved(&under, real) && !buried.contains(&reference) {
                    asked.push(shelf.path().join(real));
                }
                continue;
            }
            if meta.is_file() && !leaf.ends_with(".part") {
                all.push(Local {
                    reference: format!("{HELD}/{under}/{leaf}"),
                    under: under.clone(),
                    leaf,
                    at: file.path(),
                    len: meta.len(),
                });
            }
        }
    }
    all.sort_by(|one, other| one.reference.cmp(&other.reference));
    (all, asked)
}

pub struct Letting<'a> {
    pub chosen: &'a dyn Fn(&str, u64) -> bool,
    pub elsewhere: Elsewhere<'a>,
    pub told: Told<'a>,
    pub only_counting: bool,
}

pub fn let_go(remote: &dyn Remote, index: &Index, data: &Path, letting: Letting) -> LetGo {
    let Letting {
        chosen,
        elsewhere,
        told,
        only_counting,
    } = letting;
    let mut done = LetGo::default();
    let (all, _) = locals(data, &BTreeSet::new());
    for one in all
        .into_iter()
        .filter(|one| chosen(&one.reference, one.len))
    {
        let twin = index
            .shelf
            .get(&one.reference)
            .filter(|up| up.bytes == one.len);
        let safe = !tisty_core::holes::a_hole(&one.at)
            && match twin {
                Some(up) if !up.hash.is_empty() => resembles(remote, &one.at, up),
                Some(_) => elsewhere(&one.reference),
                None => false,
            };
        match safe && (only_counting || std::fs::remove_file(&one.at).is_ok()) {
            true => {
                done.gone += 1;
                done.freed += one.len;
                done.let_go.push(one.reference);
            }
            false => done.kept.push(one.reference),
        }
        if !told(&done) {
            break;
        }
    }
    done
}
