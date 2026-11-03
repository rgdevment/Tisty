use std::sync::Mutex;

use serde::Serialize;
use tisty_core::Op;
use tisty_core::event::{Shared, Signs};
use tisty_core::model::DocId;
use tisty_share_proto::limits::Limits;

use super::citing;
use crate::handed::Voucher;
use crate::sharing::{self, Change, Chosen, Info, Net};
use crate::{Answer, Refusal, Session, held};

/// Whether a document can go out from here, which its «Shared» row says before anything is asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Shareable {
    Yes,
    /// Someone else's: only its author shares it (decisions 68 and 70).
    NotMine,
    /// What is shared is always signed, and this machine has no key a person vouches for yet.
    NoKey,
    Off,
}

// The key this machine signs with and the person it answers for; without both nothing is shared.
/// Whether this machine holds a key its person vouches for, which everything that goes up is signed with.
pub(crate) fn can_sign(session: &Session) -> bool {
    sharer(session).is_some()
}

pub(super) fn sharer(session: &Session) -> Option<(Voucher, tisty_share_proto::identity::Person)> {
    let device = &session.config.device_id;
    let voucher = crate::handed::vouched_by(&session.paths, device, &session.state.persons)?;
    let me = session.state.persons.person(device)?;
    Some((voucher, me))
}

pub(crate) fn shareable(session: &Session, kept: &tisty_core::model::Kept) -> Shareable {
    if !sharing::ON {
        return Shareable::Off;
    }
    match sharer(session) {
        Some((_, me)) if session.state.mine_to_share(kept, &me) => Shareable::Yes,
        Some(_) => Shareable::NotMine,
        // With no key to go by, the alias still tells a guest's copy or a stranger's document apart.
        None if session.state.mine(kept) => Shareable::NoKey,
        None => Shareable::NotMine,
    }
}

/// What the «Shared» card shows: the links to hand out, and how it stands when the server could be asked.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    read: String,
    edit: Option<String>,
    gated: bool,
    version: u64,
    /// Kept as it was shared rather than following what is saved.
    still: bool,
    /// Someone else's share, edited here through their edit link.
    taken: bool,
    /// Shared only because another share holds it, which it lives and goes with (decision 64).
    held: bool,
    /// How many documents go with it, its pages and what it cites.
    holds: usize,
    /// None when the server could not be reached; the links still work without it.
    info: Option<Info>,
    held_back: Option<Refusal>,
}

fn card(shared: &Shared, info: Option<Info>) -> Card {
    let (read, edit) = sharing::links(shared);
    Card {
        read,
        edit,
        gated: shared.gated,
        version: shared.version,
        still: shared.still,
        taken: shared.taken,
        held: shared.held,
        holds: shared.holds.len(),
        info,
        held_back: None,
    }
}

// The window knows a document by its file, as it reads and writes it.
pub(super) fn doc_of(session: &Session, file: &str) -> Answer<DocId> {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .map(|one| one.id)
        .ok_or(Refusal::of("noSuchDoc"))
}

// The network is slow and the work takes a moment, so neither holds the window or the session.
pub(super) async fn apart<T: Send + 'static>(
    work: impl FnOnce() -> Answer<T> + Send + 'static,
) -> Answer<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|_| Refusal::of("internal"))?
}

/// A line that reads like a key or a password, which «Share» names before anyone with the link can read it.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Secret {
    line: usize,
    /// What the line calls it, when it calls it anything, such as `AWS_SECRET_ACCESS_KEY`.
    named: String,
    /// The heading it sits under, since the page shows headings and no line numbers.
    under: Option<String>,
}

fn secrets(text: &str) -> Vec<Secret> {
    let over = headings_over(text);
    tisty_core::agent::secrets_in(text.as_bytes())
        .into_iter()
        .map(|one| Secret {
            line: one.line,
            named: one.named,
            under: over.get(one.line.saturating_sub(1)).cloned().flatten(),
        })
        .collect()
}

// The heading over each line in one pass, past the properties and outside fenced code, where a `#` is a comment.
fn headings_over(text: &str) -> Vec<Option<String>> {
    let skipped = properties_end(text).unwrap_or(0);
    let mut fenced = false;
    let mut heading = None;
    let mut over = Vec::new();
    for (at, one) in text.lines().enumerate() {
        over.push(heading.clone());
        let one = one.trim_start();
        if at < skipped {
            continue;
        }
        if one.starts_with("```") || one.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        let marks = one.len() - one.trim_start_matches('#').len();
        let said = one[marks..].strip_prefix(' ').map(str::trim);
        if !fenced
            && (1..=6).contains(&marks)
            && let Some(said) = said.filter(|said| !said.is_empty())
        {
            heading = Some(said.to_owned());
        }
    }
    over
}

