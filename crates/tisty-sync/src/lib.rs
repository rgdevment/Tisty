use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use tisty_core::config::Holds;
use tisty_core::witness::{self, Fact, channel};

pub use tisty_core::store::MARKER;

pub const STORE: &str = "store";
const HELD: &str = "attachments";
const PAPERS: &str = "docs";
const CARRIED_TO: &str = "carried-to";

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
    pub astray: Vec<String>,
    pub joined: Vec<String>,
    pub arrived: Vec<String>,
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
    if !dest.is_dir() {
        return Err(Trouble::NotThere(dest.display().to_string()));
    }
    for folder in [STORE, HELD, PAPERS] {
        straight(&dest.join(folder), dest)?;
    }
    let store = data.join(STORE);
    let ours = settled(&store, dest, carried_here(aside, dest))?;

    let again = matches!(way, Way::Again);
    let taking = matches!(way, Way::Both | Way::Pull | Way::Again);
    let giving = matches!(way, Way::Both | Way::Push | Way::Again);

    let mut moved = Moved::default();
    let mut said = None;
    if taking {
        moved.brought = bring(&store, device, dest, &mut moved.unreadable)?;
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
        let who = tisty_core::event::DeviceId(device.to_string());
        let told =
            tisty_core::store::ledger(&store).map_err(|e| Trouble::Unreadable(e.to_string()))?;
        if !told.may_write(&who) {
            return Err(Trouble::NotAllowed(device.to_string()));
        }
        let marker = dest.join(STORE).join(MARKER);
        if std::fs::read_to_string(&marker).ok().as_deref() != Some(ours.as_str()) {
            write(&marker, ours.as_bytes())?;
        }
        let mine = dest.join(STORE).join(device);
        plainly(&mine)?;
        moved.sent = copy_segments(&store.join(device), &mine, again)?;
        moved.sent += hand_on(&store, device, dest, again)?;
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
        return Ok(moved);
    };
    let shut: Vec<String> = told
        .docs
        .values()
        .filter(|paper| told.shut(paper.id))
        .map(|paper| paper.file.clone())
        .collect();
    let buried = buried_now(&told, data);
    let adrift = taking && matches!(unclaimed(dest), Holding::Strays(_));
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
        )?;
        if holds == Holds::Shared {
            moved.freed = let_go_of(data, dest, &carried, tisty_core::attach::COPIED_UP_TO);
        }
    }
    if !alive.is_empty() {
        let papers = carry_papers_leaning_on(data, dest, &alive, &shut, again)?;
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
            None,
        )?;
    }
    note_carried(aside, dest);
    Ok(moved)
}

fn carried_here(aside: Option<&Path>, dest: &Path) -> bool {
    let Some(aside) = aside else { return false };
    std::fs::read_to_string(aside.join(CARRIED_TO))
        .is_ok_and(|last| last.trim() == dest.display().to_string())
}

