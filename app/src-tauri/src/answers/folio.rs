use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tauri::Manager;
use tisty_carrier::folio_client::{
    self, Answered, Asked, Claim, Door, First, Refused, Stopped, Taken, Verb, Version, Why,
};
use tisty_carrier::folio_hosts;
use tisty_carrier::folio_limits::{self, FolioLimits, SharingNow};
use tisty_carrier::folio_pace;
use tisty_carrier::folio_terms::{self, Standing, Tongue, Unkept, Unsure};
use tisty_carrier::folio_today;
use tisty_carrier::policy;
use tisty_core::Paths;
use tisty_core::event::Signature;
use tisty_envelope::family::Going;
use tisty_share_proto::identity::{REVOCATIONS_AT_ONCE, Revocation};
use tisty_sharing::{Carrying, Desk, Remembered, Told, Under, Unshared};

use super::folio_editing::{Named, SignsAs, said_after_sending, signs_as, who_of};
use super::sharing::{doc_id, refused, sealed_by, the_maker_among, the_maker_here, turned};
use crate::{Answer, Refusal, Session, held};

const TAKEN_BACK_TOLD: &str = "folio-taken-back-told";
const PATIENCE: Duration = Duration::from_secs(120);
pub(super) const TERMS_TO_ACCEPT: &str = "folioTermsToAccept";
const ANSWER_AT_MOST: u64 = 64 * 1024 * 1024;
pub(super) const FIRST_TERM_DAYS: u64 = 1;

pub(super) struct ToFolio {
    client: reqwest::blocking::Client,
    host: String,
    api: String,
}