// Lines the properties take, only when the first `---` closes over lines that read `key: value`; a rule is no properties.
fn properties_end(text: &str) -> Option<usize> {
    let mut lines = text.lines();
    if lines.next()?.trim_end() != "---" {
        return None;
    }
    let inside = lines.position(|one| one.trim_end() == "---")?;
    let a_property = |one: &str| {
        one.split_once(':')
            .is_some_and(|(key, _)| !key.is_empty() && !key.contains(' '))
    };
    let said = text.lines().skip(1).take(inside);
    let properties = said.clone().any(a_property)
        && said
            .filter(|one| !one.trim().is_empty() && !one.starts_with([' ', '-', '#']))
            .all(a_property);
    properties.then_some(inside + 2)
}

/// What «Share» says before anything goes out: what reads like a key, and the name the share will be signed with.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Look {
    secrets: Vec<Secret>,
    /// The print of what was looked through, handed back with the share.
    seen: String,
    /// Each one only when it is fit to sign, so the dialog offers what can go and the key alone otherwise.
    name: Option<String>,
    alias: Option<String>,
    /// Its pages, which always go with it, and the other documents it cites, which go when chosen (decision 64).
    pages: usize,
    cited: usize,
    pictures: usize,
    cited_pictures: usize,
    shares: Vec<Own>,
    level: Level,
}

#[derive(Debug, Serialize)]
pub struct Own {
    file: String,
    title: String,
}

#[derive(Debug, Serialize)]
pub struct Level {
    shares: u64,
    cited: u64,
    pictures: u64,
}

impl Level {
    fn of(limits: &Limits) -> Self {
        Self {
            shares: limits.shares,
            cited: limits.cited,
            pictures: limits.pictures,
        }
    }
}

pub(crate) fn own_shares(session: &Session) -> Vec<(DocId, Shared)> {
    session
        .state
        .shares
        .iter()
        .filter(|(_, one)| !one.taken && !one.held && one.server == sharing::SERVER)
        .filter(|(doc, _)| session.state.docs.contains_key(doc))
        .map(|(doc, one)| (*doc, one.clone()))
        .collect()
}

