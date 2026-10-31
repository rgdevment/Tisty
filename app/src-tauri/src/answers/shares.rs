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
    /// None when neither the name nor the alias is fit to sign, so the key speaks alone.
    signs_as: Option<String>,
}

// The name the person signs with, or else their alias, held to what an envelope takes.
fn signing_name(signed: &tisty_core::event::Signature) -> Option<String> {
    [&signed.name, &signed.alias]
        .into_iter()
        .flatten()
        .map(|one| one.trim())
        .find(|one| tisty_share_proto::envelope::named_well(one))
        .map(str::to_owned)
}

#[tauri::command]
pub async fn share_look(session: tauri::State<'_, Mutex<Session>>, file: String) -> Answer<Look> {
    let (docs, signs_as) = {
        let session = held(&session);
        doc_of(&session, &file)?;
        (session.paths.docs(), signing_name(&session.state.signed))
    };
    // A long document takes a moment to read and look through, so neither holds the window or the session.
    apart(move || {
        let text = tisty_core::docs::read(&docs, &file)
            .map_err(|e| Refusal::about("cannotRead", e.to_string()))?;
        Ok(Look {
            secrets: secrets(&text),
            signs_as,
        })
    })
    .await
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
        let name = signing_name(&session.state.signed);
        (
            doc,
            text,
            voucher,
            sharing::taken_back_by(&session.state.persons, &me),
            name,
        )
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