impl ToFolio {
    pub(super) fn new(home: &Path) -> Answer<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(PATIENCE)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent(concat!("tisty/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| Refusal::of("folioUnreached"))?;
        let host = private_host(home);
        Ok(Self {
            client,
            api: folio_client::api_of(&host),
            host,
        })
    }

    pub(super) fn host(&self) -> &str {
        &self.host
    }
}

pub(super) fn private_host(home: &Path) -> String {
    folio_hosts::private(home, crate::reaching::brought_from_folio)
}

pub(super) async fn private_host_apart(home: std::path::PathBuf) -> Answer<String> {
    apart(move || Ok(private_host(&home))).await
}

impl Door for ToFolio {
    fn ask(&self, asked: &Asked<'_>) -> Option<Answered> {
        let url = format!("{}{}", self.api, asked.path);
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
            Refused::SharesToday { until } => Refusal::about("folioSharesToday", until.to_string()),
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

pub(super) fn tongue_of(session: &Session) -> Tongue {
    if crate::refusing::spanish(session.locale.as_deref()) {
        Tongue::Es
    } else {
        Tongue::En
    }
}

pub(super) async fn apart<T: Send + 'static>(
    work: impl FnOnce() -> Answer<T> + Send + 'static,
) -> Answer<T> {
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

pub(super) struct Agreed {
    pub(super) version: String,
    pub(super) newer_unchecked: bool,
}

// Asked before anything leaves: terms that changed stop an upload, never a link already out.
pub(super) fn may_upload(paths: &Paths, tongue: Tongue) -> Answer<Agreed> {
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
pub(super) fn new_terms_asked(agreed: &Answer<Agreed>) -> bool {
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

pub(super) fn seconds_of(ms: u64) -> u64 {
    ms.div_ceil(1000)
}

// Folio names when it takes the next one, so this computer keeps it and asks nothing before then.
pub(super) fn kept_from(stopped: &Stopped, home: &std::path::Path, held_as: Option<&str>) {
    let now = policy::now();
    match stopped {
        Stopped::Refused(Refused::Slow { until }) => {
            if let Some(id) = held_as {
                folio_pace::hold(home, id, seconds_of(*until), now);
            }
        }
        Stopped::Refused(Refused::Quota { until }) => {
            folio_pace::hold(home, folio_pace::NETWORK, seconds_of(*until), now);
        }
        Stopped::Refused(Refused::SharesToday { until }) => {
            folio_today::full_until(home, seconds_of(*until), now);
        }
        _ => {}
    }
}

pub(super) fn said_kept(
    stopped: &Stopped,
    home: &std::path::Path,
    held_as: Option<&str>,
) -> Refusal {
    kept_from(stopped, home, held_as);
    said(stopped)
}

pub(super) fn network_free(home: &std::path::Path, now: u64) -> Answer<()> {
    match folio_pace::held_until(home, folio_pace::NETWORK, now) {
        Some(until) => Err(Refusal::about("folioQuota", (until * 1000).to_string())),
        None => Ok(()),
    }
}

pub(super) fn held_back_here(home: &std::path::Path, new_ones: usize) -> Answer<()> {
    let now = policy::now();
    network_free(home, now)?;
    if new_ones > 0 && folio_today::full_now(home, now) {
        return Err(Refusal::of("folioSharesToday"));
    }
    Ok(())
}

// The waits pace what reaches Folio, so a press stopped before that leaves them as they were.
pub(super) fn press_given_back(
    refused: Refusal,
    home: &std::path::Path,
    id: &str,
    at: u64,
) -> Refusal {
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
pub(super) fn said_to_whom(refused: Refusal, owning: bool, home: &std::path::Path) -> Refusal {
    let limits = || folio_limits::kept(home);
    match (refused.code, owning) {
        ("folioKeyBlocked", false) => Refusal::of("folioOwnerKeyBlocked"),
        ("notOneOwnDoc", false) => Refusal::of("folioEditTakenBack"),
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

pub(super) fn delivered_or_asked_again(
    refused: Refusal,
    home: &std::path::Path,
    tongue: Tongue,
) -> Refusal {
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

#[derive(Debug, Serialize)]
pub struct SharingHere {
    #[serde(flatten)]
    now: SharingNow,
    site: String,
}

#[tauri::command]
pub async fn folio_sharing_now(session: tauri::State<'_, Mutex<Session>>) -> Answer<SharingHere> {
    let now = sharing_here(&session).await?;
    let home = held(&session).paths.private();
    let site = private_host_apart(home).await?;
    Ok(SharingHere { now, site })
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
    pub(super) named: String,
    pub(super) under: Option<String>,
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

pub(super) fn secrets_of_docs(
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
    new_shares_full: bool,
    held_there: Vec<String>,
    languages: Vec<tisty_sharing::Offered>,
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
fn still_shared(
    app: &tauri::AppHandle,
    home: &Path,
    shared: Vec<SharedElsewhere>,
) -> Vec<SharedElsewhere> {
    let Ok(door) = ToFolio::new(home) else {
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

pub(super) fn forget_share(app: &tauri::AppHandle, file: &str) {
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
    let (lost, home): (Vec<(String, Remembered)>, _) = {
        let session = app.state::<Mutex<Session>>();
        let session = held(&session);
        let lost = tisty_sharing::remembered(&session.state)
            .iter()
            .filter(|(file, _)| doc_of_file(&session, file).is_none())
            .map(|(file, was)| (file.clone(), was.clone()))
            .collect();
        (lost, session.paths.private())
    };
    if lost.is_empty() {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Ok(door) = ToFolio::new(&home) else {
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
    left_out: Option<Vec<String>>,
) -> Answer<Look> {
    stops_what_lost_its_document(&app);
    let doc = doc_id(&id)?;
    let left_out = left_out_chosen(&held(&session), doc, left_out)?;
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
            left_out: left_out.clone(),
        };
        let gathered = tisty_sharing::gathered_within(&desk, &asked, languages_top(&home))
            .map_err(|e| refused(&e))?;
        let cited = tisty_sharing::citable(&desk, doc).map_err(|e| refused(&e))?;
        let languages =
            tisty_sharing::languages_offered(&desk, doc, &left_out).map_err(|e| refused(&e))?;
        let others = shared.iter().filter(|one| one.doc != id).count() as u64;
        let shared = match limits.shares_at_most {
            Some(most) if others >= most => still_shared(&app, &home, shared),
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
            new_shares_full: folio_today::full_now(&home, policy::now()),
            held_there: std::iter::once(doc)
                .chain(cited.iter().copied())
                .filter(|one| folio_holds(&state, *one))
                .map(|one| one.to_string())
                .collect(),
            languages,
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
    owner: Option<Named>,
    signer: Option<Named>,
    editors: Vec<Named>,
    signs_as: Option<SignsAs>,
}

pub(super) fn card_of(
    host: &str,
    was: &Remembered,
    signature: &Signature,
    maker: Option<&str>,
) -> Option<Card> {
    let key = was.link_key.as_deref()?;
    let taken = was.admin_token.is_none();
    let who = who_of(was, maker);
    Some(Card {
        read: folio_client::read_link(host, &was.id, key),
        edit: was
            .edit_token
            .as_deref()
            .map(|token| folio_client::edit_link(host, &was.id, key, token)),
        version: was.version,
        taken,
        held: was.held_by.is_some(),
        told: Vec::new(),
        owner: who.owner,
        signer: who.signer,
        editors: who.editors,
        signs_as: signs_as(was, signature),
    })
}

pub(super) fn file_of(session: &Session, id: &str) -> Answer<String> {
    let doc = doc_id(id)?;
    let kept = session
        .state
        .docs
        .get(&doc)
        .ok_or_else(|| Refusal::of("noSuchDoc"))?;
    Ok(kept.file.clone())
}

#[tauri::command]
pub async fn doc_share_card(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Card>> {
    let home = held(&session).paths.private();
    let maker = folio_limits::maker_root(&home);
    let host = private_host_apart(home).await?;
    let session = held(&session);
    let file = file_of(&session, &id)?;
    Ok(tisty_sharing::remembered(&session.state)
        .get(&file)
        .and_then(|was| card_of(&host, was, &session.state.signed, maker.as_deref())))
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

pub(super) fn kept_on(session: &mut Session, went: &[(String, Remembered)]) -> Answer<()> {
    session.commit_all(tisty_sharing::sent(went))?;
    Ok(())
}

pub(super) fn keeping(
    app: &tauri::AppHandle,
) -> impl Fn(&[(String, Remembered)]) -> Answer<()> + use<> {
    let app = app.clone();
    move |went| {
        let session = app.state::<Mutex<Session>>();
        kept_on(&mut held(&session), went)
    }
}

pub(super) fn delivered(
    door: &dyn Door,
    home: &std::path::Path,
    held_as: Option<&str>,
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
        // A language that gets its token now is already in Folio, which refuses a version naming a permit it does not hold.
        if sealed.granted.contains(&going.id) {
            let permit = entry
                .1
                .edit_token
                .as_deref()
                .and_then(|token| tisty_envelope::sealing::permit_shown(token).ok())
                .ok_or_else(|| Refusal::of("folioGarbled"))?;
            folio_client::permit(door, &going.id, admin, Some(&permit))
                .map_err(|stopped| said_kept(&stopped, home, held_as))?;
        }
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
        .map_err(|stopped| said_kept(&stopped, home, held_as))?;
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
            folio_client::send_files(door, &going.id, files)
                .map_err(|stopped| said_kept(&stopped, home, held_as))?;
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
pub(super) fn told_of_taken_back(
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
    let no_room = sent_taken_back(door, home, &taken);
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
fn sent_taken_back(door: &dyn Door, home: &std::path::Path, taken: &[Taken]) -> bool {
    let mut no_room = false;
    for some in taken.chunks(REVOCATIONS_AT_ONCE) {
        match folio_client::taken_back(door, some) {
            Err(Stopped::Refused(Refused::MachinesFull)) => no_room = true,
            Err(stopped @ Stopped::Refused(Refused::Quota { .. } | Refused::Slow { .. })) => {
                kept_from(&stopped, home, None);
                break;
            }
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

pub(super) fn languages_top(home: &std::path::Path) -> Option<usize> {
    folio_limits::kept(home)
        .and_then(|limits| limits.languages_at_most)
        .and_then(|top| usize::try_from(top).ok())
}

pub(super) fn floors_of(home: &std::path::Path) -> Vec<String> {
    folio_limits::kept(home)
        .map(|limits| {
            [limits.oldest_contract_taken, limits.oldest_contract_served]
                .into_iter()
                .flatten()
                .collect()
        })
        .unwrap_or_default()
}

pub(super) async fn sealed_for_folio(
    app: &tauri::AppHandle,
    session: &tauri::State<'_, Mutex<Session>>,
    asked: tisty_sharing::Asked,
    editable: bool,
    terms: String,
    held_as: Option<&str>,
) -> Answer<(tisty_sharing::Sealed, BTreeMap<String, String>)> {
    let home = held(session).paths.private();
    let languages_at_most = languages_top(&home);
    let floors = floors_of(&home);
    let starts: Starts = Arc::default();
    let trouble: Trouble = Arc::default();
    let (given, stopped) = (Arc::clone(&starts), Arc::clone(&trouble));
    let reaching = home.clone();
    let sealed = sealed_by(app, session, asked, move |desk, asked| {
        let door =
            ToFolio::new(&reaching).map_err(|_| Unshared::Unread("no way to Folio".to_owned()))?;
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
        .map(|gathered| gathered.sent_again_below(&floors))
    })
    .await;
    if let Some(why) = trouble.lock().ok().and_then(|mut held| held.take()) {
        return Err(said_kept(&why, &home, held_as));
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
    #[serde(default)]
    left_out: Option<Vec<String>>,
}

// Asked without a choice, a document starts from the one its group was shared with.
fn left_out_chosen(
    session: &Session,
    doc: tisty_core::model::DocId,
    chosen: Option<Vec<String>>,
) -> Answer<Vec<String>> {
    match chosen {
        Some(ids) => ids.iter().map(|one| file_of(session, one)).collect(),
        None => Ok(tisty_sharing::left_out_of(&session.state, doc)),
    }
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
        left_out,
    } = chosen;
    let doc = doc_id(&id)?;
    let (paths, tongue, file, left_out) = {
        let session = held(&session);
        (
            session.paths.clone(),
            tongue_of(&session),
            file_of(&session, &id)?,
            left_out_chosen(&session, doc_id(&id)?, left_out)?,
        )
    };
    let asked = tisty_sharing::Asked {
        doc,
        public: false,
        cited: cited
            .iter()
            .map(|one| doc_id(one))
            .collect::<Answer<Vec<_>>>()?,
        signed_as,
        left_out,
    };
    let new_ones = {
        let session = held(&session);
        std::iter::once(&doc)
            .chain(&asked.cited)
            .filter(|one| !folio_holds(&session.state, **one))
            .count()
    };
    held_back_here(&paths.private(), new_ones)?;
    may_start(sharing_here(&session).await?)?;
    let checking = paths.clone();
    let terms = apart(move || may_upload(&checking, tongue).map(|agreed| agreed.version)).await?;
    let (mut sealed, starts) =
        sealed_for_folio(&app, &session, asked, editable, terms, None).await?;
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
        let door = ToFolio::new(&home)?;
        delivered(&door, &home, None, &keep, &sealed, &starts, ttl_days)
            .map_err(|refused| said_to_whom(refused, true, &home))
            .map_err(|refused| delivered_or_asked_again(refused, &home, tongue))?;
        let not_kept = told_of_taken_back(&door, &home, &state, &device);
        let told = said_after_sending(told.into_iter().chain(not_kept).collect());
        let top = sealed
            .remembering
            .iter()
            .find(|(_, one)| one.held_by.is_none());
        let maker = folio_limits::maker_root(&home);
        top.and_then(|(_, one)| card_of(door.host(), one, &state.signed, maker.as_deref()))
            .map(|one| Card { told, ..one })
            .ok_or_else(|| Refusal::of("folioGarbled"))
    })
    .await
}

pub(super) fn remembered_of(session: &Session, id: &str) -> Answer<(String, Remembered)> {
    let file = file_of(session, id)?;
    let was = tisty_sharing::remembered(&session.state)
        .get(&file)
        .cloned();
    was.map(|one| (file, one))
        .ok_or_else(|| Refusal::of("folioGone"))
}

pub(super) fn doc_of_file(session: &Session, file: &str) -> Option<tisty_core::model::DocId> {
    let found = session.state.docs.values().find(|one| one.file == file);
    found.map(|one| one.id)
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
    let home = held(&session).paths.private();
    apart(move || {
        let door = ToFolio::new(&home)?;
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
    let home = held(&session).paths.private();
    apart(move || {
        let door = ToFolio::new(&home)?;
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
    let home = held(&session).paths.private();
    apart(move || {
        let door = ToFolio::new(&home)?;
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
    let (file, was, along, home) = {
        let session = held(&session);
        let (file, was) = remembered_of(&session, &id)?;
        let along = going_with(
            tisty_sharing::remembered(&session.state),
            &file,
            &was.id,
            |at| doc_of_file(&session, at).map(|one| one.to_string()),
        );
        (file, was, along, session.paths.private())
    };
    apart(move || {
        let door = ToFolio::new(&home)?;
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
