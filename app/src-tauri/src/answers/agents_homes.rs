use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tisty_core::agents::{self, Card, GLOBAL, Gate, Kind, Ledger, Seen};
use tisty_core::event::{DeviceId, DocAdd, Filed, Said};
use tisty_core::model::{FolderId, Kept};
use tisty_core::{Op, State};

use super::agents_sync::{docs_in, gated, refused, rewritten, root_of, tree};
use crate::wiring::agents::{self as places, GLOBAL_FOLDER, Homes};
use crate::{Answer, Refusal, Session, signing};

pub(crate) const SINCE: &str = "1.51.0";

fn cannot(e: impl std::fmt::Display) -> Refusal {
    Refusal::about("cannotWrite", e.to_string())
}

pub(crate) struct Spot<'a> {
    pub(crate) computer: Option<DeviceId>,
    pub(crate) name: &'a str,
}

pub(crate) fn spot(state: &State, folder: Option<FolderId>) -> Option<Spot<'_>> {
    let root = root_of(state)?;
    let at = state.folders.get(&folder?)?;
    if at.parent == Some(root) {
        return at.computer.is_none().then_some(Spot {
            computer: None,
            name: &at.name,
        });
    }
    let up = state.folders.get(&at.parent?)?;
    let computer = up.computer.clone().filter(|_| up.parent == Some(root))?;
    Some(Spot {
        computer: Some(computer),
        name: &at.name,
    })
}

pub(crate) struct Rooms {
    pub(crate) own: FolderId,
    pub(crate) ops: Vec<Op>,
}

impl Rooms {
    pub(crate) fn of(state: &State, root: FolderId, me: &DeviceId) -> Self {
        let (own, made) = agents::computer_folder(state, root, me);
        Self {
            own,
            ops: made.into_iter().collect(),
        }
    }

    pub(crate) fn room(&mut self, state: &State, parent: FolderId, name: &str) -> FolderId {
        agents::room(state, &mut self.ops, parent, name)
    }

    pub(crate) fn of_assistant(&mut self, state: &State, level: &str) -> FolderId {
        let name = places::assistant(level).map_or(GLOBAL_FOLDER, |one| one.name);
        self.room(state, self.own, name)
    }
}

pub(crate) fn subfolders_made(session: &mut Session, homes: &Homes, ledger: &Ledger) -> Answer<()> {
    let Some(root) = root_of(&session.state).filter(|_| ledger.consented()) else {
        return Ok(());
    };
    let state = &session.state;
    let mut ops = agents::sub_folders(state, root, &[GLOBAL_FOLDER]);
    let detected = homes.detected();
    if !detected.is_empty() {
        let rooms = Rooms::of(state, root, session.store.device());
        let names: Vec<&str> = detected.iter().map(|one| one.name).collect();
        ops.extend(rooms.ops);
        ops.extend(agents::sub_folders(state, rooms.own, &names));
    }
    committed(session, ops)
}

fn committed(session: &mut Session, ops: Vec<Op>) -> Answer<()> {
    if ops.is_empty() {
        return Ok(());
    }
    session.commit_all(ops).map_err(cannot)
}

// An approval an earlier Tisty said of a document for one computer stands for words it may have rewritten.
fn approved(kept: &Kept, body: &str) -> bool {
    let known = kept.agreed_knowing || Card::of(body).is_none_or(|card| card.computer.is_none());
    known && kept.agreed.as_deref() == Some(tisty_core::docs::print_of_text(body).as_str())
}

pub(crate) fn of_another(session: &Session, file: &str) -> bool {
    kept_of(session, file)
        .filter(|kept| agents::holds(&session.state, kept))
        .and_then(|kept| agents::computer_of(&session.state, kept.folder))
        .is_some_and(|one| one != *session.store.device())
}

pub(crate) fn let_in(gate: &Gate, ledger: &mut Ledger) -> bool {
    gate.admitted()
        .iter()
        .filter(|one| one.card().computer.is_some())
        .fold(false, |changed, one| {
            let print = tisty_core::docs::print_of_text(one.body());
            ledger.let_in(one.file(), &print) | changed
        })
}

