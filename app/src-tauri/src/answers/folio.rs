use std::collections::BTreeMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::Serialize;
use tisty_carrier::folio_client::{
    self, Answered, Asked, BASE, Claim, Door, First, Refused, Stopped, Taken, Verb, Version,
};
use tisty_carrier::folio_limits::{self, FolioLimits};
use tisty_carrier::folio_pace;
use tisty_carrier::folio_terms::{self, Standing, Tongue, Unkept, Unsure};
use tisty_carrier::policy;
use tisty_core::Paths;
use tisty_share_proto::identity::REVOCATIONS_AT_ONCE;
use tisty_sharing::{Carrying, Desk, Remembered, Told, Under, Unshared};

use super::folio_receiving::{kept_here, opened_from};
use super::papers::other_version_kept;
use super::sharing::{doc_id, refused, sealed_by, turned};
use crate::{Answer, Refusal, Session, held};

const PATIENCE: Duration = Duration::from_secs(120);
const ANSWER_AT_MOST: u64 = 64 * 1024 * 1024;

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
            Refused::TooBig => Refusal::of("folioTooBig"),
            Refused::Slow { until } | Refused::Quota { until } => {
                Refusal::about("folioWait", until.to_string())
            }
            Refused::UploadsOff => Refusal::of("folioUploadsOff"),
            Refused::Paused => Refusal::of("folioPaused"),
            Refused::Blocked => Refusal::of("folioBlocked"),
            Refused::Behind => Refusal::of("folioBehind"),
            Refused::Outdated => Refusal::of("folioOutdated"),
            Refused::Revoked => Refusal::of("folioRevoked"),
            Refused::NotBlocked | Refused::Appealed => Refusal::of("folioAppealed"),
            Refused::Missing => Refusal::of("folioGone"),
            Refused::NotYours => Refusal::of("notOneOwnDoc"),
            Refused::BadUpload
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
    #[serde(rename_all = "camelCase")]
    ToAccept {
        version: String,
        text: String,
        accepted_before: Option<String>,
    },
}

fn terms_now(home: &std::path::Path, tongue: Tongue) -> Answer<TermsNow> {
    match folio_terms::standing(home, tongue, crate::reaching::brought) {
        Ok(Standing::Accepted) => Ok(TermsNow::Accepted),
        Ok(Standing::ToAccept {
            version,
            text,
            accepted_before,
        }) => Ok(TermsNow::ToAccept {
            version,
            text,
            accepted_before,
        }),
        Err(Unsure::NoWord) => Err(Refusal::of("folioUnreached")),
        Err(Unsure::NotPublished) => Err(Refusal::of("folioTermsNotPublished")),
        Err(Unsure::Unmatched) => Err(Refusal::of("folioTermsUnmatched")),
    }
}

// Asked before anything leaves: terms that changed stop an upload, never a link already out.
fn may_upload(paths: &Paths, tongue: Tongue) -> Answer<()> {
    let home = paths.private();
    match terms_now(&home, tongue)? {
        TermsNow::Accepted => {}
        TermsNow::ToAccept { .. } => return Err(Refusal::of("folioTermsToAccept")),
    }
    let limits = folio_limits::kept(&home);
    if folio_limits::uploads_halted(limits.as_ref(), policy::now()) {
        return Err(Refusal::of("folioUploadsOff"));
    }
    Ok(())
}

