use std::sync::Mutex;

use serde::Serialize;
use tisty_core::Op;
use tisty_core::event::Shared;
use tisty_core::model::DocId;

use crate::sharing::{self, Chosen, Info, Net};
use crate::{Answer, Refusal, Session, held};

/// What the «Shared» card shows: the links to hand out, and how it stands when the server could be asked.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    read: String,
    edit: Option<String>,
    gated: bool,
    signed: bool,
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
        signed: shared.signed,
        version: shared.version,
        info,
    }
}

fn doc_id(id: &str) -> Answer<DocId> {
    id.parse().map_err(|_| Refusal::of("noSuchDoc"))
}

// The network is slow and the work takes a moment, so neither holds the window or the session.
async fn apart<T: Send + 'static>(work: impl FnOnce() -> Answer<T> + Send + 'static) -> Answer<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|_| Refusal::of("internal"))?
}

#[tauri::command]
pub async fn share_doc(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    chosen: Chosen,
) -> Answer<Card> {
    let doc = doc_id(&id)?;
    let (text, voucher) = {
        let session = held(&session);
        if !session.state.docs.contains_key(&doc) {
            return Err(Refusal::of("noSuchDoc"));
        }
        if session.state.shares.contains_key(&doc) {
            return Err(Refusal::of("shareAlready"));
        }
        let text = tisty_core::docs::read(&session.paths.docs(), &id)
            .map_err(|e| Refusal::about("cannotRead", e.to_string()))?;
        let voucher = chosen.signed.then(|| {
            crate::handed::vouched_by(
                &session.paths,
                &session.config.device_id,
                &session.state.persons,
            )
        });
        (text, voucher)
    };
    // Asked to sign with no key a person vouches for, it says so rather than going out unsigned.
    let voucher = match voucher {
        Some(None) => return Err(Refusal::of("shareNoKey")),
        Some(Some(one)) => Some(one),
        None => None,
    };
    let shared = apart(move || {
        let wire = Net::to(sharing::SERVER)?;
        sharing::share(&wire, sharing::SERVER, &text, &chosen, voucher.as_ref())
    })
    .await?;
    held(&session)
        .commit(Op::ShareDoc {
            id: doc,
            d: shared.clone(),
        })
        .map_err(Refusal::from)?;
    Ok(card(&shared, None))
}

#[tauri::command]
pub async fn share_card(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<Card>> {
    let doc = doc_id(&id)?;
    let Some(shared) = held(&session).state.shares.get(&doc).cloned() else {
        return Ok(None);
    };
    let asked = shared.clone();
    let info = apart(move || sharing::info(&Net::to(&asked.server)?, &asked))
        .await
        .ok();
    Ok(Some(card(&shared, info)))
}

#[tauri::command]
pub async fn share_stop(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let doc = doc_id(&id)?;
    let Some(shared) = held(&session).state.shares.get(&doc).cloned() else {
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
