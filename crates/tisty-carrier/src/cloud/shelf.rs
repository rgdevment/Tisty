use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tisty_core::attach::{self, COPIED_IN_DOC, COPIED_UP_TO};
use tisty_core::config::Holds;
use tisty_core::witness::{self, Fact, channel};
use tisty_sync::{
    Attachments, Given, Giving, LetGo, Reached, Stage, Taken, Taking, Trouble, left_behind,
};

use super::index::Index;
use super::judge::{Judged, judge};
use crate::{Elsewhere, Expect, Hitch, Reason, Remote, Seen, Told};

const HELD: &str = "attachments";

static TURN: AtomicU64 = AtomicU64::new(0);

pub struct Shelf<'a> {
    remote: &'a dyn Remote,
    index: &'a mut Index,
    paused: Option<(Reason, Option<Duration>)>,
    left: usize,
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
            left: 0,
        }
    }

    pub fn paused(&self) -> Option<(Reason, Option<Duration>, usize)> {
        self.paused.map(|(reason, wait)| (reason, wait, self.left))
    }

    fn refuse(&mut self, hitch: Hitch, reference: &str, remaining: usize) -> Result<(), Trouble> {
        if matches!(
            hitch,
            Hitch::Broke(_) | Hitch::Missing(_) | Hitch::Changed(_)
        ) {
            witness::warn(
                channel::SYNC,
                "an attachment could not be carried to or from the cloud",
                &[
                    ("at", Fact::Id(reference.to_string())),
                    ("why", Fact::Why(format!("{hitch:?}"))),
                ],
            );
            return Ok(());
        }
        match judge(hitch) {
            Judged::Pause(reason, wait) => {
                self.paused = Some((reason, wait));
                self.left = remaining;
                Ok(())
            }
            Judged::Stop(trouble) => Err(trouble),
        }
    }

    fn send(&self, one: &Local) -> Result<Seen, Hitch> {
        let seen = match self.remote.put(&one.reference, &one.at, Expect::Absent) {
            Ok(seen) => seen,
            Err(Hitch::Changed(_)) => self
                .remote
                .about(&one.reference)?
                .ok_or_else(|| Hitch::Missing(one.reference.clone()))?,
            Err(other) => return Err(other),
        };
        let same = seen.bytes == one.len
            && (seen.hash.is_empty() || self.remote.hash_of(&one.at)? == seen.hash);
        match same {
            true => Ok(seen),
            false => Err(Hitch::Broke(format!(
                "{} is up there with other bytes",
                one.reference
            ))),
        }
    }

    fn fetch(&self, reference: &str, target: &Path) -> Result<PathBuf, Hitch> {
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let part = tisty_core::parting::beside(target, TURN.fetch_add(1, Ordering::Relaxed));
        let fetched = std::fs::File::create(&part)
            .map_err(Hitch::from)
            .and_then(|mut file| {
                let got = self.remote.fetch(reference, 0, &mut file)?;
                file.sync_all()?;
                Ok(got)
            })
            .and_then(|got| match std::fs::metadata(&part)?.len() == got.bytes {
                true => Ok(()),
                false => Err(Hitch::Unreachable(format!("{reference} came short"))),
            });
        match fetched {
            Ok(()) => Ok(part),
            Err(hitch) => {
                let _ = std::fs::remove_file(&part);
                Err(hitch)
            }
        }
    }
}

