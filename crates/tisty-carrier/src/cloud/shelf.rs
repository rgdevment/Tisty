use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tisty_core::attach::{self, HELD_UP_TO};
use tisty_core::witness::{self, Fact, channel};
use tisty_sync::{
    Attachments, Given, Giving, HELD, LetGo, NAMED_AT_MOST, Reached, Stage, Taken, Taking, Trouble,
};

use super::index::Index;
use super::judge::{Judged, judge};
use super::mirror::{self, resembles};
use super::pace::Pace;
use crate::{Deferred, Elsewhere, Expect, Hitch, Reason, Remote, Seen, Told};

static TURN: AtomicU64 = AtomicU64::new(0);

pub struct Shelf<'a> {
    remote: &'a dyn Remote,
    index: &'a mut Index,
    paused: Option<Deferred>,
    sent: usize,
    fetching: bool,
    tree: Option<(&'a Path, &'a dyn Remote)>,
    early: usize,
    unsent: Vec<String>,
    full: Option<Trouble>,
    home: Option<&'a Path>,
}

enum Sent {
    Landed(Seen),
    /// Up there under our name with other bytes: what is ours goes over it on the next round.
    Spoiled(Seen),
}

struct Local {
    reference: String,
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
            tree: None,
            early: 0,
            unsent: Vec::new(),
            full: None,
            home: None,
        }
    }

    /// Where what landed is written down as the shelf goes, not only once the round ends.
    pub fn kept_in(mut self, home: &'a Path) -> Self {
        self.home = Some(home);
        self
    }

    pub fn history_first(mut self, tree: &'a Path, history: &'a dyn Remote) -> Self {
        self.tree = Some((tree, history));
        self
    }

    pub fn early(&self) -> usize {
        self.early
    }

    fn history_sent(&mut self, pace: &mut Pace) -> Option<Hitch> {
        let (tree, history) = self.tree.take()?;
        let (pushed, hitch) = mirror::push(history, tree, self.index, pace);
        self.early += pushed.put;
        hitch.filter(|hitch| {
            !matches!(
                hitch,
                Hitch::Broke(_) | Hitch::Missing(_) | Hitch::Changed(_)
            )
        })
    }

    pub fn fetching_all(remote: &'a dyn Remote, index: &'a mut Index) -> Self {
        Self {
            fetching: true,
            ..Self::over(remote, index)
        }
    }

    pub fn paused(&self) -> Option<Deferred> {
        self.paused.clone()
    }

    /// A full provider stops the shelf alone, so the documents and the stamp still go.
    pub fn full(&self) -> Option<Trouble> {
        self.full.clone()
    }

    fn stopped(&self) -> bool {
        self.paused.is_some() || self.full.is_some()
    }

    fn unsent(&mut self, reference: &str, why: &'static str) {
        witness::warn(
            channel::CLOUD,
            "an attachment could not go up to the cloud, so it stays here and is told",
            &[
                ("at", Fact::Id(reference.to_string())),
                ("why", Fact::Word(why)),
            ],
        );
        if !self.unsent.iter().any(|one| one == reference) {
            self.unsent.push(reference.to_string());
        }
    }

    fn told_unsent(&self) -> Option<Deferred> {
        (!self.unsent.is_empty()).then(|| Deferred {
            reason: Reason::Unsent,
            left: self.unsent.len(),
            retry_after: None,
            names: self.unsent.iter().take(NAMED_AT_MOST).cloned().collect(),
            cause: None,
        })
    }

    pub fn sent(&self) -> usize {
        self.sent
    }

    fn owed(&self, one: &Local, round: &Round) -> bool {
        one.len <= HELD_UP_TO && self.not_up_there(one, round)
    }

    fn not_up_there(&self, one: &Local, round: &Round) -> bool {
        !round.buried.contains(&one.reference)
            && attach::stamped(&one.leaf)
            && (round.again
                || self.index.spoiled(&one.reference)
                || !self
                    .index
                    .shelf
                    .get(&one.reference)
                    .is_some_and(|up| up.bytes == one.len))
    }

    /// True when only this one was passed over; the rest of the shelf goes on.
    fn refuse(
        &mut self,
        hitch: Hitch,
        reference: &str,
        remaining: Vec<String>,
    ) -> Result<bool, Trouble> {
        if matches!(
            hitch,
            Hitch::Broke(_) | Hitch::Missing(_) | Hitch::Changed(_)
        ) {
            // The kind alone: what a provider says back can carry the attachment's own name.
            witness::warn(
                channel::CLOUD,
                "an attachment could not be carried to or from the cloud",
                &[
                    ("at", Fact::Id(reference.to_string())),
                    ("why", Fact::Word(kind_of(&hitch))),
                ],
            );
            return Ok(true);
        }
        match judge(hitch, self.remote.limits().provider) {
            Judged::Pause(reason, wait, cause) => {
                witness::note(
                    channel::CLOUD,
                    "the shelf pauses, so what is left of it goes on a later round",
                    &[
                        ("why", Fact::Word(paused_for(reason))),
                        ("count", Fact::Count(remaining.len())),
                    ],
                );
                self.paused = Some(Deferred {
                    reason,
                    left: remaining.len(),
                    retry_after: wait,
                    names: remaining.into_iter().take(NAMED_AT_MOST).collect(),
                    cause,
                });
                Ok(false)
            }
            Judged::Stop(trouble @ Trouble::CloudFull { .. }) => {
                self.full = Some(trouble);
                Ok(false)
            }
            Judged::Stop(trouble) => Err(trouble),
        }
    }

    fn send(&self, one: &Local) -> Result<Sent, Hitch> {
        // What is up there was refused for its bytes, so ours, already held to its name and the log, goes over it.
        let expect = match self.index.spoiled(&one.reference) {
            true => Expect::Any,
            false => Expect::Absent,
        };
        let (seen, ours) = match self.remote.put(&one.reference, &one.at, expect) {
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
            true => Ok(Sent::Landed(seen)),
            false => Ok(Sent::Spoiled(seen)),
        }
    }

    fn fetch(&self, reference: &str, target: &Path) -> Result<PathBuf, Hitch> {
        fetched_part(self.remote, reference, target)
    }
}