fn kept_of<'a>(session: &'a Session, file: &str) -> Option<&'a Kept> {
    session.state.docs.values().find(|one| one.file == file)
}

fn read(session: &Session, file: &str) -> Answer<String> {
    tisty_core::docs::read(&session.paths.docs(), file).map_err(cannot)
}

// Changing whom it is for approves nothing: the words stay approved only if they were.
fn meant(session: &mut Session, file: &str, computer: Option<&DeviceId>) -> Answer<bool> {
    let body = read(session, file)?;
    let Some(fresh) = agents::meant_for(&body, computer) else {
        return Ok(false);
    };
    if kept_of(session, file).is_some_and(|kept| approved(kept, &body)) {
        rewritten(session, file, &fresh)?;
        return Ok(true);
    }
    let writer = tisty_core::history::Writer::now(session.paths.data(), None);
    let print = tisty_core::docs::print_of_text(&body);
    let docs = session.paths.docs();
    match tisty_core::docs::rewrite(&docs, &writer, file, &fresh, &print).map_err(cannot)? {
        tisty_core::docs::Rewrite::Made { .. } => {
            session.retell(file, &fresh, None);
            Ok(false)
        }
        tisty_core::docs::Rewrite::Moved => Err(Refusal::about("documentMoved", file.to_string())),
    }
}

// A proposal names the text as it was, so moving it to another computer would leave it with nothing to approve.
pub(crate) fn resolved(
    state: &State,
    id: tisty_core::model::DocId,
    folder: Option<FolderId>,
) -> Answer<()> {
    let goes = state.docs.get(&id).is_some_and(|kept| {
        agents::holds(state, kept)
            && agents::computer_of(state, kept.folder) != agents::computer_of(state, folder)
    });
    match goes && state.proposals.contains_key(&id) {
        true => Err(Refusal::of("agentProposalFirst")),
        false => Ok(()),
    }
}

pub(crate) fn moved_here(session: &mut Session, files: &[String]) -> Answer<()> {
    let here = super::agents_sync::here();
    moved(session, Homes::here().as_ref(), &here, files)
}

pub(crate) fn moved(
    session: &mut Session,
    homes: Option<&Homes>,
    here: &str,
    files: &[String],
) -> Answer<()> {
    let mut applied = Vec::new();
    let mut gone = Vec::new();
    for file in files {
        let Some(computer) = went_to(session, file) else {
            continue;
        };
        let mine = computer
            .as_ref()
            .is_none_or(|one| one == session.store.device());
        let agreed = meant(session, file, computer.as_ref())?;
        match (mine, agreed) {
            (true, true) => applied.push(file.clone()),
            (true, false) => {}
            (false, _) => gone.push(file.clone()),
        }
    }
    left_as_moved(session, here, &gone)?;
    match homes {
        Some(homes) if !applied.is_empty() => taken_as_moved(session, homes, here, &applied),
        _ => Ok(()),
    }
}

// Sent away by hand on this computer, its file here is asked about rather than kept as this computer's own.
fn left_as_moved(session: &Session, here: &str, files: &[String]) -> Answer<()> {
    let data = session.paths.data();
    let Some(mut ledger) = Ledger::mine(data, here).filter(|_| !files.is_empty()) else {
        return Ok(());
    };
    for file in files {
        ledger.take(file, false);
    }
    ledger.keep(data).map_err(refused)
}

// What an agent remembers of one computer is never another's.
fn went_to(session: &Session, file: &str) -> Option<Option<DeviceId>> {
    let kept = kept_of(session, file).filter(|kept| agents::holds(&session.state, kept))?;
    let computer = agents::computer_of(&session.state, kept.folder);
    let card = Card::of(&read(session, file).ok()?)?;
    let foreign = card.kind == Kind::Memory && computer.is_some() && computer != kept.made_by;
    (card.computer != computer && !foreign).then_some(computer)
}

