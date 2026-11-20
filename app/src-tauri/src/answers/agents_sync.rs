use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::agents::{self, Card, Found, GLOBAL, Gate, Kind, Ledger, Seen, Standing, Target};
use tisty_core::model::{FolderId, Kept};
use tisty_core::{Op, State};

use crate::wiring::agents::{self as places, ASSISTANTS, GLOBAL_FOLDER, Homes, Placed};
use crate::{Answer, Refusal, Session, held, signing};

// Past 2,000 lines a side the line matcher gives up, and every line would read as changed.
const LINES_AT_MOST: usize = 2_000;

fn here() -> String {
    tisty_core::machine::here().unwrap_or_default()
}

fn homes() -> Answer<Homes> {
    Homes::here().ok_or_else(|| Refusal::about("cannotWrite", "no home folder"))
}

pub fn root_of(state: &State) -> Option<FolderId> {
    agents::root_of(state)
}

pub(crate) fn tree(state: &State, root: FolderId) -> BTreeSet<FolderId> {
    agents::tree(state, root)
}

pub(crate) fn docs_in<'a>(state: &'a State, tree: &BTreeSet<FolderId>) -> Vec<&'a Kept> {
    state
        .docs
        .values()
        .filter(|one| one.task_of.is_none() && one.folder.is_some_and(|at| tree.contains(&at)))
        .collect()
}

pub fn bodies(session: &Session) -> Vec<(String, String)> {
    let Some(root) = root_of(&session.state) else {
        return Vec::new();
    };
    let tree = tree(&session.state, root);
    let docs = session.paths.docs();
    docs_in(&session.state, &tree)
        .into_iter()
        .filter(|one| !session.state.held_away(one))
        .filter_map(|one| {
            tisty_core::docs::read(&docs, &one.file)
                .ok()
                .map(|body| (one.file.clone(), body))
        })
        .collect()
}

// Tags an earlier build took from an assistant's words go; the approval stays, since it goes by print.
pub(crate) fn untagged(session: &mut Session) {
    let Some(root) = root_of(&session.state) else {
        return;
    };
    let tagged: Vec<String> = docs_in(&session.state, &tree(&session.state, root))
        .into_iter()
        .filter(|one| !one.tags.is_empty())
        .map(|one| one.file.clone())
        .collect();
    let docs = session.paths.docs();
    for file in tagged {
        if let Ok(body) = tisty_core::docs::read(&docs, &file)
            && Card::of(&body).is_some()
        {
            session.retell(&file, &body, None);
        }
    }
}

/// The only way to what may be written into an agent's file on this computer.
pub fn gated(session: &Session, ledger: &Ledger) -> Gate {
    let docs = session.paths.docs();
    agents::gate(&agents::Asked {
        state: &session.state,
        docs: &docs,
        root: root_of(&session.state),
        ledger,
        me: session.store.device(),
    })
}

