use std::sync::Mutex;

use serde::Serialize;
use tisty_core::Op;
use tisty_core::event::Shared;
use tisty_core::model::DocId;

use crate::handed::Voucher;
use crate::sharing::{self, Chosen, Info, Net};
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
}

// The key this machine signs with and the person it answers for; without both nothing is shared.
fn sharer(session: &Session) -> Option<(Voucher, tisty_share_proto::identity::Person)> {
    let device = &session.config.device_id;
    let voucher = crate::handed::vouched_by(&session.paths, device, &session.state.persons)?;
    let me = session.state.persons.person(device)?;
    Some((voucher, me))
}

// Every key this person took back, the root's own signed word on each.
fn taken_back_by(
    session: &Session,
    me: &tisty_share_proto::identity::Person,
) -> Vec<tisty_share_proto::identity::Revocation> {
    session
        .state
        .persons
        .revocations
        .iter()
        .filter(|one| one.unchecked().is_some_and(|(person, _)| person == *me))
        .cloned()
        .collect()
}

pub(crate) fn shareable(session: &Session, kept: &tisty_core::model::Kept) -> Shareable {
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
    /// None when the server could not be reached; the links still work without it.
    info: Option<Info>,
}

fn card(shared: &Shared, info: Option<Info>) -> Card {
    let (read, edit) = sharing::links(shared);
    Card {
        read,
        edit,
        gated: shared.gated,
        version: shared.version,
        info,
    }
}

// The window knows a document by its file, as it reads and writes it.
fn doc_of(session: &Session, file: &str) -> Answer<DocId> {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .map(|one| one.id)
        .ok_or(Refusal::of("noSuchDoc"))
}

// The network is slow and the work takes a moment, so neither holds the window or the session.
async fn apart<T: Send + 'static>(work: impl FnOnce() -> Answer<T> + Send + 'static) -> Answer<T> {
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
}

fn secrets(text: &str) -> Vec<Secret> {
    tisty_core::agent::secrets_in(text.as_bytes())
        .into_iter()
        .map(|one| Secret {
            line: one.line,
            named: one.named,
        })
        .collect()
}

#[tauri::command]
pub fn share_secrets(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
) -> Answer<Vec<Secret>> {
    let session = held(&session);
    doc_of(&session, &file)?;
    let text = tisty_core::docs::read(&session.paths.docs(), &file)
        .map_err(|e| Refusal::about("cannotRead", e.to_string()))?;
    Ok(secrets(&text))
}

#[tauri::command]
pub async fn share_doc(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    chosen: Chosen,
) -> Answer<Card> {
    let (doc, text, voucher, revoked, name) = {
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
        }
        let text = tisty_core::docs::read(&session.paths.docs(), &file)
            .map_err(|e| Refusal::about("cannotRead", e.to_string()))?;
        let (voucher, me) = sharer(&session).ok_or(Refusal::of("shareNoKey"))?;
        // The name the person signs with, or else their alias; with neither one fit to sign, the key speaks alone.
        let signed = &session.state.signed;
        let name = [&signed.name, &signed.alias]
            .into_iter()
            .flatten()
            .map(|one| one.trim())
            .find(|one| tisty_share_proto::envelope::named_well(one))
            .map(str::to_owned);
        (doc, text, voucher, taken_back_by(&session, &me), name)
    };
    let (shared, info) = apart(move || {
        let wire = Net::to(sharing::SERVER)?;
        // What this person took back may not have reached the server when it happened, so every share tells it again.
        let _ = sharing::revoke(&wire, &revoked);
        let shared = sharing::share(
            &wire,
            sharing::SERVER,
            &text,
            &chosen,
            &voucher,
            name.as_deref(),
        )?;
        let info = sharing::info(&wire, &shared).ok();
        Ok((shared, info))
    })
    .await?;
    let committed = {
        let mut session = held(&session);
        // A second press or another window may have shared it while this one was on the way.
        match session.state.shares.contains_key(&doc) {
            true => Err(Refusal::of("shareAlready")),
            false => session
                .commit(Op::ShareDoc {
                    id: doc,
                    d: shared.clone(),
                })
                .map_err(Refusal::from),
        }
    };
    if let Err(refused) = committed {
        // A link nobody can see is a link nobody can stop, so it goes.
        let _ = apart(move || sharing::stop(&Net::to(&shared.server)?, &shared)).await;
        return Err(refused);
    }
    Ok(card(&shared, info))
}

#[tauri::command]
pub async fn share_card(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
) -> Answer<Option<Card>> {
    let shared = {
        let session = held(&session);
        session.state.shares.get(&doc_of(&session, &file)?).cloned()
    };
    let Some(shared) = shared else {
        return Ok(None);
    };
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
                    .commit(Op::ShareStop { id: doc })
                    .map_err(Refusal::from)?;
            }
            Ok(None)
        }
        info => Ok(Some(card(&shared, info.ok()))),
    }
}

#[tauri::command]
pub async fn share_stop(session: tauri::State<'_, Mutex<Session>>, file: String) -> Answer<()> {
    let (doc, shared) = {
        let session = held(&session);
        let doc = doc_of(&session, &file)?;
        (doc, session.state.shares.get(&doc).cloned())
    };
    let Some(shared) = shared else {
        return Ok(());
    };
    let gone = apart(move || sharing::stop(&Net::to(&shared.server)?, &shared)).await;
    // A share the server already let go of is stopped all the same.
    match gone {
        Err(refused) if !sharing::already_gone(&refused) => return Err(refused),
        _ => {}
    }
    held(&session)
        .commit(Op::ShareStop { id: doc })
        .map_err(Refusal::from)
}

#[cfg(test)]
#[path = "shares_test.rs"]
mod tests;
