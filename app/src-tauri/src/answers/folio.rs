use std::collections::BTreeMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::Manager;
use tisty_carrier::folio_client::{
    self, Answered, Asked, BASE, Claim, Door, First, Refused, Stopped, Taken, Verb, Version, Why,
};
use tisty_carrier::folio_limits::{self, FolioLimits, SharingNow};
use tisty_carrier::folio_pace;
use tisty_carrier::folio_terms::{self, Standing, Tongue, Unkept, Unsure};
use tisty_carrier::folio_today;
use tisty_carrier::policy;
use tisty_core::Paths;
use tisty_envelope::family::Going;
use tisty_share_proto::identity::{REVOCATIONS_AT_ONCE, Revocation};
use tisty_sharing::{Carrying, Desk, Received, Remembered, Told, Under, Unshared};

use super::folio_receiving::{guest_page, guests_kept, kept_here, opened_from};
use super::papers::other_version_kept;
use super::sharing::{doc_id, refused, sealed_by, the_maker_among, the_maker_here, turned};
use crate::{Answer, Refusal, Session, held};

const TAKEN_BACK_TOLD: &str = "folio-taken-back-told";
const PATIENCE: Duration = Duration::from_secs(120);
const TERMS_TO_ACCEPT: &str = "folioTermsToAccept";
const ANSWER_AT_MOST: u64 = 64 * 1024 * 1024;
const FIRST_TERM_DAYS: u64 = 1;

pub(super) struct ToFolio {
    client: reqwest::blocking::Client,
}

impl ToFolio {
    pub(super) fn new() -> Answer<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(PATIENCE)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| Refusal::of("folioUnreached"))?;
        Ok(Self { client })
    }
}

impl Door for ToFolio {
    fn ask(&self, asked: &Asked<'_>) -> Option<Answered> {
        let url = format!("{BASE}{}", asked.path);
        let mut request = match asked.verb {
            Verb::Get => self.client.get(url),
            Verb::Post => self.client.post(url),
            Verb::Put => self.client.put(url),
            Verb::Delete => self.client.delete(url),
        };
        for (name, value) in &asked.headers {
            request = request.header(*name, value);
        }
        if !asked.body.is_empty() {
            let kind = match asked.verb {
                Verb::Post => "application/json",
                _ => "application/octet-stream",
            };
            request = request
                .header("Content-Type", kind)
                .body(asked.body.to_vec());
        }
        let answer = request.send().ok()?;
        let status = answer.status().as_u16();
        let headers = answer
            .headers()
            .iter()
            .filter_map(|(name, value)| Some((name.to_string(), value.to_str().ok()?.to_owned())))
            .collect();
        let mut body = Vec::new();
        answer.take(ANSWER_AT_MOST).read_to_end(&mut body).ok()?;
        Some(Answered {
            status,
            headers,
            body,
        })
    }
}

pub(super) fn said(stopped: &Stopped) -> Refusal {
    match stopped {
        Stopped::Unreached => Refusal::of("folioUnreached"),
        Stopped::Garbled => Refusal::of("folioGarbled"),
        Stopped::Refused(refused) => match refused {
            Refused::SharesFull => Refusal::of("folioFull"),
            Refused::RoomFull {
                held: Some(held),
                arriving: Some(arriving),
                at_most: Some(most),
            } => Refusal::about("folioRoomFullOf", format!("{held} {arriving} {most}")),
            Refused::RoomFull { .. } => Refusal::of("folioRoomFull"),
            Refused::LanguagesFull => Refusal::of("folioLanguagesFull"),
            Refused::TooBig => Refusal::of("folioTooBig"),
            Refused::Slow { until } => Refusal::about("folioWait", until.to_string()),
            Refused::Quota { until } => Refusal::about("folioQuota", until.to_string()),
            Refused::SharesToday { .. } => Refusal::of("folioSharesToday"),
            Refused::UploadsOff => Refusal::of("folioUploadsOff"),
            Refused::Terms { version } => version.clone().map_or_else(
                || Refusal::of(TERMS_TO_ACCEPT),
                |one| Refusal::about(TERMS_TO_ACCEPT, one),
            ),
            Refused::KeyBlocked => Refusal::of("folioKeyBlocked"),
            Refused::Paused => Refusal::of("folioPaused"),
            Refused::Blocked => Refusal::of("folioBlocked"),
            Refused::Behind => Refusal::of("folioBehind"),
            Refused::Outdated | Refused::Unknown | Refused::Envelope { why: Why::Unknown } => {
                Refusal::of("folioOutdated")
            }
            Refused::Revoked => Refusal::of("folioRevoked"),
            Refused::NotBlocked | Refused::Appealed => Refusal::of("folioAppealed"),
            Refused::Missing => Refusal::of("folioGone"),
            Refused::NotYours => Refusal::of("notOneOwnDoc"),
            Refused::BadUpload
            | Refused::MachinesFull
            | Refused::Envelope { .. }
            | Refused::Proof { .. }
            | Refused::Failed => Refusal::of("folioRefused"),
        },
    }
}

fn tongue_of(session: &Session) -> Tongue {
    if crate::refusing::spanish(session.locale.as_deref()) {
        Tongue::Es
    } else {
        Tongue::En
    }
}

async fn apart<T: Send + 'static>(work: impl FnOnce() -> Answer<T> + Send + 'static) -> Answer<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|_| Refusal::of("internal"))?
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum TermsNow {
    Accepted,
    StillAccepted,
    #[serde(rename_all = "camelCase")]
    ToAccept {
        version: String,
        text: String,
        accepted_before: Option<String>,
    },
}

fn terms_now(home: &std::path::Path, tongue: Tongue) -> Answer<TermsNow> {
    match folio_terms::standing(home, tongue, crate::reaching::brought_from_folio) {
        Ok(Standing::Accepted) => Ok(TermsNow::Accepted),
        Ok(Standing::StillAccepted) => Ok(TermsNow::StillAccepted),
        Ok(Standing::ToAccept {
            version,
            text,
            accepted_before,
        }) => Ok(TermsNow::ToAccept {
            version,
            text,
            accepted_before,
        }),
        Err(unsure) => Err(unsure_said(unsure)),
    }
}