fn note_carried(aside: Option<&Path>, dest: &Path) {
    let Some(aside) = aside else { return };
    if std::fs::create_dir_all(aside).is_ok() {
        let _ = written(
            &aside.join(CARRIED_TO),
            dest.display().to_string().as_bytes(),
        );
    }
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

pub fn unclaimed(dest: &Path) -> Holding {
    let here = tisty_core::docs::names(&dest.join(PAPERS));
    let named: std::collections::BTreeSet<String> =
        match tisty_core::store::read_all(dest.join(STORE)) {
            Ok(events) => {
                let told = tisty_core::State::replay(&events);
                told.docs
                    .values()
                    .map(|one| one.file.clone())
                    .chain(told.shed.iter().cloned())
                    .collect()
            }
            Err(_) => return Holding::Unreadable,
        };
    match here.difference(&named).count() {
        0 => Holding::Whole,
        adrift => Holding::Strays(adrift),
    }
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

enum Grew {
    Yes,
    No,
    Cannot,
    Unread,
}

enum Whole {
    Said(Vec<u8>),
    Empty,
    Unread,
}

fn whole_of(device_dir: &Path) -> Whole {
    let Ok(segments) = tisty_core::store::segments_in(device_dir) else {
        return Whole::Unread;
    };
    let mut said = Vec::new();
    for at in segments {
        let Ok(more) = std::fs::read(at) else {
            return Whole::Unread;
        };
        said.extend(more);
    }
    if said.is_empty() {
        Whole::Empty
    } else {
        Whole::Said(said)
    }
}

fn one_grew_from_the_other(here: &Path, there: &Path) -> Grew {
    let (ours, theirs) = match (whole_of(here), whole_of(there)) {
        (Whole::Said(ours), Whole::Said(theirs)) => (ours, theirs),
        (Whole::Unread, _) | (_, Whole::Unread) => return Grew::Unread,
        _ => return Grew::Cannot,
    };
    let grew = if ours.len() <= theirs.len() {
        theirs.starts_with(&ours)
    } else {
        ours.starts_with(&theirs)
    };
    if grew { Grew::Yes } else { Grew::No }
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

fn bring(
    store: &Path,
    device: &str,
    dest: &Path,
    unreadable: &mut Vec<String>,
) -> Result<usize, Trouble> {
    let mut brought = 0;
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
        let mine = store.join(named);
        if named.eq_ignore_ascii_case(device) {
            if !settled_already(&entry.path(), &mine) && ours_went_missing(&mine, &entry.path()) {
                match tisty_core::store::alone(&mine) {
                    Some(_held) if ours_went_missing(&mine, &entry.path()) => {
                        witness::warn(
                            channel::SYNC,
                            "this machine's own history was shorter here than in the shared folder, so it was taken back",
                            &[("at", Fact::Id(named.to_string()))],
                        );
                        plainly(&mine)?;
                        brought += copy_segments(&entry.path(), &mine, false)?;
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
        if !settled_already(&entry.path(), &mine) {
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
                    unreadable.push(named.to_string());
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
        brought += copy_segments(&entry.path(), &mine, false)?;
    }

    if brought > 0 {
        tisty_core::store::read_all(store).map_err(|e| Trouble::Unreadable(e.to_string()))?;
    }
    Ok(brought)
}

fn hand_on(store: &Path, device: &str, dest: &Path, again: bool) -> Result<usize, Trouble> {
    let there = dest.join(STORE);
    let Ok(entries) = std::fs::read_dir(store) else {
        return Ok(0);
    };
    let mut sent = 0;
    for entry in entries.filter_map(|e| e.ok()) {
        let named = entry.file_name();
        let Some(named) = named.to_str() else {
            continue;
        };
        if named.eq_ignore_ascii_case(device) || !entry.path().is_dir() {
            continue;
        }
        let theirs = there.join(named);
        if settled_already(&entry.path(), &theirs) || !ours_reaches_further(&entry.path(), &theirs)
        {
            continue;
        }
        plainly(&theirs)?;
        let done = copy_segments(&entry.path(), &theirs, again)?;
        if done > 0 {
            witness::note(
                channel::SYNC,
                "a history this machine was holding for another was handed on",
                &[
                    ("at", Fact::Id(named.to_string())),
                    ("sent", Fact::Count(done)),
                ],
            );
        }
        sent += done;
    }
    Ok(sent)
}

fn ours_went_missing(mine: &Path, theirs: &Path) -> bool {
    let held = match tisty_core::store::distinct_in(mine) {
        Ok(held) => held,
        Err(tisty_core::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => 0,
        Err(_) => return false,
    };
    if tisty_core::store::check_device(theirs).is_err() {
        return false;
    }
    let Ok(coming) = tisty_core::store::distinct_in(theirs) else {
        return false;
    };
    coming > held && (held == 0 || matches!(one_grew_from_the_other(mine, theirs), Grew::Yes))
}

fn ours_reaches_further(mine: &Path, theirs: &Path) -> bool {
    let Ok(ours) = tisty_core::store::distinct_in(mine) else {
        return false;
    };
    match tisty_core::store::check_device(theirs) {
        Ok(_) => {}
        Err(tisty_core::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            return ours > 0;
        }
        Err(_) => return false,
    }
    let Ok(held) = tisty_core::store::distinct_in(theirs) else {
        return false;
    };
    ours > held && (held == 0 || matches!(one_grew_from_the_other(mine, theirs), Grew::Yes))
}

fn settled_already(theirs: &Path, mine: &Path) -> bool {
    let Ok(offered) = tisty_core::store::segments_in(theirs) else {
        return false;
    };
    !offered.is_empty()
        && offered.iter().all(|at| {
            at.file_name()
                .is_some_and(|named| same(at, &mine.join(named)))
        })
}

fn copy_segments(from: &Path, into: &Path, again: bool) -> Result<usize, Trouble> {
    let carried = match tisty_core::store::segments_in(from) {
        Ok(carried) => carried,
        Err(e) => {
            if !matches!(&e, tisty_core::Error::Io(io) if io.kind() == std::io::ErrorKind::NotFound)
            {
                witness::warn(
                    channel::SYNC,
                    "segments unlistable",
                    &[
                        ("at", Fact::Path(from.to_path_buf())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
            return Ok(0);
        }
    };
    if carried.is_empty() {
        return Ok(0);
    }
    std::fs::create_dir_all(into).map_err(io)?;
    sweep(into);
    let mut done = 0;
    for at in carried {
        let Some(named) = at.file_name() else {
            continue;
        };
        let counter = at.with_extension("count");
        if let Some(tally) = counter.file_name().filter(|_| counter.is_file()) {
            let target = into.join(tally);
            if again || !same(&counter, &target) {
                copy_onto(&counter, &target)?;
            }
        }

        let target = into.join(named);
        if !again && same(&at, &target) {
            continue;
        }
        copy_onto(&at, &target)?;
        done += 1;
    }
    Ok(done)
}

fn sweep(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mine = format!(".{}.", std::process::id());
    for at in entries.filter_map(|e| e.ok()).map(|e| e.path()) {
        let ours = at
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".part") && n.contains(&mine));
        if ours && let Err(e) = std::fs::remove_file(&at) {
            witness::warn(
                channel::SYNC,
                "leftover not removed",
                &[
                    ("at", Fact::Path(at.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LetGo {
    pub gone: usize,
    pub freed: u64,
    pub kept: Vec<String>,
}

/// Deletes a local copy only after the one up there is found to hash the same. `told` hears each
/// one as it goes and answers whether to carry on.
/// What a round just put up there, checked by its size where it landed: the bytes were hashed on
/// the way and the name was renamed into place, so reading it back would only ask the cloud for
/// what we wrote a second ago.
fn let_go_of(data: &Path, dest: &Path, carried: &[(String, u64)], above: u64) -> u64 {
    let mut freed = 0;
    for (reference, bytes) in carried {
        if *bytes <= above {
            continue;
        }
        let (Ok(here), Ok(there)) = (
            tisty_core::attach::resolve(reference, data),
            tisty_core::attach::resolve(reference, dest),
        ) else {
            continue;
        };
        let landed =
            std::fs::metadata(&there).is_ok_and(|told| told.is_file() && told.len() == *bytes);
        if landed && std::fs::remove_file(&here).is_ok() {
            freed += bytes;
        }
    }
    freed
}

pub fn let_go_telling(
    data: &Path,
    dest: &Path,
    above: u64,
    told: &mut dyn FnMut(&LetGo) -> bool,
) -> Result<LetGo, Trouble> {
    let mut done = LetGo::default();
    let shed = data.join(HELD);
    let Ok(shelves) = std::fs::read_dir(&shed) else {
        return Ok(done);
    };
    for shelf in shelves.filter_map(|one| one.ok()) {
        if !shelf.path().is_dir() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|one| one.ok()) {
            let at = file.path();
            let Ok(about) = std::fs::metadata(&at) else {
                continue;
            };
            let weighs = about.len();
            if !about.is_file() || weighs <= above {
                continue;
            }
            let under = shelf.file_name();
            let (Some(under), Some(named)) =
                (under.to_str(), at.file_name().and_then(|n| n.to_str()))
            else {
                continue;
            };
            let reference = format!("attachments/{under}/{named}");
            match twinned(
                &dest.join(HELD).join(under).join(named),
                weighs,
                under,
                named,
            ) {
                true => {
                    if std::fs::remove_file(&at).is_ok() {
                        done.gone += 1;
                        done.freed += weighs;
                    }
                }
                false => done.kept.push(reference),
            }
            if !told(&done) {
                return Ok(done);
            }
        }
    }
    Ok(done)
}

fn twinned(there: &Path, weighs: u64, under: &str, named: &str) -> bool {
    if !std::fs::metadata(there).is_ok_and(|told| told.is_file() && told.len() == weighs) {
        return false;
    }
    tisty_core::attach::hashed(there)
        .is_ok_and(|(sha256, _)| tisty_core::attach::vouched(under, named, &sha256))
}

fn left_behind(holds: Holds) -> Option<u64> {
    match holds {
        Holds::Everywhere => None,
        Holds::Mine | Holds::Shared => Some(tisty_core::attach::COPIED_UP_TO),
    }
}

#[allow(clippy::too_many_arguments)]
fn copy_held(
    from: &Path,
    into: &Path,
    buried: &std::collections::BTreeSet<String>,
    again: bool,
    ledger: Option<&Path>,
    above: Option<u64>,
    reachable: Option<&std::collections::BTreeSet<String>>,
    carried: Option<&mut Vec<(String, u64)>>,
) -> Result<usize, Trouble> {
    let mut done = 0;
    let mut left = 0;
    let mut carried = carried;
    let written_down = ledger.map(tisty_core::attach::digests).unwrap_or_default();
    let shelves = match std::fs::read_dir(from) {
        Ok(shelves) => shelves,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::SYNC,
                    "attachments unreadable",
                    &[
                        ("at", Fact::Path(from.to_path_buf())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
            return Ok(0);
        }
    };
    for shelf in shelves.filter_map(|e| e.ok()) {
        if !shelf.path().is_dir() {
            continue;
        }
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        let onto = into.join(shelf.file_name());
        plainly(&onto)?;
        sweep(&onto);
        for file in files.filter_map(|e| e.ok()) {
            let at = file.path();
            if !at.is_file() {
                continue;
            }
            let named = at
                .file_name()
                .and_then(|one| one.to_str())
                .unwrap_or_default();
            let under = shelf.file_name();
            let under = under.to_str().unwrap_or_default();
            // What iCloud left in place of a file is not litter, and saying so would bury the log.
            if tisty_core::icloud::marker(named) {
                continue;
            }
            if !tisty_core::attach::shelved(under, named) {
                witness::warn(
                    channel::SYNC,
                    "something in the shared folder is not shaped like an attachment",
                    &[("at", Fact::Id(format!("attachments/{under}/{named}")))],
                );
                continue;
            }
            let reference = format!("attachments/{under}/{named}");
            if buried.contains(&reference) {
                witness::note(
                    channel::SYNC,
                    "a retired attachment was left where it was instead of coming back",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            }
            if reachable.is_some_and(|named| !named.contains(&reference)) {
                left += 1;
                continue;
            }
            let weighs = std::fs::metadata(&at).map(|m| m.len()).unwrap_or(0);
            if above.is_some_and(|most| weighs > most) {
                continue;
            }
            if weighs > tisty_core::attach::COPIED_IN_DOC {
                witness::warn(
                    channel::SYNC,
                    "something in the shared folder is past what any attachment may weigh",
                    &[("at", Fact::Id(format!("attachments/{under}/{named}")))],
                );
                continue;
            }
            let Some(rest) = at.strip_prefix(from).ok() else {
                continue;
            };
            let target = into.join(rest);
            if !again
                && std::fs::metadata(&at).map(|m| m.len()).ok()
                    == std::fs::metadata(&target).map(|m| m.len()).ok()
            {
                continue;
            }
            let reference = format!("attachments/{under}/{named}");
            let part = beside(&target);
            let ferried = tisty_core::attach::copied(&at, &part, tisty_core::attach::COPIED_IN_DOC);
            let Ok((sha256, bytes)) = ferried else {
                let _ = std::fs::remove_file(&part);
                witness::warn(
                    channel::SYNC,
                    "an attachment could not be carried",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            };
            if !tisty_core::attach::vouched(under, named, &sha256) {
                let _ = std::fs::remove_file(&part);
                witness::warn(
                    channel::SYNC,
                    "an attachment does not hold the bytes its name vouches for",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            }
            if !tisty_core::attach::as_kept(&written_down, &reference, &sha256) {
                let _ = std::fs::remove_file(&part);
                witness::warn(
                    channel::SYNC,
                    "an attachment we already kept came back holding other bytes",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            }
            if std::fs::rename(&part, &target).is_err() {
                let _ = std::fs::remove_file(&part);
                witness::warn(
                    channel::SYNC,
                    "file not carried",
                    &[("at", Fact::Path(target.clone()))],
                );
                continue;
            }
            if let Some(ledger) = ledger {
                tisty_core::attach::noted(ledger, &reference, &sha256, bytes);
            }
            if let Some(carried) = carried.as_deref_mut() {
                carried.push((reference, bytes));
            }
            done += 1;
        }
    }
    if left > 0 {
        witness::note(
            channel::SYNC,
            "this folder's history does not account for its documents, so files nothing names were left up there",
            &[("left", Fact::Count(left))],
        );
    }
    Ok(done)
}

fn same(from: &Path, to: &Path) -> bool {
    use std::io::Read;

    let (Ok(a), Ok(b)) = (std::fs::metadata(from), std::fs::metadata(to)) else {
        return false;
    };
    if a.len() != b.len() {
        return false;
    }
    if a.len() == 0 {
        return true;
    }
    let (Ok(here), Ok(there)) = (std::fs::File::open(from), std::fs::File::open(to)) else {
        return false;
    };
    let mut here = std::io::BufReader::new(here);
    let mut there = std::io::BufReader::new(there);
    let mut one = [0u8; 16 * 1024];
    let mut two = [0u8; 16 * 1024];
    loop {
        let read = match here.read(&mut one) {
            Ok(0) => return true,
            Ok(read) => read,
            Err(_) => return false,
        };
        if there.read_exact(&mut two[..read]).is_err() || one[..read] != two[..read] {
            return false;
        }
    }
}

static ROUND: AtomicU64 = AtomicU64::new(0);

fn write(at: &Path, body: &[u8]) -> Result<(), Trouble> {
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

fn landed(mine: &Path, theirs: &Path) -> bool {
    use tisty_core::docs::print_of;
    match (print_of(mine), print_of(theirs)) {
        (Ok(Some(ours)), Ok(Some(yours))) => ours == yours,
        _ => false,
    }
}

fn joined(data: &Path, dest: &Path, id: &str, mine: &Path, theirs: &Path) -> Option<String> {
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
            at.starts_with(&held) && (at.is_file() || tisty_core::icloud::shed(&at).is_some())
        })
    })
}

fn settled_body(data: &Path, id: &str, mine: &Path, theirs: &Path) {
    let said = std::fs::read_to_string(mine)
        .or_else(|_| std::fs::read_to_string(theirs))
        .ok();
    let Some(said) = said else {
        tisty_core::docs::forget_carried(data, id);
        return;
    };
    if tisty_core::docs::keep_carried(data, id, &said).is_err() {
        witness::warn(
            channel::SYNC,
            "the settled body could not be kept, so the next round has no base to lean on",
            &[("at", Fact::Id(id.to_string()))],
        );
        tisty_core::docs::forget_carried(data, id);
    }
}

pub fn carry_papers(data: &Path, dest: &Path, alive: &[String]) -> Result<Moved, Trouble> {
    carry_papers_leaning_on(data, dest, alive, &[], false)
}

pub fn carry_papers_holding(
    data: &Path,
    dest: &Path,
    alive: &[String],
    shut: &[String],
) -> Result<Moved, Trouble> {
    carry_papers_leaning_on(data, dest, alive, shut, false)
}

fn carry_papers_leaning_on(
    data: &Path,
    dest: &Path,
    alive: &[String],
    shut: &[String],
    again: bool,
) -> Result<Moved, Trouble> {
    use tisty_core::docs::{Carried, Move, moved, print_of};

    let here = data.join(PAPERS);
    let there = dest.join(PAPERS);
    straight(&there, dest)?;
    let was = Carried::read(data);
    let mut said = was.clone();
    let mut done = Moved::default();

    let outcome = (|| -> Result<(), Trouble> {
        for id in alive {
            let (Ok(mine), Ok(theirs)) = (
                tisty_core::docs::resolve(&here, id),
                tisty_core::docs::resolve(&there, id),
            ) else {
                witness::warn(
                    channel::SYNC,
                    "a document was named in a way no document can be named",
                    &[("at", Fact::Id(id.clone()))],
                );
                continue;
            };
            if plainly(&theirs).is_err() || plainly(&mine).is_err() {
                done.astray.push(id.clone());
                continue;
            }
            let (ours, yours) = match (print_of(&mine), print_of(&theirs)) {
                (Ok(ours), Ok(yours)) => (ours, yours),
                (here, there) => {
                    let why = here.err().or(there.err());
                    witness::warn(
                        channel::SYNC,
                        "a document could not be read, so this turn leaves it alone",
                        &[
                            ("at", Fact::Id(id.clone())),
                            (
                                "why",
                                Fact::Why(why.map(|e| e.to_string()).unwrap_or_else(|| "?".into())),
                            ),
                        ],
                    );
                    done.astray.push(id.clone());
                    continue;
                }
            };

            match moved(said.of(id), ours.as_deref(), yours.as_deref()) {
                Move::Nothing => {
                    if again && mine.is_file() {
                        std::fs::create_dir_all(&there).map_err(io)?;
                        copy_onto(&mine, &theirs)?;
                    }
                    if let Some(print) = ours.or(yours) {
                        let steady = said.of(id) == Some(print.as_str())
                            && tisty_core::docs::carried_print(data, id).as_deref()
                                == Some(print.as_str());
                        if !steady {
                            settled_body(data, id, &mine, &theirs);
                        }
                        said.keep(id, &print);
                    }
                }
                Move::Send => {
                    std::fs::create_dir_all(&there).map_err(io)?;
                    copy_onto(&mine, &theirs)?;
                    done.sent += 1;
                    if let Some(print) = ours {
                        settled_body(data, id, &mine, &theirs);
                        said.keep(id, &print);
                    }
                }
                Move::Bring if shut.contains(id) => {
                    witness::warn(
                        channel::SYNC,
                        "a locked document arrived changed, so it waits for the person",
                        &[("at", Fact::Id(id.clone()))],
                    );
                    done.undecided.push(Undecided {
                        id: id.clone(),
                        theirs: yours.unwrap_or_default(),
                    });
                }
                Move::TheyDecide if shut.contains(id) => {
                    witness::warn(
                        channel::SYNC,
                        "a locked document was written on both sides, and no join writes over it",
                        &[("at", Fact::Id(id.clone()))],
                    );
                    done.undecided.push(Undecided {
                        id: id.clone(),
                        theirs: yours.unwrap_or_default(),
                    });
                }
                Move::Bring => {
                    std::fs::create_dir_all(&here).map_err(io)?;
                    let _held = docs_lock(&here, id);
                    copy_onto(&theirs, &mine)?;
                    done.brought += 1;
                    done.arrived.push(id.clone());
                    if let Some(print) = yours {
                        settled_body(data, id, &mine, &theirs);
                        said.keep(id, &print);
                    }
                }
                Move::TheyDecide => {
                    let _held = docs_lock(&here, id);
                    match joined(data, dest, id, &mine, &theirs) {
                        Some(whole) => {
                            write(&mine, whole.as_bytes())?;
                            copy_onto(&mine, &theirs)?;
                            done.sent += 1;
                            done.brought += 1;
                            done.joined.push(id.clone());
                            done.arrived.push(id.clone());
                            if landed(&mine, &theirs) {
                                if let Ok(Some(print)) = print_of(&mine) {
                                    settled_body(data, id, &mine, &theirs);
                                    said.keep(id, &print);
                                }
                            } else {
                                witness::warn(
                                    channel::SYNC,
                                    "another machine wrote while this one joined, so the base stays put",
                                    &[("at", Fact::Id(id.clone()))],
                                );
                            }
                        }
                        None => done.undecided.push(Undecided {
                            id: id.clone(),
                            theirs: yours.unwrap_or_default(),
                        }),
                    }
                }
            }
        }
        Ok(())
    })();

    if said != was {
        said.save(data)
            .map_err(|e| Trouble::Unreadable(e.to_string()))?;
    }
    outcome?;
    Ok(done)
}

/// Unheld beats unwritten: a round that skipped a document comes back for it.
fn docs_lock(here: &Path, id: &str) -> Option<tisty_core::docs::Alone> {
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

fn plainly(at: &Path) -> Result<(), Trouble> {
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
    let mine = ROUND.fetch_add(1, Ordering::Relaxed);
    at.with_extension(format!("{}.{mine}.part", std::process::id()))
}

fn written(at: &Path, body: &[u8]) -> Result<(), Trouble> {
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
    let mine = ROUND.fetch_add(1, Ordering::Relaxed);
    let tmp = at.with_extension(format!("{}.{mine}.part", std::process::id()));

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
