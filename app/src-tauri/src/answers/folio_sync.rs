use std::sync::Mutex;

use serde::Serialize;
use tisty_carrier::folio_client::{self, Door, Refused, Stopped};
use tisty_carrier::folio_limits::{self, FolioLimits};
use tisty_carrier::folio_pace;
use tisty_carrier::folio_terms::Tongue;
use tisty_carrier::policy;
use tisty_core::Paths;
use tisty_sharing::{Desk, Received, Remembered, Told};

use super::folio_editing::{
    link_matches_block, others_asked, said_after_sending, sent_or_put_back,
};

use super::folio::{
    FIRST_TERM_DAYS, TERMS_TO_ACCEPT, ToFolio, apart, delivered, delivered_or_asked_again,
    doc_of_file, file_of, floors_of, forget_share, keeping, kept_on, languages_top, may_upload,
    network_free, new_terms_asked, press_given_back, remembered_of, said_kept, said_to_whom,
    sealed_for_folio, seconds_of, secrets_of_docs, told_of_taken_back, tongue_of,
};
use super::folio_receiving::{
    Opened, guest_page, guests_kept, kept_here, opened_from, owner_apart,
};
use super::papers::{Arrived, other_version_kept};
use super::sharing::{doc_id, refused, the_maker_here, turned};
use crate::{Answer, Refusal, Session, held};

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
    OthersEdited {
        editors: Vec<String>,
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
        let home = self.paths.private();
        tisty_sharing::gathered_within(&desk, asked, languages_top(&home))
            .map(|gathered| gathered.sent_again_below(&floors_of(&home)))
            .map_err(|e| refused(&e))
    }
}