// Another computer's memory never stands for this one's, so its file here can still be brought.
fn addresses(bodies: &[(String, String)], gate: &Gate) -> BTreeSet<String> {
    bodies
        .iter()
        .filter(|(file, _)| gate.seen(file) != Some(Seen::OtherComputersMemory))
        .filter_map(|(_, body)| Card::of(body).map(|card| card.address()))
        .collect()
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSeen {
    pub at: String,
    pub name: String,
    pub kind: &'static str,
    pub skip: Option<String>,
    /// A document in AGENTS already stands for it, so it is compared rather than brought.
    pub held: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    pub id: String,
    pub name: String,
    pub on: bool,
    pub files: Vec<FileSeen>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Look {
    pub folder: Option<String>,
    pub docs: usize,
    /// Files changed outside Tisty, or on both sides, that wait for the person.
    pub changes: usize,
    /// Documents from another computer that wait for «apply here» or «not on this computer».
    pub waiting: usize,
    /// Documents whose words the person has not approved, held back from every agent's file.
    pub unagreed: usize,
    /// Documents with an assistant's change waiting beside the approved text.
    pub proposed: usize,
    pub consented: bool,
    pub all_on: bool,
    pub asks: bool,
    pub assistants: Vec<Shown>,
    pub both: Vec<Both>,
    pub loose: Vec<Loose>,
    pub unled: Vec<String>,
}

fn address_of(one: &Found) -> String {
    format!("{}/{}/{}", one.assistant, one.kind.word(), one.name)
}

fn found_here(homes: &Homes) -> Vec<Found> {
    let spots: Vec<agents::Place> = homes
        .detected()
        .into_iter()
        .flat_map(|one| places::spots(homes, one))
        .collect();
    agents::found(&spots)
}

pub struct Gathered {
    root: Option<FolderId>,
    docs: usize,
    proposed: usize,
    bodies: Vec<(String, String)>,
    ledger: Ledger,
    gate: Gate,
}

pub fn gathered(session: &Session, here: &str) -> Gathered {
    let root = root_of(&session.state);
    let ledger = Ledger::open(session.paths.data(), here);
    Gathered {
        root,
        docs: root
            .map(|root| docs_in(&session.state, &tree(&session.state, root)).len())
            .unwrap_or(0),
        proposed: proposed_in(&session.state),
        bodies: bodies(session),
        gate: gated(session, &ledger),
        ledger,
    }
}

fn proposed_in(state: &tisty_core::State) -> usize {
    state
        .proposals
        .keys()
        .filter_map(|id| state.docs.get(id))
        .filter(|kept| agents::holds(state, kept))
        .count()
}

// A document no assistant here would read has nothing to wait for on this computer.
fn waiting_here(seen: &Gathered, homes: &Homes) -> usize {
    seen.bodies
        .iter()
        .filter(|(file, _)| seen.gate.seen(file) == Some(Seen::WaitingHere))
        .filter_map(|(_, body)| Card::of(body))
        .filter(|card| places::lands_here(homes, card))
        .count()
}

// A file Tisty writes now would come back in with Tisty's own words inside it.
fn written_now(ledger: &Ledger, placed: &[Placed]) -> BTreeSet<PathBuf> {
    placed
        .iter()
        .filter(|one| ledger.written(&one.target.at).is_some())
        .map(|one| one.target.at.clone())
        .collect()
}

fn known(held: &BTreeSet<String>, writes: &BTreeSet<PathBuf>, one: &Found) -> bool {
    held.contains(&address_of(one)) || (one.kind != Kind::Memory && writes.contains(&one.at))
}

/// Names and sizes only: no agent file is opened before the person says yes.
pub fn looked(seen: &Gathered, homes: &Homes, found: &[Found]) -> Look {
    let ledger = &seen.ledger;
    let (root, docs) = (seen.root, seen.docs);
    let held = addresses(&seen.bodies, &seen.gate);
    let placed = places::targets(homes, &seen.gate, ledger);
    let (changes, both) = weighed(ledger, &placed);
    let loose: Vec<Loose> = loose_in(homes, &seen.gate, ledger, &placed, root.is_some())
        .into_iter()
        .map(|(one, _)| one)
        .collect();
    let unled = unled_in(ledger, &placed);
    let writes = written_now(ledger, &placed);
    let assistants = homes
        .detected()
        .into_iter()
        .map(|one| Shown {
            id: one.id.to_string(),
            name: one.name.to_string(),
            on: ledger.syncs(one.id),
            files: found
                .iter()
                .filter(|seen| seen.assistant == one.id)
                .map(|seen| FileSeen {
                    at: seen.at.to_string_lossy().into_owned(),
                    name: seen.name.clone(),
                    kind: seen.kind.word(),
                    skip: seen.skip.map(skip_word),
                    held: known(&held, &writes, seen),
                })
                .collect(),
        })
        .collect();
    Look {
        folder: root.map(|at| at.to_string()),
        docs,
        changes: changes + loose.len() + unled.len(),
        waiting: waiting_here(seen, homes),
        unagreed: seen.gate.unagreed(),
        proposed: seen.proposed,
        // Only a folder that syncs asks on its own; one brought but not synced opens like any other.
        asks: ledger.consented()
            && ledger.syncs("")
            && (!both.is_empty() || !loose.is_empty() || !unled.is_empty()),
        consented: ledger.consented(),
        all_on: ledger.syncs(""),
        assistants,
        both,
        loose,
        unled,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Loose {
    pub at: String,
    pub assistant: String,
    pub changed: bool,
}

// A document gone, uncarded, declined here or sent elsewhere frees its file; a Global rule only follows where the assistant reads.
fn frees(homes: &Homes, gate: &Gate, at: &Path, file: &str) -> Option<bool> {
    match gate.seen(file) {
        None | Some(Seen::Plain | Seen::DeclinedHere) => Some(true),
        Some(Seen::Admitted) => {
            let card = gate
                .admitted()
                .iter()
                .find(|one| one.file() == file)?
                .card();
            let elsewhere = card.level != GLOBAL
                && places::target_of(homes, card).map(|(_, there)| there) != Some(at.to_path_buf());
            Some(elsewhere)
        }
        Some(_) => None,
    }
}

// With AGENTS gone, its files wait to be brought back the next first time, never to be taken away.
pub(crate) fn loose_in(
    homes: &Homes,
    gate: &Gate,
    ledger: &Ledger,
    placed: &[Placed],
    rooted: bool,
) -> Vec<(Loose, Target)> {
    if !rooted || !ledger.consented() {
        return Vec::new();
    }
    let placing = |at: &Path| {
        placed.iter().any(|one| {
            one.target.at == at || one.lead_in.as_ref().is_some_and(|lead| lead.at == at)
        })
    };
    let mut out = Vec::new();
    for (at, files) in ledger.written_files() {
        if placing(at) || places::agents_own(homes, at) || files.is_empty() {
            continue;
        }
        let Some((assistant, root)) = places::owner_of(homes, at) else {
            continue;
        };
        let Ok(now) = std::fs::read(at) else {
            continue;
        };
        if !ledger.syncs(assistant) || files.iter().any(|file| gate.held(file)) {
            continue;
        }
        let freed: Option<Vec<bool>> = files
            .iter()
            .map(|file| frees(homes, gate, at, file))
            .collect();
        if !freed.is_some_and(|all| all.contains(&true)) {
            continue;
        }
        let changed = ledger.written(at) != Some(tisty_core::attach::printed(&now).as_str());
        out.push((
            Loose {
                at: at.to_string_lossy().into_owned(),
                assistant: assistant.to_string(),
                changed,
            },
            gate.target(ledger, root, at.to_path_buf(), assistant),
        ));
    }
    out
}

// Once Tisty put its line in, a line gone is the person's word: the assistant waits until they put it back.
pub(crate) fn unled(ledger: &Ledger, lead: &Target) -> bool {
    ledger.consented()
        && ledger.syncs(&lead.assistant)
        && ledger.led(&lead.at)
        && agents::lacks(lead, &places::lead_in_line())
}

fn unled_in(ledger: &Ledger, placed: &[Placed]) -> Vec<String> {
    placed
        .iter()
        .filter(|one| one.lead_in.as_ref().is_some_and(|lead| unled(ledger, lead)))
        .map(|one| one.target.assistant.clone())
        .collect()
}

// A choice left for later stays open here until the person makes it; every file changed outside is counted beside it.
pub(crate) fn weighed(ledger: &Ledger, placed: &[Placed]) -> (usize, Vec<Both>) {
    let mut changes = 0;
    let mut both = Vec::new();
    for one in placed {
        match agents::standing(ledger, &one.target) {
            Standing::Both { now } => {
                changes += 1;
                both.push(both_of(&one.target, &now));
            }
            Standing::Outside { .. } | Standing::Unproven => changes += 1,
            _ => {}
        }
    }
    (changes, both)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub side: &'static str,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Both {
    pub at: String,
    pub assistant: String,
    /// Made of several documents, or of a Global one others share, so only Tisty's side can be kept from here.
    pub composed: bool,
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Told {
    pub at: String,
    pub assistant: String,
    pub files: Vec<String>,
    pub standing: &'static str,
}

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rounded {
    pub told: Vec<Told>,
    pub both: Vec<Both>,
    pub loose: Vec<Loose>,
    pub unled: Vec<String>,
}

fn word(standing: &Standing) -> &'static str {
    match standing {
        Standing::NoConsent => "noConsent",
        Standing::Off => "off",
        Standing::Held => "held",
        Standing::Unproven => "unproven",
        Standing::Empty => "empty",
        Standing::Linked => "linked",
        Standing::Broken { .. } => "broken",
        Standing::UpToDate | Standing::Agreed => "upToDate",
        Standing::Write => "write",
        Standing::Wrote => "wrote",
        Standing::Outside { .. } => "outside",
        Standing::Both { .. } => "both",
        Standing::Unreadable(_) => "unreadable",
    }
}

fn lines_of(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .take(LINES_AT_MOST)
        .map(str::to_string)
        .collect()
}

pub fn compared(tisty: &[u8], file: &[u8]) -> Vec<Line> {
    let (a, b) = (lines_of(tisty), lines_of(file));
    let pairs = tisty_core::lcs::pairs(&a, &b);
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    for (pi, pj) in pairs.into_iter().chain([(a.len(), b.len())]) {
        out.extend(a[i..pi].iter().map(|text| Line {
            side: "tisty",
            text: text.clone(),
        }));
        out.extend(b[j..pj].iter().map(|text| Line {
            side: "file",
            text: text.clone(),
        }));
        if pi < a.len() && pj < b.len() {
            out.push(Line {
                side: "same",
                text: a[pi].clone(),
            });
        }
        (i, j) = (pi + 1, pj + 1);
    }
    out
}

fn own_alone(target: &Target) -> bool {
    match target.pieces.as_slice() {
        [one] => one.card().level == target.assistant,
        _ => false,
    }
}

fn both_of(target: &Target, now: &[u8]) -> Both {
    let tisty = agents::wanted(target).unwrap_or_default();
    Both {
        at: target.at.to_string_lossy().into_owned(),
        assistant: target.assistant.clone(),
        composed: !own_alone(target),
        lines: compared(&tisty, now),
    }
}

pub fn rounded(data: &Path, ledger: &mut Ledger, placed: &[Placed]) -> Rounded {
    let targets: Vec<Target> = placed.iter().map(|one| one.target.clone()).collect();
    let standings = agents::round(data, ledger, &targets);
    let mut out = Rounded::default();
    for (one, standing) in placed.iter().zip(&standings) {
        if let (Standing::Wrote | Standing::UpToDate, Some(lead)) = (standing, &one.lead_in) {
            led_in(data, ledger, lead, placed, &standings);
        }
        if let Standing::Both { now } = standing {
            out.both.push(both_of(&one.target, now));
        }
        out.told.push(Told {
            at: one.target.at.to_string_lossy().into_owned(),
            assistant: one.target.assistant.clone(),
            files: one.files.clone(),
            standing: word(standing),
        });
    }
    out
}

// Tisty's own line in the agent's file is settled at once, unless the agent had already changed that file.
fn led_in(
    data: &Path,
    ledger: &mut Ledger,
    lead: &Target,
    placed: &[Placed],
    standings: &[Standing],
) {
    let quiet = placed
        .iter()
        .zip(standings)
        .find(|(one, _)| one.target.at == lead.at)
        .is_none_or(|(_, standing)| matches!(standing, Standing::UpToDate | Standing::Wrote));
    let fresh = !ledger.led(&lead.at);
    match agents::lead_in(data, ledger, lead, &places::lead_in_line()) {
        Ok(added) => {
            ledger.lead(&lead.at);
            if added
                && quiet
                && let Ok(now) = std::fs::read(&lead.at)
            {
                ledger.settle(&lead.at, &now);
            }
            if fresh || added {
                let _ = ledger.keep(data);
            }
        }
        Err(why) => tisty_core::witness::warn(
            tisty_core::witness::channel::AGENT,
            "the line that names Tisty's rules could not be added to the agent's own file",
            &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
        ),
    }
}

pub(crate) fn round_now(session: &Session, homes: &Homes, ledger: &mut Ledger) -> Rounded {
    round_after(session, homes, ledger, &[])
}

// An earlier build marked no line; it settled the agent's own file only once it had put its line in, unless that file is a memory document here.
fn marked_led(data: &Path, ledger: &mut Ledger, placed: &[Placed]) {
    let mut marked = false;
    for lead in placed.iter().filter_map(|one| one.lead_in.as_ref()) {
        let remembered = placed.iter().any(|one| one.target.at == lead.at);
        if !ledger.led(&lead.at) && !remembered && ledger.written(&lead.at).is_some() {
            ledger.lead(&lead.at);
            marked = true;
        }
    }
    if marked {
        let _ = ledger.keep(data);
    }
}

// Files just brought are taken as they stand before the round, so what joins them is written over them with a backup.
fn round_after(
    session: &Session,
    homes: &Homes,
    ledger: &mut Ledger,
    brought: &[(String, PathBuf)],
) -> Rounded {
    let data = session.paths.data();
    let gate = gated(session, ledger);
    if ledger.taken_all(gate.admitted().iter().map(agents::Admitted::file)) {
        let _ = ledger.keep(session.paths.data());
    }
    let placed = places::targets(homes, &gate, ledger);
    let mut adopted = false;
    for (file, at) in brought {
        if let Some(one) = placed.iter().find(|one| one.target.at == *at) {
            adopted |= agents::adopted(ledger, &one.target, file);
        }
    }
    if adopted {
        let _ = ledger.keep(data);
    }
    marked_led(data, ledger, &placed);
    let rooted = root_of(&session.state).is_some();
    let loose = loose_in(homes, &gate, ledger, &placed, rooted)
        .into_iter()
        .map(|(one, _)| one)
        .collect();
    let (paused, going): (Vec<Placed>, Vec<Placed>) = placed
        .into_iter()
        .partition(|one| one.lead_in.as_ref().is_some_and(|lead| unled(ledger, lead)));
    let mut out = rounded(data, ledger, &going);
    out.told.extend(paused.iter().map(|one| Told {
        at: one.target.at.to_string_lossy().into_owned(),
        assistant: one.target.assistant.clone(),
        files: one.files.clone(),
        standing: "unled",
    }));
    out.unled = paused
        .iter()
        .map(|one| one.target.assistant.clone())
        .collect();
    out.loose = loose;
    out
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Turned {
    pub at: String,
    pub why: String,
    /// What the window says in the person's language; `why` is only shown when it has no words for it.
    pub reason: String,
}

pub(crate) fn refused(why: agents::Refused) -> Refusal {
    use agents::Refused as R;
    match why {
        R::NoConsent => Refusal::of("agentNoConsent"),
        R::Off => Refusal::of("agentOff"),
        R::Held => Refusal::of("agentHeld"),
        R::AgentsOwn => Refusal::of("agentOwn"),
        R::Empty => Refusal::of("agentEmpty"),
        R::Linked => Refusal::of("agentLinked"),
        R::Changed => Refusal::of("agentChanged"),
        R::Unattributed => Refusal::of("agentUnattributed"),
        R::NoSuchBackup => Refusal::of("agentNoBackup"),
        R::TooBig => Refusal::of("agentTooBig"),
        R::NotText => Refusal::of("agentNotText"),
        R::BrokenFence => Refusal::of("agentBrokenFence"),
        R::Secret(told) => Refusal::about("agentSecret", told),
        other => Refusal::about("internalNamed", other.to_string()),
    }
}

// The window's `agentsSkip_*` words are keyed by this.
fn skip_word(skip: agents::Skip) -> String {
    format!("{skip:?}").to_lowercase()
}

fn reason_of(why: &agents::Refused) -> String {
    use agents::Refused as R;
    let said = match why {
        R::Skipped(skip) => return format!("skip_{}", skip_word(*skip)),
        R::NoConsent => "noConsent",
        R::Secret(_) => "secret",
        R::TooBig => "tooBig",
        R::NotText => "notText",
        R::Linked => "linked",
        R::Name => "name",
        _ => "other",
    };
    said.to_string()
}

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Brought {
    pub made: usize,
    pub refused: Vec<Turned>,
    pub round: Rounded,
    /// Whether the folder syncs once brought: «Bring» alone leaves every agent file as it is.
    pub syncs: bool,
}

pub fn read_chosen(
    ledger: &Ledger,
    found: &[Found],
    chosen: &BTreeSet<String>,
    held: &BTreeSet<String>,
    writes: &BTreeSet<PathBuf>,
) -> (Vec<(Found, String)>, Vec<Turned>) {
    let mut bodies = Vec::new();
    let mut refused = Vec::new();
    for one in found {
        let at = one.at.to_string_lossy().into_owned();
        if !chosen.contains(&at) || known(held, writes, one) {
            continue;
        }
        match agents::brought_in(ledger, one) {
            Ok(body) => bodies.push((one.clone(), body)),
            Err(why) => refused.push(Turned {
                at,
                reason: reason_of(&why),
                why: why.to_string(),
            }),
        }
    }
    (bodies, refused)
}

fn sub_name(level: &str) -> &'static str {
    places::assistant(level).map_or(GLOBAL_FOLDER, |one| one.name)
}

pub fn filed(session: &mut Session, bodies: &[(Found, String)]) -> Answer<Vec<(String, PathBuf)>> {
    if bodies.is_empty() && root_of(&session.state).is_some() {
        return Ok(Vec::new());
    }
    let mut ops = Vec::new();
    let mut orders: BTreeMap<Option<FolderId>, Vec<String>> = BTreeMap::new();
    let mut next = |state: &State, parent: Option<FolderId>, docs: bool| -> String {
        let taken = orders.entry(parent).or_insert_with(|| match docs {
            true => state
                .docs
                .values()
                .filter(|one| one.folder == parent && one.page_of.is_none())
                .map(|one| one.order.clone())
                .collect(),
            false => state
                .under(parent)
                .iter()
                .map(|one| one.order.clone())
                .collect(),
        });
        let order = tisty_core::order::last_of(taken.iter().map(String::as_str));
        taken.push(order.clone());
        order
    };
    let folder = |ops: &mut Vec<Op>, name: &str, parent: Option<FolderId>, mark: bool| {
        let id = ulid::Ulid::generate();
        ops.push(Op::FolderAdd {
            id,
            d: tisty_core::event::FolderAdd {
                name: name.to_string(),
                order: String::new(),
                parent,
                icon: mark.then(|| places::ICON.to_string()),
                color: mark.then(|| places::HUE.to_string()),
            },
        });
        id
    };
    let root = match root_of(&session.state) {
        Some(root) => root,
        None => {
            let root = folder(&mut ops, places::FOLDER, None, true);
            folder(&mut ops, GLOBAL_FOLDER, Some(root), false);
            root
        }
    };
    let mut subs: BTreeMap<&'static str, FolderId> = session
        .state
        .folders
        .values()
        .filter(|one| one.parent == Some(root))
        .filter_map(|one| {
            ASSISTANTS
                .iter()
                .map(|known| known.name)
                .chain([GLOBAL_FOLDER])
                .find(|name| *name == one.name)
                .map(|name| (name, one.id))
        })
        .collect();
    let device = session.store.device().clone();
    let docs = session.paths.docs();
    let signed = signing(&session.state);
    let mut made = Vec::new();
    for (one, body) in bodies {
        let name = sub_name(&one.assistant);
        let sub = match subs.get(name) {
            Some(sub) => *sub,
            None => {
                let sub = folder(&mut ops, name, Some(root), false);
                subs.insert(name, sub);
                sub
            }
        };
        let kept = match tisty_core::docs::create(&docs, &device, body) {
            Ok(kept) => kept,
            Err(e) => return Err(undone(&docs, &made, e)),
        };
        made.push((kept.id.clone(), one.at.clone()));
        ops.push(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: kept.id,
                folder: Some(sub),
                order: next(&session.state, Some(sub), true),
                said: Some(tisty_core::event::Said::of(body).agreed()),
                by: signed.clone(),
                ..Default::default()
            },
        });
    }
    for op in &mut ops {
        if let Op::FolderAdd { d, .. } = op {
            d.order = next(&session.state, d.parent, false);
        }
    }
    match session.commit_all(ops) {
        Ok(()) => Ok(made),
        Err(e) => Err(undone(&docs, &made, e)),
    }
}

// A file written for an import that did not land would sit in the store with nothing naming it.
fn undone(docs: &Path, made: &[(String, PathBuf)], why: tisty_core::Error) -> Refusal {
    for (id, _) in made {
        let _ = tisty_core::docs::remove(docs, id);
    }
    Refusal::about("cannotWrite", why.to_string())
}

type Known = (Ledger, BTreeSet<String>, BTreeSet<PathBuf>);

/// `sync` is the person's word: none keeps the switch, and a first yes without it never syncs.
pub fn consented_here(
    session: &Session,
    homes: &Homes,
    here: &str,
    sync: Option<bool>,
) -> Answer<Known> {
    let mut ledger = Ledger::open(session.paths.data(), here);
    let syncs = sync.unwrap_or(ledger.consented() && ledger.syncs(""));
    ledger.consent(true);
    ledger.turn_all(syncs);
    ledger.keep(session.paths.data()).map_err(refused)?;
    let gate = gated(session, &ledger);
    let writes = written_now(&ledger, &places::targets(homes, &gate, &ledger));
    Ok((ledger, addresses(&bodies(session), &gate), writes))
}

pub fn brought_home(
    session: &mut Session,
    homes: &Homes,
    mut ledger: Ledger,
    read: &[(Found, String)],
    refused: Vec<Turned>,
) -> Answer<Brought> {
    let made = filed(session, read)?;
    let round = round_after(session, homes, &mut ledger, &made);
    Ok(Brought {
        made: made.len(),
        refused,
        round,
        syncs: ledger.syncs(""),
    })
}

fn placed_at(session: &Session, homes: &Homes, ledger: &Ledger, at: &str) -> Answer<Placed> {
    places::targets(homes, &gated(session, ledger), ledger)
        .into_iter()
        .find(|one| one.target.at.to_string_lossy() == at)
        .ok_or_else(|| Refusal::about("cannotWrite", at.to_string()))
}

pub fn kept_side(
    session: &mut Session,
    homes: &Homes,
    here: &str,
    at: &str,
    tisty: bool,
) -> Answer<Rounded> {
    let mut ledger = Ledger::open(session.paths.data(), here);
    let placed = placed_at(session, homes, &ledger, at)?;
    match tisty {
        true => {
            agents::overwrite(session.paths.data(), &mut ledger, &placed.target).map_err(refused)?
        }
        false => {
            let (true, [file]) = (own_alone(&placed.target), placed.files.as_slice()) else {
                return Err(refused(agents::Refused::Unattributed));
            };
            let body = agents::bring(&ledger, &placed.target, 0).map_err(refused)?;
            rewritten(session, file, &body)?;
        }
    }
    Ok(round_now(session, homes, &mut ledger))
}

// The document's former words stay in its history, which is Tisty's side of the backup.
fn rewritten(session: &mut Session, file: &str, body: &str) -> Answer<()> {
    rewritten_by(session, file, body, None)
}

/// Written as `via`, so the document's history keeps what it said before whoever wrote it last.
pub(crate) fn rewritten_by(
    session: &mut Session,
    file: &str,
    body: &str,
    via: Option<String>,
) -> Answer<()> {
    let docs = session.paths.docs();
    let cannot = |e: tisty_core::Error| Refusal::about("cannotWrite", e.to_string());
    let at = tisty_core::docs::resolve(&docs, file).map_err(cannot)?;
    let print = tisty_core::docs::print_of(&at)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?
        .unwrap_or_default();
    let writer = tisty_core::history::Writer::now(session.paths.data(), via.clone());
    let made = tisty_core::docs::rewrite(&docs, &writer, file, body, &print).map_err(cannot)?;
    match made {
        tisty_core::docs::Rewrite::Made { .. } => {
            session.retell_written(file, body, via);
            Ok(())
        }
        tisty_core::docs::Rewrite::Moved => Err(Refusal::about("documentMoved", file.to_string())),
    }
}

pub fn turned(
    session: &Session,
    homes: &Homes,
    here: &str,
    assistant: Option<&str>,
    on: bool,
) -> Answer<Rounded> {
    let mut ledger = Ledger::open(session.paths.data(), here);
    match assistant {
        Some(id) => ledger.turn(id, on),
        None => ledger.turn_all(on),
    }
    ledger.keep(session.paths.data()).map_err(refused)?;
    Ok(match on {
        true => round_now(session, homes, &mut ledger),
        false => Rounded::default(),
    })
}

pub fn loosened(
    session: &Session,
    homes: &Homes,
    here: &str,
    at: &str,
    away: bool,
) -> Answer<Rounded> {
    let data = session.paths.data();
    let mut ledger = Ledger::open(data, here);
    let gate = gated(session, &ledger);
    let placed = places::targets(homes, &gate, &ledger);
    let rooted = root_of(&session.state).is_some();
    let Some((_, target)) = loose_in(homes, &gate, &ledger, &placed, rooted)
        .into_iter()
        .find(|(one, _)| one.at == at)
    else {
        return Err(refused(agents::Refused::Held));
    };
    match away {
        true => {
            agents::removed(data, &mut ledger, &target).map_err(refused)?;
            unlead(data, &mut ledger, &target);
        }
        false => {
            ledger.forget(&target.at);
            ledger.keep(data).map_err(refused)?;
        }
    }
    Ok(round_now(session, homes, &mut ledger))
}

// With the file of rules gone, the line naming it would point at nothing.
fn unlead(data: &Path, ledger: &mut Ledger, gone: &Target) {
    let Some(lead) = places::lead_of(gone) else {
        return;
    };
    if agents::lead_out(data, ledger, &lead, &places::lead_in_line()).is_ok() {
        ledger.forget(&lead.at);
        let _ = ledger.keep(data);
    }
}

// A file its last rule left waits for the person's answer first, or AGENTS would go and leave it unasked.
pub fn emptied(session: &mut Session, homes: &Homes, here: &str) -> Answer<bool> {
    let Some(root) = root_of(&session.state) else {
        return Ok(false);
    };
    let tree = tree(&session.state, root);
    if !docs_in(&session.state, &tree).is_empty() {
        return Ok(false);
    }
    let ledger = Ledger::open(session.paths.data(), here);
    let gate = gated(session, &ledger);
    let placed = places::targets(homes, &gate, &ledger);
    if !loose_in(homes, &gate, &ledger, &placed, true).is_empty() {
        return Ok(false);
    }
    // Deepest first: a folder deleted before its children would lift them to the top.
    let mut gone: Vec<FolderId> = tree.into_iter().collect();
    gone.sort_by_key(|at| std::cmp::Reverse(session.state.depth(Some(*at))));
    let ops = gone.into_iter().map(|id| Op::FolderDelete { id }).collect();
    session
        .commit_all(ops)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    Ok(true)
}

pub fn led_again(session: &Session, homes: &Homes, here: &str) -> Answer<Rounded> {
    let data = session.paths.data();
    let mut ledger = Ledger::open(data, here);
    let placed = places::targets(homes, &gated(session, &ledger), &ledger);
    for lead in placed.iter().filter_map(|one| one.lead_in.as_ref()) {
        if unled(&ledger, lead) {
            agents::lead_in(data, &ledger, lead, &places::lead_in_line()).map_err(refused)?;
        }
    }
    Ok(round_now(session, homes, &mut ledger))
}

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Present {
    pub id: String,
    pub name: String,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stance {
    pub folder: Option<String>,
    /// Documents in the folder, so a folder brought on another computer reads as brought here too.
    pub held: usize,
    pub consented: bool,
    pub all_on: bool,
    pub assistants: Vec<Present>,
}

/// Whether AGENTS is on and which assistants have a folder here: before the person's yes, nothing inside one is looked at.
pub fn stance(session: &Session, homes: &Homes, here: &str) -> Stance {
    let ledger = Ledger::open(session.paths.data(), here);
    let root = root_of(&session.state);
    Stance {
        folder: root.map(|at| at.to_string()),
        held: root.map_or(0, |at| {
            docs_in(&session.state, &tree(&session.state, at))
                .into_iter()
                .filter(|kept| !session.state.held_away(kept))
                .count()
        }),
        consented: ledger.consented(),
        all_on: ledger.syncs(""),
        assistants: homes
            .detected()
            .into_iter()
            .map(|one| Present {
                id: one.id.to_string(),
                name: one.name.to_string(),
            })
            .collect(),
    }
}

#[tauri::command(async)]
pub fn agents_state(session: tauri::State<'_, Mutex<Session>>) -> Answer<Stance> {
    let homes = homes()?;
    Ok(stance(&held(&session), &homes, &here()))
}

#[tauri::command(async)]
pub fn agents_look(session: tauri::State<'_, Mutex<Session>>) -> Answer<Look> {
    let homes = homes()?;
    let found = found_here(&homes);
    let mut session = held(&session);
    // An assistant's proposal arrives in the log alone, so the look reads the log again first.
    session.reload()?;
    let seen = gathered(&session, &here());
    Ok(looked(&seen, &homes, &found))
}

#[tauri::command(async)]
pub fn agents_bring(
    session: tauri::State<'_, Mutex<Session>>,
    chosen: Vec<String>,
    sync: Option<bool>,
) -> Answer<Brought> {
    let homes = homes()?;
    let (ledger, known, writes) = consented_here(&held(&session), &homes, &here(), sync)?;
    let chosen: BTreeSet<String> = chosen.into_iter().collect();
    let (read, refused) = read_chosen(&ledger, &found_here(&homes), &chosen, &known, &writes);
    brought_home(&mut held(&session), &homes, ledger, &read, refused)
}

#[tauri::command(async)]
pub fn agents_keep(
    session: tauri::State<'_, Mutex<Session>>,
    at: String,
    tisty: bool,
) -> Answer<Rounded> {
    let homes = homes()?;
    kept_side(&mut held(&session), &homes, &here(), &at, tisty)
}

#[tauri::command(async)]
pub fn agents_turn(
    session: tauri::State<'_, Mutex<Session>>,
    assistant: Option<String>,
    on: bool,
) -> Answer<Rounded> {
    let homes = homes()?;
    turned(&held(&session), &homes, &here(), assistant.as_deref(), on)
}

#[tauri::command(async)]
pub fn agents_round(session: tauri::State<'_, Mutex<Session>>) -> Answer<Rounded> {
    let homes = homes()?;
    let mut session = held(&session);
    session.reload()?;
    untagged(&mut session);
    let mut ledger = Ledger::open(session.paths.data(), &here());
    Ok(round_now(&session, &homes, &mut ledger))
}

#[tauri::command(async)]
pub fn agents_loose(
    session: tauri::State<'_, Mutex<Session>>,
    at: String,
    away: bool,
) -> Answer<Rounded> {
    let homes = homes()?;
    loosened(&held(&session), &homes, &here(), &at, away)
}

#[tauri::command]
pub fn agents_empty(session: tauri::State<'_, Mutex<Session>>) -> Answer<bool> {
    let homes = homes()?;
    emptied(&mut held(&session), &homes, &here())
}

#[tauri::command(async)]
pub fn agents_lead(session: tauri::State<'_, Mutex<Session>>) -> Answer<Rounded> {
    let homes = homes()?;
    led_again(&held(&session), &homes, &here())
}

#[cfg(test)]
#[path = "agents_sync_test.rs"]
mod tests;