struct Agreed {
    version: String,
    newer_unchecked: bool,
}

// Asked before anything leaves: terms that changed stop an upload, never a link already out.
fn may_upload(paths: &Paths, tongue: Tongue) -> Answer<Agreed> {
    let home = paths.private();
    let newer_unchecked = match terms_now(&home, tongue)? {
        TermsNow::Accepted => false,
        TermsNow::StillAccepted => true,
        TermsNow::ToAccept { .. } => return Err(Refusal::of(TERMS_TO_ACCEPT)),
    };
    let limits = folio_limits::kept(&home);
    if folio_limits::uploads_halted(limits.as_ref(), policy::now()) {
        return Err(Refusal::of("folioUploadsOff"));
    }
    let version =
        folio_terms::accepted_version(&home).ok_or_else(|| Refusal::of(TERMS_TO_ACCEPT))?;
    Ok(Agreed {
        version,
        newer_unchecked,
    })
}

// Nothing goes on a Sync with no changes, yet newer terms are still asked for as soon as they check out.
fn new_terms_asked(agreed: &Answer<Agreed>) -> bool {
    matches!(agreed, Err(refused) if refused.code == TERMS_TO_ACCEPT)
}

fn unsure_said(unsure: Unsure) -> Refusal {
    match unsure {
        Unsure::NoWord => Refusal::of("folioUnreached"),
        Unsure::NotPublished => Refusal::of("folioTermsNotPublished"),
        Unsure::Settling => Refusal::of("folioTermsUpdating"),
        Unsure::Unmatched => Refusal::of("folioTermsUnmatched"),
    }
}

fn after_terms_refused(standing: Result<Standing, Unsure>) -> Refusal {
    match standing {
        Ok(Standing::ToAccept { .. }) => Refusal::of(TERMS_TO_ACCEPT),
        Ok(Standing::Accepted | Standing::StillAccepted) => Refusal::of("folioTermsUnmatched"),
        Err(unsure) => unsure_said(unsure),
    }
}

// The waits pace what reaches Folio, so a press stopped before that leaves them as they were.
fn press_given_back(refused: Refusal, home: &std::path::Path, id: &str, at: u64) -> Refusal {
    if matches!(
        refused.code,
        TERMS_TO_ACCEPT
            | "folioTermsUpdating"
            | "folioTermsUnmatched"
            | "folioUnreached"
            | "folioUploadsOff"
    ) {
        folio_pace::take_back(home, id, at);
    }
    refused
}

// Folio holds both against the owner's key without saying whose, and an editor signs with their own.
fn said_to_whom(refused: Refusal, owning: bool, home: &std::path::Path) -> Refusal {
    let limits = || folio_limits::kept(home);
    match (refused.code, owning) {
        ("folioKeyBlocked", false) => Refusal::of("folioOwnerKeyBlocked"),
        ("folioRoomFullOf", false) => Refusal {
            code: "folioOwnerRoomFullOf",
            ..refused
        },
        ("folioRoomFull", true) => {
            numbered("folioRoomFull", "folioRoomFullAll", key_room(limits()))
        }
        ("folioRoomFull", false) => numbered(
            "folioOwnerRoomFull",
            "folioOwnerRoomFullAll",
            key_room(limits()),
        ),
        ("folioFull", _) => numbered(
            "folioFullOf",
            "folioFull",
            limits().and_then(|one| one.shares_at_most),
        ),
        ("folioLanguagesFull", _) => numbered(
            "folioLanguagesFullOf",
            "folioLanguagesFull",
            limits().and_then(|one| one.languages_at_most),
        ),
        _ => refused,
    }
}

fn key_room(limits: Option<folio_limits::FolioLimits>) -> Option<u64> {
    limits.and_then(|one| one.key_bytes_at_most)
}

// Folio's own figure when its signed limits name one, and words that need none when they do not.
fn numbered(with: &'static str, without: &'static str, most: Option<u64>) -> Refusal {
    most.map_or_else(
        || Refusal::of(without),
        |one| Refusal::about(with, one.to_string()),
    )
}

fn delivered_or_asked_again(refused: Refusal, home: &std::path::Path, tongue: Tongue) -> Refusal {
    if refused.code != TERMS_TO_ACCEPT {
        return refused;
    }
    after_terms_refused(folio_terms::read_again(
        home,
        tongue,
        crate::reaching::brought_from_folio,
        refused.name.as_deref(),
    ))
}

const OTHER_LANGUAGE_ENDS: [&str; 2] = ["](TERMS.md)", "](TERMINOS.md)"];

fn names_the_other_language(line: &str) -> bool {
    let line = line.trim();
    line.starts_with('[') && OTHER_LANGUAGE_ENDS.iter().any(|end| line.ends_with(end))
}

fn may_start(now: SharingNow) -> Answer<()> {
    match now {
        SharingNow::Open => Ok(()),
        SharingNow::Closed { .. } => Err(Refusal::of("folioClosed")),
        SharingNow::Unreached => Err(Refusal::of("folioUnreached")),
    }
}

async fn sharing_here(session: &tauri::State<'_, Mutex<Session>>) -> Answer<SharingNow> {
    let (paths, persons, device) = {
        let session = held(session);
        (
            session.paths.clone(),
            session.state.persons.clone(),
            session.config.device_id.clone(),
        )
    };
    apart(move || {
        let now = folio_limits::sharing_now(&paths.private(), crate::reaching::brought_from_folio);
        // Asked only once the limits are fresh, since they are what names the maker.
        Ok(now.opened_to(the_maker_among(&paths, &persons, &device).is_some()))
    })
    .await
}

#[tauri::command]
pub async fn folio_sharing_now(session: tauri::State<'_, Mutex<Session>>) -> Answer<SharingNow> {
    sharing_here(&session).await
}

#[tauri::command]
pub async fn folio_terms_standing(session: tauri::State<'_, Mutex<Session>>) -> Answer<TermsNow> {
    let (home, tongue) = {
        let session = held(&session);
        (session.paths.private(), tongue_of(&session))
    };
    apart(move || terms_now(&home, tongue)).await
}