// Moving it where this computer reads is the person's «apply here», unless another would write the same file.
fn taken_as_moved(session: &Session, homes: &Homes, here: &str, files: &[String]) -> Answer<()> {
    let data = session.paths.data();
    let Some(mut ledger) = Ledger::mine(data, here).filter(Ledger::consented) else {
        return Ok(());
    };
    let gate = gated(session, &ledger);
    for file in files {
        match places::sides(homes, &gate, file).rivals.is_empty() {
            true => chosen(session, &mut ledger, &gate, homes, file, true),
            false => {
                ledger.wait_all([file.as_str()]);
            }
        }
    }
    ledger.keep(data).map_err(refused)
}

pub(crate) fn under(state: &State, folder: FolderId) -> Vec<String> {
    docs_in(state, &agents::tree(state, folder))
        .into_iter()
        .map(|one| one.file.clone())
        .collect()
}

// An earlier Tisty elsewhere would read the move as documents leaving and touch that computer's files.
fn all_read_computers(session: &Session) -> bool {
    let me = session.store.device();
    session.state.not_yet_on(SINCE).iter().all(|one| one == me)
}

/// What an earlier Tisty left goes where it belongs now; whatever cannot go yet stays, and the round goes on.
pub(crate) fn settled_in(session: &mut Session, homes: &Homes, ledger: &mut Ledger) {
    let went = homed(session, ledger).and_then(|()| files_kept(session, homes, ledger));
    if let Err(why) = went {
        untold(
            "what an earlier Tisty left in Agents could not be settled",
            &why,
        );
    }
}

fn untold(said: &'static str, why: &Refusal) {
    tisty_core::witness::warn(
        tisty_core::witness::channel::AGENT,
        said,
        &[("why", tisty_core::witness::Fact::Why(format!("{why:?}")))],
    );
}

pub(crate) fn homed(session: &mut Session, ledger: &mut Ledger) -> Answer<()> {
    let Some(root) = root_of(&session.state).filter(|_| ledger.consented() && !ledger.homed())
    else {
        return Ok(());
    };
    let data = session.paths.data().to_path_buf();
    let me = session.store.device().clone();
    if ledger.homing().is_none() {
        ledger.home_all(left_at_root(session, root, &me));
        ledger.keep(&data).map_err(refused)?;
    }
    if !all_read_computers(session) {
        return Ok(());
    }
    for file in ledger.homing().cloned().unwrap_or_default() {
        match went_home(session, root, &me, &file) {
            Ok(true) => ledger.went_home(&file),
            Ok(false) => {}
            Err(why) => untold("a document could not go to its computer's folder", &why),
        }
    }
    ledger.keep(&data).map_err(refused)
}

// Words approved by a Tisty that knows computers sit at the root because the person wanted them there.
fn left_at_root(session: &Session, root: FolderId, me: &DeviceId) -> BTreeSet<String> {
    let state = &session.state;
    docs_in(state, &tree(state, root))
        .into_iter()
        .filter(|kept| kept.page_of.is_none() && kept.made_by.as_ref() == Some(me))
        .filter(|kept| agents::computer_of(state, kept.folder).is_none() && !kept.agreed_knowing)
        .filter(|kept| {
            read(session, &kept.file).is_ok_and(|body| {
                approved(kept, &body) && Card::of(&body).is_some_and(|card| card.level != GLOBAL)
            })
        })
        .map(|kept| kept.file.clone())
        .collect()
}

// Each goes in a step of its own, and one left halfway is picked up by its line the next time.
fn went_home(session: &mut Session, root: FolderId, me: &DeviceId, file: &str) -> Answer<bool> {
    let Some(kept) = kept_of(session, file).filter(|kept| agents::holds(&session.state, kept))
    else {
        return Ok(true);
    };
    if agents::computer_of(&session.state, kept.folder).is_some() {
        return Ok(true);
    }
    if session.state.proposals.contains_key(&kept.id) {
        return Ok(false);
    }
    let (id, folder) = (kept.id, kept.folder);
    let body = read(session, file)?;
    let halfway = Card::of(&body).is_some_and(|card| card.computer.as_ref() == Some(me));
    let Some(card) = Card::of(&body).filter(|_| halfway || approved(kept, &body)) else {
        return Ok(true);
    };
    if !halfway {
        meant(session, file, Some(me))?;
    }
    let mut rooms = Rooms::of(&session.state, root, me);
    let mut into = rooms.of_assistant(&session.state, &card.level);
    if let Some(name) = own_folder_name(&session.state, root, folder) {
        into = rooms.room(&session.state, into, &name);
    }
    let mut ops = rooms.ops;
    ops.push(Op::DocMove {
        id,
        d: Filed {
            folder: Some(Some(into)),
            page_of: None,
            order: None,
        },
    });
    committed(session, ops).map(|()| true)
}