struct Looked {
    unchanged: bool,
    secret: Option<String>,
    others: Vec<String>,
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
            others: others_asked(gathered.told()),
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

pub(super) fn asked_again(
    doc: tisty_core::model::DocId,
    was: &Remembered,
) -> Answer<tisty_sharing::Asked> {
    Ok(tisty_sharing::Asked {
        doc,
        public: false,
        cited: was
            .cited
            .iter()
            .map(|one| doc_id(one))
            .collect::<Answer<Vec<_>>>()?,
        signed_as: was.signed_as.clone(),
        left_out: was.left_out.clone(),
    })
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Ahead {
    Unchanged,
    Gone,
    Newer,
    Upload,
}

fn waited(free: u64) -> Refusal {
    Refusal::about("folioWait", (free * 1000).to_string())
}

// The sync contract asks on every press, so another's newer version is brought apart (149); only an upload counts for the waits.
pub(super) fn before_upload(
    door: &dyn Door,
    home: &std::path::Path,
    was: &Remembered,
    unchanged: bool,
    limits: Option<&FolioLimits>,
    now: u64,
) -> Answer<Ahead> {
    if !unchanged {
        network_free(home, now)?;
        folio_pace::press(home, &was.id, now, limits).map_err(waited)?;
    }
    let ready = if unchanged {
        Ahead::Unchanged
    } else {
        Ahead::Upload
    };
    match folio_client::latest(door, &was.id) {
        Ok(latest) => {
            let next = held_until_next(&latest, home, &was.id, now);
            if latest.version > was.version {
                return Ok(Ahead::Newer);
            }
            match next {
                Some(until) if !unchanged => {
                    folio_pace::take_back(home, &was.id, now);
                    Err(waited(until))
                }
                _ => Ok(ready),
            }
        }
        Err(Stopped::Refused(Refused::Missing)) => Ok(Ahead::Gone),
        Err(Stopped::Refused(Refused::Blocked)) if correctable(door, was) => Ok(ready),
        Err(stopped) => {
            let refused = said_kept(&stopped, home, Some(&was.id));
            if unchanged {
                return Err(refused);
            }
            Err(press_given_back(refused, home, &was.id, now))
        }
    }
}

/// The languages sent with this anchor, each with what this Tisty last knew of it.
fn its_other_languages(
    session: &Session,
    doc: tisty_core::model::DocId,
    file: &str,
    was: &Remembered,
) -> Vec<(String, Remembered)> {
    let shelf = tisty_sharing::remembered(&session.state);
    session
        .state
        .tongues_of(doc)
        .into_iter()
        .filter(|one| one.file != file)
        .filter_map(|one| {
            shelf
                .get(&one.file)
                .map(|its| (one.file.clone(), its.clone()))
        })
        .filter(|(_, its)| its.held_by.as_deref() == Some(was.id.as_str()))
        .collect()
}

pub(super) fn first_ahead(
    door: &dyn Door,
    languages: Vec<(String, Remembered)>,
) -> Option<(String, Remembered)> {
    languages.into_iter().find(|(_, its)| {
        folio_client::latest(door, &its.id).is_ok_and(|latest| latest.version > its.version)
    })
}

// The family's next version may wait on another computer's or an editor's upload, which only Folio sees.
fn held_until_next(
    latest: &folio_client::Latest,
    home: &std::path::Path,
    id: &str,
    now: u64,
) -> Option<u64> {
    let until = seconds_of(latest.next_at?);
    if until <= now {
        return None;
    }
    folio_pace::hold(home, id, until, now);
    Some(until)
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
        Ok(ToFolio::new(&home)
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
    let looked = if was.read_only() {
        Looked {
            unchanged: true,
            secret: None,
            others: Vec::new(),
            print: String::new(),
        }
    } else {
        looked_over(&session, &asked).await?
    };
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
    if !looked.others.is_empty() && !looked.unchanged && accepted.as_ref() != Some(&looked.print) {
        return Ok(Synced::OthersEdited {
            editors: looked.others,
            print: looked.print,
        });
    }
    let home = paths.private();
    let pressed_at = policy::now();
    let languages = its_other_languages(&held(&session), doc, &file, &was);
    let (ahead, language_ahead) = {
        let (home, was, unchanged) = (home.clone(), was.clone(), looked.unchanged);
        apart(move || {
            let limits = folio_limits::kept(&home);
            let door = ToFolio::new(&home)?;
            let ahead = before_upload(&door, &home, &was, unchanged, limits.as_ref(), pressed_at)?;
            let language_ahead = match ahead {
                Ahead::Upload | Ahead::Unchanged => first_ahead(&door, languages),
                _ => None,
            };
            Ok((ahead, language_ahead))
        })
        .await?
    };
    // Each language has its own versions, and another's newer one is brought apart as the anchor's is (166).
    if let Some((its_file, its_was)) = language_ahead {
        let its_doc = doc_of_file(&held(&session), &its_file)
            .ok_or_else(|| Refusal::of("noSuchDoc"))?
            .to_string();
        return other_version_brought(&session, &its_doc, &its_was).await;
    }
    match ahead {
        Ahead::Unchanged => return unchanged_answered(&session, &paths, tongue).await,
        Ahead::Gone => {
            forget_share(&app, &file);
            return Ok(Synced::Gone);
        }
        Ahead::Newer => return other_version_brought(&session, &id, &was).await,
        Ahead::Upload => {}
    }
    if !link_matches_block(&was) {
        return Err(Refusal::of("folioEditLinkMismatch"));
    }
    let checking = paths.clone();
    let agreed = apart(move || may_upload(&checking, tongue)).await;
    let terms_unchecked = agreed.as_ref().is_ok_and(|one| one.newer_unchecked);
    let terms = agreed
        .map(|one| one.version)
        .map_err(|refused| press_given_back(refused, &home, &was.id, pressed_at))?;
    let (sealed, starts) =
        sealed_for_folio(&app, &session, asked, false, terms, Some(&was.id)).await?;
    let mut sealed = sealed.only_what_changed();
    if sealed.going.is_empty() {
        folio_pace::take_back(&home, &was.id, pressed_at);
        return unchanged_answered(&session, &paths, tongue).await;
    }
    let ttl_days = was.ttl_days.unwrap_or(FIRST_TERM_DAYS);
    for (_, one) in &mut sealed.remembering {
        one.ttl_days = Some(ttl_days);
    }
    let told = said_after_sending(sealed.told.clone());
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
    let owned = owned_with_tokens(&was, &sealed);
    let not_kept = apart(move || {
        let door = ToFolio::new(&home)?;
        let sent = std::cell::RefCell::new(std::collections::BTreeSet::new());
        let keep_noting = |went: &[(String, Remembered)]| {
            keep(went)?;
            sent.borrow_mut()
                .extend(went.iter().map(|(_, one)| one.id.clone()));
            Ok(())
        };
        // A retry after putting a permit back never sends again what Folio already stored.
        let deliver = || {
            let owed = (!sent.borrow().is_empty()).then(|| still_owed(&sealed, &sent.borrow()));
            delivered(
                &door,
                &home,
                Some(&pressed_id),
                &keep_noting,
                owed.as_ref().unwrap_or(&sealed),
                &starts,
                ttl_days,
            )
        };
        sent_or_put_back(&door, &owned, deliver)
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
    let pending =
        look && !was.read_only() && !unchanged_here(&session, asked_again(doc, &was)?).await?;
    Ok(Waiting {
        until: Some(until * 1000),
        pending,
    })
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

/// The documents of this share whose permit the owner can put back, each once.
pub(super) fn owned_with_tokens(
    was: &Remembered,
    sealed: &tisty_sharing::Sealed,
) -> Vec<Remembered> {
    let mut owned: Vec<Remembered> = Vec::new();
    for one in std::iter::once(was).chain(sealed.remembering.iter().map(|(_, one)| one)) {
        if one.admin_token.is_some()
            && one.edit_token.is_some()
            && owned.iter().all(|kept| kept.id != one.id)
        {
            owned.push(one.clone());
        }
    }
    owned
}

pub(super) fn still_owed(
    sealed: &tisty_sharing::Sealed,
    sent: &std::collections::BTreeSet<String>,
) -> tisty_sharing::Sealed {
    tisty_sharing::Sealed {
        going: sealed
            .going
            .iter()
            .filter(|one| !sent.contains(&one.id))
            .cloned()
            .collect(),
        told: sealed.told.clone(),
        remembering: sealed.remembering.clone(),
        anchors: sealed.anchors.clone(),
        changed: sealed.changed.clone(),
        granted: sealed.granted.clone(),
    }
}

pub(super) fn kept_after(was: &Remembered, opened: &Opened, received: &Received) -> Remembered {
    let mut now = was.clone();
    now.version = opened.version;
    now.sealed_under = opened.contract.clone();
    now.version_by = owner_apart(opened).and(received.signed_as.clone());
    now.version_root = owner_apart(opened).map(|_| opened.signer.clone());
    // Folio holds whoever edits to the block of the latest version, which moves when its owner changes machine, name or link.
    if was.admin_token.is_none() {
        now.owner.clone_from(&opened.owner);
        now.owner_name.clone_from(&received.owner_name);
    }
    now.carried =
        tisty_sharing::editors_brought(was.carried.as_deref(), received.carried.as_deref());
    if was.admin_token.is_none() {
        now.carried =
            tisty_sharing::map_brought(now.carried.as_deref(), received.carried.as_deref());
    }
    now.files_print = None;
    now.files_hash = None;
    now.files_key = None;
    now
}

// What another sent stays apart, and one's own changes wait until the person chooses and presses again (149).
async fn other_version_brought(
    session: &tauri::State<'_, Mutex<Session>>,
    id: &str,
    was: &Remembered,
) -> Answer<Synced> {
    let (shared_id, key) = (was.id.clone(), was.link_key.clone().unwrap_or_default());
    let home = held(session).paths.private();
    let opened = apart(move || {
        let door = ToFolio::new(&home)?;
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
    let kept = other_version_kept(
        &mut session,
        &file,
        received.body.clone(),
        Some(marked),
        Arrived::Folio {
            root: &opened.signer,
            name: received.signed_as.as_deref(),
        },
    )?;
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
    let now = kept_after(was, &opened, &received);
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
