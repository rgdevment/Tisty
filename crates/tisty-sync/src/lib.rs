mod guarding;
mod held;
mod papers;
mod place;
mod segments;
mod shape;
pub mod turned;
mod verified;

pub use held::let_go_telling;
use held::{copy_held, left_behind, let_go_of};
pub use papers::{carry_papers, carry_papers_holding, unclaimed};
use papers::{carry_papers_leaning_on, settled_body, unclaimed_leaning_on};
use place::{carried_here, note_carried};
use segments::{Alike, Grew, Toward, hand_on, one_grew_from_the_other, ours_went_missing, sweep};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use tisty_core::config::Holds;
use tisty_core::witness::{self, Fact, channel};

pub use tisty_core::store::MARKER;

pub const STORE: &str = "store";
const HELD: &str = "attachments";
const PAPERS: &str = "docs";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Trouble {
    NotThere(String),
    OtherStore { theirs: String },
    Unreadable(String),
    Refused(String),
    Broke(String),
    WouldReset { theirs: String },
    NotAllowed(String),
    SameName(String),
    Emptied(String),
    Newer(String),
    Shape(String),
    Unshaped(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Way {
    Both,
    Push,
    Pull,
    Again,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undecided {
    pub id: String,
    pub theirs: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Moved {
    pub sent: usize,
    pub brought: usize,
    pub freed: u64,
    pub undecided: Vec<Undecided>,
    pub unreadable: Vec<String>,
    /// Machines whose history carries a signature that does not answer to the key they published.
    pub disowned: Vec<String>,
    pub unconfirmed: Vec<String>,
    pub astray: Vec<String>,
    pub unprojected: bool,
    pub joined: Vec<String>,
    pub arrived: Vec<String>,
    pub let_go: Vec<String>,
    pub took_in: Vec<(String, String, u64)>,
}

impl Moved {
    pub fn undecided_ids(&self) -> Vec<String> {
        self.undecided.iter().map(|one| one.id.clone()).collect()
    }
}

pub fn carry(
    data: &Path,
    device: &str,
    dest: &Path,
    way: Way,
    alive: &[String],
) -> Result<Moved, Trouble> {
    carry_holding(data, None, device, dest, way, alive, Holds::Everywhere)
}

pub fn carry_leaning_on(
    data: &Path,
    aside: Option<&Path>,
    device: &str,
    dest: &Path,
    way: Way,
    alive: &[String],
) -> Result<Moved, Trouble> {
    carry_holding(data, aside, device, dest, way, alive, Holds::Everywhere)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reached {
    Log,
    Papers,
}

pub fn carry_holding(
    data: &Path,
    aside: Option<&Path>,
    device: &str,
    dest: &Path,
    way: Way,
    alive: &[String],
    holds: Holds,
) -> Result<Moved, Trouble> {
    carry_telling(data, aside, device, dest, way, alive, holds, &mut |_| {})
}

#[allow(clippy::too_many_arguments)]
pub fn carry_telling(
    data: &Path,
    aside: Option<&Path>,
    device: &str,
    dest: &Path,
    way: Way,
    alive: &[String],
    holds: Holds,
    saying: &mut dyn FnMut(Reached),
) -> Result<Moved, Trouble> {
    guarding::before_carrying(dest, device)?;
    shape::before_reading(data, dest)?;
    let store = data.join(STORE);
    let again = matches!(way, Way::Again);
    let ours = settled(&store, dest, carried_here(aside, dest) && !again)?;

    let taking = matches!(way, Way::Both | Way::Pull | Way::Again);
    let giving = matches!(way, Way::Both | Way::Push | Way::Again);

    let mut moved = Moved::default();
    let mut alike = Alike::default();
    let mut said = None;
    if taking {
        moved.brought = bring(data, &store, device, dest, &mut moved, &mut alike)?;
        said = as_told(&store, aside);
        if moved.brought > 0 {
            saying(Reached::Log);
        }
    }
    let mut pushed = None;
    if giving {
        if said.is_none() {
            pushed = as_told(&store, aside);
        }
        if said.as_ref().or(pushed.as_ref()).is_none() {
            return Err(Trouble::Unreadable(store.display().to_string()));
        }
        guarding::allowed_to_write(&store, device)?;
        let marker = dest.join(STORE).join(MARKER);
        if std::fs::read_to_string(&marker).ok().as_deref() != Some(ours.as_str()) {
            write(&marker, ours.as_bytes())?;
        }
        let there = dest.join(STORE).join(device);
        plainly(&there)?;
        moved.sent = alike.carried(device, &there, &store.join(device), Toward::Folder, again)?;
        moved.sent += hand_on(&store, device, dest, again, &mut alike)?;
    }
    let alive: Vec<String> = match &said {
        Some(one) => one.docs.values().map(|paper| paper.file.clone()).collect(),
        None => alive.to_vec(),
    };
    let told = said.or(pushed).or_else(|| as_told(&store, aside));
    let Some(told) = told else {
        witness::warn(
            channel::SYNC,
            "this store would not project, so neither document nor attachment was carried",
            &[],
        );
        moved.astray = alive;
        moved.unprojected = true;
        if giving {
            shape::stamp(data, dest);
        }
        return Ok(moved);
    };
    let shut: Vec<String> = told
        .docs
        .values()
        .filter(|paper| told.shut(paper.id))
        .map(|paper| paper.file.clone())
        .collect();
    let empty: Vec<String> = told
        .docs
        .values()
        .filter(|paper| paper.bytes == Some(0))
        .map(|paper| paper.file.clone())
        .collect();
    let printed: std::collections::BTreeMap<String, String> = told
        .docs
        .values()
        .filter_map(|paper| Some((paper.file.clone(), paper.print.clone()?)))
        .collect();
    let buried = buried_now(&told, data);
    let adrift = taking && matches!(unclaimed_leaning_on(dest, &told), Holding::Strays(_));
    if giving {
        let mut carried = Vec::new();
        moved.sent += copy_held(
            &data.join(HELD),
            &dest.join(HELD),
            &buried,
            again,
            None,
            None,
            None,
            Some(&mut carried),
            &told.kept,
        )?;
        if holds == Holds::Shared {
            (moved.freed, moved.let_go) =
                let_go_of(data, dest, &carried, tisty_core::attach::COPIED_UP_TO);
        }
    }
    if !alive.is_empty() {
        let papers = carry_papers_leaning_on(
            data,
            dest,
            &alive,
            &shut,
            Some(&empty),
            Some(&printed),
            again,
        )?;
        moved.sent += papers.sent;
        moved.brought += papers.brought;
        moved.undecided = papers.undecided;
        moved.astray = papers.astray;
        moved.joined = papers.joined;
        moved.arrived = papers.arrived;
        if papers.brought > 0 {
            saying(Reached::Papers);
        }
    }
    if taking {
        let reachable = adrift.then(|| named_now(&told, data));
        moved.brought += copy_held(
            &dest.join(HELD),
            &data.join(HELD),
            &buried_now(&told, data),
            false,
            Some(data),
            left_behind(holds),
            reachable.as_ref(),
            Some(&mut moved.took_in),
            &told.kept,
        )?;
    }
    // Last of all, so finding it is finding a round that got to the end.
    if giving {
        shape::stamp(data, dest);
    }
    note_carried(aside, dest);
    Ok(moved)
}

fn buried_now(told: &tisty_core::State, data: &Path) -> std::collections::BTreeSet<String> {
    if told.retired.is_empty() {
        return Default::default();
    }
    let named = named_now(told, data);
    told.retired.difference(&named).cloned().collect()
}

fn named_now(told: &tisty_core::State, data: &Path) -> std::collections::BTreeSet<String> {
    let mut named: std::collections::BTreeSet<String> = told
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();
    named.extend(tisty_core::docs::referenced(&data.join(PAPERS)));
    named
}

fn as_told(store: &Path, aside: Option<&Path>) -> Option<tisty_core::State> {
    if let Some(at) = aside
        && let Ok(state) = tisty_core::cache::project(store, at)
    {
        return Some(state);
    }
    tisty_core::store::read_all(store)
        .ok()
        .map(|events| tisty_core::State::replay(&events))
}

fn settled(store: &Path, dest: &Path, carried_here: bool) -> Result<String, Trouble> {
    let ours = tisty_core::store::peek_identity(store);
    let theirs = theirs(dest);
    // Written when the store is made, so having a name of its own says nothing about
    // having a history: what makes a machine new is that it has never written anything.
    let we_are_new = !tisty_core::store::inhabited(store);
    let they_are_new = theirs.is_none() && !tisty_core::store::inhabited(dest.join(STORE));

    if let Some(theirs) = &theirs
        && we_are_new
    {
        write(&store.join(MARKER), theirs.as_bytes())?;
        return Ok(theirs.clone());
    }
    if let (Some(ours), Some(theirs)) = (&ours, &theirs) {
        claims(theirs, ours)?;
        return Ok(ours.clone());
    }
    match (&ours, &theirs) {
        (Some(ours), None) if they_are_new => {
            if carried_here {
                return Err(Trouble::Emptied(dest.display().to_string()));
            }
            return Ok(ours.clone());
        }
        (None, None) if they_are_new => {
            return tisty_core::store::identity(store)
                .map_err(|e| Trouble::Unreadable(e.to_string()));
        }
        _ => {}
    }

    Err(Trouble::WouldReset {
        theirs: theirs.unwrap_or_else(|| dest.display().to_string()),
    })
}

pub fn claims(theirs: &str, ours: &str) -> Result<(), Trouble> {
    let theirs = theirs.trim();
    if theirs.is_empty() || theirs == ours.trim() {
        Ok(())
    } else {
        Err(Trouble::OtherStore {
            theirs: theirs.to_string(),
        })
    }
}

pub fn theirs(dest: &Path) -> Option<String> {
    tisty_core::store::peek_identity(dest.join(STORE))
}

pub fn paper_waiting(dest: &Path, id: &str) -> bool {
    tisty_core::docs::resolve(&dest.join(PAPERS), id).is_ok_and(|at| at.is_file())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Holding {
    Whole,
    Strays(usize),
    Unreadable,
}

pub fn signed_at(dest: &Path) -> Option<String> {
    let events = tisty_core::store::read_all(dest.join(STORE)).ok()?;
    tisty_core::State::replay(&events).signed.alias
}

/// Metadata only, so a window can ask often: nothing here reads a byte of what the folder holds.
pub fn stirring(dest: &Path) -> u64 {
    use std::hash::{Hash, Hasher};

    let mut seen: Vec<(std::path::PathBuf, u64, u64)> = Vec::new();
    let when = |at: &Path| {
        std::fs::metadata(at)
            .and_then(|one| Ok((one.len(), one.modified()?)))
            .ok()
            .map(|(len, when)| {
                (
                    len,
                    when.duration_since(std::time::UNIX_EPOCH)
                        .map(|since| since.as_secs())
                        .unwrap_or(0),
                )
            })
    };

    if let Ok(entries) = std::fs::read_dir(dest.join(STORE)) {
        for entry in entries.filter_map(|one| one.ok()) {
            let Ok(segments) = tisty_core::store::segments_in(&entry.path()) else {
                continue;
            };
            for at in segments {
                if let Some((len, stamped)) = when(&at) {
                    seen.push((at, len, stamped));
                }
            }
        }
    }

    seen.sort();
    let mut told = std::collections::hash_map::DefaultHasher::new();
    seen.hash(&mut told);
    told.finish()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kin {
    Strangers,
    SameLineage,
    Clash(String),
    Unsure(String),
}

pub fn kinship(store: &Path, dest: &Path) -> Kin {
    let there = dest.join(STORE);
    let mut shared = false;

    let Ok(mine) = std::fs::read_dir(store) else {
        return Kin::Unsure(String::new());
    };
    for one in mine.filter_map(|e| e.ok()) {
        let named = one.file_name();
        let Some(named) = named.to_str() else {
            continue;
        };
        if !one.path().is_dir() {
            continue;
        }
        let theirs = there.join(named);
        if !theirs.is_dir() {
            continue;
        }
        match one_grew_from_the_other(&one.path(), &theirs) {
            Grew::Yes => shared = true,
            Grew::No => return Kin::Clash(named.to_string()),
            Grew::Unread => return Kin::Unsure(named.to_string()),
            Grew::Cannot => {}
        }
    }

    if shared {
        return Kin::SameLineage;
    }
    match named_on_both(store, &there).into_iter().next() {
        Some(named) => Kin::Clash(named),
        None => Kin::Strangers,
    }
}

fn every_name(store: &Path) -> std::collections::BTreeSet<String> {
    let mut said: std::collections::BTreeSet<String> = std::fs::read_dir(store)
        .into_iter()
        .flatten()
        .filter_map(|one| one.ok())
        .filter(|one| one.path().is_dir())
        .filter_map(|one| one.file_name().to_str().map(str::to_string))
        .collect();
    if let Ok(events) = tisty_core::store::read_all(store) {
        for one in events {
            match one.op {
                tisty_core::Op::DeviceJoin { d, .. } | tisty_core::Op::DeviceRemove { d } => {
                    said.insert(d.0);
                }
                _ => {}
            }
        }
    }
    said
}

fn named_on_both(store: &Path, there: &Path) -> std::collections::BTreeSet<String> {
    let mine = every_name(store);
    every_name(there)
        .into_iter()
        .filter(|one| mine.contains(one))
        .collect()
}

#[derive(Debug)]
pub struct Stitched {
    pub kin: Kin,
    pub stitch: Option<tisty_core::event::Stitch>,
}

pub fn stitch(data: &Path, device: &str, dest: &Path) -> Result<Stitched, Trouble> {
    if !dest.is_dir() {
        return Err(Trouble::NotThere(dest.display().to_string()));
    }
    for folder in [STORE, HELD, PAPERS] {
        straight(&dest.join(folder), dest)?;
    }
    let store = data.join(STORE);

    let kin = kinship(&store, dest);
    match &kin {
        Kin::Clash(named) => return Err(Trouble::SameName(named.clone())),
        Kin::Unsure(named) => return Err(Trouble::Unreadable(named.clone())),
        _ => {}
    }

    let who = tisty_core::event::DeviceId(device.to_string());
    let told = tisty_core::store::ledger(&store).map_err(|e| Trouble::Unreadable(e.to_string()))?;
    if !told.may_write(&who) {
        return Err(Trouble::NotAllowed(device.to_string()));
    }

    let ours = tisty_core::store::peek_identity(&store)
        .ok_or_else(|| Trouble::Unreadable("this machine has no identity".into()))?;
    let theirs =
        theirs(dest).ok_or_else(|| Trouble::Unreadable("that folder has no identity".into()))?;
    if ours == theirs {
        return Ok(Stitched {
            kin: Kin::SameLineage,
            stitch: None,
        });
    }

    let mine = seats(&store);
    let yours = seats(&dest.join(STORE));

    if kin == Kin::SameLineage {
        write(&store.join(MARKER), theirs.as_bytes())?;
        return Ok(Stitched { kin, stitch: None });
    }

    let seam = tisty_core::event::Stitch {
        absorbed: ours,
        survivor: theirs.clone(),
        ours: mine,
        theirs: yours,
    };
    let mut held = tisty_core::Store::open(&store, tisty_core::DeviceId(device.to_string()))
        .map_err(|e| Trouble::Unreadable(e.to_string()))?;
    held.append(tisty_core::Op::StoresJoined { d: seam.clone() })
        .map_err(|e| Trouble::Broke(e.to_string()))?;
    drop(held);

    write(&store.join(MARKER), theirs.as_bytes())?;
    Ok(Stitched {
        kin,
        stitch: Some(seam),
    })
}

fn seats(store: &Path) -> std::collections::BTreeSet<tisty_core::event::DeviceId> {
    let Ok(entries) = std::fs::read_dir(store) else {
        return Default::default();
    };
    entries
        .filter_map(|e| e.ok())
        .filter(|one| one.path().is_dir())
        .filter_map(|one| one.file_name().to_str().map(str::to_string))
        .map(tisty_core::event::DeviceId)
        .collect()
}

fn anything_signed_in(dir: &Path) -> bool {
    tisty_core::store::segments_in(dir).is_ok_and(|found| {
        found
            .iter()
            .any(|one| one.with_extension(tisty_core::signing::SIG).is_file())
    })
}

/// A machine's own word for what it signs with, read from the log only where nobody has answered
/// for it yet. It catches a history altered without that machine's key, which is the hand anybody
/// who reaches the folder can lay on it; what it cannot catch is the key itself being planted,
/// and that is what a person confirming one closes.
fn claimed(
    store: &Path,
    who: &tisty_core::DeviceId,
    knew: &mut Option<tisty_core::store::Ledger>,
) -> Result<Option<String>, ()> {
    let told = match knew {
        Some(told) => told,
        None => match tisty_core::store::ledger(store) {
            Ok(told) => knew.insert(told),
            Err(e) => {
                witness::warn(
                    channel::SYNC,
                    "this machine's own log would not say what the others sign with, so nothing signed was taken in",
                    &[("why", Fact::Why(e.to_string()))],
                );
                return Err(());
            }
        },
    };
    Ok(told.keys.get(who).cloned())
}

/// Checked before any of it is taken in, and only from where the last round left off.
#[allow(clippy::too_many_arguments)]
fn answers_for_itself(
    data: &Path,
    store: &Path,
    dest: &Path,
    theirs: &Path,
    named: &str,
    device: &str,
    adopting: bool,
    knew: &mut Option<tisty_core::store::Ledger>,
    alike: &mut Alike,
) -> Answered {
    let who = tisty_core::DeviceId(named.to_string());
    let ours = named.eq_ignore_ascii_case(device);
    let from = verified::of(data, dest, named);
    let signed = anything_signed_in(theirs);
    let mut stood = tisty_core::vouched::confirmed(data, &who).map(|one| one.key);
    if stood.is_none() && !ours && signed {
        let says = match claimed(store, &who, knew) {
            Ok(Some(claim)) => Some(claim),
            Ok(None) if store.join(named).is_dir() => None,
            Ok(None) => tisty_core::store::key_said_in(theirs, &who),
            Err(()) => return Answered::Unreadable,
        };
        match says {
            Some(says) if adopting && tisty_core::vouched::confirm(data, &who, &says) => {
                witness::note(
                    channel::SYNC,
                    "reaching this folder for the first time answered for the key of a machine already writing in it",
                    &[("at", Fact::Id(named.to_string()))],
                );
                stood = Some(says);
            }
            Some(_) => {
                witness::note(
                    channel::SYNC,
                    "a machine says it signs what it writes and nobody here has answered for its key, so what it writes waits",
                    &[("at", Fact::Id(named.to_string()))],
                );
                return Answered::Unconfirmed;
            }
            None => {}
        }
    }
    if !signed {
        let owed = from.signing
            || stood.is_some()
            || tisty_core::store::key_said_in(theirs, &who).is_some()
            || tisty_core::store::newest_schema(theirs)
                .is_ok_and(|was| was >= tisty_core::event::SIGNED_FROM);
        if !owed {
            return Answered::Yes;
        }
        witness::warn(
            channel::SYNC,
            "a history that owes a signature arrived with none at all, so none of it was taken in",
            &[("at", Fact::Id(named.to_string()))],
        );
        return Answered::Disowned;
    }
    let said = match stood {
        Some(key) => Some(key),
        None => match claimed(store, &who, knew) {
            Ok(said) => said,
            Err(()) => return Answered::Unreadable,
        },
    };
    let by = said.and_then(|said| tisty_core::signing::read(&said));
    let Some(by) = by else {
        // A machine we hold no key for has nothing to check — but our own name is not one of
        // those: a history signed under it that we cannot answer for is not ours to take back.
        if !ours {
            return Answered::Yes;
        }
        witness::warn(
            channel::SYNC,
            "a history signed under this machine's own name cannot be checked from here, so it was left in the folder",
            &[("at", Fact::Id(named.to_string()))],
        );
        return Answered::Unreadable;
    };
    use tisty_core::answering::Adrift;
    let mine = store.join(named);
    let ours_already = alike.of(named, theirs, &mine).clone();
    let answers = tisty_core::answering::answers(theirs, &who, &by, from, &|segment| {
        ours_already.contains(std::ffi::OsStr::new(segment))
    });
    match answers {
        Ok(held) => {
            verified::keep(data, dest, named, held);
            Answered::Yes
        }
        Err(Adrift::Unreadable(why)) => {
            witness::warn(
                channel::SYNC,
                "a history in the shared folder could not be read through to its signature, so it was left out",
                &[("at", Fact::Id(named.to_string())), ("why", Fact::Why(why))],
            );
            Answered::Unreadable
        }
        Err(Adrift::Disowned(segment)) => {
            // Read again from the first line next round, but never give up knowing it signed:
            // that latch is what tells a signature taken away from a history written before one.
            verified::keep(
                data,
                dest,
                named,
                tisty_core::answering::Reached {
                    signing: true,
                    ..Default::default()
                },
            );
            witness::warn(
                channel::SYNC,
                "a history in the shared folder does not answer to the key kept for that machine, so none of it was taken in",
                &[
                    ("at", Fact::Id(named.to_string())),
                    ("segment", Fact::Id(segment)),
                ],
            );
            Answered::Disowned
        }
    }
}

enum Answered {
    Yes,
    Unreadable,
    Disowned,
    Unconfirmed,
}

fn bring(
    data: &Path,
    store: &Path,
    device: &str,
    dest: &Path,
    moved: &mut Moved,
    alike: &mut Alike,
) -> Result<usize, Trouble> {
    let mut brought = 0;
    let mut knew: Option<tisty_core::store::Ledger> = None;
    let mut away: std::collections::BTreeMap<String, turned::Away> = Default::default();
    let adopting = tisty_core::vouched::all_confirmed(data)
        .keys()
        .all(|who| who.0.eq_ignore_ascii_case(device));
    let at = dest.join(STORE);
    let entries = match std::fs::read_dir(&at) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::SYNC,
                    "folder unreadable",
                    &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
                );
            }
            return Ok(0);
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let named = entry.file_name();
        let Some(named) = named.to_str() else {
            continue;
        };
        if !entry.path().is_dir() {
            continue;
        }
        // Anybody who reaches the folder can name a directory, and a name is what every memo and
        // every ledger line is keyed by. One that could not be a machine of ours never becomes one.
        if !tisty_core::store::is_device_name(named) {
            witness::warn(
                channel::SYNC,
                "the shared folder holds a directory that no machine of ours could be named, so it was left alone",
                &[("at", Fact::Id(named.to_string()))],
            );
            continue;
        }
        let mine = store.join(named);
        if named.eq_ignore_ascii_case(device) {
            if !alike.settled(named, &entry.path(), &mine, Toward::Home)
                && ours_went_missing(&mine, &entry.path())
            {
                match tisty_core::store::alone(&mine) {
                    // Our own name is the one worth wearing: what comes back under it is checked
                    // like anybody else's, or the next thing we write would sign it as ours.
                    Some(_held)
                        if !matches!(
                            answers_for_itself(
                                data,
                                store,
                                dest,
                                &entry.path(),
                                named,
                                device,
                                adopting,
                                &mut knew,
                                alike,
                            ),
                            Answered::Yes
                        ) =>
                    {
                        moved.disowned.push(named.to_string());
                    }
                    Some(_held) if ours_went_missing(&mine, &entry.path()) => {
                        witness::warn(
                            channel::SYNC,
                            "this machine's own history was shorter here than in the shared folder, so it was taken back",
                            &[("at", Fact::Id(named.to_string()))],
                        );
                        plainly(&mine)?;
                        brought +=
                            alike.carried(named, &entry.path(), &mine, Toward::Home, false)?;
                    }
                    Some(_) => {}
                    None => witness::warn(
                        channel::SYNC,
                        "this machine's own history is being written, so it was left as it is",
                        &[("at", Fact::Id(named.to_string()))],
                    ),
                }
            }
            continue;
        }
        plainly(&mine)?;
        match answers_for_itself(
            data,
            store,
            dest,
            &entry.path(),
            named,
            device,
            adopting,
            &mut knew,
            alike,
        ) {
            Answered::Yes => {}
            Answered::Unreadable => {
                away.insert(named.to_string(), turned::Away::Unreadable);
                moved.unreadable.push(named.to_string());
                continue;
            }
            Answered::Disowned => {
                away.insert(named.to_string(), turned::Away::Disowned);
                moved.disowned.push(named.to_string());
                continue;
            }
            Answered::Unconfirmed => {
                away.insert(named.to_string(), turned::Away::Unconfirmed);
                moved.unconfirmed.push(named.to_string());
                continue;
            }
        }
        if !alike.settled(named, &entry.path(), &mine, Toward::Home) {
            let coming = match tisty_core::store::check_device(&entry.path())
                .and_then(|_| tisty_core::store::distinct_in(&entry.path()))
            {
                Ok(coming) => coming,
                Err(tisty_core::Error::UnsupportedVersion(_)) => {
                    witness::warn(
                        channel::SYNC,
                        "another machine writes a newer schema, so nothing was carried either way",
                        &[("at", Fact::Id(named.to_string()))],
                    );
                    return Err(Trouble::Newer(named.to_string()));
                }
                Err(why) => {
                    witness::warn(
                        channel::SYNC,
                        "a machine's history in the shared folder could not be read, so it was left out",
                        &[
                            ("at", Fact::Id(named.to_string())),
                            ("why", Fact::Why(why.to_string())),
                        ],
                    );
                    moved.unreadable.push(named.to_string());
                    continue;
                }
            };
            let held = match tisty_core::store::distinct_in(&mine) {
                Ok(held) => held,
                Err(tisty_core::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => 0,
                Err(why) => {
                    witness::warn(
                        channel::SYNC,
                        "what we hold for a machine cannot be counted, so nothing replaces it",
                        &[
                            ("at", Fact::Id(named.to_string())),
                            ("why", Fact::Why(why.to_string())),
                        ],
                    );
                    continue;
                }
            };
            if coming < held {
                if !matches!(one_grew_from_the_other(&mine, &entry.path()), Grew::Yes) {
                    witness::warn(
                        channel::SYNC,
                        "a shorter history for a machine was left where it was",
                        &[
                            ("at", Fact::Id(named.to_string())),
                            ("held", Fact::Count(held)),
                            ("coming", Fact::Count(coming)),
                        ],
                    );
                }
                continue;
            }
        }
        brought += alike.carried(named, &entry.path(), &mine, Toward::Home, false)?;
    }

    turned::keep(data, &away);
    if brought > 0 {
        tisty_core::store::read_all(store).map_err(|e| Trouble::Unreadable(e.to_string()))?;
    }
    Ok(brought)
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LetGo {
    pub gone: usize,
    pub freed: u64,
    pub kept: Vec<String>,
    pub let_go: Vec<String>,
}

static ROUND: AtomicU64 = AtomicU64::new(0);

pub(crate) fn write(at: &Path, body: &[u8]) -> Result<(), Trouble> {
    written(at, body)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keep {
    Mine,
    Theirs,
    Both,
}

pub fn forget_paper(dest: &Path, id: &str) {
    let Ok(theirs) = tisty_core::docs::resolve(&dest.join(PAPERS), id) else {
        return;
    };
    match std::fs::remove_file(&theirs) {
        Ok(()) => {}
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {}
        Err(why) => witness::warn(
            channel::SYNC,
            "a deleted document kept its file in the shared folder, where the next round will find it again",
            &[
                ("at", Fact::Id(id.to_string())),
                ("why", Fact::Why(why.to_string())),
            ],
        ),
    }
}

pub fn both_papers(data: &Path, dest: &Path, id: &str) -> Result<(String, String), Trouble> {
    straight(&dest.join(PAPERS), dest)?;
    let mine = tisty_core::docs::resolve(&data.join(PAPERS), id)
        .map_err(|_| Trouble::Refused(id.to_string()))?;
    let theirs = tisty_core::docs::resolve(&dest.join(PAPERS), id)
        .map_err(|_| Trouble::Refused(id.to_string()))?;
    plainly(&theirs)?;
    plainly(&mine)?;

    let mine = tisty_core::docs::read(&data.join(PAPERS), id)
        .map_err(|e| Trouble::Unreadable(e.to_string()))?;
    let theirs = tisty_core::docs::read(&dest.join(PAPERS), id)
        .map_err(|e| Trouble::Unreadable(e.to_string()))?;
    Ok((mine, theirs))
}

pub fn settle(data: &Path, dest: &Path, id: &str, keep: Keep) -> Result<Option<String>, Trouble> {
    use tisty_core::docs::{Carried, print_of};

    straight(&dest.join(PAPERS), dest)?;
    let mine = tisty_core::docs::resolve(&data.join(PAPERS), id)
        .map_err(|_| Trouble::Refused(id.to_string()))?;
    let theirs = tisty_core::docs::resolve(&dest.join(PAPERS), id)
        .map_err(|_| Trouble::Refused(id.to_string()))?;
    let mut said = Carried::read(data);
    if said.facing(dest) {
        tisty_core::docs::forget_what_was_carried(data);
    }
    plainly(&theirs)?;
    plainly(&mine)?;

    if keep == Keep::Both {
        let body = tisty_core::docs::read(&dest.join(PAPERS), id)
            .map_err(|e| Trouble::Unreadable(e.to_string()))?;
        witness::warn(
            channel::SYNC,
            "a document that two machines disagreed on was kept twice",
            &[("at", Fact::Id(id.to_string()))],
        );
        return Ok(Some(body));
    }

    match keep {
        Keep::Theirs => {
            let body = tisty_core::docs::read(&dest.join(PAPERS), id)
                .map_err(|e| Trouble::Unreadable(e.to_string()))?;
            std::fs::create_dir_all(data.join(PAPERS)).map_err(io)?;
            written(&mine, body.as_bytes())?;
        }
        _ => {
            std::fs::create_dir_all(dest.join(PAPERS)).map_err(io)?;
            copy_onto(&mine, &theirs)?;
        }
    }

    match print_of(&mine) {
        Ok(Some(print)) => {
            settled_body(data, id, &mine, &theirs);
            said.keep(id, &print);
        }
        Ok(None) => {
            tisty_core::docs::forget_carried(data, id);
            said.forget(id);
        }
        Err(e) => {
            witness::error(
                channel::SYNC,
                "a document was settled but its print could not be read, so the next round may ask again",
                &[
                    ("at", Fact::Id(id.to_string())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            return Err(io(e));
        }
    }
    said.save(data)
        .map_err(|e| Trouble::Unreadable(e.to_string()))?;
    witness::warn(
        channel::SYNC,
        "a document that two machines disagreed on was settled by hand",
        &[
            ("at", Fact::Id(id.to_string())),
            (
                "kept",
                Fact::Word(match keep {
                    Keep::Mine => "mine",
                    Keep::Theirs => "theirs",
                    Keep::Both => "both",
                }),
            ),
        ],
    );
    Ok(None)
}

pub(crate) fn landed(mine: &Path, theirs: &Path) -> bool {
    use tisty_core::docs::print_of;
    match (print_of(mine), print_of(theirs)) {
        (Ok(Some(ours)), Ok(Some(yours))) => ours == yours,
        _ => false,
    }
}

pub(crate) fn joined(
    data: &Path,
    dest: &Path,
    id: &str,
    mine: &Path,
    theirs: &Path,
    stood: Option<&str>,
) -> Option<String> {
    let gave_up = |why: &'static str| {
        witness::note(
            channel::SYNC,
            "two versions of a document could not be joined on their own, so the person is asked",
            &[("at", Fact::Id(id.to_string())), ("why", Fact::Word(why))],
        );
        None::<String>
    };
    let Some(base) = tisty_core::docs::read_carried(data, id) else {
        return gave_up("no version they both came from");
    };
    let printed = tisty_core::attach::printed(tisty_core::docs::settled(&base).as_bytes());
    if Some(printed.as_str()) != stood {
        return gave_up("the version kept beside it is not the one the ledger names");
    }
    let Ok(ours) = std::fs::read_to_string(mine) else {
        return gave_up("this side could not be read");
    };
    let Ok(yours) = std::fs::read_to_string(theirs) else {
        return gave_up("the other side could not be read");
    };
    let Some(whole) = tisty_core::merge::merged(&base, &ours, &yours) else {
        return gave_up("both sides changed the same lines");
    };

    for one in tisty_core::refs::extract(&whole)
        .into_iter()
        .map(|one| one.target)
    {
        if !one.starts_with("attachments/") {
            continue;
        }
        // The shared folder counts: a machine that leaves the big ones there still has them.
        if !anywhere(&one, data, dest) {
            witness::warn(
                channel::SYNC,
                "a joined document would name an attachment nobody here can reach",
                &[("at", Fact::Id(one))],
            );
            return None;
        }
    }
    witness::note(
        channel::SYNC,
        "two versions of a document were joined without asking",
        &[("at", Fact::Id(id.to_string()))],
    );
    Some(whole)
}

fn anywhere(reference: &str, data: &Path, dest: &Path) -> bool {
    [data, dest].iter().any(|root| {
        let held = root.join(HELD);
        tisty_core::attach::resolve(reference, root).is_ok_and(|at| {
            at.starts_with(&held) && (at.is_file() || tisty_core::holes::a_hole(&at))
        })
    })
}

/// Unheld beats unwritten: a round that skipped a document comes back for it.
pub(crate) fn docs_lock(here: &Path, id: &str) -> Option<tisty_core::docs::Alone> {
    let held = tisty_core::docs::hold(here);
    if held.is_none() {
        witness::warn(
            channel::SYNC,
            "a document was written while somebody else held it",
            &[("at", Fact::Id(id.to_string()))],
        );
    }
    held
}

fn copy_onto(from: &Path, at: &Path) -> Result<(), Trouble> {
    plainly(from)?;
    let mut source = std::fs::File::open(from).map_err(io)?;
    laid(at, |file| {
        std::io::copy(&mut source, &mut &*file).map(|_| ())
    })
}

pub(crate) fn plainly(at: &Path) -> Result<(), Trouble> {
    if std::fs::symlink_metadata(at).is_ok_and(|one| one.file_type().is_symlink()) {
        witness::warn(
            channel::SYNC,
            "something in the meeting place points somewhere else, so it was left alone",
            &[("at", Fact::Path(at.to_path_buf()))],
        );
        return Err(Trouble::Refused(at.display().to_string()));
    }
    Ok(())
}

fn straight(at: &Path, under: &Path) -> Result<(), Trouble> {
    let mut walk = at;
    while walk != under {
        if std::fs::symlink_metadata(walk).is_ok_and(|one| one.file_type().is_symlink()) {
            witness::warn(
                channel::SYNC,
                "the meeting place points somewhere else, so nothing was written",
                &[("at", Fact::Path(walk.to_path_buf()))],
            );
            return Err(Trouble::Refused(walk.display().to_string()));
        }
        match walk.parent() {
            Some(up) => walk = up,
            None => break,
        }
    }
    Ok(())
}

/// Beside the file it will become, so the rename that finishes it never crosses a volume.
fn beside(at: &Path) -> std::path::PathBuf {
    if let Some(parent) = at.parent() {
        let _ = std::fs::create_dir_all(parent);
        let _ = tisty_core::paths::ours_alone(parent);
    }
    tisty_core::parting::beside(at, ROUND.fetch_add(1, Ordering::Relaxed))
}

pub(crate) fn written(at: &Path, body: &[u8]) -> Result<(), Trouble> {
    laid(at, |file| std::io::Write::write_all(&mut &*file, body))
}

/// The bytes go through a part file and a rename, so a reader in the shared folder never meets
/// half of anything — and never through memory, because an attachment is as big as it likes.
fn laid(
    at: &Path,
    fill: impl FnOnce(&std::fs::File) -> std::io::Result<()>,
) -> Result<(), Trouble> {
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent).map_err(io)?;
        let _ = tisty_core::paths::ours_alone(parent);
    }
    let tmp = tisty_core::parting::beside(at, ROUND.fetch_add(1, Ordering::Relaxed));

    let done = (|| {
        let file = std::fs::File::create(&tmp)?;
        fill(&file)?;
        file.sync_all()?;
        std::fs::rename(&tmp, at)
    })();

    if let Err(e) = done {
        let _ = std::fs::remove_file(&tmp);
        witness::warn(
            channel::SYNC,
            "file not carried",
            &[
                ("at", Fact::Path(at.to_path_buf())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
        return Err(io(e));
    }
    Ok(())
}

fn io(e: std::io::Error) -> Trouble {
    match e.kind() {
        std::io::ErrorKind::NotFound => Trouble::NotThere(e.to_string()),
        std::io::ErrorKind::PermissionDenied => Trouble::Refused(e.to_string()),
        _ => Trouble::Broke(e.to_string()),
    }
}

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;