fn fetched_part(remote: &dyn Remote, reference: &str, target: &Path) -> Result<PathBuf, Hitch> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
        let _ = tisty_core::paths::ours_alone(parent);
    }
    let part = tisty_core::parting::beside(target, TURN.fetch_add(1, Ordering::Relaxed));
    let fetched = std::fs::File::create(&part)
        .map_err(Hitch::from)
        .and_then(|mut file| {
            let _ = tisty_core::paths::ours_alone(&part);
            let got = remote.fetch(reference, 0, &mut file)?;
            file.sync_all()?;
            Ok(got)
        })
        .and_then(|got| match std::fs::metadata(&part)?.len() == got.bytes {
            true if got.bytes <= HELD_UP_TO => Ok(()),
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

struct Round<'r> {
    buried: &'r BTreeSet<String>,
    again: bool,
}

pub fn named_well(reference: &str) -> bool {
    attach::names_an_attachment(reference)
}

/// Where it lies, and what it read when no history answers for it yet.
pub type Fetched = (PathBuf, Option<(String, u64)>);

pub fn brought(
    remote: &dyn Remote,
    data: &Path,
    reference: &str,
    logged: Option<&str>,
) -> Result<Option<Fetched>, Hitch> {
    let Some(leaf) = placed(reference) else {
        return Ok(None);
    };
    let target = attach::resolve(reference, data).map_err(|e| Hitch::Broke(e.to_string()))?;
    let written_down = attach::digests(data);
    let holds = |at: &Path| {
        attach::hashed(at).ok().filter(|(sha256, _)| {
            attach::vouched(leaf, sha256)
                && match logged {
                    Some(logged) => sha256.eq_ignore_ascii_case(logged),
                    None => attach::as_kept(&written_down, reference, sha256),
                }
        })
    };
    let unvouched = |read: (String, u64)| logged.is_none().then_some(read);
    if target.is_file()
        && let Some(read) = holds(&target)
    {
        return Ok(Some((target, unvouched(read))));
    }
    let part = match fetched_part(remote, reference, &target) {
        Ok(part) => part,
        Err(Hitch::Missing(_)) => return Ok(None),
        Err(hitch) => return Err(hitch),
    };
    let Some(read) = holds(&part) else {
        let _ = std::fs::remove_file(&part);
        return Err(Hitch::Broke(format!(
            "{reference} does not hold the bytes its name and the log vouch for"
        )));
    };
    if let Err(e) = std::fs::rename(&part, &target) {
        let _ = std::fs::remove_file(&part);
        return Err(e.into());
    }
    let _ = tisty_core::paths::ours_alone(&target);
    attach::noted(data, reference, &read.0, read.1);
    Ok(Some((target, unvouched(read))))
}

impl Attachments for Shelf<'_> {
    fn give(&mut self, giving: Giving) -> Result<Given, Trouble> {
        let Giving {
            data,
            buried,
            avowed,
            again,
            saying,
            ..
        } = giving;
        let round = Round { buried, again };
        let mut pace = Pace::new(self.home, saying);
        let mut given = Given::default();
        let (all, mut asked_for) = locals(data, buried);
        let (whole, weighs) = all
            .iter()
            .filter(|one| self.owed(one, &round) && !tisty_core::holes::a_hole(&one.at))
            .fold((0, 0), |(files, bytes), one| (files + 1, bytes + one.len));
        pace.owe(whole, weighs);
        for (at, one) in all.iter().enumerate() {
            if self.stopped() {
                break;
            }
            if tisty_core::holes::a_hole(&one.at) {
                asked_for.push(one.at.clone());
                continue;
            }
            if one.len > HELD_UP_TO && self.not_up_there(one, &round) {
                self.unsent(&one.reference, "past what the cloud takes");
                continue;
            }
            if !self.owed(one, &round) {
                continue;
            }
            if let Some(hitch) = self.history_sent(&mut pace) {
                let remaining = all[at..]
                    .iter()
                    .filter(|rest| self.owed(rest, &round))
                    .map(|rest| rest.reference.clone())
                    .collect();
                self.refuse(hitch, &one.reference, remaining)?;
                continue;
            }
            let Ok((sha256, _)) = attach::hashed(&one.at) else {
                self.unsent(&one.reference, "could not be read here");
                pace.went(one.len);
                continue;
            };
            if !attach::vouched(&one.leaf, &sha256)
                || !attach::as_kept(avowed, &one.reference, &sha256)
            {
                self.unsent(&one.reference, "does not hold what its name or the log say");
                pace.went(one.len);
                continue;
            }
            match self.send(one) {
                Ok(Sent::Landed(seen)) => {
                    self.index.shelved(&one.reference, seen);
                    given.sent += 1;
                    self.sent += 1;
                    self.remote.moved_on();
                    pace.landed(self.index, one.len);
                }
                Ok(Sent::Spoiled(seen)) => {
                    self.index
                        .spoiled
                        .insert(one.reference.clone(), seen.revision.clone());
                    self.index.shelved(&one.reference, seen);
                    self.unsent(&one.reference, "up there with other bytes");
                    self.remote.moved_on();
                    pace.went(one.len);
                }
                Err(hitch) => {
                    let remaining = all[at..]
                        .iter()
                        .filter(|rest| self.owed(rest, &round))
                        .map(|rest| rest.reference.clone())
                        .collect();
                    if self.refuse(hitch, &one.reference, remaining)? {
                        self.unsent(&one.reference, "the cloud would not take it");
                    }
                    pace.went(one.len);
                }
            }
        }
        pace.kept(self.index);
        tisty_core::holes::ask_for(asked_for);
        given.unsent = self.told_unsent();
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
            if self.stopped() {
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
            let Some(leaf) = placed(reference) else {
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
                        .map(|(rest, _)| rest.to_string())
                        .collect();
                    self.refuse(hitch, reference, remaining)?;
                    continue;
                }
            };
            let read = attach::hashed(&part);
            let fits = read.as_ref().is_ok_and(|(sha256, _)| {
                attach::vouched(leaf, sha256) && attach::as_kept(&written_down, reference, sha256)
            });
            match read {
                Ok((sha256, bytes)) if fits && std::fs::rename(&part, &target).is_ok() => {
                    let _ = tisty_core::paths::ours_alone(&target);
                    attach::noted(round.data, reference, &sha256, bytes);
                    taken.brought += 1;
                    // Only what the log already answers for is said to be held: a name alone vouches for too little.
                    if !round.avowed.contains_key(reference) {
                        continue;
                    }
                    (round.saying)(Reached::Kept {
                        at: reference.clone(),
                        sha256: sha256.clone(),
                        bytes,
                    });
                    taken.took_in.push((reference.clone(), sha256, bytes));
                }
                Ok(_) if !fits => {
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
                _ => {
                    let _ = std::fs::remove_file(&part);
                    witness::warn(
                        channel::CLOUD,
                        "an attachment from the cloud could not be kept here, so the next step tries again",
                        &[("at", Fact::Id(reference.clone()))],
                    );
                }
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

fn placed(reference: &str) -> Option<&str> {
    attach::leaf_of(reference)
}

fn wanted(round: &Taking, index: &Index, reference: &str, seen: &Seen) -> bool {
    if placed(reference).is_none() {
        return false;
    }
    let Ok(target) = attach::resolve(reference, round.data) else {
        return false;
    };
    !round.buried.contains(reference)
        && (round
            .reachable
            .is_none_or(|named| named.contains(reference))
            || round.avowed.contains_key(reference))
        && !index.spoiled(reference)
        && seen.bytes <= HELD_UP_TO
        && !std::fs::metadata(&target).is_ok_and(|one| one.is_file() && one.len() == seen.bytes)
}

fn paused_for(reason: Reason) -> &'static str {
    match reason {
        Reason::Limit => "limit",
        Reason::Budget => "budget",
        Reason::Paused => "paused",
        Reason::Busy => "busy",
        Reason::Offline => "offline",
        Reason::Changed => "changed",
        Reason::Troubled => "troubled",
        Reason::Unsent => "unsent",
    }
}

fn kind_of(hitch: &Hitch) -> &'static str {
    match hitch {
        Hitch::Missing(_) => "missing",
        Hitch::Changed(_) => "changed",
        _ => "broke",
    }
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
    let Ok(files) = std::fs::read_dir(data.join(HELD)) else {
        return (all, asked);
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
            let reference = format!("{HELD}/{real}");
            if attach::stamped(real) && !buried.contains(&reference) {
                asked.push(data.join(HELD).join(real));
            }
            continue;
        }
        if meta.is_file() && !leaf.ends_with(".part") {
            all.push(Local {
                reference: format!("{HELD}/{leaf}"),
                leaf,
                at: file.path(),
                len: meta.len(),
            });
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
        // Counting reads no bytes: the same size under a hash the provider keeps is enough for an estimate.
        let safe = !tisty_core::holes::a_hole(&one.at)
            && match twin {
                Some(up) if !up.hash.is_empty() => only_counting || resembles(remote, &one.at, up),
                Some(_) => elsewhere(&one.reference),
                None => false,
            };
        let still = only_counting || chosen(&one.reference, one.len);
        match safe && still && (only_counting || std::fs::remove_file(&one.at).is_ok()) {
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

#[cfg(test)]
#[path = "shelf_test.rs"]
mod tests;

fn flat_name(name: &str) -> Option<String> {
    let rest = name.strip_prefix(&format!("{HELD}/"))?;
    let (shelf, leaf) = rest.split_once('/')?;
    (shelf.len() == 2 && attach::lower_hex(shelf) && attach::stamped(leaf))
        .then(|| format!("{HELD}/{leaf}"))
}

/// What an older build left on shelves up there moves to the one flat folder before a round reads
/// it; a twin already flat is the same bytes by its stamp, so the shelved one only goes. Whatever
/// the cloud would not take now is left for the next round.
pub fn flattened(remote: &dyn Remote, index: &mut Index) -> usize {
    let shelved: Vec<(String, String)> = index
        .shelf
        .keys()
        .filter_map(|name| flat_name(name).map(|flat| (name.clone(), flat)))
        .collect();
    let mut moved = 0;
    for (from, to) in shelved {
        let done = match index.shelf.contains_key(&to) {
            true => remote.delete(&from, None),
            false => remote.renamed(&from, &to),
        };
        match done {
            Ok(()) | Err(Hitch::Missing(_)) => {
                if let Some(seen) = index.shelf.remove(&from) {
                    index.shelf.entry(to.clone()).or_insert(Seen {
                        name: to.clone(),
                        ..seen
                    });
                }
                if let Some(was) = index.spoiled.remove(&from) {
                    index.spoiled.insert(to.clone(), was);
                }
                if let Some(seen) = index.owed.remove(&from) {
                    index.owed.insert(
                        to.clone(),
                        Seen {
                            name: to.clone(),
                            ..seen
                        },
                    );
                }
                if index.vanished.remove(&from) {
                    index.vanished.insert(to);
                }
                moved += 1;
            }
            Err(hitch) => {
                witness::warn(
                    channel::SYNC,
                    "an attachment up there could not leave its shelf yet",
                    &[
                        ("at", Fact::Id(from)),
                        ("why", Fact::Why(format!("{hitch:?}"))),
                    ],
                );
                break;
            }
        }
    }
    moved
}
