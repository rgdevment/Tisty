use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tisty_carrier::folio_client::{self, Door, Fetched};
use tisty_contracts::{EnvelopeKind, Opening, Served};
use tisty_core::Op;
use tisty_core::attach::{self, Via};
use tisty_core::witness::{self, Fact, channel};
use tisty_envelope::reading::{self, Unopened};
use tisty_sharing::{Asked, Desk, Received, Remembered};

use super::folio::{ToFolio, said};
use super::sharing::{refused, turned};
use crate::{Answer, Refusal, Session, held};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Link {
    pub id: String,
    pub key: String,
    pub token: Option<String>,
}

pub fn link_of(web: &url::Url) -> Option<Link> {
    let id = web.path().trim_start_matches('/').to_owned();
    if id.len() != 8 || !id.chars().all(|one| one.is_ascii_alphanumeric()) {
        return None;
    }
    let fragment = web.fragment()?;
    let (key, token) = match fragment.split_once('.') {
        Some((key, token)) => (key, Some(token.to_owned())),
        None => (fragment, None),
    };
    (!key.is_empty()).then(|| Link {
        id,
        key: key.to_owned(),
        token,
    })
}

fn served_by(fetched: &Fetched) -> Served {
    let received = jiff::Timestamp::from_millisecond(fetched.arrived_ms)
        .ok()
        .map(|at| at.to_string());
    let revocations = fetched
        .revocations
        .as_deref()
        .and_then(|said| serde_json::from_str(said).ok());
    Served {
        received,
        revocations,
    }
}

fn unopened(wrong: &Unopened) -> Refusal {
    witness::warn(
        channel::WINDOW,
        "a shared document could not be opened",
        &[("why", Fact::Why(wrong.to_string()))],
    );
    match wrong {
        Unopened::Newer => Refusal::of("folioOutdated"),
        _ => Refusal::of("folioUnopened"),
    }
}

pub(super) struct Opened {
    pub document: serde_json::Value,
    pub pictures: Vec<Vec<u8>>,
    pub files: Vec<Vec<u8>>,
    pub owner: Option<String>,
    pub signer: String,
    pub version: u64,
}

pub(super) fn opened_from(door: &dyn Door, id: &str, key: &str) -> Answer<Opened> {
    let fetched = folio_client::fetch(door, id).map_err(|stopped| said(&stopped))?;
    let served = served_by(&fetched);
    let opening = |kind| Opening {
        id: id.to_owned(),
        kind,
        link_key: Some(key.to_owned()),
    };
    let main = opening(EnvelopeKind::Main);
    let sealed = reading::opened(&fetched.bytes, &main, &served).map_err(|e| unopened(&e))?;
    let owner = reading::owner_block(&sealed);
    let read = reading::reading(&fetched.bytes, &main, &served).map_err(|e| unopened(&e))?;
    let files = match read.document["files_hash"].as_str() {
        Some(hash) => {
            let bytes =
                folio_client::fetch_files(door, id, hash).map_err(|stopped| said(&stopped))?;
            reading::files_for(
                &read.document,
                &bytes,
                &opening(EnvelopeKind::Files),
                &served,
            )
            .map_err(|e| unopened(&e))?
            .parts
        }
        None => Vec::new(),
    };
    Ok(Opened {
        document: read.document,
        pictures: read.pictures,
        files,
        owner,
        signer: read.signer,
        version: fetched.version,
    })
}

pub(super) fn kept_here(session: &mut Session, opened: &Opened) -> Received {
    let data = session.paths.data().to_path_buf();
    let room = std::env::temp_dir().join(format!("tisty-received-{}", ulid::Ulid::generate()));
    let mut told = Vec::new();
    let received = tisty_sharing::received(
        &opened.document,
        &opened.pictures,
        &opened.files,
        &mut |name, bytes| {
            let leaf = std::path::Path::new(name).file_name()?;
            std::fs::create_dir_all(&room).ok()?;
            let beside = room.join(leaf);
            std::fs::write(&beside, bytes).ok()?;
            let kept = attach::keep(&beside, &data, attach::COPIED_IN_DOC).ok()?;
            told.push(kept.told(Via::Window));
            Some(attach::leaf_of(&kept.at)?.to_owned())
        },
    );
    let _ = std::fs::remove_dir_all(&room);
    for op in told {
        let _ = session.commit(op);
    }
    received
}