fn listed(session: &Session, blocked: &[DocId]) -> Vec<Own> {
    own_shares(session)
        .into_iter()
        .filter(|(doc, _)| !blocked.contains(doc))
        .filter_map(|(doc, _)| {
            let kept = session.state.docs.get(&doc)?;
            Some(Own {
                file: kept.file.clone(),
                title: kept
                    .title
                    .clone()
                    .filter(|title| !title.trim().is_empty())
                    .unwrap_or_else(|| kept.file.clone()),
            })
        })
        .collect()
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Standing {
    pub(crate) gone: Vec<(DocId, String)>,
    pub(crate) blocked: Vec<DocId>,
}

impl Standing {
    pub(crate) fn counted(&self, own: &[(DocId, Shared)]) -> u64 {
        own.len()
            .saturating_sub(self.gone.len() + self.blocked.len()) as u64
    }
}

pub(crate) fn standing_of(wire: &impl sharing::Wire, own: &[(DocId, Shared)]) -> Standing {
    let mut standing = Standing::default();
    for (doc, shared) in own {
        match sharing::info(wire, shared) {
            Ok(info) if info.blocked => standing.blocked.push(*doc),
            Err(refused) if sharing::already_gone(&refused) => {
                standing.gone.push((*doc, shared.link.clone()));
            }
            _ => {}
        }
    }
    standing
}

fn forget_gone(session: &mut Session, gone: Vec<(DocId, String)>) -> Answer<()> {
    let ops: Vec<Op> = gone
        .into_iter()
        .filter(|(doc, link)| {
            session
                .state
                .shares
                .get(doc)
                .is_some_and(|now| now.link == *link)
        })
        .map(|(id, link)| Op::ShareStop { id, link })
        .collect();
    if ops.is_empty() {
        return Ok(());
    }
    session.commit_all(ops).map_err(Refusal::from)
}

// The name a share goes out with, as chosen, held to what an envelope takes.
pub(super) fn signing_name(signed: &tisty_core::event::Signature, signs: Signs) -> Option<String> {
    match signs {
        Signs::Name => fit(&signed.name).or_else(|| fit(&signed.alias)),
        Signs::Alias => fit(&signed.alias),
        Signs::Key => None,
    }
}

fn fit(one: &Option<String>) -> Option<String> {
    one.as_deref()
        .map(str::trim)
        .filter(|one| tisty_share_proto::envelope::named_well(one))
        .map(str::to_owned)
}

#[tauri::command]
pub async fn share_look(session: tauri::State<'_, Mutex<Session>>, file: String) -> Answer<Look> {
    sharing::switched(sharing::ON)?;
    let limits = Limits::default();
    let (docs, data, name, alias, pages, cited, own) = {
        let session = held(&session);
        let doc = doc_of(&session, &file)?;
        let signed = &session.state.signed;
        let reach = citing::reach(&session, doc);
        let room = citing::cited_at_most().saturating_sub(reach.pages.len());
        let pages = citing::texts_of(&session, reach.pages.iter().map(|one| one.doc));
        let cited = citing::texts_of(&session, reach.cited.iter().take(room).map(|one| one.doc));
        (
            session.paths.docs(),
            session.paths.data().to_path_buf(),
            fit(&signed.name),
            fit(&signed.alias),
            (reach.pages.len(), pages),
            (reach.cited.len(), cited),
            own_shares(&session),
        )
    };
    let full = own.len() as u64 >= limits.shares;
    // A long document takes a moment to read and look through, so neither holds the window or the session.
    let (look, standing) = apart(move || {
        let text = tisty_core::docs::read(&docs, &file)
            .map_err(|e| Refusal::about("cannotRead", e.to_string()))?;
        let pictures = citing::pictures_of(std::iter::once(&text).chain(&pages.1), &data);
        let look = Look {
            secrets: secrets(&text),
            seen: sharing::print_of(&text),
            name,
            alias,
            pages: pages.0,
            cited: cited.0,
            pictures,
            cited_pictures: citing::pictures_of(&cited.1, &data),
            shares: Vec::new(),
            level: Level::of(&limits),
        };
        let standing = match full {
            true => standing_of(&Net::to(sharing::SERVER)?, &own),
            false => Standing::default(),
        };
        Ok((look, standing))
    })
    .await?;
    let mut session = held(&session);
    forget_gone(&mut session, standing.gone)?;
    Ok(Look {
        shares: listed(&session, &standing.blocked),
        ..look
    })
}

// A new share as it went up: its links, what went with it, and how the server holds it.
type SharedUp = (Shared, Vec<(citing::Citable, Shared)>, Option<Info>);

#[tauri::command]
pub async fn share_doc(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    following: tauri::State<'_, super::versions::Following>,
    file: String,
    chosen: Chosen,
) -> Answer<Card> {
    sharing::switched(sharing::ON)?;
    let (doc, text, voucher, revoked, name, data, going, own) = {
        let session = held(&session);
        let doc = doc_of(&session, &file)?;
        if session.state.shares.contains_key(&doc) {
            return Err(Refusal::of("shareAlready"));
        }
        let kept = session
            .state
            .docs
            .get(&doc)
            .ok_or(Refusal::of("noSuchDoc"))?;
        // Only what is one's own goes out under one's link and key; what came from someone else is theirs to share.
        match shareable(&session, kept) {
            Shareable::Yes => {}
            Shareable::NotMine => return Err(Refusal::of("shareNotMine")),
            Shareable::NoKey => return Err(Refusal::of("shareNoKey")),
            Shareable::Off => return Err(Refusal::of("shareOff")),
        }
        let text = tisty_core::docs::read(&session.paths.docs(), &file)
            .map_err(|e| Refusal::about("cannotRead", e.to_string()))?;
        // Saved after the dialog looked, the words could hold a key nobody was warned of.
        if chosen
            .seen
            .as_ref()
            .is_some_and(|seen| *seen != sharing::print_of(&text))
        {
            return Err(Refusal::of("shareChanged"));
        }
        let (voucher, me) = sharer(&session).ok_or(Refusal::of("shareNoKey"))?;
        let name = signing_name(&session.state.signed, chosen.signs);
        let going = citing::going_of(&session, citing::reached(&session, doc, chosen.cites)?);
        (
            doc,
            text,
            voucher,
            sharing::taken_back_by(&session.state.persons, &me),
            name,
            session.paths.data().to_path_buf(),
            going,
            own_shares(&session),
        )
    };
    // Never beside a round that keeps what shares cite in step, waiting for one that runs, so no document is reserved twice.
    let Some(_alone) = super::versions::waited(&following, super::versions::CITING).await else {
        return Err(Refusal::of("shareSlow"));
    };
    // The share itself must fit the hour; what it takes along goes as far as the hour allows, and the rest with later saves.
    let room = following
        .take(1, going.len().min(citing::NEW_A_ROUND))
        .ok_or(Refusal::of("shareSlow"))?;
    let rewritten: Vec<String> = going
        .iter()
        .filter(|one| one.was.is_some())
        .map(|one| one.citable.file.clone())
        .collect();
    let (spent, shared_up) = apart(move || {
        let mut spent = citing::Spent::default();
        let shared_up = (|| -> Answer<SharedUp> {
            let most = Limits::default().pictures;
            let texts = std::iter::once(&text).chain(going.iter().map(|one| &one.text));
            if citing::pictures_of(texts, &data) as u64 > most {
                return Err(Refusal::about("sharePicturesFull", most.to_string()));
            }
            let wire = Net::to(sharing::SERVER)?;
            let shares = Limits::default().shares;
            if own.len() as u64 >= shares && standing_of(&wire, &own).counted(&own) >= shares {
                return Err(Refusal::about("shareFull", shares.to_string()));
            }
            // What this person took back may not have reached the server when it happened, so every share tells it again.
            let _ = sharing::revoke(&wire, &revoked);
            let root = sharing::reserve(&wire, sharing::SERVER, &chosen)?;
            spent.reserves += 1;
            let signing = citing::Signing {
                voucher,
                name,
                data,
            };
            // What it cites goes up first, so its own envelope can name each with the key that opens it.
            let members = citing::took_up(&wire, &root, &going, &signing, room, &mut spent)
                .inspect_err(|_| {
                    let _ = sharing::stop(&wire, &root);
                })?;
            let cited = citing::cited_in(&text, &members);
            let (words, pictures) =
                sharing::pictured(&text, |target| sharing::attachment(target, &signing.data));
            let first = sharing::Words {
                text: &words,
                cited: &cited,
                pictures: &pictures,
                voucher: &signing.voucher,
                name: signing.name.as_deref(),
            };
            spent.uploads += 1;
            sharing::first_up(&wire, &root, &first, &chosen)?;
            let info = sharing::info(&wire, &root).ok();
            let shared = Shared {
                print: citing::printed(&text, &cited),
                holds: members.iter().map(|(one, _)| one.doc).collect(),
                ..root
            };
            Ok((shared, members, info))
        })();
        Ok((spent, shared_up))
    })
    .await?;
    // What reached the server stays spent, since its hour counts it whatever came of it.
    let taken = 1 + room;
    following.refund(
        taken - spent.reserves.min(taken),
        taken - spent.uploads.min(taken),
    );
    let (shared, members, info) = shared_up?;
    let committed = {
        let mut session = held(&session);
        // A second press or another window may have shared it while this one was on the way.
        match session.state.shares.contains_key(&doc) {
            true => Err(Refusal::of("shareAlready")),
            false => {
                let mut ops = citing::written(&session, members);
                ops.push(Op::ShareDoc {
                    id: doc,
                    d: shared.clone(),
                });
                session.commit_all(ops).map_err(Refusal::from)
            }
        }
    };
    if let Err(refused) = committed {
        // A link nobody can see is a link nobody can stop, so it goes.
        let _ = apart(move || sharing::stop(&Net::to(&shared.server)?, &shared)).await;
        return Err(refused);
    }
    // What it holds beside things already shared changed where those lead, so their envelopes go up again.
    for file in rewritten {
        super::versions::follow(&app, &file);
    }
    Ok(card(&shared, info))
}

#[tauri::command]
pub async fn share_card(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    following: tauri::State<'_, super::versions::Following>,
    file: String,
) -> Answer<Option<Card>> {
    sharing::switched(sharing::ON)?;
    let held_back = following.held_back_of(&file);
    let card = |shared: &Shared, info: Option<Info>| Card {
        held_back: held_back.clone(),
        ..card(shared, info)
    };
    // Looking at a share is when its newer versions are most wanted, so they are brought first.
    let _ = super::versions::brought(&app, &file).await;
    let shared = {
        let session = held(&session);
        session.state.shares.get(&doc_of(&session, &file)?).cloned()
    };
    let Some(shared) = shared else {
        return Ok(None);
    };
    // Someone else's share has no token to manage it, so there is nothing more to ask the server.
    if shared.taken {
        return Ok(Some(card(&shared, None)));
    }
    let asked = shared.clone();
    let info = apart(move || sharing::info(&Net::to(&asked.server)?, &asked)).await;
    match info {
        // It ran out or was stopped from another machine: the store forgets it, and the row offers to share again.
        Err(refused) if sharing::already_gone(&refused) => {
            let mut session = held(&session);
            let doc = doc_of(&session, &file)?;
            // Another window may have shared it anew meanwhile; only the share the server let go of is forgotten.
            if session
                .state
                .shares
                .get(&doc)
                .is_some_and(|now| now.link == shared.link)
            {
                session
                    .commit(Op::ShareStop {
                        id: doc,
                        link: shared.link,
                    })
                    .map_err(Refusal::from)?;
            }
            Ok(None)
        }
        info => Ok(Some(card(&shared, info.ok()))),
    }
}

/// Changes a share already up, and hands back its row as it stands after.
#[tauri::command]
pub async fn share_change(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    change: sharing::Change,
) -> Answer<Card> {
    sharing::switched(sharing::ON)?;
    let (doc, shared) = {
        let session = held(&session);
        let doc = doc_of(&session, &file)?;
        let shared = session.state.shares.get(&doc).cloned();
        (doc, shared.ok_or(Refusal::of("shareOpenGone"))?)
    };
    // Only whoever shared it holds the token that changes it.
    if shared.taken {
        return Err(Refusal::of("notAllowed"));
    }
    // A reader reaches it through the share that holds it, which already asked for any password.
    if shared.held && matches!(change, sharing::Change::Password { .. }) {
        return Err(Refusal::of("shareHeld"));
    }
    let asked = shared.clone();
    let wanted = change.clone();
    let info = apart(move || {
        let wire = Net::to(&asked.server)?;
        sharing::changed(&wire, &asked, &wanted)?;
        Ok(sharing::info(&wire, &asked).ok())
    })
    .await?;
    let Change::Password { with } = change else {
        return Ok(card(&shared, info));
    };
    // Whether it asks for a password travels with the links, so every machine's row says the same.
    let now = Shared {
        gated: with.is_some(),
        ..shared
    };
    let mut session = held(&session);
    if session
        .state
        .shares
        .get(&doc)
        .is_some_and(|one| one.link == now.link)
    {
        session
            .commit(Op::ShareDoc {
                id: doc,
                d: now.clone(),
            })
            .map_err(Refusal::from)?;
    }
    Ok(card(&now, info))
}

#[tauri::command]
pub async fn share_stop(session: tauri::State<'_, Mutex<Session>>, file: String) -> Answer<()> {
    sharing::switched(sharing::ON)?;
    let (doc, shared) = {
        let session = held(&session);
        let doc = doc_of(&session, &file)?;
        (doc, session.state.shares.get(&doc).cloned())
    };
    let Some(shared) = shared else {
        return Ok(());
    };
    let link = shared.link.clone();
    let shared_was = shared.clone();
    // Someone else's share stays up for them; this store only stops editing it.
    let kept = match shared.taken {
        true => Ok(Some(false)),
        false => {
            apart(move || {
                let wire = Net::to(&shared.server)?;
                sharing::stop(&wire, &shared)?;
                // Still held by a share that cites it, the server keeps it for that one; unasked, this store's word stands.
                Ok(match sharing::info(&wire, &shared) {
                    Ok(_) => Some(true),
                    Err(refused) if sharing::already_gone(&refused) => Some(false),
                    Err(_) => None,
                })
            })
            .await
        }
    };
    // A share the server already let go of is stopped all the same.
    let kept = match kept {
        Err(refused) if !sharing::already_gone(&refused) => return Err(refused),
        Err(_) => Some(false),
        Ok(kept) => kept,
    };
    let mut session = held(&session);
    let kept = kept.unwrap_or_else(|| citing::held_here(&session, doc));
    // The server kept it for a share that cites it: it stays here as going only through that one.
    if kept {
        let held_only = Shared {
            held: true,
            ..shared_was
        };
        return session
            .commit(Op::ShareDoc {
                id: doc,
                d: held_only,
            })
            .map_err(Refusal::from);
    }
    // What went only through it went with it on the server, so this store forgets those too.
    let mut ops = citing::let_go_of(&session, doc, &shared_was, &[]);
    ops.push(Op::ShareStop { id: doc, link });
    session.commit_all(ops).map_err(Refusal::from)
}

/// Keeps a share as it was shared, or lets it follow what is saved again, putting the words up at once when it does.
#[tauri::command]
pub async fn share_still(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    still: bool,
) -> Answer<Card> {
    sharing::switched(sharing::ON)?;
    let now = {
        let mut session = held(&session);
        let doc = doc_of(&session, &file)?;
        let shared = session.state.shares.get(&doc).cloned();
        let now = Shared {
            still,
            ..shared.ok_or(Refusal::of("shareOpenGone"))?
        };
        session
            .commit(Op::ShareDoc {
                id: doc,
                d: now.clone(),
            })
            .map_err(Refusal::from)?;
        now
    };
    if !still {
        super::versions::follow(&app, &file);
    }
    let asked = now.clone();
    let info = apart(move || sharing::info(&Net::to(&asked.server)?, &asked)).await;
    Ok(card(&now, info.ok()))
}

/// Set while loose shares are being let go of, which the start and every round ask for.
#[derive(Default)]
pub struct Loosening(std::sync::atomic::AtomicBool);

// Cleared however the run ends, so one that failed halfway never keeps the next from starting.
struct Loosened<'a>(&'a std::sync::atomic::AtomicBool);