#[tauri::command]
pub async fn folio_terms_standing(session: tauri::State<'_, Mutex<Session>>) -> Answer<TermsNow> {
    let (home, tongue) = {
        let session = held(&session);
        (session.paths.private(), tongue_of(&session))
    };
    apart(move || terms_now(&home, tongue)).await
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
    name: Option<String>,
    alias: Option<String>,
    carrying: Carrying,
    cited: Vec<String>,
    shared: Vec<SharedElsewhere>,
    limits: FolioLimits,
    told: Vec<Told>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedElsewhere {
    doc: String,
    title: String,
}

fn shared_elsewhere(
    session: &Session,
    shelf: &BTreeMap<String, Remembered>,
) -> Vec<SharedElsewhere> {
    session
        .state
        .docs
        .values()
        .filter(|one| shelf.get(&one.file).is_some_and(|was| !was.held) && one.page_of.is_none())
        .map(|one| SharedElsewhere {
            doc: one.id.to_string(),
            title: one.title.clone().unwrap_or_default(),
        })
        .collect()
}

#[tauri::command]
pub async fn doc_share_look(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Look> {
    let doc = doc_id(&id)?;
    let (paths, state, also, name, alias, shared) = {
        let session = held(&session);
        let shelf = tisty_sharing::remembered(&session.paths).map_err(|e| refused(&e))?;
        (
            session.paths.clone(),
            session.state.clone(),
            session.dest(),
            session.state.signed.name.clone(),
            session.state.signed.alias.clone(),
            shared_elsewhere(&session, &shelf),
        )
    };
    apart(move || {
        let kept = state
            .docs
            .get(&doc)
            .ok_or_else(|| Refusal::of("noSuchDoc"))?;
        let body = tisty_core::docs::read(&paths.docs(), &kept.file)
            .map_err(|_| Refusal::of("cannotSeal"))?;
        let desk = Desk {
            paths: &paths,
            state: &state,
            also: also.as_deref(),
            turned: &turned,
        };
        let asked = tisty_sharing::Asked {
            doc,
            public: false,
            cited: Vec::new(),
            signed_as: None,
        };
        let gathered = tisty_sharing::gathered(&desk, &asked).map_err(|e| refused(&e))?;
        let cited = tisty_sharing::citable(&desk, doc).map_err(|e| refused(&e))?;
        let home = paths.private();
        let limits = match folio_limits::heed(&home, crate::reaching::brought) {
            Some(Ok(limits)) => Some(limits),
            _ => folio_limits::kept(&home),
        };
        Ok(Look {
            secrets: secrets_in(&body),
            name,
            alias,
            carrying: gathered.carrying(),
            cited: cited.iter().map(ToString::to_string).collect(),
            shared,
            limits: limits.unwrap_or_default(),
            told: gathered.told().to_vec(),
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
    let shelf = tisty_sharing::remembered(&session.paths).map_err(|e| refused(&e))?;
    Ok(shelf.get(&file).and_then(card_of))
}

type Starts = Arc<Mutex<BTreeMap<String, String>>>;
type Trouble = Arc<Mutex<Option<Stopped>>>;

fn delivered(
    door: &dyn Door,
    paths: &Paths,
    sealed: &tisty_sharing::Sealed,
    starts: &BTreeMap<String, String>,
    ttl_days: u64,
) -> Answer<()> {
    for going in &sealed.going {
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
        if stored.files_wanted {
            let files = going
                .sealed
                .files
                .as_deref()
                .ok_or_else(|| Refusal::of("folioGarbled"))?;
            folio_client::send_files(door, &going.id, files).map_err(|stopped| said(&stopped))?;
        }
        tisty_sharing::sent(paths, std::slice::from_ref(entry)).map_err(|e| refused(&e))?;
    }
    Ok(())
}

// Sent again on each sync (81): the server keeps them only while it holds something of that root, and a late one still counts.
fn told_of_taken_back(door: &dyn Door, state: &tisty_core::State, device: &tisty_core::DeviceId) {
    let Some(root) = state.persons.person(device) else {
        return;
    };
    let taken: Vec<Taken> = state
        .persons
        .revocations
        .iter()
        .filter(|one| one.unchecked().is_some_and(|(person, _)| person == root))
        .map(|one| Taken {
            root: one.root.clone(),
            machine: one.share.clone(),
            at: one.at.to_string(),
            signed: one.sig.clone(),
        })
        .collect();
    for some in taken.chunks(REVOCATIONS_AT_ONCE) {
        let _ = folio_client::taken_back(door, some);
    }
}

async fn sealed_for_folio(
    app: &tauri::AppHandle,
    session: &tauri::State<'_, Mutex<Session>>,
    asked: tisty_sharing::Asked,
    editable: bool,
) -> Answer<(tisty_sharing::Sealed, BTreeMap<String, String>)> {
    let starts: Starts = Arc::default();
    let trouble: Trouble = Arc::default();
    let door = ToFolio::new()?;
    let (given, stopped) = (Arc::clone(&starts), Arc::clone(&trouble));
    let sealed = sealed_by(app, session, asked, move |desk, asked| {
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

#[tauri::command]
pub async fn doc_shared(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    cited: Vec<String>,
    signed_as: Option<String>,
    editable: bool,
    ttl_days: u64,
) -> Answer<Card> {
    let doc = doc_id(&id)?;
    let (paths, tongue, file) = {
        let session = held(&session);
        (
            session.paths.clone(),
            tongue_of(&session),
            file_of(&session, &id)?,
        )
    };
    let checking = paths.clone();
    apart(move || may_upload(&checking, tongue)).await?;
    let asked = tisty_sharing::Asked {
        doc,
        public: false,
        cited: cited
            .iter()
            .map(|one| doc_id(one))
            .collect::<Answer<Vec<_>>>()?,
        signed_as,
    };
    let (mut sealed, starts) = sealed_for_folio(&app, &session, asked, editable).await?;
    if let Some((_, top)) = sealed.remembering.iter_mut().find(|(at, _)| *at == file) {
        top.cited = cited;
    }
    let (state, device) = {
        let session = held(&session);
        (session.state.clone(), session.config.device_id.clone())
    };
    apart(move || {
        let door = ToFolio::new()?;
        delivered(&door, &paths, &sealed, &starts, ttl_days)?;
        told_of_taken_back(&door, &state, &device);
        let shelf = tisty_sharing::remembered(&paths).map_err(|e| refused(&e))?;
        shelf
            .get(&file)
            .and_then(card_of)
            .ok_or_else(|| Refusal::of("folioGarbled"))
    })
    .await
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Synced {
    Unchanged,
    Sent { version: u64 },
    OtherVersion { version: u64, signer: String },
}

fn remembered_of(session: &Session, id: &str) -> Answer<(String, Remembered)> {
    let file = file_of(session, id)?;
    let shelf = tisty_sharing::remembered(&session.paths).map_err(|e| refused(&e))?;
    let was = shelf.get(&file).cloned();
    was.map(|one| (file, one))
        .ok_or_else(|| Refusal::of("folioGone"))
}

#[tauri::command]
pub async fn doc_share_sync(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Synced> {
    let doc = doc_id(&id)?;
    let (paths, tongue, was, also, state) = {
        let session = held(&session);
        (
            session.paths.clone(),
            tongue_of(&session),
            remembered_of(&session, &id)?.1,
            session.dest(),
            session.state.clone(),
        )
    };
    let home = paths.private();
    let limits = folio_limits::kept(&home);
    folio_pace::press(&home, &was.id, policy::now(), limits.as_ref())
        .map_err(|free| Refusal::about("folioWait", (free * 1000).to_string()))?;
    let remembered_id = was.id.clone();
    let remembered_version = was.version;
    let latest = apart(move || {
        let door = ToFolio::new()?;
        folio_client::latest(&door, &remembered_id).map_err(|stopped| said(&stopped))
    })
    .await?;
    if latest.version > remembered_version {
        return other_version_brought(&session, &id, &was).await;
    }
    let asked = tisty_sharing::Asked {
        doc,
        public: false,
        cited: was
            .cited
            .iter()
            .map(|one| doc_id(one))
            .collect::<Answer<Vec<_>>>()?,
        signed_as: was.signed_as.clone(),
    };
    let looking = asked.clone();
    let reading = paths.clone();
    let unchanged = apart(move || {
        let desk = Desk {
            paths: &reading,
            state: &state,
            also: also.as_deref(),
            turned: &turned,
        };
        let shelf = tisty_sharing::remembered(&reading).map_err(|e| refused(&e))?;
        let gathered = tisty_sharing::gathered(&desk, &looking).map_err(|e| refused(&e))?;
        Ok(gathered.same_as_sent(&shelf))
    })
    .await?;
    if unchanged {
        return Ok(Synced::Unchanged);
    }
    let checking = paths.clone();
    apart(move || may_upload(&checking, tongue)).await?;
    let (sealed, starts) = sealed_for_folio(&app, &session, asked, false).await?;
    let version = sealed
        .remembering
        .iter()
        .find(|(_, one)| one.id == was.id)
        .map_or(was.version + 1, |(_, one)| one.version);
    let (state, device) = {
        let session = held(&session);
        (session.state.clone(), session.config.device_id.clone())
    };
    apart(move || {
        let door = ToFolio::new()?;
        delivered(&door, &paths, &sealed, &starts, 0)?;
        told_of_taken_back(&door, &state, &device);
        Ok(())
    })
    .await?;
    Ok(Synced::Sent { version })
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
    let mut session = held(session);
    let file = file_of(&session, id)?;
    let received = kept_here(&mut session, &opened);
    let marked = if crate::refusing::spanish(session.locale.as_deref()) {
        "otra versión"
    } else {
        "other version"
    };
    other_version_kept(&mut session, &file, received.body, Some(marked))?;
    let mut now = was.clone();
    now.version = opened.version;
    tisty_sharing::sent(&session.paths, &[(file, now)]).map_err(|e| refused(&e))?;
    Ok(Synced::OtherVersion {
        version: opened.version,
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
}

fn admin_of(session: &tauri::State<'_, Mutex<Session>>, id: &str) -> Answer<(String, String)> {
    let session = held(session);
    let (_, was) = remembered_of(&session, id)?;
    let admin = was.admin_token.ok_or_else(|| Refusal::of("folioGone"))?;
    Ok((was.id, admin))
}

#[tauri::command]
pub async fn doc_share_status(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<ShareStatus> {
    let (id, admin) = admin_of(&session, &id)?;
    apart(move || {
        let door = ToFolio::new()?;
        let told = folio_client::status(&door, &id, &admin).map_err(|stopped| said(&stopped))?;
        Ok(ShareStatus {
            dies_on: told.dies_on,
            paused: told.paused_at.is_some(),
            blocked: told.blocked_at.is_some(),
            reads: told.read_total,
        })
    })
    .await
}

#[tauri::command]
pub async fn doc_share_pause(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    paused: bool,
) -> Answer<()> {
    let (id, admin) = admin_of(&session, &id)?;
    apart(move || {
        let door = ToFolio::new()?;
        let done = if paused {
            folio_client::pause(&door, &id, &admin)
        } else {
            folio_client::resume(&door, &id, &admin)
        };
        done.map_err(|stopped| said(&stopped))
    })
    .await
}

#[tauri::command]
pub async fn doc_share_appeal(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<bool> {
    let (id, admin) = admin_of(&session, &id)?;
    apart(move || {
        let door = ToFolio::new()?;
        let claim = folio_client::appeal(&door, &id, &admin).map_err(|stopped| said(&stopped))?;
        Ok(claim == Claim::Accepted)
    })
    .await
}

#[tauri::command]
pub async fn doc_share_stop(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let (file, paths) = {
        let session = held(&session);
        (file_of(&session, &id)?, session.paths.clone())
    };
    let manages = admin_of(&session, &id).ok();
    apart(move || {
        if let Some((id, admin)) = manages {
            let door = ToFolio::new()?;
            folio_client::stop(&door, &id, &admin).map_err(|stopped| said(&stopped))?;
        }
        tisty_sharing::forgotten(&paths, &file).map_err(|e| refused(&e))
    })
    .await
}

#[cfg(test)]
#[path = "folio_test.rs"]
mod tests;