fn guest_doc(
    session: &mut Session,
    body: &str,
    by: Option<String>,
    page_of: Option<tisty_core::model::DocId>,
) -> Answer<(tisty_core::model::DocId, String)> {
    let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body)
        .map_err(|_| Refusal::of("folioGarbled"))?;
    let order = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.page_of == page_of && one.folder.is_none())
            .map(|one| one.order.as_str()),
    );
    let id = ulid::Ulid::generate();
    session.commit(Op::DocAdd {
        id,
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: true,
            made: None,
            by,
            file: made.id.clone(),
            order,
            said: Some(tisty_core::event::Said::of(body)),
            folder: None,
            page_of,
            task_of: None,
        },
    })?;
    Ok((id, made.id))
}

fn with_its_print(session: &Session, doc: tisty_core::model::DocId) -> Option<String> {
    let desk = Desk {
        paths: &session.paths,
        state: &session.state,
        also: None,
        turned: &turned,
    };
    let asked = Asked {
        doc,
        public: false,
        cited: Vec::new(),
        signed_as: None,
    };
    let gathered = tisty_sharing::gathered(&desk, &asked).ok()?;
    gathered.print_of_first()
}

fn taken_in(session: &mut Session, link: &Link, opened: &Opened) -> Answer<String> {
    let received = kept_here(session, opened);
    let by = received.owner_name.clone().or(received.signed_as.clone());
    let (doc, file) = guest_doc(session, &received.body, by.clone(), None)?;
    for page in &received.pages {
        guest_doc(session, page, by.clone(), Some(doc))?;
    }
    // A link to read only leaves a copy of one's own to change: nothing of it goes back (80).
    if link.token.is_some() {
        let entry = Remembered {
            id: link.id.clone(),
            version: opened.version,
            link_key: Some(link.key.clone()),
            files_print: None,
            files_hash: opened.document["files_hash"].as_str().map(str::to_owned),
            files_key: opened.document["files_hash"]
                .as_str()
                .map(|_| link.key.clone()),
            admin_token: None,
            edit_token: link.token.clone(),
            words_print: None,
            signed_as: None,
            owner: opened.owner.clone(),
            owner_name: received.owner_name.clone(),
            cited: Vec::new(),
            held: false,
        };
        let as_of_now = (file.clone(), entry);
        tisty_sharing::sent(&session.paths, std::slice::from_ref(&as_of_now))
            .map_err(|e| refused(&e))?;
        if let Some(print) = with_its_print(session, doc) {
            let (file, mut entry) = as_of_now;
            entry.words_print = Some(print);
            tisty_sharing::sent(&session.paths, &[(file, entry)]).map_err(|e| refused(&e))?;
        }
    }
    Ok(file)
}

fn asked_to_keep(app: &AppHandle, spanish: bool, opened: &Opened) -> bool {
    let title = opened.document["title"].as_str().unwrap_or_default();
    let name = opened.document["owner_name"]
        .as_str()
        .or(opened.document["signed_as"].as_str())
        .unwrap_or_default();
    let (words, keep, leave) =
        crate::refusing::edit_asked_words(spanish, title, name, &opened.signer);
    app.dialog()
        .message(words)
        .title("Tisty")
        .buttons(MessageDialogButtons::OkCancelCustom(
            keep.to_owned(),
            leave.to_owned(),
        ))
        .blocking_show()
}

fn brought(app: &AppHandle, web: &url::Url, spanish: bool) -> Answer<Option<String>> {
    let link = link_of(web).ok_or_else(|| Refusal::of("folioUnopened"))?;
    let door = ToFolio::new()?;
    let opened = opened_from(&door, &link.id, &link.key)?;
    if !asked_to_keep(app, spanish, &opened) {
        return Ok(None);
    }
    let session = app
        .try_state::<Mutex<Session>>()
        .ok_or_else(|| Refusal::of("internal"))?;
    let mut session = held(&session);
    taken_in(&mut session, &link, &opened).map(Some)
}

pub fn bring_apart(app: &AppHandle, web: url::Url) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let spanish = app
            .try_state::<Mutex<Session>>()
            .is_some_and(|session| crate::refusing::spanish(held(&session).locale.as_deref()));
        match brought(&app, &web, spanish) {
            Ok(Some(file)) => {
                let _ = app.emit_to("main", "showdoc", file);
                crate::tray::surface(&app);
            }
            Ok(None) => {}
            Err(_) => {
                app.dialog()
                    .message(crate::refusing::edit_failed_words(spanish))
                    .title("Tisty")
                    .show(|_| {});
            }
        }
    });
}

#[cfg(test)]
#[path = "folio_receiving_test.rs"]
mod tests;