fn own_folder_name(state: &State, root: FolderId, folder: Option<FolderId>) -> Option<String> {
    let mut at = state.folders.get(&folder?)?;
    loop {
        let up = state.folders.get(&at.parent?)?;
        if up.parent == Some(root) {
            return Some(at.name.clone());
        }
        at = up;
    }
}

pub(crate) fn files_kept(session: &mut Session, homes: &Homes, ledger: &mut Ledger) -> Answer<()> {
    if !ledger.consented() {
        return Ok(());
    }
    let Some(root) = root_of(&session.state) else {
        return Ok(());
    };
    let gate = gated(session, ledger);
    for one in orphans(session, homes, ledger, &gate) {
        file_kept(session, homes, ledger, root, one)?;
    }
    Ok(())
}

struct Orphan {
    at: PathBuf,
    bytes: Vec<u8>,
    level: &'static str,
    kind: Kind,
    name: String,
    mixed: bool,
}

fn orphans(session: &Session, homes: &Homes, ledger: &Ledger, gate: &Gate) -> Vec<Orphan> {
    ledger
        .written_files()
        .filter(|(at, _)| !agents::shipped_copy(ledger, at) && !ledger.stays(at))
        .filter(|(_, files)| stands_no_more(session, ledger, gate, files))
        .filter(|(at, _)| !places::agents_own(homes, at))
        .filter_map(|(at, files)| {
            let (level, kind, name) = places::card_at(homes, at)?;
            let bytes = as_written(ledger, at).filter(|_| ledger.syncs(level))?;
            Some(Orphan {
                at: at.to_path_buf(),
                bytes,
                level,
                kind,
                name,
                mixed: files.len() > 1,
            })
        })
        .collect()
}

// Joined files wait until what they are made of went home; another computer's document is gone at once.
fn stands_no_more(
    session: &Session,
    ledger: &Ledger,
    gate: &Gate,
    files: &BTreeSet<String>,
) -> bool {
    let kept = |file: &&String| !ledger.left(file);
    if files.len() > 1 {
        return ledger.homed();
    }
    files.iter().filter(kept).any(|file| {
        gate.seen(file) == Some(Seen::OtherComputer)
            || (ledger.homed() && global_rule(session, file))
    })
}

fn global_rule(session: &Session, file: &str) -> bool {
    read(session, file)
        .ok()
        .and_then(|body| Card::of(&body))
        .is_some_and(|card| card.level == GLOBAL && card.kind == Kind::Rule)
}

fn as_written(ledger: &Ledger, at: &Path) -> Option<Vec<u8>> {
    let meta = std::fs::symlink_metadata(at).ok()?;
    if !meta.is_file() || meta.len() > tisty_core::docs::BODY_AT_MOST {
        return None;
    }
    let bytes = std::fs::read(at).ok()?;
    (ledger.written(at) == Some(tisty_core::attach::printed(&bytes).as_str())).then_some(bytes)
}

fn holds_whole(bytes: &[u8], part: &[u8]) -> bool {
    let part = part.trim_ascii_end();
    part.is_empty() || bytes.windows(part.len()).any(|one| one == part)
}

fn file_kept(
    session: &mut Session,
    homes: &Homes,
    ledger: &mut Ledger,
    root: FolderId,
    one: Orphan,
) -> Answer<()> {
    let me = session.store.device().clone();
    let same = match own_of(session, ledger, &me, &one) {
        Some(file) if same_words(session, &file, &one)? => Some(file),
        // Joined words newer than the file are the person's to settle, so it stays as it is.
        Some(_) if one.mixed => return Ok(()),
        _ => None,
    };
    let Some(file) = same.map_or_else(|| made_own(session, root, &me, &one), |it| Ok(Some(it)))?
    else {
        return Ok(());
    };
    let gate = gated(session, ledger);
    chosen(session, ledger, &gate, homes, &file, true);
    ledger.applied_to(&one.at, [file]);
    ledger.keep(session.paths.data()).map_err(refused)
}