impl Drop for Loosened<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

// A replaced link may be the one just sent, so its reader gets a week to open it before an unread one is stopped.
const UNREAD_FOR: jiff::SignedDuration = jiff::SignedDuration::from_hours(7 * 24);

/// Stops in the background every share nothing in Tisty leads to any more, and forgets each one the server let go of.
pub(crate) fn let_loose_go(app: tauri::AppHandle) {
    if !sharing::ON {
        return;
    }
    use tauri::Manager;
    tauri::async_runtime::spawn(async move {
        let loosening = app.state::<Loosening>();
        // One at a time, or two would stop the same shares and both write that they did.
        if loosening.0.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let _running = Loosened(&loosening.0);
        loosened(&app).await;
        super::versions::brought_all(&app).await;
    });
}

async fn loosened(app: &tauri::AppHandle) {
    use tauri::Manager;
    let session = app.state::<Mutex<Session>>();
    let (gone, replaced) = {
        let session = held(&session);
        (session.state.gone_shares(), session.state.replaced.clone())
    };
    if gone.is_empty() && replaced.is_empty() {
        return;
    }
    let now = jiff::Timestamp::now();
    let stopped = apart(move || {
        // Only an unread replaced share goes, and one the server let go of is forgotten rather than asked after again.
        let unread = replaced
            .into_iter()
            .filter(|one| one.share.taken || now.duration_since(one.at) >= UNREAD_FOR)
            .filter(|one| {
                // Someone else's link is theirs to stop: this store only forgets it.
                if one.share.taken {
                    return true;
                }
                match Net::to(&one.share.server).and_then(|wire| sharing::info(&wire, &one.share)) {
                    Ok(info) => info.reads == 0,
                    Err(refused) => sharing::already_gone(&refused),
                }
            })
            .map(|one| (one.id, one.share));
        Ok(gone
            .into_iter()
            .chain(unread)
            .filter(|(_, shared)| {
                if shared.taken {
                    return true;
                }
                match Net::to(&shared.server).and_then(|wire| sharing::stop(&wire, shared)) {
                    Ok(()) => true,
                    // Closed by reports, the server never lets it be stopped, and asking again only costs.
                    Err(refused) => {
                        sharing::already_gone(&refused)
                            || refused.name.as_deref() == Some("blocked")
                    }
                }
            })
            .map(|(id, shared)| Op::ShareStop {
                id,
                link: shared.link,
            })
            .collect::<Vec<_>>())
    })
    .await;
    if let Ok(ops) = stopped
        && !ops.is_empty()
        && let Err(e) = held(&session).commit_all(ops)
    {
        tisty_core::witness::warn(
            tisty_core::witness::channel::WINDOW,
            "shares nothing leads to were stopped but could not be forgotten",
            &[("why", tisty_core::witness::Fact::Why(e.to_string()))],
        );
    }
}

#[cfg(test)]
#[path = "shares_test.rs"]
mod tests;