impl Attachments for Shelf<'_> {
    fn give(&mut self, round: Giving) -> Result<Given, Trouble> {
        let mut given = Given::default();
        let mut landed = Vec::new();
        let all = locals(round.data);
        let whole = all.len();
        for (at, one) in all.into_iter().enumerate() {
            if self.paused.is_some() {
                break;
            }
            if round.buried.contains(&one.reference)
                || one.len > COPIED_IN_DOC
                || !attach::shelved(&one.under, &one.leaf)
                || tisty_core::holes::a_hole(&one.at)
                || (!round.again
                    && self
                        .index
                        .shelf
                        .get(&one.reference)
                        .is_some_and(|up| up.bytes == one.len))
            {
                continue;
            }
            let Ok((sha256, bytes)) = attach::hashed(&one.at) else {
                continue;
            };
            if !attach::vouched(&one.under, &one.leaf, &sha256)
                || !attach::as_kept(round.avowed, &one.reference, &sha256)
            {
                witness::warn(
                    channel::SYNC,
                    "an attachment does not hold what its name or the log say, so it was not sent",
                    &[("at", Fact::Id(one.reference.clone()))],
                );
                continue;
            }
            match self.send(&one) {
                Ok(seen) => {
                    self.index.shelf.insert(one.reference.clone(), seen);
                    landed.push((one.reference, bytes));
                    given.sent += 1;
                }
                Err(hitch) => self.refuse(hitch, &one.reference, whole - at)?,
            }
        }
        if round.holds == Holds::Shared {
            for (reference, bytes) in landed
                .into_iter()
                .filter(|(_, bytes)| *bytes > COPIED_UP_TO)
            {
                let freed = attach::resolve(&reference, round.data)
                    .is_ok_and(|at| std::fs::remove_file(at).is_ok());
                if freed {
                    given.freed += bytes;
                    given.let_go.push(reference);
                }
            }
        }
        Ok(given)
    }

    fn take(&mut self, round: Taking) -> Result<Taken, Trouble> {
        let mut taken = Taken::default();
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
        let above = left_behind(round.holds);
        let up: Vec<(String, Seen)> = self
            .index
            .shelf
            .iter()
            .map(|(reference, seen)| (reference.clone(), seen.clone()))
            .collect();
        let whole = up.len();
        for (done, (reference, seen)) in up.into_iter().enumerate() {
            if self.paused.is_some() {
                break;
            }
            (round.saying)(Reached::Along {
                stage: Stage::Attachments,
                done,
                whole,
            });
            let Some((under, leaf)) = reference
                .strip_prefix("attachments/")
                .and_then(|rest| rest.split_once('/'))
            else {
                continue;
            };
            let Ok(target) = attach::resolve(&reference, round.data) else {
                continue;
            };
            let wanted = attach::shelved(under, leaf)
                && !round.buried.contains(&reference)
                && round
                    .reachable
                    .is_none_or(|named| named.contains(&reference))
                && above.is_none_or(|most| seen.bytes <= most)
                && seen.bytes <= COPIED_IN_DOC
                && !std::fs::metadata(&target)
                    .is_ok_and(|one| one.is_file() && one.len() == seen.bytes);
            if !wanted {
                continue;
            }
            let part = match self.fetch(&reference, &target) {
                Ok(part) => part,
                Err(hitch) => {
                    self.refuse(hitch, &reference, whole - done)?;
                    continue;
                }
            };
            let kept = attach::hashed(&part).is_ok_and(|(sha256, bytes)| {
                let fits = attach::vouched(under, leaf, &sha256)
                    && attach::as_kept(&written_down, &reference, &sha256)
                    && std::fs::rename(&part, &target).is_ok();
                if fits {
                    attach::noted(round.data, &reference, &sha256, bytes);
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
                witness::warn(
                    channel::SYNC,
                    "an attachment from the cloud does not hold the bytes its name vouches for",
                    &[("at", Fact::Id(reference))],
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

fn locals(data: &Path) -> Vec<Local> {
    let mut all = Vec::new();
    let Ok(shelves) = std::fs::read_dir(data.join(HELD)) else {
        return all;
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
    all
}

pub fn let_go(
    remote: &dyn Remote,
    index: &Index,
    data: &Path,
    above: u64,
    elsewhere: Elsewhere,
    told: Told,
) -> LetGo {
    let mut done = LetGo::default();
    for one in locals(data).into_iter().filter(|one| one.len > above) {
        let twin = index
            .shelf
            .get(&one.reference)
            .filter(|up| up.bytes == one.len);
        let safe = match twin {
            Some(up) if !up.hash.is_empty() => {
                remote.hash_of(&one.at).is_ok_and(|hash| hash == up.hash)
            }
            Some(_) => elsewhere(&one.reference),
            None => false,
        };
        match safe && std::fs::remove_file(&one.at).is_ok() {
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