fn own_of(session: &Session, ledger: &Ledger, me: &DeviceId, one: &Orphan) -> Option<String> {
    let address = format!("{}/{}/{}", one.level, one.kind.word(), one.name);
    gated(session, ledger)
        .admitted()
        .iter()
        .find(|it| it.card().computer.as_ref() == Some(me) && it.card().address() == address)
        .map(|it| it.file().to_string())
}

fn same_words(session: &mut Session, file: &str, one: &Orphan) -> Answer<bool> {
    let body = read(session, file)?;
    if !agents::rendered(&body).is_ok_and(|said| holds_whole(&one.bytes, &said)) {
        return Ok(false);
    }
    let Ok(fresh) = agents::brought(&body, &one.bytes) else {
        return Ok(false);
    };
    if !tisty_core::docs::unchanged(&body, &fresh) {
        rewritten(session, file, &fresh)?;
    }
    Ok(true)
}

fn made_own(
    session: &mut Session,
    root: FolderId,
    me: &DeviceId,
    one: &Orphan,
) -> Answer<Option<String>> {
    let Some(body) = agents::doc_of(one.level, one.kind, &one.name, &one.bytes)
        .ok()
        .and_then(|body| agents::meant_for(&body, Some(me)))
    else {
        return Ok(None);
    };
    let mut rooms = Rooms::of(&session.state, root, me);
    let room = rooms.of_assistant(&session.state, one.level);
    let docs = session.paths.docs();
    let made = tisty_core::docs::create(&docs, me, &body).map_err(cannot)?;
    let taken: Vec<String> = session
        .state
        .docs
        .values()
        .filter(|kept| kept.folder == Some(room) && kept.page_of.is_none())
        .map(|kept| kept.order.clone())
        .collect();
    let mut ops = rooms.ops;
    ops.push(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: DocAdd {
            file: made.id.clone(),
            folder: Some(room),
            order: tisty_core::order::last_of(taken.iter().map(String::as_str)),
            said: Some(Said::of(&body).agreed()),
            by: signing(&session.state),
            ..Default::default()
        },
    });
    match committed(session, ops) {
        Ok(()) => Ok(Some(made.id)),
        Err(e) => {
            let _ = tisty_core::docs::remove(&docs, &made.id);
            Err(e)
        }
    }
}

fn agreed_print(session: &Session, file: &str) -> Option<String> {
    kept_of(session, file)?.agreed.clone()
}

fn took(session: &Session, ledger: &mut Ledger, file: &str) {
    ledger.take(file, true);
    if let Some(print) = agreed_print(session, file) {
        ledger.let_in(file, &print);
    }
}

// Choosing one side covers the places both would write and nothing else, whole skills and all.
pub(crate) fn chosen(
    session: &Session,
    ledger: &mut Ledger,
    gate: &Gate,
    homes: &Homes,
    file: &str,
    here: bool,
) {
    let sides = places::sides(homes, gate, file);
    if here {
        took(session, ledger, file);
        let along = sides.mine.iter().filter(|_| !sides.rivals.is_empty());
        for one in along {
            if !ledger.left(one) {
                took(session, ledger, one);
            }
        }
        for (place, side) in &sides.places {
            ledger.choose(place, side);
        }
        return;
    }
    let standing: Vec<&String> = sides
        .rivals
        .iter()
        .filter(|(_, files)| files.iter().any(|it| gate.seen(it) == Some(Seen::Admitted)))
        .map(|(side, _)| side)
        .collect();
    match (gate.seen(file), standing.as_slice()) {
        (Some(Seen::Admitted), [rival]) => {
            for (place, _) in &sides.places {
                ledger.choose(place, rival);
            }
        }
        _ => ledger.take(file, false),
    }
}

#[cfg(test)]
#[path = "agents_homes_test.rs"]
mod tests;
