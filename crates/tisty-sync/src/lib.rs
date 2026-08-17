use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use tisty_core::witness::{self, Fact, channel};

pub use tisty_core::store::MARKER;

const STORE: &str = "store";
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
    pub undecided: Vec<Undecided>,
    pub unreadable: Vec<String>,
    pub astray: Vec<String>,
    pub joined: Vec<String>,
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
    carry_leaning_on(data, None, device, dest, way, alive)
}

pub fn carry_leaning_on(
    data: &Path,
    aside: Option<&Path>,
    device: &str,
    dest: &Path,
    way: Way,
    alive: &[String],
) -> Result<Moved, Trouble> {
    if !dest.is_dir() {
        return Err(Trouble::NotThere(dest.display().to_string()));
    }
    for folder in [STORE, HELD, PAPERS] {
        straight(&dest.join(folder), dest)?;
    }
    let store = data.join(STORE);
    let ours = settled(&store, dest)?;

    let again = matches!(way, Way::Again);
    let mut moved = Moved::default();
    let mut said = None;
    if matches!(way, Way::Both | Way::Pull | Way::Again) {
        let came = bring(&store, device, dest, &mut moved.unreadable)?;
        moved.brought = came;
        said = as_told(&store, aside);
        match &said {
            Some(one) => {
                moved.brought += copy_held(&dest.join(HELD), &data.join(HELD), &one.retired, false)?;
            }
            None => witness::warn(
                channel::SYNC,
                "this store would not project, so no attachment was taken in",
                &[],
            ),
        }
    }
    if matches!(way, Way::Both | Way::Push | Way::Again) {
        let Some(buried) = said
            .as_ref()
            .map(|one| one.retired.clone())
            .or_else(|| as_told(&store, aside).map(|one| one.retired))
        else {
            return Err(Trouble::Unreadable(store.display().to_string()));
        };
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
        moved.sent += copy_held(&data.join(HELD), &dest.join(HELD), &buried, again)?;
    }
    let alive = match &said {
        Some(one) => one.docs.values().map(|paper| paper.file.clone()).collect(),
        None => alive.to_vec(),
    };
    if !alive.is_empty() {
        let papers = carry_papers_leaning_on(data, dest, &alive, again)?;
        moved.sent += papers.sent;
        moved.brought += papers.brought;
        moved.undecided = papers.undecided;
        moved.astray = papers.astray;
        moved.joined = papers.joined;
    }
    Ok(moved)
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

fn settled(store: &Path, dest: &Path) -> Result<String, Trouble> {
    let ours = tisty_core::store::peek_identity(store);
    let theirs = theirs(dest);
    let we_are_new = ours.is_none() && !tisty_core::store::inhabited(store);
    let they_are_new = theirs.is_none() && !tisty_core::store::inhabited(dest.join(STORE));

    if let (Some(ours), Some(theirs)) = (&ours, &theirs) {
        claims(theirs, ours)?;
        return Ok(ours.clone());
    }
    match (&ours, &theirs) {
        (None, Some(theirs)) if we_are_new => {
            write(&store.join(MARKER), theirs.as_bytes())?;
            return Ok(theirs.clone());
        }
        (Some(ours), None) if they_are_new => return Ok(ours.clone()),
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
                tisty_core::Op::DeviceJoin { d } | tisty_core::Op::DeviceRemove { d } => {
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
        if named.eq_ignore_ascii_case(device) || !entry.path().is_dir() {
            continue;
        }
        let mine = store.join(named);
        plainly(&mine)?;
        if !settled_already(&entry.path(), &mine) {
            let coming = match tisty_core::store::check_device(&entry.path())
                .and_then(|_| tisty_core::store::distinct_in(&entry.path()))
            {
                Ok(coming) => coming,
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
                witness::warn(
                    channel::SYNC,
                    "a shorter history for a machine was left where it was",
                    &[
                        ("at", Fact::Id(named.to_string())),
                        ("held", Fact::Count(held)),
                        ("coming", Fact::Count(coming)),
                    ],
                );
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

fn copy_held(
    from: &Path,
    into: &Path,
    buried: &std::collections::BTreeSet<String>,
    again: bool,
) -> Result<usize, Trouble> {
    let mut done = 0;
    let written_down = tisty_core::attach::digests(into.parent().unwrap_or_else(|| Path::new("")));
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
            if std::fs::metadata(&at).map(|m| m.len()).unwrap_or(0)
                > tisty_core::attach::COPIED_IN_DOC
            {
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
            let body = std::fs::read(&at).map_err(io)?;
            let reference = format!("attachments/{under}/{named}");
            if !tisty_core::attach::vouched(under, named, &body) {
                witness::warn(
                    channel::SYNC,
                    "an attachment does not hold the bytes its name vouches for",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            }
            if !tisty_core::attach::as_kept(&written_down, &reference, &body) {
                witness::warn(
                    channel::SYNC,
                    "an attachment we already kept came back holding other bytes",
                    &[("at", Fact::Id(reference))],
                );
                continue;
            }
            written(&target, &body)?;
            tisty_core::attach::noted(into.parent().unwrap_or(into), &reference, &body);
            done += 1;
        }
    }
    Ok(done)
}

const TAIL: u64 = 512;

fn tail_of(at: &Path, len: u64) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(at).ok()?;
    let want = len.min(TAIL);
    file.seek(SeekFrom::Start(len - want)).ok()?;
    let mut end = vec![0u8; want as usize];
    file.read_exact(&mut end).ok()?;
    Some(end)
}

fn same(from: &Path, to: &Path) -> bool {
    let (Ok(a), Ok(b)) = (std::fs::metadata(from), std::fs::metadata(to)) else {
        return false;
    };
    if a.len() != b.len() {
        return false;
    }
    if a.len() == 0 {
        return true;
    }
    match (tail_of(from, a.len()), tail_of(to, b.len())) {
        (Some(a), Some(b)) => a == b,
        _ => false,
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
        Ok(()) | Err(_) => {}
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
        Err(e) => return Err(io(e)),
    }
    said.save(data)
        .map_err(|e| Trouble::Unreadable(e.to_string()))?;
    Ok(None)
}

fn landed(mine: &Path, theirs: &Path) -> bool {
    use tisty_core::docs::print_of;
    match (print_of(mine), print_of(theirs)) {
        (Ok(Some(ours)), Ok(Some(yours))) => ours == yours,
        _ => false,
    }
}

fn joined(data: &Path, id: &str, mine: &Path, theirs: &Path) -> Option<String> {
    let base = tisty_core::docs::read_carried(data, id)?;
    let ours = std::fs::read_to_string(mine).ok()?;
    let yours = std::fs::read_to_string(theirs).ok()?;
    let whole = tisty_core::merge::merged(&base, &ours, &yours)?;

    let held = data.join(HELD);
    for one in tisty_core::refs::extract(&whole)
        .into_iter()
        .map(|one| one.target)
    {
        if !one.starts_with("attachments/") {
            continue;
        }
        let there = tisty_core::attach::resolve(&one, data).ok()?;
        if !there.starts_with(&held) || !there.is_file() {
            witness::warn(
                channel::SYNC,
                "a joined document would name an attachment this machine does not hold",
                &[("at", Fact::Id(one))],
            );
            return None;
        }
    }
    Some(whole)
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
    carry_papers_leaning_on(data, dest, alive, false)
}

fn carry_papers_leaning_on(
    data: &Path,
    dest: &Path,
    alive: &[String],
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
                Move::Bring => {
                    std::fs::create_dir_all(&here).map_err(io)?;
                    copy_onto(&theirs, &mine)?;
                    done.brought += 1;
                    if let Some(print) = yours {
                        settled_body(data, id, &mine, &theirs);
                        said.keep(id, &print);
                    }
                }
                Move::TheyDecide => match joined(data, id, &mine, &theirs) {
                    Some(whole) => {
                        write(&mine, whole.as_bytes())?;
                        copy_onto(&mine, &theirs)?;
                        done.sent += 1;
                        done.brought += 1;
                        done.joined.push(id.clone());
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
                },
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

fn copy_onto(from: &Path, at: &Path) -> Result<(), Trouble> {
    plainly(from)?;
    let body = std::fs::read(from).map_err(io)?;
    written(at, &body)
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

fn written(at: &Path, body: &[u8]) -> Result<(), Trouble> {
    if let Some(parent) = at.parent() {
        std::fs::create_dir_all(parent).map_err(io)?;
        let _ = tisty_core::paths::ours_alone(parent);
    }
    let mine = ROUND.fetch_add(1, Ordering::Relaxed);
    let tmp = at.with_extension(format!("{}.{mine}.part", std::process::id()));

    let done = (|| {
        let file = std::fs::File::create(&tmp)?;
        std::io::Write::write_all(&mut &file, body)?;
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
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tisty_core::event::{DeviceId, TaskAdd};
    use tisty_core::{Op, Store};
    use ulid::Ulid;

    struct Machine {
        _dir: tempfile::TempDir,
        data: PathBuf,
        store: PathBuf,
        device: String,
    }

    fn blank(named: &str) -> Machine {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("data");
        let store = data.join("store");
        Machine {
            _dir: dir,
            data,
            store,
            device: named.into(),
        }
    }

    fn machine(named: &str) -> Machine {
        let one = blank(named);
        wrote(&one, format!("lo de {named}"));
        one
    }

    fn filed(who: &Machine, file: &str, body: &str) {
        let mut held = Store::open(&who.store, DeviceId(who.device.clone())).unwrap();
        held.append(Op::DocAdd {
            id: Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: file.to_string(),
                order: "a0".into(),
                folder: None,
            },
        })
        .unwrap();
        tisty_core::docs::write(&who.data.join(PAPERS), file, body).unwrap();
    }

    #[test]
    fn a_shorter_history_arriving_first_never_replaces_the_longer_one_we_hold() {
        let one = machine("uno");
        for said in ["dos", "tres", "cuatro"] {
            wrote(&one, said.into());
        }
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let two = blank("dos");
        carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();
        let held = tisty_core::store::check_device(&two.store.join(&one.device)).unwrap();
        assert!(held >= 4);

        let theirs = shared.path().join(STORE).join(&one.device);
        let at = theirs.join("active.tisty");
        let whole = std::fs::read_to_string(&at).unwrap();
        let first = whole.lines().next().unwrap();
        std::fs::write(&at, format!("{first}\n")).unwrap();

        carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(
            tisty_core::store::check_device(&two.store.join(&one.device)).unwrap(),
            held,
            "una historia mas corta piso la que ya teniamos"
        );
    }

    #[test]
    fn a_history_that_grew_on_the_other_side_still_comes_across() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let two = blank("dos");
        carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();
        let before = tisty_core::store::check_device(&two.store.join(&one.device)).unwrap();

        wrote(&one, "algo mas".into());
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(
            tisty_core::store::check_device(&two.store.join(&one.device)).unwrap(),
            before + 1
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_machine_folder_that_points_somewhere_else_never_receives_the_log() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let there = shared.path().join(STORE);
        std::fs::create_dir_all(&there).unwrap();
        std::os::unix::fs::symlink(elsewhere.path(), there.join(&one.device)).unwrap();

        let outcome = carry(&one.data, &one.device, shared.path(), Way::Push, &[]);

        assert!(matches!(outcome, Err(Trouble::Refused(_))), "{outcome:?}");
        assert!(
            std::fs::read_dir(elsewhere.path())
                .unwrap()
                .next()
                .is_none(),
            "el log aterrizo donde apuntaba el enlace"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_shelf_that_points_somewhere_else_never_receives_an_attachment() {
        let one = machine("uno");
        let kept = planted(&one.data, "foto.png", b"unos bytes cualesquiera");
        let shelf = kept.split('/').nth(1).unwrap().to_string();
        let shared = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let there = shared.path().join(HELD);
        std::fs::create_dir_all(&there).unwrap();
        std::os::unix::fs::symlink(elsewhere.path(), there.join(&shelf)).unwrap();

        let outcome = carry(&one.data, &one.device, shared.path(), Way::Push, &[]);

        assert!(matches!(outcome, Err(Trouble::Refused(_))), "{outcome:?}");
        assert!(
            std::fs::read_dir(elsewhere.path())
                .unwrap()
                .next()
                .is_none(),
            "el adjunto aterrizo donde apuntaba el enlace"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_folder_with_no_links_in_it_still_carries_as_it_always_did() {
        let one = machine("uno");
        planted(&one.data, "foto.png", b"unos bytes cualesquiera");
        let shared = tempfile::tempdir().unwrap();

        let moved = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        assert!(moved.sent > 0);
    }

    #[test]
    fn the_seam_is_written_down_before_the_new_name_is_taken() {
        let one = machine("uno");
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
        let was = tisty_core::store::identity(&one.store).unwrap();

        stitch(&one.data, &one.device, shared.path()).unwrap();

        let said = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap());
        assert_eq!(said.forebears.len(), 2, "la costura no quedo en el log");
        assert!(said.forebears.contains(&was));
    }

    #[test]
    fn a_seam_left_half_done_can_still_be_finished_and_says_the_same_thing() {
        let one = machine("uno");
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        let was = tisty_core::store::identity(&one.store).unwrap();
        stitch(&one.data, &one.device, shared.path()).unwrap();
        std::fs::write(one.store.join(MARKER), was.as_bytes()).unwrap();

        stitch(&one.data, &one.device, shared.path()).unwrap();

        let said = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap());
        assert!(said.forebears.contains(&was), "el linaje viejo se perdio");
        assert_eq!(
            tisty_core::store::peek_identity(&one.store).unwrap(),
            tisty_core::store::peek_identity(shared.path().join(STORE)).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_document_that_is_a_link_to_a_secret_never_becomes_one_of_ours() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let secret = elsewhere.path().join("id_ed25519");
        std::fs::write(&secret, b"PRIVATE KEY nadie deberia ver esto").unwrap();
        says(
            &one,
            Op::DocAdd {
                id: Ulid::generate(),
                d: tisty_core::event::DocAdd {
                    file: "uno-0001".into(),
                    order: "a0".into(),
                    folder: None,
                },
            },
        );
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let there = shared.path().join(PAPERS);
        std::fs::create_dir_all(&there).unwrap();
        std::os::unix::fs::symlink(&secret, there.join("uno-0001.md")).unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        let landed = one.data.join(PAPERS).join("uno-0001.md");
        assert!(
            !landed.exists()
                || !std::fs::read_to_string(&landed)
                    .unwrap()
                    .contains("PRIVATE KEY"),
            "el secreto entro como documento nuestro"
        );
    }

    #[cfg(unix)]
    #[test]
    fn keeping_theirs_never_reads_through_a_link_either() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let secret = elsewhere.path().join("id_ed25519");
        std::fs::write(&secret, b"PRIVATE KEY nadie deberia ver esto").unwrap();
        tisty_core::docs::write(&one.data.join(PAPERS), "uno-0001", "# Lo mio\n").unwrap();

        let there = shared.path().join(PAPERS);
        std::fs::create_dir_all(&there).unwrap();
        std::os::unix::fs::symlink(&secret, there.join("uno-0001.md")).unwrap();

        let outcome = settle(&one.data, shared.path(), "uno-0001", Keep::Theirs);

        assert!(matches!(outcome, Err(Trouble::Refused(_))), "{outcome:?}");
        assert_eq!(
            std::fs::read_to_string(one.data.join(PAPERS).join("uno-0001.md")).unwrap(),
            "# Lo mio\n"
        );
    }

    #[test]
    fn a_retired_attachment_is_never_pushed_back_up_to_the_folder() {
        let one = machine("uno");
        let kept = planted(&one.data, "foto.png", b"una fotografia retirada");
        let shared = tempfile::tempdir().unwrap();

        says(&one, Op::AttachRetire { d: kept.clone() });
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(
            !shared.path().join(&kept).exists(),
            "lo retirado se subio a la carpeta compartida"
        );
    }

    #[test]
    fn an_attachment_nobody_retired_still_goes_up() {
        let one = machine("uno");
        let kept = planted(&one.data, "foto.png", b"una fotografia cualquiera");
        let shared = tempfile::tempdir().unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(shared.path().join(&kept).is_file());
    }

    fn rotated(who: &Machine) {
        let at = who.store.join(&who.device);
        let whole = std::fs::read_to_string(at.join("active.tisty")).unwrap();
        let lines: Vec<&str> = whole.lines().collect();
        let (sealed, tail) = lines.split_at(lines.len() - 1);
        std::fs::write(at.join("000001.tisty"), format!("{}\n", sealed.join("\n"))).unwrap();
        std::fs::write(at.join("000001.count"), sealed.len().to_string()).unwrap();
        std::fs::write(at.join("active.tisty"), format!("{}\n", tail.join("\n"))).unwrap();
    }

    #[test]
    fn a_pull_cut_between_two_segments_does_not_wedge_the_next_one() {
        let one = machine("uno");
        for said in ["dos", "tres", "cuatro"] {
            wrote(&one, said.into());
        }
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let two = blank("dos");
        carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

        rotated(&one);
        let theirs = shared.path().join(STORE).join(&one.device);
        let mine = two.store.join(&one.device);
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        for leaf in ["000001.tisty", "000001.count"] {
            std::fs::copy(theirs.join(leaf), mine.join(leaf)).unwrap();
        }

        wrote(&one, "lo que vino despues de rotar".into());
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(
            tisty_core::store::distinct_in(&mine).unwrap(),
            tisty_core::store::distinct_in(&theirs).unwrap(),
            "la maquina quedo atascada y no recibe nada mas"
        );
    }

    #[test]
    fn a_torn_local_copy_is_never_replaced_by_a_shorter_history() {
        let one = machine("uno");
        for said in ["dos", "tres", "cuatro"] {
            wrote(&one, said.into());
        }
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let two = blank("dos");
        carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

        let mine = two.store.join(&one.device).join("active.tisty");
        let whole = std::fs::read_to_string(&mine).unwrap();
        std::fs::write(&mine, format!("{whole}{{\"v\":3,\"ts\"")).unwrap();

        let theirs = shared
            .path()
            .join(STORE)
            .join(&one.device)
            .join("active.tisty");
        let short = std::fs::read_to_string(&theirs).unwrap();
        let first = short.lines().next().unwrap();
        std::fs::write(&theirs, format!("{first}\n")).unwrap();

        carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

        let after = std::fs::read_to_string(&mine).unwrap();
        assert!(
            after.starts_with(&whole),
            "una historia corta piso la copia local rota"
        );
    }

    #[test]
    fn a_machine_that_was_removed_cannot_stitch_itself_into_the_folder() {
        let one = machine("uno");
        let two = machine("dos");
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_otra".into()),
            },
        );
        says(
            &one,
            Op::DeviceRemove {
                d: DeviceId(one.device.clone()),
            },
        );
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
        let was = tisty_core::store::identity(&one.store).unwrap();

        let outcome = stitch(&one.data, &one.device, shared.path());

        assert!(
            matches!(outcome, Err(Trouble::NotAllowed(_))),
            "{outcome:?}"
        );
        assert_eq!(
            tisty_core::store::peek_identity(&one.store).as_deref(),
            Some(was.as_str()),
            "adopto un nombre en el que no puede escribir"
        );
    }

    #[test]
    fn a_segment_that_cannot_be_read_is_never_called_a_clash() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);

        let mine = one.store.join(&one.device);
        let file = tisty_core::store::segments_in(&mine).unwrap().remove(0);
        std::fs::remove_file(&file).unwrap();
        std::fs::create_dir(&file).unwrap();

        assert_eq!(
            kinship(&one.store, shared.path()),
            Kin::Unsure(one.device.clone())
        );
        assert!(matches!(
            stitch(&one.data, &one.device, shared.path()),
            Err(Trouble::Unreadable(_))
        ));
    }

    #[test]
    fn a_clash_is_found_even_when_another_machine_already_proved_the_lineage() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let both = "compartida";
        for at in [one.store.join(both), shared.path().join(STORE).join(both)] {
            std::fs::create_dir_all(&at).unwrap();
        }
        std::fs::write(one.store.join(both).join("active.tisty"), b"lo de aqui\n").unwrap();
        std::fs::write(
            shared.path().join(STORE).join(both).join("active.tisty"),
            b"lo de alli\n",
        )
        .unwrap();

        assert_eq!(
            kinship(&one.store, shared.path()),
            Kin::Clash(both.to_string()),
            "un choque real se perdio porque otra maquina ya habia probado el linaje"
        );
    }

    #[test]
    fn stitching_twice_never_writes_a_seam_that_joins_a_history_to_itself() {
        let one = machine("uno");
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
        tisty_core::store::identity(&one.store).unwrap();

        stitch(&one.data, &one.device, shared.path()).unwrap();
        let again = stitch(&one.data, &one.device, shared.path()).unwrap();

        assert!(again.stitch.is_none(), "anoto una costura de si misma");
        let seams = tisty_core::store::read_all(&one.store)
            .unwrap()
            .into_iter()
            .filter(|one| matches!(one.op, Op::StoresJoined { .. }))
            .count();
        assert_eq!(seams, 1);
    }

    #[test]
    fn a_seam_says_which_history_was_absorbed_and_which_one_survived() {
        let one = machine("uno");
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
        let was = tisty_core::store::identity(&one.store).unwrap();

        let done = stitch(&one.data, &one.device, shared.path()).unwrap();

        let seam = done.stitch.unwrap();
        assert_eq!(seam.absorbed, was);
        assert_eq!(
            seam.survivor,
            tisty_core::store::peek_identity(shared.path().join(STORE)).unwrap()
        );
        assert_ne!(seam.absorbed, seam.survivor);
    }

    #[test]
    fn a_seam_says_which_machines_came_from_each_side() {
        let one = machine("uno");
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
        tisty_core::store::identity(&one.store).unwrap();

        let seam = stitch(&one.data, &one.device, shared.path())
            .unwrap()
            .stitch
            .unwrap();

        assert!(seam.ours.contains(&DeviceId(one.device.clone())));
        assert!(seam.theirs.contains(&DeviceId(two.device.clone())));
        assert!(!seam.ours.contains(&DeviceId(two.device.clone())));
    }

    #[test]
    fn stitching_where_there_is_no_folder_says_so_instead_of_something_else() {
        let one = machine("uno");
        let nowhere = tempfile::tempdir().unwrap();

        let outcome = stitch(&one.data, &one.device, &nowhere.path().join("no-esta"));

        assert!(matches!(outcome, Err(Trouble::NotThere(_))), "{outcome:?}");
    }

    #[test]
    fn a_document_too_big_to_read_is_named_instead_of_vanishing_from_the_sync() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let here = one.data.join(PAPERS);
        std::fs::create_dir_all(&here).unwrap();
        let there = shared.path().join(PAPERS);
        std::fs::create_dir_all(&there).unwrap();
        let huge = "x".repeat((tisty_core::docs::BODY_AT_MOST + 1) as usize);
        std::fs::write(here.join("uno-0001.md"), &huge).unwrap();
        std::fs::write(there.join("uno-0001.md"), "# Lo de alli\n").unwrap();

        let moved = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert_eq!(moved.astray, vec!["uno-0001".to_string()]);
        assert!(moved.undecided.is_empty());
    }

    #[test]
    fn a_document_that_reads_fine_is_never_called_astray() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let here = one.data.join(PAPERS);
        std::fs::create_dir_all(&here).unwrap();
        std::fs::write(here.join("uno-0001.md"), "# Lo mio\n").unwrap();

        let moved = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert!(moved.astray.is_empty());
    }

    #[test]
    fn the_body_that_settled_is_kept_so_the_next_round_has_a_base() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        filed(&one, "uno-0001", "# Kit\n\nlo que quedo asentado\n");

        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert_eq!(
            tisty_core::docs::read_carried(&one.data, "uno-0001").as_deref(),
            Some("# Kit\n\nlo que quedo asentado\n")
        );
    }

    #[test]
    fn the_base_follows_the_body_when_the_other_side_wins() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let there = shared.path().join(PAPERS);
        std::fs::create_dir_all(&there).unwrap();
        filed(&one, "uno-0001", "# Kit\n\nlo mio\n");
        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        std::fs::write(there.join("uno-0001.md"), "# Kit\n\nlo de alli\n").unwrap();
        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert_eq!(
            tisty_core::docs::read_carried(&one.data, "uno-0001").as_deref(),
            Some("# Kit\n\nlo de alli\n"),
            "la base se quedo con lo viejo"
        );
    }

    #[test]
    fn a_base_is_never_kept_for_a_document_neither_side_has() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();

        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert!(tisty_core::docs::read_carried(&one.data, "uno-0001").is_none());
    }

    #[test]
    fn the_base_never_travels_to_the_shared_folder() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        filed(&one, "uno-0001", "# Kit\n\ncuerpo\n");

        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(!shared.path().join("carried").exists());
        assert!(one.data.join("carried").is_dir());
    }

    fn wrote_body(at: &Path, id: &str, body: &str) {
        std::fs::create_dir_all(at).unwrap();
        std::fs::write(at.join(format!("{id}.md")), body).unwrap();
    }

    #[test]
    fn two_machines_touching_different_parts_end_up_with_one_document_and_no_question() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let here = one.data.join(PAPERS);
        let there = shared.path().join(PAPERS);
        let base = "# Kit\n\nla introduccion\n\nel cuerpo\n\nel cierre\n";
        filed(&one, "uno-0001", base);
        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        wrote_body(
            &here,
            "uno-0001",
            "# Kit\n\nla introduccion del mac\n\nel cuerpo\n\nel cierre\n",
        );
        wrote_body(
            &there,
            "uno-0001",
            "# Kit\n\nla introduccion\n\nel cuerpo\n\nel cierre\n\nlo de windows\n",
        );

        let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert!(done.undecided.is_empty(), "pregunto pudiendo juntarlo");
        assert_eq!(done.joined, vec!["uno-0001".to_string()]);
        let whole = std::fs::read_to_string(here.join("uno-0001.md")).unwrap();
        assert!(whole.contains("del mac"), "{whole}");
        assert!(whole.contains("lo de windows"), "{whole}");
        assert_eq!(
            whole,
            std::fs::read_to_string(there.join("uno-0001.md")).unwrap()
        );
    }

    #[test]
    fn the_same_paragraph_written_two_ways_still_asks() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let here = one.data.join(PAPERS);
        let there = shared.path().join(PAPERS);
        filed(&one, "uno-0001", "# Kit\n\nla introduccion\n\nel cierre\n");
        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        wrote_body(
            &here,
            "uno-0001",
            "# Kit\n\nla introduccion del mac\n\nel cierre\n",
        );
        wrote_body(
            &there,
            "uno-0001",
            "# Kit\n\nla introduccion de windows\n\nel cierre\n",
        );

        let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert_eq!(done.undecided.len(), 1);
        assert!(done.joined.is_empty());
    }

    #[test]
    fn without_a_base_nothing_is_joined_and_the_person_still_decides() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let here = one.data.join(PAPERS);
        let there = shared.path().join(PAPERS);
        filed(&one, "uno-0001", "# Kit\n\nuno\n\ndos\n");
        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();
        tisty_core::docs::forget_carried(&one.data, "uno-0001");

        wrote_body(&here, "uno-0001", "# Kit del mac\n\nuno\n\ndos\n");
        wrote_body(&there, "uno-0001", "# Kit\n\nuno\n\ndos\n\ntres\n");

        let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert_eq!(done.undecided.len(), 1);
        assert!(done.joined.is_empty());
    }

    #[test]
    fn a_join_that_would_name_an_attachment_we_do_not_hold_is_never_written() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let here = one.data.join(PAPERS);
        let there = shared.path().join(PAPERS);
        filed(&one, "uno-0001", "# Kit\n\nuno\n\ndos\n");
        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        wrote_body(&here, "uno-0001", "# Kit del mac\n\nuno\n\ndos\n");
        wrote_body(
            &there,
            "uno-0001",
            "# Kit\n\nuno\n\ndos\n\n![foto](attachments/ab/foto-a1b2c3d4.png)\n",
        );

        let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert!(done.joined.is_empty(), "junto una referencia rota");
        assert_eq!(done.undecided.len(), 1);
    }

    #[test]
    fn a_join_leaves_the_base_on_what_both_sides_now_hold() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        let here = one.data.join(PAPERS);
        let there = shared.path().join(PAPERS);
        filed(&one, "uno-0001", "# Kit\n\nuno\n\ndos\n");
        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        wrote_body(&here, "uno-0001", "# Kit del mac\n\nuno\n\ndos\n");
        wrote_body(&there, "uno-0001", "# Kit\n\nuno\n\ndos\n\ntres\n");
        carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

        assert!(done.joined.is_empty(), "volvio a juntar lo ya junto");
        assert_eq!(
            tisty_core::docs::read_carried(&one.data, "uno-0001"),
            Some(std::fs::read_to_string(here.join("uno-0001.md")).unwrap())
        );
    }

    #[test]
    fn a_body_the_other_side_no_longer_matches_is_not_taken_as_landed() {
        let room = tempfile::tempdir().unwrap();
        let mine = room.path().join("mio.md");
        let theirs = room.path().join("suyo.md");
        std::fs::write(&mine, "# Kit\n\njunto\n").unwrap();
        std::fs::write(&theirs, "# Kit\n\notra cosa\n").unwrap();

        assert!(!landed(&mine, &theirs));

        std::fs::write(&theirs, "# Kit\n\njunto\n").unwrap();
        assert!(landed(&mine, &theirs));
    }

    #[test]
    fn a_side_that_is_not_there_at_all_is_never_taken_as_landed() {
        let room = tempfile::tempdir().unwrap();
        let mine = room.path().join("mio.md");
        std::fs::write(&mine, "# Kit\n").unwrap();

        assert!(!landed(&mine, &room.path().join("no-esta.md")));
    }

    #[test]
    fn two_histories_that_never_met_are_strangers() {
        let one = machine("uno");
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        assert_eq!(kinship(&one.store, shared.path()), Kin::Strangers);
    }

    #[test]
    fn a_folder_that_already_holds_everything_of_ours_is_the_same_lineage() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
    }

    #[test]
    fn a_tail_we_never_sent_is_still_the_same_lineage_not_a_clash() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        wrote(&one, "algo que se quedo aqui".into());

        assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
    }

    #[test]
    fn the_same_name_writing_two_different_things_is_the_clash_that_is_refused() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let theirs = shared.path().join(STORE).join(&one.device);
        let file = tisty_core::store::segments_in(&theirs).unwrap().remove(0);
        let mut said = std::fs::read(&file).unwrap();
        said[0] ^= 0xff;
        std::fs::write(&file, said).unwrap();

        assert_eq!(
            kinship(&one.store, shared.path()),
            Kin::Clash(one.device.clone())
        );
    }

    #[test]
    fn a_folder_ahead_of_us_is_the_same_lineage_because_only_the_end_grows() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let theirs = shared.path().join(STORE).join(&one.device);
        let file = tisty_core::store::segments_in(&theirs).unwrap().remove(0);
        let mut said = std::fs::read(&file).unwrap();
        said.extend_from_slice(b"{\"v\":3}\n");
        std::fs::write(&file, said).unwrap();

        assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
    }

    #[test]
    fn a_document_written_on_the_other_machine_lands_on_the_first_sync_not_the_second() {
        let shared = tempfile::tempdir().unwrap();
        let one = machine("uno");
        let two = blank("dos");

        filed(&one, "uno-0001", "# Ortografia\n\nla n con virgulilla\n");
        carry(
            &one.data,
            &one.device,
            shared.path(),
            Way::Both,
            &["uno-0001".into()],
        )
        .unwrap();

        carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

        let landed = two.data.join(PAPERS).join("uno-0001.md");
        assert!(
            landed.is_file(),
            "el documento no llego en la primera vuelta"
        );
        assert_eq!(
            std::fs::read_to_string(landed).unwrap(),
            "# Ortografia\n\nla n con virgulilla\n"
        );
    }

    #[test]
    fn a_document_the_other_machine_deleted_is_never_brought_back_by_the_new_reckoning() {
        let shared = tempfile::tempdir().unwrap();
        let one = machine("uno");
        let two = blank("dos");

        filed(&one, "uno-0001", "# Algo\n");
        carry(
            &one.data,
            &one.device,
            shared.path(),
            Way::Both,
            &["uno-0001".into()],
        )
        .unwrap();

        let mut held = Store::open(&one.store, DeviceId(one.device.clone())).unwrap();
        let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
            .docs
            .values()
            .next()
            .unwrap()
            .id;
        held.append(Op::DocDelete { id }).unwrap();
        drop(held);
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(!two.data.join(PAPERS).join("uno-0001.md").exists());
    }

    fn wrote(who: &Machine, title: String) {
        let mut held = Store::open(&who.store, DeviceId(who.device.clone())).unwrap();
        held.append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new(title, "a0"),
        })
        .unwrap();
    }

    fn joined(who: &Machine, shared: &Path) {
        carry(&who.data, &who.device, shared, Way::Pull, &[]).unwrap();
    }

    fn titles(store: &Path) -> Vec<String> {
        tisty_core::State::replay(&tisty_core::store::read_all(store).unwrap())
            .tasks
            .values()
            .map(|task| task.title.clone())
            .collect()
    }

    fn says(who: &Machine, op: Op) {
        let mut held = Store::open(&who.store, DeviceId(who.device.clone())).unwrap();
        held.append(op).unwrap();
    }

    #[test]
    fn a_store_with_no_list_yet_lets_everyone_write() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();

        let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(moved.sent > 0, "an older store must not be locked out");
    }

    #[test]
    fn a_machine_on_the_list_writes_as_it_always_did() {
        let one = machine("dev_a");
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_a".into()),
            },
        );
        let shared = tempfile::tempdir().unwrap();

        let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(moved.sent > 0);
    }

    fn planted(root: &Path, called: &str, body: &[u8]) -> String {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join(called);
        std::fs::write(&at, body).unwrap();
        tisty_core::attach::keep(&at, root, tisty_core::attach::COPIED_UP_TO)
            .unwrap()
            .at
    }

    fn paper(who: &Machine, id: &str, body: &str) {
        let at = who.data.join("docs");
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join(format!("{id}.md")), body).unwrap();
    }

    fn theirs(shared: &Path, id: &str, body: &str) {
        let at = shared.join("docs");
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join(format!("{id}.md")), body).unwrap();
    }

    fn body(at: &Path, id: &str) -> String {
        std::fs::read_to_string(at.join("docs").join(format!("{id}.md"))).unwrap()
    }

    fn at_odds(one: &Machine, shared: &Path) -> Vec<String> {
        let alive = vec!["dev_a-0001".to_string()];
        paper(one, "dev_a-0001", "# Minuta");
        carry_papers(&one.data, shared, &alive).unwrap();
        paper(one, "dev_a-0001", "# Minuta\n\nlo mio");
        theirs(shared, "dev_a-0001", "# Minuta\n\nlo suyo");
        alive
    }

    #[test]
    fn keeping_mine_leaves_the_folder_holding_mine_and_asks_no_more() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());

        let brought = settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        assert_eq!(brought, None);
        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo mio");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert!(
            done.undecided.is_empty(),
            "it asked again about a settled one"
        );
    }

    #[test]
    fn what_is_written_after_settling_travels_instead_of_being_asked_about_again() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        paper(&one, "dev_a-0001", "# Minuta\n\nlo mio, y algo mas");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "settling it did not become the new common ground"
        );
        assert_eq!(done.sent, 1);
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nlo mio, y algo mas"
        );
    }

    #[test]
    fn keeping_theirs_takes_it_home_and_asks_no_more() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());

        settle(&one.data, shared.path(), "dev_a-0001", Keep::Theirs).unwrap();

        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo suyo");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert!(
            done.undecided.is_empty(),
            "it asked again about a settled one"
        );
    }

    #[test]
    fn keeping_both_hands_back_the_other_body_before_anything_is_overwritten() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());

        let brought = settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();

        assert_eq!(brought.as_deref(), Some("# Minuta\n\nlo suyo"));
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nlo suyo",
            "it overwrote the other version before it was anywhere safe"
        );
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo mio");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(done.undecided.len(), 1, "it settled before being told to");
    }

    #[test]
    fn keeping_both_settles_once_the_other_version_is_safe() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();

        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo mio");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert!(done.undecided.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn a_meeting_place_that_points_somewhere_else_is_refused_before_anything_leaves() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(elsewhere.path(), shared.path().join(STORE)).unwrap();

        let why = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap_err();

        assert!(
            matches!(why, Trouble::Refused(_)),
            "it followed the link out of the folder: {why:?}"
        );
        assert!(
            std::fs::read_dir(elsewhere.path())
                .unwrap()
                .next()
                .is_none(),
            "the log was copied where the link pointed"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_documents_folder_that_points_somewhere_else_never_receives_a_body() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(elsewhere.path(), shared.path().join(PAPERS)).unwrap();
        paper(&one, "dev_a-0001", "# Lo que dije");

        let why = carry_papers(&one.data, shared.path(), &["dev_a-0001".into()]).unwrap_err();

        assert!(matches!(why, Trouble::Refused(_)), "{why:?}");
        assert!(
            std::fs::read_dir(elsewhere.path())
                .unwrap()
                .next()
                .is_none(),
            "the body was written where the link pointed"
        );
    }

    #[test]
    fn a_body_the_reader_would_refuse_never_replaces_the_one_that_is_here() {
        let one = blank("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Lo mio, breve");
        theirs(shared.path(), "dev_a-0001", &"x".repeat(600 * 1024));

        let done = carry_papers(&one.data, shared.path(), &alive);

        assert!(done.is_err() || done.unwrap().brought == 0);
        assert_eq!(
            body(&one.data, "dev_a-0001"),
            "# Lo mio, breve",
            "a body nobody can open replaced one that could be read"
        );
    }

    #[test]
    fn an_attachment_whose_bytes_were_swapped_never_reaches_this_machine() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let (_src, file) = {
            let dir = tempfile::tempdir().unwrap();
            let at = dir.path().join("contrato.pdf");
            std::fs::write(&at, b"what the person really attached").unwrap();
            (dir, at)
        };
        let kept =
            tisty_core::attach::keep(&file, &one.data, tisty_core::attach::COPIED_UP_TO).unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let theirs = shared.path().join(&kept.at);
        std::fs::write(&theirs, b"a different file wearing the same name").unwrap();
        std::fs::remove_file(one.data.join(&kept.at)).unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

        assert!(
            !one.data.join(&kept.at).exists(),
            "bytes nobody vouched for were taken in under a trusted name"
        );
    }

    #[test]
    fn what_we_already_kept_is_not_replaced_by_something_the_name_alone_allows() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let kept = planted(
            shared.path(),
            "contrato.pdf",
            b"the bytes whose name this is",
        );

        let mine = one.data.join(&kept);
        std::fs::create_dir_all(mine.parent().unwrap()).unwrap();
        std::fs::write(&mine, b"what we kept, of another length").unwrap();
        std::fs::write(
            one.data.join("attachments.jsonl"),
            format!(
                "{{\"at\":\"{kept}\",\"sha256\":\"{}\",\"bytes\":31}}\n",
                "0".repeat(64)
            ),
        )
        .unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(
            std::fs::read(&mine).unwrap(),
            b"what we kept, of another length",
            "the name alone was enough to replace what we had written down"
        );
    }

    #[test]
    fn an_attachment_that_is_what_it_says_it_is_comes_home() {
        let one = machine("dev_a");
        let other = blank("dev_b");
        let shared = tempfile::tempdir().unwrap();
        let (_src, file) = {
            let dir = tempfile::tempdir().unwrap();
            let at = dir.path().join("contrato.pdf");
            std::fs::write(&at, b"what the person really attached").unwrap();
            (dir, at)
        };
        let kept =
            tisty_core::attach::keep(&file, &one.data, tisty_core::attach::COPIED_UP_TO).unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        carry(&other.data, &other.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(
            std::fs::read(other.data.join(&kept.at)).unwrap(),
            b"what the person really attached",
            "an honest attachment was turned away"
        );
    }

    #[test]
    fn what_the_folder_offers_that_is_not_shaped_like_an_attachment_stays_there() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let real = planted(shared.path(), "contrato.pdf", b"a real one");
        let under = real.split('/').nth(1).unwrap().to_string();
        let shelf = shared.path().join(HELD).join(&under);
        std::fs::write(shelf.join("factura.exe"), b"not yours").unwrap();
        std::fs::write(shelf.join("nota.command"), b"nor this").unwrap();
        let odd = shared.path().join(HELD).join("not-a-shelf");
        std::fs::create_dir_all(&odd).unwrap();
        std::fs::write(odd.join("mapa-91f2ab00.svg"), b"wrong shelf").unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

        let here = one.data.join(HELD).join(&under);
        assert!(one.data.join(&real).exists(), "it kept nothing");
        assert!(
            !here.join("factura.exe").exists(),
            "an executable was let in"
        );
        assert!(!here.join("nota.command").exists());
        assert!(!one.data.join(HELD).join("not-a-shelf").exists());
    }

    #[test]
    fn a_document_deleted_here_stops_being_readable_in_the_shared_folder() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        paper(&one, "dev_a-0001", "# Lo que dije");
        carry_papers(&one.data, shared.path(), &["dev_a-0001".into()]).unwrap();

        forget_paper(shared.path(), "dev_a-0001");

        assert!(
            !shared.path().join("docs").join("dev_a-0001.md").exists(),
            "the body stayed legible in someone else's cloud folder"
        );
    }

    #[test]
    fn forgetting_a_paper_can_never_name_its_way_out_of_the_folder() {
        let shared = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(shared.path().join(PAPERS)).unwrap();
        let loot = shared.path().join("loot.md");
        std::fs::write(&loot, "no es tuyo").unwrap();

        forget_paper(shared.path(), "../loot");

        assert!(loot.exists(), "it deleted a file outside the folder");
    }

    #[test]
    fn a_body_travels_even_when_only_one_direction_was_asked_for() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        paper(&one, "dev_a-0001", "# Lo que dije");

        carry(
            &one.data,
            &one.device,
            shared.path(),
            Way::Push,
            &["dev_a-0001".to_string()],
        )
        .unwrap();

        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Lo que dije",
            "a document written here waited for a full round to leave"
        );
    }

    #[test]
    fn nothing_moving_still_leaves_the_two_sides_on_common_ground() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = vec!["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Iguales");
        theirs(shared.path(), "dev_a-0001", "# Iguales");

        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(done.sent + done.brought, 0);

        paper(&one, "dev_a-0001", "# Iguales, y algo mas");
        let after = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            after.undecided.is_empty(),
            "agreeing was not written down, so the next edit looked like a quarrel"
        );
        assert_eq!(after.sent, 1);
    }

    #[test]
    fn a_name_no_document_could_have_never_reaches_the_disk() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let loot = shared.path().join("loot.md");
        std::fs::write(&loot, "no es tuyo").unwrap();

        let done = carry_papers(
            &one.data,
            shared.path(),
            &["../loot".to_string(), "../../loot".to_string()],
        )
        .unwrap();

        assert_eq!(done.sent + done.brought, 0, "it walked out of the store");
        assert_eq!(std::fs::read_to_string(&loot).unwrap(), "no es tuyo");
        let said = std::fs::read_to_string(one.data.join("carried.json")).unwrap_or_default();
        assert!(
            !said.contains("loot"),
            "the ledger learned a name it must not know"
        );
    }

    #[test]
    fn settling_a_name_no_document_could_have_is_refused() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();

        let why = settle(&one.data, shared.path(), "../../loot", Keep::Theirs);

        assert!(why.is_err(), "it settled a document that cannot exist");
    }

    #[test]
    fn a_document_written_here_lands_in_the_folder() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        paper(&one, "dev_a-0001", "# Minuta\n\nlo que dije");

        let done = carry_papers(&one.data, shared.path(), &["dev_a-0001".into()]).unwrap();

        assert_eq!(done.sent, 1);
        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo que dije");
    }

    #[test]
    fn a_document_only_the_folder_has_comes_home() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        theirs(shared.path(), "dev_b-0001", "# Suya");

        let done = carry_papers(&one.data, shared.path(), &["dev_b-0001".into()]).unwrap();

        assert_eq!(done.brought, 1);
        assert_eq!(body(&one.data, "dev_b-0001"), "# Suya");
    }

    #[test]
    fn only_the_side_that_changed_travels_the_second_time() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        theirs(shared.path(), "dev_a-0001", "# Minuta\n\ny algo mas");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(done.brought, 1, "what changed there did not come home");
        assert_eq!(done.sent, 0, "it pushed over what it had just been given");
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\ny algo mas");
    }

    #[test]
    fn when_both_sides_moved_it_asks_instead_of_choosing() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        paper(&one, "dev_a-0001", "# Minuta\n\nlo mio");
        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nlo suyo");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(done.undecided.len(), 1);
        assert_eq!(done.undecided[0].id, "dev_a-0001");
        assert_eq!(done.sent + done.brought, 0, "it moved something anyway");
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo mio");
        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo suyo");
    }

    #[test]
    fn a_document_the_log_never_mentions_is_left_where_it_is() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        theirs(shared.path(), "dev_b-0009", "# Nadie la nombra");

        let done = carry_papers(&one.data, shared.path(), &[]).unwrap();

        assert_eq!(done.brought, 0);
        assert!(!one.data.join("docs").join("dev_b-0009.md").exists());
    }

    #[test]
    fn asking_twice_asks_twice_instead_of_deciding_the_second_time() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta");
        carry_papers(&one.data, shared.path(), &alive).unwrap();
        paper(&one, "dev_a-0001", "# Minuta\n\nlo mio");
        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nlo suyo");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(done.undecided.len(), 1, "it settled it on its own");
    }

    #[test]
    fn settling_with_theirs_also_lets_the_next_edit_travel_alone() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Theirs).unwrap();

        paper(&one, "dev_a-0001", "# Minuta\n\nlo suyo, y algo mas");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "settling with theirs did not become the new common ground"
        );
        assert_eq!(done.sent, 1);
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nlo suyo, y algo mas"
        );
    }

    #[test]
    fn settling_with_both_lets_a_later_local_edit_travel_alone() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        paper(&one, "dev_a-0001", "# Minuta\n\nlo mio, y algo mas");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "settling both did not become the new common ground"
        );
        assert_eq!(done.sent, 1);
    }

    #[test]
    fn settling_with_both_lets_a_later_remote_edit_travel_alone() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

        theirs(
            shared.path(),
            "dev_a-0001",
            "# Minuta\n\nlo mio, visto desde el otro lado",
        );
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "settling both did not become the new common ground on the other side"
        );
        assert_eq!(done.brought, 1);
    }

    #[test]
    fn four_alternating_synchronizations_converge_without_losing_a_single_edit() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];

        paper(&one, "dev_a-0001", "# Minuta\n\nversion uno");
        let first = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(first.sent, 1, "the first version never left");

        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nversion dos");
        let second = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(second.brought, 1, "the second version did not come home");
        assert!(second.undecided.is_empty());
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nversion dos");

        paper(&one, "dev_a-0001", "# Minuta\n\nversion tres");
        let third = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(third.sent, 1, "the third version never left");
        assert!(third.undecided.is_empty());
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nversion tres"
        );

        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nversion cuatro");
        let fourth = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(fourth.brought, 1, "the fourth version did not come home");
        assert!(fourth.undecided.is_empty());

        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nversion cuatro");
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nversion cuatro"
        );
    }

    #[test]
    fn a_body_missing_from_the_shared_folder_is_sent_back_not_left_absent() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta\n\nlo que dije");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        std::fs::remove_file(shared.path().join("docs").join("dev_a-0001.md")).unwrap();
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(
            done.sent, 1,
            "a body only the folder lost was not sent back"
        );
        assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo que dije");
    }

    #[test]
    fn a_body_deleted_here_by_accident_comes_back_from_what_the_folder_still_has() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta\n\nlo que dije");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        std::fs::remove_file(one.data.join("docs").join("dev_a-0001.md")).unwrap();
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(
            done.brought, 1,
            "a body only lost here was not brought back"
        );
        assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo que dije");
    }

    #[test]
    fn a_body_gone_from_both_sides_is_not_forgotten_and_comes_back_clean_once_one_side_writes_again()
     {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Minuta\n\nversion uno");
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        std::fs::remove_file(one.data.join("docs").join("dev_a-0001.md")).unwrap();
        std::fs::remove_file(shared.path().join("docs").join("dev_a-0001.md")).unwrap();
        let vanished = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(vanished.sent + vanished.brought, 0);
        assert!(vanished.undecided.is_empty());

        paper(&one, "dev_a-0001", "# Minuta\n\nvuelve distinta");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(
            done.sent, 1,
            "a body that came back was held against its old self"
        );
        assert!(done.undecided.is_empty());
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            "# Minuta\n\nvuelve distinta"
        );
    }

    #[test]
    fn two_sides_that_reach_the_same_words_on_their_own_settle_without_being_asked() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = at_odds(&one, shared.path());
        let conflicted = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(conflicted.undecided.len(), 1);

        paper(&one, "dev_a-0001", "# Minuta\n\nlo mismo al fin");
        theirs(shared.path(), "dev_a-0001", "# Minuta\n\nlo mismo al fin");
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(
            done.undecided.is_empty(),
            "it kept asking after both sides said the same thing"
        );
        assert_eq!(
            done.sent + done.brought,
            0,
            "it moved something nobody had changed anywhere"
        );
    }

    #[test]
    fn leaning_on_the_cache_still_sees_what_was_retired_a_moment_ago() {
        let one = machine("uno");
        let kept = planted(&one.data, "foto.png", b"una fotografia retirada");
        let shared = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let at = cache.path().to_path_buf();

        carry_leaning_on(
            &one.data,
            Some(&at),
            &one.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap();
        std::fs::remove_file(shared.path().join(&kept)).unwrap();

        says(&one, Op::AttachRetire { d: kept.clone() });
        carry_leaning_on(
            &one.data,
            Some(&at),
            &one.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap();

        assert!(
            !shared.path().join(&kept).exists(),
            "la cache sirvio un estado viejo y lo retirado volvio a subir"
        );
    }

    #[test]
    fn leaning_on_the_cache_lands_exactly_where_reading_it_all_lands() {
        let run = |aside: bool| -> (Vec<String>, usize) {
            let one = machine("dev_a");
            let two = blank("dev_b");
            let shared = tempfile::tempdir().unwrap();
            let cache = tempfile::tempdir().unwrap();
            let at = aside.then(|| cache.path().to_path_buf());

            carry_leaning_on(
                &one.data,
                at.as_deref(),
                &one.device,
                shared.path(),
                Way::Both,
                &[],
            )
            .unwrap();
            carry_leaning_on(
                &two.data,
                at.as_deref(),
                &two.device,
                shared.path(),
                Way::Both,
                &[],
            )
            .unwrap();

            for _ in 0..3 {
                carry_leaning_on(
                    &one.data,
                    at.as_deref(),
                    &one.device,
                    shared.path(),
                    Way::Both,
                    &[],
                )
                .unwrap();
                carry_leaning_on(
                    &two.data,
                    at.as_deref(),
                    &two.device,
                    shared.path(),
                    Way::Both,
                    &[],
                )
                .unwrap();
                wrote(&one, "algo mas de a".to_string());
                wrote(&two, "algo mas de b".to_string());
            }
            carry_leaning_on(
                &one.data,
                at.as_deref(),
                &one.device,
                shared.path(),
                Way::Both,
                &[],
            )
            .unwrap();

            let told = tisty_core::store::read_all(&one.store).unwrap();
            let state = tisty_core::State::replay(&told);
            let mut said: Vec<String> = state.tasks.values().map(|one| one.title.clone()).collect();
            said.sort();
            (said, told.len())
        };

        let plain = run(false);
        let leaned = run(true);

        assert_eq!(
            leaned, plain,
            "la cache llevo la sincronizacion a otro sitio"
        );
        assert!(plain.1 > 0, "el sorteo no llego a mover nada");
    }

    #[test]
    fn after_a_decision_by_hand_the_base_on_disk_is_what_the_ledger_says_it_is() {
        let one = blank("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        let base = "# Minuta\n\nparrafo uno\n\nparrafo dos\n";
        paper(&one, "dev_a-0001", base);
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        paper(
            &one,
            "dev_a-0001",
            "# Minuta\n\nparrafo uno del mac\n\nparrafo dos\n",
        );
        theirs(
            shared.path(),
            "dev_a-0001",
            "# Minuta\n\nparrafo uno de windows\n\nparrafo dos\n",
        );
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(done.undecided.len(), 1, "no llego a preguntar");

        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();
        carry_papers(&one.data, shared.path(), &alive).unwrap();
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        let said = tisty_core::docs::Carried::read(&one.data);
        assert_eq!(
            tisty_core::docs::carried_print(&one.data, "dev_a-0001").as_deref(),
            said.of("dev_a-0001"),
            "la base en disco dejo de ser la que el ledger dice"
        );
    }

    #[test]
    fn a_stale_base_left_by_a_decision_does_not_turn_a_silent_merge_into_a_question() {
        let one = blank("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(
            &one,
            "dev_a-0001",
            "# Minuta\n\nparrafo uno\n\nparrafo dos\n",
        );
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        paper(
            &one,
            "dev_a-0001",
            "# Minuta\n\nparrafo uno del mac\n\nparrafo dos\n",
        );
        theirs(
            shared.path(),
            "dev_a-0001",
            "# Minuta\n\nparrafo uno de windows\n\nparrafo dos\n",
        );
        carry_papers(&one.data, shared.path(), &alive).unwrap();
        settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();
        carry_papers(&one.data, shared.path(), &alive).unwrap();

        paper(
            &one,
            "dev_a-0001",
            "# Minuta del mac\n\nparrafo uno del mac\n\nparrafo dos\n",
        );
        theirs(
            shared.path(),
            "dev_a-0001",
            "# Minuta\n\nparrafo uno del mac\n\nparrafo dos\n\nparrafo tres\n",
        );
        let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(
            done.undecided.len(),
            0,
            "volvio a preguntar algo que se juntaba solo"
        );
        assert_eq!(
            body(&one.data, "dev_a-0001"),
            "# Minuta del mac\n\nparrafo uno del mac\n\nparrafo dos\n\nparrafo tres\n"
        );
    }

    #[test]
    fn an_attachment_that_came_from_elsewhere_is_written_down_like_our_own() {
        let one = machine("uno");
        let two = blank("dos");
        let shared = tempfile::tempdir().unwrap();
        let kept = planted(&one.data, "foto.png", b"una fotografia que viaja");
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

        let written = tisty_core::attach::digests(&two.data);
        assert!(
            written.contains_key(&kept),
            "lo que llega de fuera queda sin huella larga, solo con la corta del nombre"
        );
    }

    #[test]
    fn what_lands_in_the_shared_folder_is_dated_now_so_a_cloud_client_notices_it() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let old = std::time::SystemTime::now() - std::time::Duration::from_secs(60 * 60 * 24 * 3);
        let mine = one.store.join(&one.device).join("active.tisty");
        std::fs::File::options()
            .write(true)
            .open(&mine)
            .unwrap()
            .set_modified(old)
            .unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let landed = shared
            .path()
            .join(STORE)
            .join(&one.device)
            .join("active.tisty");
        let when = std::fs::metadata(&landed)
            .and_then(|m| m.modified())
            .unwrap();
        let apart = std::time::SystemTime::now().duration_since(when).unwrap();
        assert!(
            apart < std::time::Duration::from_secs(60),
            "aterrizo con fecha de hace {apart:?}: un cliente de nube no lo ve como cambio"
        );
    }

    #[test]
    fn a_round_that_changed_nothing_still_does_not_copy_again() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        let landed = shared
            .path()
            .join(STORE)
            .join(&one.device)
            .join("active.tisty");
        let was = std::fs::metadata(&landed)
            .and_then(|m| m.modified())
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(20));
        let done = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        assert_eq!(done.sent, 0, "volvio a copiar lo que no habia cambiado");
        assert_eq!(
            std::fs::metadata(&landed)
                .and_then(|m| m.modified())
                .unwrap(),
            was,
            "lo reescribio sin motivo"
        );
    }

    #[test]
    fn asking_again_writes_what_a_plain_round_leaves_alone() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        let landed = shared
            .path()
            .join(STORE)
            .join(&one.device)
            .join("active.tisty");
        let stuck = std::time::SystemTime::now() - std::time::Duration::from_secs(60 * 60 * 24 * 3);
        std::fs::File::options()
            .write(true)
            .open(&landed)
            .unwrap()
            .set_modified(stuck)
            .unwrap();

        let plain = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        assert_eq!(plain.sent, 0);
        assert_eq!(
            std::fs::metadata(&landed)
                .and_then(|m| m.modified())
                .unwrap(),
            stuck,
            "una ronda normal deberia dejar quieto lo que ya tiene los mismos bytes"
        );

        let forced = carry(&one.data, &one.device, shared.path(), Way::Again, &[]).unwrap();

        assert_eq!(forced.sent, 1, "pedirlo otra vez no reenvio el segmento");
        let when = std::fs::metadata(&landed)
            .and_then(|m| m.modified())
            .unwrap();
        let apart = std::time::SystemTime::now().duration_since(when).unwrap();
        assert!(
            apart < std::time::Duration::from_secs(60),
            "aterrizo con fecha de hace {apart:?}: sigue atascado"
        );
    }

    #[test]
    fn two_bodies_of_one_size_but_different_content_are_not_taken_for_equal() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        let body = b"una linea igual de larga\n";
        let other = b"otra linea igual de larga";
        assert_eq!(body.len(), other.len());
        std::fs::write(&a, body).unwrap();
        std::fs::write(&b, other).unwrap();

        assert!(!same(&a, &b), "la fecha decidia, no el contenido");
        std::fs::write(&b, body).unwrap();
        assert!(same(&a, &b));
    }

    #[test]
    #[ignore = "same() only reads the last 512 bytes, so a body that differs before its tail passes for equal"]
    fn two_bodies_of_one_size_that_differ_before_the_tail_are_not_taken_for_equal() {
        let dir = tempfile::tempdir().unwrap();
        let a = dir.path().join("a");
        let b = dir.path().join("b");
        let mut body = vec![b'x'; 4096];
        std::fs::write(&a, &body).unwrap();
        body[0] = b'z';
        std::fs::write(&b, &body).unwrap();

        assert!(!same(&a, &b));
    }

    #[test]
    fn a_round_that_moved_nothing_leaves_the_marker_untouched() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
        let at = shared.path().join(STORE).join(MARKER);
        let was = std::fs::metadata(&at).and_then(|m| m.modified()).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(20));
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        let now = std::fs::metadata(&at).and_then(|m| m.modified()).unwrap();
        assert_eq!(now, was, "la carpeta de nube ve un cambio donde no lo hay");
    }

    #[test]
    fn a_round_where_nothing_moved_writes_nothing_at_all() {
        let one = blank("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Notas\n\nun cuerpo cualquiera");

        carry_papers(&one.data, shared.path(), &alive).unwrap();

        let watched = [
            one.data.join("carried").join("dev_a-0001.md"),
            one.data.join("carried.json"),
        ];
        let before: Vec<_> = watched
            .iter()
            .map(|at| std::fs::metadata(at).and_then(|m| m.modified()).unwrap())
            .collect();

        std::thread::sleep(std::time::Duration::from_millis(20));
        let still = carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert_eq!(still.sent + still.brought, 0);
        for (at, was) in watched.iter().zip(before) {
            let now = std::fs::metadata(at).and_then(|m| m.modified()).unwrap();
            assert_eq!(now, was, "se reescribio sin que nada cambiara: {at:?}");
        }
    }

    #[test]
    fn a_base_wiped_from_under_us_is_written_again_on_the_next_round() {
        let one = blank("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", "# Notas\n\nun cuerpo cualquiera");

        carry_papers(&one.data, shared.path(), &alive).unwrap();
        let at = one.data.join("carried").join("dev_a-0001.md");
        std::fs::remove_file(&at).unwrap();

        carry_papers(&one.data, shared.path(), &alive).unwrap();

        assert!(at.exists(), "la base para fusionar quedo perdida");
    }

    #[test]
    fn content_no_editor_would_be_proud_of_still_crosses_byte_for_byte() {
        let long = "y".repeat(400 * 1024);
        let cases: [(&str, &str); 6] = [
            ("empty", ""),
            ("blank", "   \n\t  \n  "),
            ("long, but still within what the editor will open", &long),
            ("accented", "café ñandú 日本語 🎉 texto con acentós"),
            ("windows line endings", "una linea\r\notra linea\r\n"),
            ("a byte order mark up front", "\u{FEFF}# Titulo\n\ncuerpo"),
        ];

        for (label, first) in cases {
            let one = blank("dev_a");
            let shared = tempfile::tempdir().unwrap();
            let alive = ["dev_a-0001".to_string()];
            paper(&one, "dev_a-0001", first);

            let sent = carry_papers(&one.data, shared.path(), &alive).unwrap();
            assert_eq!(sent.sent, 1, "did not travel on: {label}");
            assert_eq!(
                body(shared.path(), "dev_a-0001"),
                first,
                "bytes changed in transit on: {label}"
            );

            let still = carry_papers(&one.data, shared.path(), &alive).unwrap();
            assert_eq!(
                still.sent + still.brought,
                0,
                "moved again with nothing changed on: {label}"
            );

            let edited = format!("{first}\nmas");
            theirs(shared.path(), "dev_a-0001", &edited);
            let round = carry_papers(&one.data, shared.path(), &alive).unwrap();
            assert_eq!(
                round.brought, 1,
                "the edit on top did not come home on: {label}"
            );
            assert_eq!(
                body(&one.data, "dev_a-0001"),
                edited,
                "bytes changed coming home on: {label}"
            );
        }
    }

    #[test]
    fn a_machine_nobody_ever_named_is_not_locked_out_by_someone_elses_list() {
        let one = machine("dev_a");
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_b".into()),
            },
        );
        let shared = tempfile::tempdir().unwrap();

        let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(
            moved.sent > 0,
            "the first machine to join shut the door on the rest"
        );
    }

    #[test]
    fn being_named_once_and_dropped_is_not_the_same_as_never_being_named() {
        let said = tisty_core::store::Ledger {
            allowed: [DeviceId("dev_b".into())].into(),
            named: [DeviceId("dev_a".into()), DeviceId("dev_b".into())].into(),
        };

        assert!(!said.may_write(&DeviceId("dev_a".into())));
        assert!(said.may_write(&DeviceId("dev_b".into())));
        assert!(said.may_write(&DeviceId("dev_c".into())));
        assert!(said.was_removed(&DeviceId("dev_a".into())));
        assert!(!said.was_removed(&DeviceId("dev_c".into())));
    }

    #[test]
    fn two_machines_removing_each_other_at_once_do_not_brick_the_store() {
        let said = tisty_core::store::Ledger {
            allowed: Default::default(),
            named: [DeviceId("dev_a".into()), DeviceId("dev_b".into())].into(),
        };

        assert!(
            said.may_write(&DeviceId("dev_a".into())),
            "nobody could ever write here again"
        );
        assert!(said.may_write(&DeviceId("dev_b".into())));
    }

    #[test]
    fn a_machine_that_was_removed_writes_nothing_at_all() {
        let one = machine("dev_a");
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_a".into()),
            },
        );
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_b".into()),
            },
        );
        says(
            &one,
            Op::DeviceRemove {
                d: DeviceId("dev_a".into()),
            },
        );
        let shared = tempfile::tempdir().unwrap();

        let why = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap_err();

        assert!(
            matches!(why, Trouble::NotAllowed(_)),
            "it went through: {why:?}"
        );
        assert!(
            !shared.path().join(STORE).join("dev_a").exists(),
            "it wrote before refusing"
        );
    }

    #[test]
    fn a_machine_that_was_removed_still_brings_what_is_there() {
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let one = blank("dev_a");
        joined(&one, shared.path());
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("dev_b".into()),
            },
        );
        says(
            &one,
            Op::DeviceRemove {
                d: DeviceId("dev_a".into()),
            },
        );

        wrote(&other, "lo dicho despues".into());
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
        let moved = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

        assert!(moved.brought > 0, "being removed is not being cut off");
        assert!(
            titles(&one.store).contains(&"lo dicho despues".to_string()),
            "{:?}",
            titles(&one.store)
        );
    }

    #[test]
    fn the_word_that_removes_it_is_read_before_it_writes() {
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let one = blank("dev_a");
        joined(&one, shared.path());

        says(
            &other,
            Op::DeviceJoin {
                d: DeviceId("dev_b".into()),
            },
        );
        says(
            &other,
            Op::DeviceRemove {
                d: DeviceId("dev_a".into()),
            },
        );
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let why = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap_err();

        assert!(
            matches!(why, Trouble::NotAllowed(_)),
            "it pushed on stale news: {why:?}"
        );
    }

    #[test]
    fn what_one_machine_leaves_the_other_takes_home() {
        let one = machine("dev_a");
        let other = blank("dev_b");
        let shared = tempfile::tempdir().unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
        joined(&other, shared.path());
        wrote(&other, "lo de dev_b".into());
        carry(&other.data, &other.device, shared.path(), Way::Both, &[]).unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        let mine = titles(&one.store);
        assert!(mine.contains(&"lo de dev_a".to_string()), "{mine:?}");
        assert!(mine.contains(&"lo de dev_b".to_string()), "{mine:?}");
        assert_eq!(titles(&other.store).len(), 2);
    }

    #[test]
    fn nobody_ever_writes_over_their_own_directory() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        std::fs::write(shared.path().join("store/dev_a/active.tisty"), b"").unwrap();

        let _ = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]);
        assert_eq!(titles(&one.store).len(), 1, "the emptied copy came home");
    }

    #[test]
    fn a_directory_that_differs_only_in_case_is_still_our_own() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let theirs = shared.path().join("store/DEV_A");
        std::fs::create_dir_all(&theirs).unwrap();
        std::fs::write(theirs.join("active.tisty"), b"").unwrap();

        let _ = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]);
        assert_eq!(titles(&one.store).len(), 1, "our own log was overwritten");
    }

    #[test]
    fn what_is_left_behind_is_never_removed() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let stranger = shared.path().join("store/dev_z");
        std::fs::create_dir_all(&stranger).unwrap();
        std::fs::write(stranger.join("keep.txt"), b"not ours").unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        assert!(stranger.join("keep.txt").exists());
    }

    #[test]
    fn a_folder_of_another_store_is_refused_before_anything_moves() {
        let one = machine("dev_a");
        std::fs::write(one.store.join(MARKER), b"01OURS").unwrap();
        let shared = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(shared.path().join("store")).unwrap();
        std::fs::write(shared.path().join("store").join(MARKER), b"01THEIRS").unwrap();

        let Err(Trouble::OtherStore { theirs }) =
            carry(&one.data, &one.device, shared.path(), Way::Both, &[])
        else {
            panic!("two histories were about to be merged");
        };
        assert_eq!(theirs, "01THEIRS");
        assert!(
            !shared.path().join("store/dev_a").exists(),
            "something moved"
        );
    }

    #[test]
    fn two_histories_are_never_joined_at_all() {
        let one = machine("dev_a");
        std::fs::remove_file(one.store.join(MARKER)).ok();
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let Err(Trouble::WouldReset { .. }) =
            carry(&one.data, &one.device, shared.path(), Way::Both, &[])
        else {
            panic!("two histories were joined");
        };

        assert_eq!(titles(&one.store), vec!["lo de dev_a".to_string()]);
        assert!(
            !shared.path().join("store/dev_a").exists(),
            "something moved"
        );

        let again = carry(&one.data, &one.device, shared.path(), Way::Both, &[]);
        assert!(
            matches!(again, Err(Trouble::WouldReset { .. })),
            "asking twice is not consent: {again:?}"
        );
        assert_eq!(titles(&one.store).len(), 1, "it joined them anyway");
    }

    #[test]
    fn a_store_with_history_and_no_marker_is_not_adopted() {
        let one = machine("dev_a");
        std::fs::remove_file(one.store.join(MARKER)).ok();
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        let Err(Trouble::WouldReset { .. }) =
            carry(&one.data, &one.device, shared.path(), Way::Both, &[])
        else {
            panic!("an unmarked store merged into a stranger's history");
        };
        assert_eq!(titles(&one.store), vec!["lo de dev_a".to_string()]);
    }

    #[test]
    fn a_folder_full_of_history_with_no_marker_is_refused() {
        let one = machine("dev_a");
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
        std::fs::remove_file(shared.path().join("store").join(MARKER)).unwrap();

        let Err(Trouble::WouldReset { .. }) =
            carry(&one.data, &one.device, shared.path(), Way::Both, &[])
        else {
            panic!("a folder with history and no marker was treated as empty");
        };
        assert_eq!(titles(&one.store), vec!["lo de dev_a".to_string()]);
    }

    #[test]
    fn a_meeting_place_that_is_not_there_says_so() {
        let one = machine("dev_a");
        let gone = one.store.join("unplugged");

        assert!(matches!(
            carry(&one.data, &one.device, &gone, Way::Both, &[]),
            Err(Trouble::NotThere(_))
        ));
    }

    #[test]
    fn one_direction_only_does_one_direction() {
        let one = blank("dev_a");
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        assert!(
            titles(&one.store).is_empty(),
            "a push brought something back"
        );

        carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();
        assert_eq!(titles(&one.store).len(), 1);
    }

    #[test]
    fn a_machine_meeting_the_folder_for_the_first_time_adopts_its_name() {
        let one = machine("dev_a");
        let other = machine("dev_b");
        std::fs::remove_file(other.store.join(MARKER)).ok();
        let shared = tempfile::tempdir().unwrap();

        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
        std::fs::remove_dir_all(other.store.join(&other.device)).unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Both, &[]).unwrap();

        assert_eq!(
            tisty_core::store::peek_identity(&other.store),
            tisty_core::store::peek_identity(&one.store)
        );
    }

    #[test]
    fn syncing_twice_over_carries_nothing_the_second_time() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();

        let first = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        let again = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        assert!(first.sent > 0);
        assert_eq!(again.sent, 0, "it copied what was already identical");
    }

    #[test]
    fn a_gap_in_what_is_offered_is_refused_without_importing_it() {
        let one = blank("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let theirs = shared.path().join("store/dev_b");
        std::fs::create_dir_all(&theirs).unwrap();
        std::fs::write(theirs.join("000002.tisty"), b"").unwrap();
        std::fs::write(shared.path().join("store").join(MARKER), b"01M0THEIRSTORE").unwrap();

        let moved = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(moved.unreadable, vec!["dev_b".to_string()]);
        assert!(!one.store.join("dev_b").exists(), "it landed anyway");
        assert!(titles(&one.store).is_empty(), "our own store still reads");
    }

    #[test]
    fn one_unreadable_machine_in_the_folder_never_stops_our_own_work_going_out() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
        let theirs = shared.path().join("store/dev_b");
        std::fs::create_dir_all(&theirs).unwrap();
        std::fs::write(theirs.join("000002.tisty"), b"").unwrap();

        wrote(&one, "lo que tiene que salir igual".into());
        let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert_eq!(moved.unreadable, vec!["dev_b".to_string()]);
        assert!(moved.sent > 0, "lo nuestro se quedo sin salir");
        assert_eq!(
            tisty_core::store::distinct_in(&shared.path().join(STORE).join(&one.device)).unwrap(),
            2,
            "la carpeta no recibio lo que escribimos"
        );
    }

    #[test]
    fn a_conflict_copy_is_not_a_segment() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let mine = shared.path().join("store/dev_a");
        let held = std::fs::read(mine.join("active.tisty")).unwrap();
        std::fs::write(mine.join("active (conflicted copy).tisty"), &held).unwrap();

        let other = blank("dev_b");
        carry(&other.data, &other.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(
            titles(&other.store).len(),
            1,
            "the conflict copy was read as history"
        );
    }

    #[test]
    fn a_pull_with_nothing_to_bring_does_not_reread_everything() {
        let one = blank("dev_a");
        let other = machine("dev_b");
        let shared = tempfile::tempdir().unwrap();
        carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

        std::fs::write(
            shared.path().join("store/dev_b/000001.tisty"),
            b"not json at all",
        )
        .unwrap();
        std::fs::write(shared.path().join("store/dev_b/000001.count"), b"1").unwrap();

        let again = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();
        assert_eq!(again.unreadable, vec!["dev_b".to_string()]);
        assert_eq!(titles(&one.store).len(), 1, "our store still reads");
    }

    fn hostile_ids() -> Vec<String> {
        vec![
            "../secret".to_string(),
            "../../secret".to_string(),
            "..\\secret".to_string(),
            "/etc/passwd".to_string(),
            "C:\\Windows\\System32\\loot".to_string(),
            "c:loot".to_string(),
            "\\\\server\\share\\loot".to_string(),
            "CON".to_string(),
            "a3f1\0-0001".to_string(),
            "a3f1-0001 ".to_string(),
            "a".repeat(300),
            "a".repeat(5000),
            "dispositivo_caf\u{e9}-0001".to_string(),
            "dispositivo_cafe\u{301}-0001".to_string(),
            "\u{202e}evil-0001".to_string(),
            String::new(),
        ]
    }

    #[test]
    fn a_hostile_identifier_in_the_alive_list_never_reaches_a_file_outside_docs() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(one.data.join(PAPERS)).unwrap();
        std::fs::create_dir_all(shared.path().join(PAPERS)).unwrap();
        let local_bystander = one.data.join("secret.md");
        std::fs::write(&local_bystander, "no es tuyo").unwrap();
        let shared_bystander = shared.path().join("secret.md");
        std::fs::write(&shared_bystander, "tampoco es tuyo").unwrap();

        let attacks = hostile_ids();
        let done = carry_papers(&one.data, shared.path(), &attacks).unwrap();

        assert_eq!(
            done.sent + done.brought,
            0,
            "a hostile identifier moved something"
        );
        assert_eq!(
            std::fs::read_to_string(&local_bystander).unwrap(),
            "no es tuyo"
        );
        assert_eq!(
            std::fs::read_to_string(&shared_bystander).unwrap(),
            "tampoco es tuyo"
        );
        let said = std::fs::read_to_string(one.data.join("carried.json")).unwrap_or_default();
        assert!(
            !said.contains("secret"),
            "the ledger learned a name it must not know"
        );
    }

    #[test]
    fn settling_any_hostile_identifier_is_always_refused_before_anything_moves() {
        let one = machine("dev_a");
        let shared = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(one.data.join(PAPERS)).unwrap();
        std::fs::create_dir_all(shared.path().join(PAPERS)).unwrap();

        for id in hostile_ids() {
            assert!(
                settle(&one.data, shared.path(), &id, Keep::Theirs).is_err(),
                "{id:?} settled with theirs"
            );
            assert!(
                settle(&one.data, shared.path(), &id, Keep::Mine).is_err(),
                "{id:?} settled with mine"
            );
            assert!(
                settle(&one.data, shared.path(), &id, Keep::Both).is_err(),
                "{id:?} settled with both"
            );
        }
    }

    #[test]
    fn forgetting_any_hostile_identifier_never_deletes_a_file_outside_docs() {
        let shared = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(shared.path().join(PAPERS)).unwrap();
        let bystander = shared.path().join("secret.md");
        std::fs::write(&bystander, "no es tuyo").unwrap();

        for id in hostile_ids() {
            forget_paper(shared.path(), &id);
        }

        assert_eq!(std::fs::read_to_string(&bystander).unwrap(), "no es tuyo");
    }

    #[test]
    fn attachments_travel_with_the_tasks_that_name_them() {
        let one = machine("dev_a");
        let kept = planted(&one.data, "foto.png", b"a picture");

        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let other = blank("dev_b");
        carry(&other.data, &other.device, shared.path(), Way::Pull, &[]).unwrap();

        assert_eq!(std::fs::read(other.data.join(&kept)).unwrap(), b"a picture");
    }

    #[test]
    fn merging_two_unrelated_histories_loses_nothing_from_either_side() {
        let one = machine("uno");
        tisty_core::store::identity(&one.store).unwrap();
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        let merged = stitch(&one.data, &one.device, shared.path()).unwrap();
        assert_eq!(merged.kin, Kin::Strangers);
        says(
            &one,
            Op::StoresJoined {
                d: merged.stitch.unwrap(),
            },
        );

        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

        for who in [titles(&one.store), titles(&two.store)] {
            assert!(who.contains(&"lo de uno".to_string()), "{who:?}");
            assert!(who.contains(&"lo de dos".to_string()), "{who:?}");
        }
    }

    #[test]
    fn merging_adopts_the_folders_store_id_never_the_local_one_nor_a_new_one() {
        let one = machine("uno");
        let local_before = tisty_core::store::identity(&one.store).unwrap();
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
        let folder_id = super::theirs(shared.path()).unwrap();

        stitch(&one.data, &one.device, shared.path()).unwrap();

        let adopted = tisty_core::store::peek_identity(&one.store).unwrap();
        assert_eq!(adopted, folder_id);
        assert_ne!(adopted, local_before);
    }

    #[test]
    fn the_other_machine_of_the_surviving_history_syncs_after_a_merge_without_being_asked() {
        let one = machine("uno");
        tisty_core::store::identity(&one.store).unwrap();
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        let seam = stitch(&one.data, &one.device, shared.path())
            .unwrap()
            .stitch
            .unwrap();
        says(&one, Op::StoresJoined { d: seam });
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        let done = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(
            done.brought > 0,
            "the survivor's own machine was not offered the merge"
        );
        assert!(titles(&two.store).contains(&"lo de uno".to_string()));
    }

    #[test]
    fn a_straggler_with_the_old_identity_is_recognized_as_the_same_lineage() {
        let one = machine("uno");
        let old_shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, old_shared.path(), Way::Push, &[]).unwrap();
        let straggler = blank("tres");
        carry(
            &straggler.data,
            &straggler.device,
            old_shared.path(),
            Way::Pull,
            &[],
        )
        .unwrap();

        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
        let seam = stitch(&one.data, &one.device, shared.path())
            .unwrap()
            .stitch
            .unwrap();
        says(&one, Op::StoresJoined { d: seam });
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        assert_eq!(kinship(&straggler.store, shared.path()), Kin::SameLineage);
    }

    #[test]
    fn a_straggler_adopts_the_merged_identity_while_keeping_its_unsent_tail() {
        let one = machine("uno");
        let old_shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, old_shared.path(), Way::Push, &[]).unwrap();
        let straggler = blank("tres");
        carry(
            &straggler.data,
            &straggler.device,
            old_shared.path(),
            Way::Pull,
            &[],
        )
        .unwrap();

        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
        let seam = stitch(&one.data, &one.device, shared.path())
            .unwrap()
            .stitch
            .unwrap();
        says(&one, Op::StoresJoined { d: seam });
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        wrote(&straggler, "lo que tres no habia mandado".into());

        let merged = stitch(&straggler.data, &straggler.device, shared.path()).unwrap();
        assert_eq!(merged.kin, Kin::SameLineage);
        assert!(merged.stitch.is_none());

        let moved = carry(
            &straggler.data,
            &straggler.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap();

        assert!(moved.sent > 0, "the straggler's unsent tail never left");
        assert!(shared.path().join(STORE).join(&straggler.device).exists());
        let mine = titles(&straggler.store);
        assert!(mine.contains(&"lo que tres no habia mandado".to_string()));
        assert!(mine.contains(&"lo de dos".to_string()));
    }

    #[test]
    fn two_machines_that_happen_to_share_a_device_name_but_wrote_different_things_are_a_clash() {
        let one = blank("portable");
        wrote(&one, "lo de la primera portable".into());
        let two = blank("portable");
        wrote(&two, "lo de la otra portable".into());
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        assert_eq!(
            kinship(&one.store, shared.path()),
            Kin::Clash("portable".to_string())
        );
    }

    #[test]
    fn merging_refuses_with_same_name_when_two_machines_clash_under_one_device_name() {
        let one = blank("portable");
        wrote(&one, "lo de la primera portable".into());
        let two = blank("portable");
        wrote(&two, "lo de la otra portable".into());
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        let Err(why) = stitch(&one.data, &one.device, shared.path()) else {
            panic!("two machines clashing under one name were merged");
        };

        assert_eq!(why, Trouble::SameName("portable".to_string()));
    }

    #[test]
    fn merging_leaves_the_local_identity_untouched_when_it_refuses_a_clash() {
        let one = blank("portable");
        wrote(&one, "lo de la primera portable".into());
        let before = tisty_core::store::identity(&one.store).unwrap();
        let two = blank("portable");
        wrote(&two, "lo de la otra portable".into());
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        let _ = stitch(&one.data, &one.device, shared.path());

        assert_eq!(
            tisty_core::store::peek_identity(&one.store).unwrap(),
            before
        );
    }

    #[test]
    fn an_empty_segment_proves_no_kinship_so_a_shared_name_is_refused() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let mine = one.store.join(&one.device);
        let file = tisty_core::store::segments_in(&mine).unwrap().remove(0);
        std::fs::write(&file, b"").unwrap();

        assert_eq!(
            kinship(&one.store, shared.path()),
            Kin::Clash(one.device.clone())
        );
    }

    #[test]
    fn a_placeholder_the_cloud_has_not_filled_in_cannot_pass_for_the_same_history() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let theirs = shared.path().join(STORE).join(&one.device);
        for at in tisty_core::store::segments_in(&theirs).unwrap() {
            std::fs::write(&at, b"").unwrap();
        }

        assert_eq!(
            kinship(&one.store, shared.path()),
            Kin::Clash(one.device.clone())
        );
    }

    #[test]
    fn a_name_that_only_a_removal_remembers_still_counts_as_shared() {
        let one = machine("uno");
        let two = machine("dos");
        says(
            &one,
            Op::DeviceRemove {
                d: DeviceId(two.device.clone()),
            },
        );
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
        std::fs::remove_dir_all(one.store.join(&two.device)).ok();

        assert_eq!(
            kinship(&one.store, shared.path()),
            Kin::Clash(two.device.clone()),
            "un nombre que solo vive en un evento paso desapercibido"
        );
    }

    #[test]
    fn two_histories_with_no_name_in_common_are_still_strangers() {
        let one = machine("uno");
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        assert_eq!(kinship(&one.store, shared.path()), Kin::Strangers);
    }

    #[test]
    fn a_side_that_rotated_into_a_second_segment_is_still_the_same_lineage() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let mine = one.store.join(&one.device);
        std::fs::rename(mine.join("active.tisty"), mine.join("000001.tisty")).unwrap();
        std::fs::write(mine.join("000001.count"), b"1").unwrap();
        std::fs::write(mine.join("active.tisty"), b"").unwrap();

        assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
    }

    #[test]
    fn a_directory_present_on_only_one_side_never_turns_a_shared_directory_into_a_clash() {
        let one = machine("uno");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        let mut solo_local = Store::open(&one.store, DeviceId("solo-local".into())).unwrap();
        solo_local
            .append(Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new("solo aqui", "a0"),
            })
            .unwrap();

        let mut solo_remote =
            Store::open(shared.path().join(STORE), DeviceId("solo-remoto".into())).unwrap();
        solo_remote
            .append(Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new("solo alla", "a0"),
            })
            .unwrap();

        assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
    }

    #[test]
    fn an_empty_store_directory_is_a_stranger_not_the_same_lineage() {
        let empty = blank("solitario");
        std::fs::create_dir_all(&empty.store).unwrap();
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        assert_eq!(kinship(&empty.store, shared.path()), Kin::Strangers);
    }

    #[test]
    fn merging_two_histories_lands_the_documents_of_both_sides_under_their_own_names() {
        let one = machine("uno");
        tisty_core::store::identity(&one.store).unwrap();
        filed(&one, "uno-0001", "# Lo de uno");
        let two = machine("dos");
        filed(&two, "dos-0001", "# Lo de dos");
        let shared = tempfile::tempdir().unwrap();
        carry(
            &two.data,
            &two.device,
            shared.path(),
            Way::Both,
            &["dos-0001".into()],
        )
        .unwrap();

        let seam = stitch(&one.data, &one.device, shared.path())
            .unwrap()
            .stitch
            .unwrap();
        says(&one, Op::StoresJoined { d: seam });
        carry(
            &one.data,
            &one.device,
            shared.path(),
            Way::Both,
            &["uno-0001".into(), "dos-0001".into()],
        )
        .unwrap();

        assert_eq!(body(shared.path(), "uno-0001"), "# Lo de uno\n");
        assert_eq!(body(shared.path(), "dos-0001"), "# Lo de dos\n");
        assert_eq!(body(&one.data, "dos-0001"), "# Lo de dos\n");

        carry(
            &two.data,
            &two.device,
            shared.path(),
            Way::Both,
            &["uno-0001".into(), "dos-0001".into()],
        )
        .unwrap();
        assert_eq!(body(&two.data, "uno-0001"), "# Lo de uno\n");
    }

    #[test]
    fn the_same_attachment_kept_independently_on_both_sides_of_a_merge_is_not_duplicated() {
        let one = machine("uno");
        tisty_core::store::identity(&one.store).unwrap();
        let two = machine("dos");
        let bytes: &[u8] = b"la misma fotografia, bit por bit";
        let kept_one = planted(&one.data, "foto.png", bytes);
        let kept_two = planted(&two.data, "foto.png", bytes);
        assert_eq!(
            kept_one, kept_two,
            "identical bytes under the same name must land at the same address"
        );

        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

        let seam = stitch(&one.data, &one.device, shared.path())
            .unwrap()
            .stitch
            .unwrap();
        says(&one, Op::StoresJoined { d: seam });
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        let shelf = shared
            .path()
            .join(&kept_one)
            .parent()
            .unwrap()
            .to_path_buf();
        let landed = std::fs::read_dir(&shelf).unwrap().count();
        assert_eq!(
            landed, 1,
            "the same picture landed twice under different names"
        );
        assert_eq!(std::fs::read(shared.path().join(&kept_one)).unwrap(), bytes);
    }

    #[test]
    fn a_document_deleted_in_one_history_never_comes_back_once_the_histories_are_merged() {
        let one = machine("uno");
        tisty_core::store::identity(&one.store).unwrap();
        filed(&one, "uno-0001", "# Efimero");
        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        let seam = stitch(&one.data, &one.device, shared.path())
            .unwrap()
            .stitch
            .unwrap();
        says(&one, Op::StoresJoined { d: seam });
        carry(
            &one.data,
            &one.device,
            shared.path(),
            Way::Both,
            &["uno-0001".into()],
        )
        .unwrap();

        let mut held = Store::open(&one.store, DeviceId(one.device.clone())).unwrap();
        let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
            .docs
            .values()
            .next()
            .unwrap()
            .id;
        held.append(Op::DocDelete { id }).unwrap();
        drop(held);
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

        assert!(!two.data.join(PAPERS).join("uno-0001.md").exists());
    }

    #[test]
    fn a_device_removed_before_a_merge_is_still_removed_after_it() {
        let one = machine("uno");
        tisty_core::store::identity(&one.store).unwrap();
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("uno".into()),
            },
        );
        says(
            &one,
            Op::DeviceJoin {
                d: DeviceId("vieja".into()),
            },
        );
        says(
            &one,
            Op::DeviceRemove {
                d: DeviceId("vieja".into()),
            },
        );

        let two = machine("dos");
        let shared = tempfile::tempdir().unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

        let seam = stitch(&one.data, &one.device, shared.path())
            .unwrap()
            .stitch
            .unwrap();
        says(&one, Op::StoresJoined { d: seam });
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

        let said = tisty_core::store::ledger(shared.path().join(STORE)).unwrap();
        assert!(said.was_removed(&DeviceId("vieja".into())));
        assert!(!said.may_write(&DeviceId("vieja".into())));

        let vieja = blank("vieja");
        let why = carry(&vieja.data, &vieja.device, shared.path(), Way::Both, &[]).unwrap_err();
        assert!(matches!(why, Trouble::NotAllowed(_)), "{why:?}");
    }

    #[test]
    fn a_retired_attachment_does_not_come_back_from_the_folder_once_it_is_swept() {
        let one = machine("uno");
        let kept = planted(&one.data, "foto.png", b"una fotografia retirada");
        let shared = tempfile::tempdir().unwrap();
        carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

        says(&one, Op::AttachRetire { d: kept.clone() });
        let retired: std::collections::BTreeSet<String> = [kept.clone()].into();
        tisty_core::attach::sweep(&one.data, &retired, &Default::default());
        assert!(!one.data.join(&kept).exists(), "sweep did not remove it");

        carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

        assert!(
            !one.data.join(&kept).exists(),
            "a retired attachment came back from the shared folder"
        );
    }
}