// Folio writes them for any Markdown reader, cut at a column, and Tisty's own reading would keep each cut as a break.
#[tauri::command(async)]
pub fn folio_terms_drawn(text: String) -> String {
    let shown: Vec<&str> = text
        .lines()
        .filter(|line| !names_the_other_language(line))
        .collect();
    tisty_markdown::html(&tisty_markdown::read_file(&shown.join("\n")))
}

#[tauri::command]
pub fn folio_terms_accept(
    session: tauri::State<'_, Mutex<Session>>,
    version: String,
) -> Answer<()> {
    let session = held(&session);
    folio_terms::accept(&session.paths.private(), tongue_of(&session), &version).map_err(|why| {
        match why {
            Unkept::Moved => Refusal::of("folioTermsToAccept"),
            Unkept::Unwritten => Refusal::of("folioGarbled"),
        }
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Secret {
    named: String,
    under: Option<String>,
}

fn secrets_in(text: &str) -> Vec<Secret> {
    let over = headings_over(text);
    tisty_core::agent::secrets_in(text.as_bytes())
        .into_iter()
        .map(|one| Secret {
            named: one.named,
            under: over.get(one.line.saturating_sub(1)).cloned().flatten(),
        })
        .collect()
}

fn secrets_of_docs(
    paths: &Paths,
    state: &tisty_core::State,
    docs: &[tisty_core::model::DocId],
) -> Vec<Secret> {
    let root = paths.docs();
    let mut all = Vec::new();
    for doc in docs {
        let Some(kept) = state.docs.get(doc) else {
            continue;
        };
        let group = state.tongues_of(*doc);
        let members = if group.is_empty() { vec![kept] } else { group };
        for one in members {
            let Ok(body) = tisty_core::docs::read(&root, &one.file) else {
                continue;
            };
            all.extend(secrets_in(&body));
            for page in state.pages_read(one.id, &body) {
                if let Ok(words) = tisty_core::docs::read(&root, &page.file) {
                    all.extend(secrets_in(&words));
                }
            }
        }
    }
    all
}

fn headings_over(text: &str) -> Vec<Option<String>> {
    let mut over = Vec::new();
    let mut heading = None;
    let mut fenced = false;
    let mut in_properties = text.starts_with("---");
    for (at, one) in text.lines().enumerate() {
        over.push(heading.clone());
        let trimmed = one.trim_start();
        if in_properties {
            in_properties = at == 0 || trimmed.trim_end() != "---";
            continue;
        }
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            fenced = !fenced;
        } else if !fenced && trimmed.starts_with('#') {
            let words = trimmed.trim_start_matches('#').trim();
            if !words.is_empty() {
                heading = Some(words.to_owned());
            }
        }
    }
    over
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Look {
    secrets: Vec<Secret>,
    cited_secrets: Vec<Secret>,
    name: Option<String>,
    alias: Option<String>,
    carrying: Carrying,
    cited: Vec<String>,
    shared: Vec<SharedElsewhere>,
    limits: FolioLimits,
    told: Vec<Told>,
    seen: Option<String>,
    signs: bool,
    new_shares_today: u64,
    held_there: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedElsewhere {
    doc: String,
    title: String,
    #[serde(skip)]
    file: String,
    #[serde(skip)]
    was: Remembered,
}

// What is edited through another's link counts for whoever shared it, and what is held for what holds it (93 bis).
fn counted_as_shared(was: &Remembered) -> bool {
    was.held_by.is_none() && was.admin_token.is_some()
}

fn shared_elsewhere(
    session: &Session,
    shelf: &BTreeMap<String, Remembered>,
) -> Vec<SharedElsewhere> {
    session
        .state
        .docs
        .values()
        .filter(|one| one.page_of.is_none())
        .filter_map(|one| {
            let was = shelf.get(&one.file).filter(|was| counted_as_shared(was))?;
            Some(SharedElsewhere {
                doc: one.id.to_string(),
                title: one.title.clone().unwrap_or_default(),
                file: one.file.clone(),
                was: was.clone(),
            })
        })
        .collect()
}

// Only when the count would stop the person: a term that ran out frees its place without telling anyone.
fn still_shared(app: &tauri::AppHandle, shared: Vec<SharedElsewhere>) -> Vec<SharedElsewhere> {
    let Ok(door) = ToFolio::new() else {
        return shared;
    };
    shared
        .into_iter()
        .filter(|one| {
            let Some(admin) = &one.was.admin_token else {
                return true;
            };
            let gone = folio_client::status(&door, &one.was.id, admin)
                == Err(Stopped::Refused(Refused::Missing));
            if gone {
                forget_share(app, &one.file);
            }
            !gone
        })
        .collect()
}

fn forget_share(app: &tauri::AppHandle, file: &str) {
    let session = app.state::<Mutex<Session>>();
    let mut session = held(&session);
    let shelf = tisty_sharing::remembered(&session.state);
    let Some(was) = shelf.get(file).cloned() else {
        return;
    };
    let mut gone = going_with(shelf, file, &was.id, |at| {
        doc_of_file(&session, at).map(|one| one.to_string())
    });
    gone.push((file.to_owned(), was));
    let _ = session.commit_all(tisty_sharing::forgotten(&gone));
}

fn told_of(app: &tauri::AppHandle, file: &str, stopped: &Stopped) -> Refusal {
    if *stopped == Stopped::Refused(Refused::Missing) {
        forget_share(app, file);
    }
    said(stopped)
}

// Decisions 48 and 51: deleting a document stops its share, here or on whichever computer sees it next.
pub fn stops_what_lost_its_document(app: &tauri::AppHandle) {
    let lost: Vec<(String, Remembered)> = {
        let session = app.state::<Mutex<Session>>();
        let session = held(&session);
        tisty_sharing::remembered(&session.state)
            .iter()
            .filter(|(file, _)| doc_of_file(&session, file).is_none())
            .map(|(file, was)| (file.clone(), was.clone()))
            .collect()
    };
    if lost.is_empty() {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Ok(door) = ToFolio::new() else {
            return;
        };
        for (file, was) in lost {
            let stopped = was
                .admin_token
                .as_deref()
                .is_none_or(|admin| folio_client::stop(&door, &was.id, admin).is_ok());
            if stopped {
                forget_share(&app, &file);
            }
        }
    });
}

#[tauri::command]
pub async fn doc_share_look(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Look> {
    stops_what_lost_its_document(&app);
    let doc = doc_id(&id)?;
    crate::identity::seated_to_sign(&session).await;
    let (paths, state, device, also, name, alias, shared) = {
        let session = held(&session);
        (
            session.paths.clone(),
            session.state.clone(),
            session.config.device_id.clone(),
            session.dest(),
            session.state.signed.name.clone(),
            session.state.signed.alias.clone(),
            shared_elsewhere(&session, tisty_sharing::remembered(&session.state)),
        )
    };
    apart(move || {
        let kept = state
            .docs
            .get(&doc)
            .ok_or_else(|| Refusal::of("noSuchDoc"))?;
        let home = paths.private();
        // Heeded first: the limits are what name the maker the desk is set for.
        let limits = match folio_limits::heed(&home, crate::reaching::brought_from_folio) {
            Some(Ok(limits)) => Some(limits),
            _ => folio_limits::kept(&home),
        }
        .unwrap_or_default();
        let desk = Desk {
            paths: &paths,
            state: &state,
            also: also.as_deref(),
            turned: &turned,
            maker: the_maker_here(&paths, &state, &device),
        };
        let asked = tisty_sharing::Asked {
            doc,
            public: false,
            cited: Vec::new(),
            signed_as: None,
        };
        let gathered = tisty_sharing::gathered_within(&desk, &asked, languages_top(&home))
            .map_err(|e| refused(&e))?;
        let cited = tisty_sharing::citable(&desk, doc).map_err(|e| refused(&e))?;
        let others = shared.iter().filter(|one| one.doc != id).count() as u64;
        let shared = match limits.shares_at_most {
            Some(most) if others >= most => still_shared(&app, shared),
            _ => shared,
        };
        Ok(Look {
            signs: tisty_sharing::signer(&paths, &state, &device).is_some(),
            secrets: secrets_of_docs(&paths, &state, &[doc]),
            cited_secrets: secrets_of_docs(&paths, &state, &cited),
            seen: gathered.print_of(&kept.file),
            name,
            alias,
            carrying: gathered.carrying(),
            cited: cited.iter().map(ToString::to_string).collect(),
            shared,
            limits,
            told: gathered.told().to_vec(),
            new_shares_today: folio_today::shared_today(&home, policy::now()),
            held_there: std::iter::once(doc)
                .chain(cited.iter().copied())
                .filter(|one| folio_holds(&state, *one))
                .map(|one| one.to_string())
                .collect(),
        })
    })
    .await
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    read: String,
    edit: Option<String>,
    version: u64,
    taken: bool,
    held: bool,
    told: Vec<Told>,
}

fn card_of(was: &Remembered) -> Option<Card> {
    let key = was.link_key.as_deref()?;
    Some(Card {
        read: folio_client::read_link(&was.id, key),
        edit: was
            .edit_token
            .as_deref()
            .map(|token| folio_client::edit_link(&was.id, key, token)),
        version: was.version,
        taken: was.admin_token.is_none(),
        held: was.held_by.is_some(),
        told: Vec::new(),
    })
}

fn file_of(session: &Session, id: &str) -> Answer<String> {
    let doc = doc_id(id)?;
    let kept = session
        .state
        .docs
        .get(&doc)
        .ok_or_else(|| Refusal::of("noSuchDoc"))?;
    Ok(kept.file.clone())
}

#[tauri::command]
pub fn doc_share_card(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Card>> {
    let session = held(&session);
    let file = file_of(&session, &id)?;
    Ok(tisty_sharing::remembered(&session.state)
        .get(&file)
        .and_then(card_of))
}

type Keep<'a> = &'a dyn Fn(&[(String, Remembered)]) -> Answer<()>;
type Starts = Arc<Mutex<BTreeMap<String, String>>>;
type Trouble = Arc<Mutex<Option<Stopped>>>;

fn files_still_owed((file, was): &(String, Remembered)) -> (String, Remembered) {
    let mut owed = was.clone();
    owed.files_hash = None;
    owed.files_key = None;
    owed.words_print = None;
    (file.clone(), owed)
}

fn kept_on(session: &mut Session, went: &[(String, Remembered)]) -> Answer<()> {
    session.commit_all(tisty_sharing::sent(went))?;
    Ok(())
}

fn keeping(app: &tauri::AppHandle) -> impl Fn(&[(String, Remembered)]) -> Answer<()> + use<> {
    let app = app.clone();
    move |went| {
        let session = app.state::<Mutex<Session>>();
        kept_on(&mut held(&session), went)
    }
}

fn delivered(
    door: &dyn Door,
    home: &std::path::Path,
    keep: Keep<'_>,
    sealed: &tisty_sharing::Sealed,
    starts: &BTreeMap<String, String>,
    ttl_days: u64,
) -> Answer<()> {
    for going in cited_first(sealed) {
        let Some(entry) = sealed
            .remembering
            .iter()
            .find(|(_, was)| was.id == going.id)
        else {
            continue;
        };
        let admin = entry.1.admin_token.as_deref().unwrap_or_default();
        let first = starts.get(&going.id).map(|mac| First {
            mac,
            admin,
            ttl_days,
            anchor: sealed.anchors.get(&going.id).map(String::as_str),
        });
        let stored = folio_client::send_version(
            door,
            &Version {
                id: &going.id,
                envelope: &going.sealed.main,
                files_hash: going.sealed.files_hash.as_deref(),
                first,
            },
        )
        .map_err(|stopped| said(&stopped))?;
        if first_of_its_own(starts, sealed, &going.id) {
            folio_today::counted(home, &going.id, policy::now());
        }
        if stored.files_wanted {
            // Folio already holds this version, so it is remembered now; only its files are still owed.
            keep(&[files_still_owed(entry)])?;
            let files = going
                .sealed
                .files
                .as_deref()
                .ok_or_else(|| Refusal::of("folioGarbled"))?;
            folio_client::send_files(door, &going.id, files).map_err(|stopped| said(&stopped))?;
        }
        keep(std::slice::from_ref(entry))?;
    }
    Ok(())
}

// A refusal on the way then never leaves a document that cites what Folio does not hold.
fn cited_first(sealed: &tisty_sharing::Sealed) -> Vec<&Going> {
    let top = sealed
        .remembering
        .iter()
        .find(|(_, one)| one.held_by.is_none())
        .map(|(_, one)| one.id.as_str());
    let of_top = |id: &str| Some(id) == top || sealed.anchors.get(id).map(String::as_str) == top;
    let (own, cited): (Vec<&Going>, Vec<&Going>) =
        sealed.going.iter().partition(|one| of_top(&one.id));
    cited.into_iter().chain(own).collect()
}

// Folio counts each new document a day, and a language that goes with its anchor is not one.
fn first_of_its_own(
    starts: &BTreeMap<String, String>,
    sealed: &tisty_sharing::Sealed,
    id: &str,
) -> bool {
    starts.contains_key(id) && !sealed.anchors.contains_key(id)
}

// Sent again on each sync (81): the server keeps them only while it holds something of that root, and a late one still counts.
fn told_of_taken_back(
    door: &dyn Door,
    home: &std::path::Path,
    state: &tisty_core::State,
    device: &tisty_core::DeviceId,
) -> Option<Told> {
    let root = state.persons.person(device)?;
    let taken = newest_first(
        state
            .persons
            .revocations
            .iter()
            .filter(|one| one.unchecked().is_some_and(|(person, _)| person == root)),
    );
    let no_room = sent_taken_back(door, &taken);
    (no_room && first_time_without_room(home, taken.len())).then_some(Told::TakenBackNotKept)
}

// Folio counts all of them against its hour, so the one that matters most cannot wait behind the old ones.
fn newest_first<'a>(revocations: impl Iterator<Item = &'a Revocation>) -> Vec<Taken> {
    let mut mine: Vec<&Revocation> = revocations.collect();
    mine.sort_by_key(|one| std::cmp::Reverse(one.at));
    mine.into_iter()
        .map(|one| Taken {
            root: one.root.clone(),
            machine: one.share.clone(),
            at: one.at.to_string(),
            signed: one.sig.clone(),
        })
        .collect()
}

// Whether Folio had no room: any other failure goes again on the next sync, and past its hour the rest would only be refused too.
fn sent_taken_back(door: &dyn Door, taken: &[Taken]) -> bool {
    let mut no_room = false;
    for some in taken.chunks(REVOCATIONS_AT_ONCE) {
        match folio_client::taken_back(door, some) {
            Err(Stopped::Refused(Refused::MachinesFull)) => no_room = true,
            Err(Stopped::Refused(Refused::Quota { .. } | Refused::Slow { .. })) => break,
            _ => {}
        }
    }
    no_room
}

// Said once for each key that finds no room, or it would turn into a line nobody reads.
fn first_time_without_room(home: &std::path::Path, many: usize) -> bool {
    let kept = home.join(TAKEN_BACK_TOLD);
    let told = std::fs::read_to_string(&kept)
        .ok()
        .and_then(|said| said.trim().parse::<usize>().ok())
        .unwrap_or(0);
    if many <= told {
        return false;
    }
    let _ = std::fs::create_dir_all(home);
    let _ = std::fs::write(&kept, many.to_string());
    true
}

fn folio_holds(state: &tisty_core::State, doc: tisty_core::model::DocId) -> bool {
    let shelf = tisty_sharing::remembered(state);
    state.docs.get(&doc).is_some_and(|kept| {
        let anchor = state.docs.get(&state.anchor_of(kept));
        shelf.contains_key(&kept.file) || anchor.is_some_and(|one| shelf.contains_key(&one.file))
    })
}

fn languages_top(home: &std::path::Path) -> Option<usize> {
    folio_limits::kept(home)
        .and_then(|limits| limits.languages_at_most)
        .and_then(|top| usize::try_from(top).ok())
}

async fn sealed_for_folio(
    app: &tauri::AppHandle,
    session: &tauri::State<'_, Mutex<Session>>,
    asked: tisty_sharing::Asked,
    editable: bool,
    terms: String,
) -> Answer<(tisty_sharing::Sealed, BTreeMap<String, String>)> {
    let languages_at_most = languages_top(&held(session).paths.private());
    let starts: Starts = Arc::default();
    let trouble: Trouble = Arc::default();
    let (given, stopped) = (Arc::clone(&starts), Arc::clone(&trouble));
    let sealed = sealed_by(app, session, asked, move |desk, asked| {
        let door = ToFolio::new().map_err(|_| Unshared::Unread("no way to Folio".to_owned()))?;
        let mint = || match folio_client::start(&door) {
            Ok(start) => {
                if let Ok(mut held) = given.lock() {
                    held.insert(start.id.clone(), start.mac);
                }
                Ok(start.id)
            }
            Err(why) => {
                if let Ok(mut held) = stopped.lock() {
                    *held = Some(why);
                }
                Err(Unshared::Unread("Folio gave no id".to_owned()))
            }
        };
        tisty_sharing::gathered_under(
            desk,
            asked,
            &Under {
                mint: &mint,
                editable,
                terms: Some(&terms),
                languages_at_most,
            },
        )
    })
    .await;
    if let Some(why) = trouble.lock().ok().and_then(|mut held| held.take()) {
        return Err(said(&why));
    }
    let starts = starts.lock().map(|held| held.clone()).unwrap_or_default();
    Ok((sealed?, starts))
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chosen {
    cited: Vec<String>,
    signed_as: Option<String>,
    editable: bool,
    ttl_days: u64,
    seen: Option<String>,
}

#[tauri::command]
pub async fn doc_shared(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    chosen: Chosen,
) -> Answer<Card> {
    let Chosen {
        cited,
        signed_as,
        editable,
        ttl_days,
        seen,
    } = chosen;
    let doc = doc_id(&id)?;
    let (paths, tongue, file) = {
        let session = held(&session);
        (
            session.paths.clone(),
            tongue_of(&session),
            file_of(&session, &id)?,
        )
    };
    may_start(sharing_here(&session).await?)?;
    let checking = paths.clone();
    let terms = apart(move || may_upload(&checking, tongue).map(|agreed| agreed.version)).await?;
    let asked = tisty_sharing::Asked {
        doc,
        public: false,
        cited: cited
            .iter()
            .map(|one| doc_id(one))
            .collect::<Answer<Vec<_>>>()?,
        signed_as,
    };
    let (mut sealed, starts) = sealed_for_folio(&app, &session, asked, editable, terms).await?;
    let print = sealed.remembering.iter().find(|(at, _)| *at == file);
    if seen.is_some() && seen != print.and_then(|(_, one)| one.words_print.clone()) {
        return Err(Refusal::of("folioChanged"));
    }
    for (_, one) in &mut sealed.remembering {
        one.ttl_days = Some(ttl_days);
        if one.held_by.is_none() {
            one.cited.clone_from(&cited);
        }
    }
    let told = sealed.told.clone();
    let (state, device) = {
        let session = held(&session);
        (session.state.clone(), session.config.device_id.clone())
    };
    let keep = keeping(&app);
    let home = paths.private();
    apart(move || {
        let door = ToFolio::new()?;
        delivered(&door, &home, &keep, &sealed, &starts, ttl_days)
            .map_err(|refused| said_to_whom(refused, true, &home))
            .map_err(|refused| delivered_or_asked_again(refused, &home, tongue))?;
        let not_kept = told_of_taken_back(&door, &home, &state, &device);
        let told = told.into_iter().chain(not_kept).collect();
        let top = sealed
            .remembering
            .iter()
            .find(|(_, one)| one.held_by.is_none());
        top.and_then(|(_, one)| card_of(one))
            .map(|one| Card { told, ..one })
            .ok_or_else(|| Refusal::of("folioGarbled"))
    })
    .await
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Synced {
    #[serde(rename_all = "camelCase")]
    Unchanged {
        told: Vec<Told>,
        terms_unchecked: bool,
    },
    Secrets {
        place: String,
        print: String,
    },
    #[serde(rename_all = "camelCase")]
    Sent {
        version: Option<u64>,
        told: Vec<Told>,
        terms_unchecked: bool,
    },
    Gone,
    OtherVersion {
        version: u64,
        signer: String,
        doubtful: bool,
    },
}

fn remembered_of(session: &Session, id: &str) -> Answer<(String, Remembered)> {
    let file = file_of(session, id)?;
    let was = tisty_sharing::remembered(&session.state)
        .get(&file)
        .cloned();
    was.map(|one| (file, one))
        .ok_or_else(|| Refusal::of("folioGone"))
}

struct Here {
    paths: Paths,
    also: Option<std::path::PathBuf>,
    state: tisty_core::State,
    device: tisty_core::DeviceId,
}

impl Here {
    fn of(session: &tauri::State<'_, Mutex<Session>>) -> Self {
        let session = held(session);
        Self {
            paths: session.paths.clone(),
            also: session.dest(),
            state: session.state.clone(),
            device: session.config.device_id.clone(),
        }
    }

    fn gathered(&self, asked: &tisty_sharing::Asked) -> Answer<tisty_sharing::Gathered> {
        let desk = Desk {
            paths: &self.paths,
            state: &self.state,
            also: self.also.as_deref(),
            turned: &turned,
            maker: the_maker_here(&self.paths, &self.state, &self.device),
        };
        tisty_sharing::gathered_within(&desk, asked, languages_top(&self.paths.private()))
            .map_err(|e| refused(&e))
    }
}

struct Looked {
    unchanged: bool,
    secret: Option<String>,
    print: String,
}

async fn looked_over(
    session: &tauri::State<'_, Mutex<Session>>,
    asked: &tisty_sharing::Asked,
) -> Answer<Looked> {
    let here = Here::of(session);
    let looking = asked.clone();
    apart(move || {
        let gathered = here.gathered(&looking)?;
        let mut docs = vec![looking.doc];
        docs.extend(looking.cited.iter().copied());
        let secret = secrets_of_docs(&here.paths, &here.state, &docs)
            .into_iter()
            .next()
            .map(|one| one.under.unwrap_or(one.named));
        Ok(Looked {
            unchanged: gathered.same_as_sent(),
            secret,
            print: gathered.whole_print(),
        })
    })
    .await
}

async fn unchanged_here(
    session: &tauri::State<'_, Mutex<Session>>,
    asked: tisty_sharing::Asked,
) -> Answer<bool> {
    let here = Here::of(session);
    apart(move || Ok(here.gathered(&asked)?.same_as_sent())).await
}

fn asked_again(doc: tisty_core::model::DocId, was: &Remembered) -> Answer<tisty_sharing::Asked> {
    Ok(tisty_sharing::Asked {
        doc,
        public: false,
        cited: was
            .cited
            .iter()
            .map(|one| doc_id(one))
            .collect::<Answer<Vec<_>>>()?,
        signed_as: was.signed_as.clone(),
    })
}

#[derive(Debug, PartialEq, Eq)]
enum Ahead {
    Unchanged,
    Gone,
    Newer,
    Upload,
}

fn waited(free: u64) -> Refusal {
    Refusal::about("folioWait", (free * 1000).to_string())
}

// The sync contract asks on every press, so another's newer version is brought apart (149); only an upload counts for the waits.
fn before_upload(
    door: &dyn Door,
    home: &std::path::Path,
    was: &Remembered,
    unchanged: bool,
    limits: Option<&FolioLimits>,
    now: u64,
) -> Answer<Ahead> {
    if !unchanged {
        folio_pace::press(home, &was.id, now, limits).map_err(waited)?;
    }
    let ready = if unchanged {
        Ahead::Unchanged
    } else {
        Ahead::Upload
    };
    match folio_client::latest(door, &was.id) {
        Ok(latest) if latest.version > was.version => Ok(Ahead::Newer),
        Ok(_) => Ok(ready),
        Err(Stopped::Refused(Refused::Missing)) => Ok(Ahead::Gone),
        Err(Stopped::Refused(Refused::Blocked)) if correctable(door, was) => Ok(ready),
        Err(stopped) if unchanged => Err(said(&stopped)),
        Err(stopped) => Err(press_given_back(said(&stopped), home, &was.id, now)),
    }
}

// An appeal accepted leaves the document blocked until its owner sends a version, which Folio's latest will not name.
fn correctable(door: &dyn Door, was: &Remembered) -> bool {
    was.admin_token.as_deref().is_some_and(|admin| {
        folio_client::status(door, &was.id, admin).is_ok_and(|told| told.may_correct)
    })
}

async fn unchanged_answered(
    session: &tauri::State<'_, Mutex<Session>>,
    paths: &Paths,
    tongue: Tongue,
) -> Answer<Synced> {
    let checking = paths.clone();
    let agreed = apart(move || may_upload(&checking, tongue)).await;
    if new_terms_asked(&agreed) {
        return Err(Refusal::of(TERMS_TO_ACCEPT));
    }
    let (state, device) = {
        let session = held(session);
        (session.state.clone(), session.config.device_id.clone())
    };
    let home = paths.private();
    let not_kept = apart(move || {
        Ok(ToFolio::new()
            .ok()
            .and_then(|door| told_of_taken_back(&door, &home, &state, &device)))
    })
    .await?;
    Ok(Synced::Unchanged {
        told: not_kept.into_iter().collect(),
        terms_unchecked: agreed.as_ref().is_ok_and(|one| one.newer_unchecked),
    })
}

#[tauri::command]
pub async fn doc_share_sync(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    accepted: Option<String>,
) -> Answer<Synced> {
    let doc = doc_id(&id)?;
    let (paths, tongue, (file, was)) = {
        let session = held(&session);
        (
            session.paths.clone(),
            tongue_of(&session),
            remembered_of(&session, &id)?,
        )
    };
    let asked = asked_again(doc, &was)?;
    let looked = looked_over(&session, &asked).await?;
    // Looked at before a press counts, and an answer holds only for the words it was given about (81).
    if let Some(place) = looked
        .secret
        .filter(|_| !looked.unchanged && accepted.as_ref() != Some(&looked.print))
    {
        return Ok(Synced::Secrets {
            place,
            print: looked.print,
        });
    }
    let home = paths.private();
    let pressed_at = policy::now();
    let ahead = {
        let (home, was, unchanged) = (home.clone(), was.clone(), looked.unchanged);
        apart(move || {
            let limits = folio_limits::kept(&home);
            let door = ToFolio::new()?;
            before_upload(&door, &home, &was, unchanged, limits.as_ref(), pressed_at)
        })
        .await?
    };
    match ahead {
        Ahead::Unchanged => return unchanged_answered(&session, &paths, tongue).await,
        Ahead::Gone => {
            forget_share(&app, &file);
            return Ok(Synced::Gone);
        }
        Ahead::Newer => return other_version_brought(&session, &id, &was).await,
        Ahead::Upload => {}
    }
    let checking = paths.clone();
    let agreed = apart(move || may_upload(&checking, tongue)).await;
    let terms_unchecked = agreed.as_ref().is_ok_and(|one| one.newer_unchecked);
    let terms = agreed
        .map(|one| one.version)
        .map_err(|refused| press_given_back(refused, &home, &was.id, pressed_at))?;
    let (sealed, starts) = sealed_for_folio(&app, &session, asked, false, terms).await?;
    let mut sealed = sealed.only_what_changed();
    if sealed.going.is_empty() {
        folio_pace::take_back(&home, &was.id, pressed_at);
        return unchanged_answered(&session, &paths, tongue).await;
    }
    let ttl_days = was.ttl_days.unwrap_or(FIRST_TERM_DAYS);
    for (_, one) in &mut sealed.remembering {
        one.ttl_days = Some(ttl_days);
    }
    let told = sealed.told.clone();
    let version = sealed
        .remembering
        .iter()
        .find(|(_, one)| one.id == was.id)
        .map(|(_, one)| one.version);
    let (state, device) = {
        let session = held(&session);
        (session.state.clone(), session.config.device_id.clone())
    };
    let keep = keeping(&app);
    let pressed_id = was.id.clone();
    let owning = was.admin_token.is_some();
    let not_kept = apart(move || {
        let door = ToFolio::new()?;
        delivered(&door, &home, &keep, &sealed, &starts, ttl_days)
            .map_err(|refused| said_to_whom(refused, owning, &home))
            .map_err(|refused| delivered_or_asked_again(refused, &home, tongue))
            .map_err(|refused| press_given_back(refused, &home, &pressed_id, pressed_at))?;
        Ok(told_of_taken_back(&door, &home, &state, &device))
    })
    .await?;
    let told = told.into_iter().chain(not_kept).collect();
    Ok(Synced::Sent {
        version,
        told,
        terms_unchecked,
    })
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Waiting {
    until: Option<u64>,
    pending: bool,
}

#[tauri::command]
pub async fn doc_share_waiting(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    look: bool,
) -> Answer<Waiting> {
    let doc = doc_id(&id)?;
    let (home, was) = {
        let session = held(&session);
        (session.paths.private(), remembered_of(&session, &id)?.1)
    };
    let limits = folio_limits::kept(&home);
    let until = folio_pace::waiting_until(&home, &was.id, policy::now(), limits.as_ref());
    let Some(until) = until else {
        return Ok(Waiting {
            until: None,
            pending: false,
        });
    };
    let pending = look && !unchanged_here(&session, asked_again(doc, &was)?).await?;
    Ok(Waiting {
        until: Some(until * 1000),
        pending,
    })
}

fn doc_of_file(session: &Session, file: &str) -> Option<tisty_core::model::DocId> {
    let found = session.state.docs.values().find(|one| one.file == file);
    found.map(|one| one.id)
}

fn pages_differ(session: &Session, file: &str, theirs: &[String]) -> bool {
    let root = session.paths.docs();
    let Some(kept) = session.state.docs.values().find(|one| one.file == file) else {
        return !theirs.is_empty();
    };
    let body = tisty_core::docs::read(&root, file).unwrap_or_default();
    let mine: Vec<String> = session
        .state
        .pages_read(kept.id, &body)
        .iter()
        .filter_map(|page| tisty_core::docs::read(&root, &page.file).ok())
        .collect();
    mine.len() != theirs.len()
        || mine
            .iter()
            .zip(theirs)
            .any(|(one, other)| !tisty_core::docs::unchanged(one, other))
}

// What another sent stays apart, and one's own changes wait until the person chooses and presses again (149).
async fn other_version_brought(
    session: &tauri::State<'_, Mutex<Session>>,
    id: &str,
    was: &Remembered,
) -> Answer<Synced> {
    let (shared_id, key) = (was.id.clone(), was.link_key.clone().unwrap_or_default());
    let opened = apart(move || {
        let door = ToFolio::new()?;
        opened_from(&door, &shared_id, &key)
    })
    .await?;
    // Folio never serves below what it saved, so a lower number is not to be taken for news (85).
    if opened.version <= was.version {
        return Err(Refusal::of("folioRefused"));
    }
    let (data, file) = {
        let session = held(session);
        (session.paths.data().to_path_buf(), file_of(&session, id)?)
    };
    let (received, told) = {
        let (data, opened) = (data, opened.clone());
        apart(move || Ok(kept_here(&data, &opened))).await?
    };
    let mut session = held(session);
    session.commit_all(told)?;
    let marked = if crate::refusing::spanish(session.locale.as_deref()) {
        "otra versión"
    } else {
        "other version"
    };
    let by = received.signed_as.clone().or(received.owner_name.clone());
    let kept = other_version_kept(&mut session, &file, received.body.clone(), Some(marked))?;
    let holder = match kept {
        Some(made) => doc_of_file(&session, &made),
        None if pages_differ(&session, &file, &received.pages) => {
            let marked_copy = Received {
                body: tisty_core::docs::marked(&received.body, marked),
                ..received.clone()
            };
            guests_kept(&mut session, &marked_copy, by.clone())?;
            None
        }
        None => None,
    };
    if let Some(up) = holder {
        for page in &received.pages {
            guest_page(&mut session, page, by.clone(), up)?;
        }
    }
    let mut now = was.clone();
    now.version = opened.version;
    now.files_print = None;
    now.files_hash = None;
    now.files_key = None;
    kept_on(&mut session, &[(file, now)])?;
    let doubtful = opened.doubtful();
    Ok(Synced::OtherVersion {
        version: opened.version,
        doubtful,
        signer: received
            .signed_as
            .or(received.owner_name)
            .unwrap_or(opened.signer),
    })
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareStatus {
    dies_on: u64,
    paused: bool,
    blocked: bool,
    reads: u64,
    appealed: bool,
    may_correct: bool,
    fix_by: Option<u64>,
}

fn admin_of(
    session: &tauri::State<'_, Mutex<Session>>,
    id: &str,
) -> Answer<(String, String, String)> {
    let session = held(session);
    let (file, was) = remembered_of(&session, id)?;
    let admin = was.admin_token.ok_or_else(|| Refusal::of("folioGone"))?;
    Ok((was.id, admin, file))
}

#[tauri::command]
pub async fn doc_share_status(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<ShareStatus> {
    let (id, admin, file) = admin_of(&session, &id)?;
    apart(move || {
        let door = ToFolio::new()?;
        let told = folio_client::status(&door, &id, &admin)
            .map_err(|stopped| told_of(&app, &file, &stopped))?;
        Ok(ShareStatus {
            dies_on: told.dies_on,
            paused: told.paused_at.is_some(),
            blocked: told.blocked_at.is_some(),
            reads: told.read_total,
            appealed: told.appealed_at.is_some(),
            may_correct: told.may_correct,
            fix_by: told.fix_by,
        })
    })
    .await
}

#[tauri::command]
pub async fn doc_share_pause(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    paused: bool,
) -> Answer<()> {
    let (id, admin, file) = admin_of(&session, &id)?;
    apart(move || {
        let door = ToFolio::new()?;
        let done = if paused {
            folio_client::pause(&door, &id, &admin)
        } else {
            folio_client::resume(&door, &id, &admin)
        };
        done.map_err(|stopped| told_of(&app, &file, &stopped))
    })
    .await
}

#[tauri::command]
pub async fn doc_share_appeal(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<bool> {
    let (id, admin, _) = admin_of(&session, &id)?;
    apart(move || {
        let door = ToFolio::new()?;
        let claim = folio_client::appeal(&door, &id, &admin).map_err(|stopped| said(&stopped))?;
        Ok(claim == Claim::Accepted)
    })
    .await
}

// A language or a cited document lives through what holds it, unless another share still cites it.
fn going_with(
    shelf: &BTreeMap<String, Remembered>,
    file: &str,
    top: &str,
    doc_of: impl Fn(&str) -> Option<String>,
) -> Vec<(String, Remembered)> {
    let still_cited: Vec<&String> = shelf
        .iter()
        .filter(|(at, one)| at.as_str() != file && one.held_by.is_none())
        .flat_map(|(_, one)| &one.cited)
        .collect();
    shelf
        .iter()
        .filter(|(_, one)| one.held_by.as_deref() == Some(top))
        .filter(|(at, _)| !doc_of(at).is_some_and(|doc| still_cited.contains(&&doc)))
        .map(|(at, one)| (at.clone(), one.clone()))
        .collect()
}

#[tauri::command]
pub async fn doc_share_stop(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<()> {
    let (file, was, along) = {
        let session = held(&session);
        let (file, was) = remembered_of(&session, &id)?;
        let along = going_with(
            tisty_sharing::remembered(&session.state),
            &file,
            &was.id,
            |at| doc_of_file(&session, at).map(|one| one.to_string()),
        );
        (file, was, along)
    };
    apart(move || {
        let door = ToFolio::new()?;
        if let Some(admin) = &was.admin_token {
            folio_client::stop(&door, &was.id, admin).map_err(|stopped| said(&stopped))?;
        }
        let mut gone = vec![(file, was)];
        for (at, one) in along {
            if let Some(admin) = &one.admin_token {
                let _ = folio_client::stop(&door, &one.id, admin);
            }
            gone.push((at, one));
        }
        let session = app.state::<Mutex<Session>>();
        held(&session).commit_all(tisty_sharing::forgotten(&gone))?;
        Ok(())
    })
    .await
}

#[cfg(test)]
#[path = "folio_test.rs"]
mod tests;
