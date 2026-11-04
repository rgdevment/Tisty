use std::path::Path;
use std::sync::Mutex;

use serde::Serialize;
use tisty_core::model::DocId;
use tisty_core::witness::{self, Fact, channel};
use tisty_envelope::sealing::Writer;
use tisty_sharing::{Asked, Desk, Told, Unshared, gathered, signer};

use crate::{Answer, Refusal, Session, held};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sealed {
    envelopes: usize,
    bytes: u64,
    told: Vec<Told>,
}

fn refused(wrong: &Unshared) -> Refusal {
    witness::warn(
        channel::WINDOW,
        "a document could not be sealed",
        &[("why", Fact::Why(wrong.to_string()))],
    );
    Refusal::of(match wrong {
        Unshared::NoSuchDoc => "noSuchDoc",
        Unshared::NotOwn => "notOneOwnDoc",
        Unshared::Shut => "documentLocked",
        Unshared::Away => "awayNotShared",
        Unshared::NotADocument => "onlyADocumentShared",
        Unshared::Unfinished(_) => "cannotFinish",
        Unshared::Unread(_) | Unshared::Unwritten(_) | Unshared::Refused(_) => "cannotSeal",
    })
}

fn doc_id(said: &str) -> Answer<DocId> {
    said.parse().map_err(|_| Refusal::of("noSuchDoc"))
}

// Nothing goes anywhere and nothing is remembered: that is for whoever sends them.
#[tauri::command]
pub async fn doc_sealed(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    public: bool,
    cited: Vec<String>,
    signed_as: Option<String>,
) -> Answer<Sealed> {
    let asked = Asked {
        doc: doc_id(&id)?,
        public,
        cited: cited
            .iter()
            .map(|one| doc_id(one))
            .collect::<Answer<Vec<_>>>()?,
        signed_as,
    };
    // A copy, so a picture being made anew never holds up whoever is writing.
    let (paths, state, device, also) = {
        let session = held(&session);
        (
            session.paths.clone(),
            session.state.clone(),
            session.config.device_id.clone(),
            session.dest(),
        )
    };
    // Asked first: with nothing to sign with, nothing else is worth doing.
    let (key, certificate) =
        signer(&paths, &state, &device).ok_or_else(|| Refusal::of("noIdentityToSign"))?;
    let mut gathered = tauri::async_runtime::spawn_blocking(move || {
        let desk = Desk {
            paths: &paths,
            state: &state,
            also: also.as_deref(),
            turned: &turned,
        };
        gathered(&desk, &asked)
    })
    .await
    .map_err(|_| Refusal::of("cannotSeal"))?
    .map_err(|wrong| refused(&wrong))?;
    let answers = crate::finishing::in_hiding(&app, gathered.unfinished()).await?;
    gathered
        .finished(answers)
        .map_err(|wrong| refused(&wrong))?;
    let sealed = tauri::async_runtime::spawn_blocking(move || {
        let mut writer = Writer::new(&key, &certificate);
        gathered
            .sealed(&mut writer)
            .map_err(|wrong| refused(&wrong))
    })
    .await
    .map_err(|_| Refusal::of("cannotSeal"))??;
    Ok(summed_up(sealed))
}

fn summed_up(sealed: tisty_sharing::Sealed) -> Sealed {
    let sizes = sealed.going.iter().flat_map(|one| {
        std::iter::once(one.sealed.main.len()).chain(one.sealed.files.as_ref().map(Vec::len))
    });
    Sealed {
        envelopes: sizes.clone().count(),
        bytes: sizes.map(|one| one as u64).sum(),
        told: sealed.told,
    }
}

// The machine's own converter is only a way in for what `image` cannot read: its bytes are read and made anew.
fn turned(from: &Path) -> Option<Vec<u8>> {
    let made = crate::drawn::reshaped(from, "turned.jpg", false)?;
    let bytes = std::fs::read(&made).ok();
    if let Some(room) = made.parent() {
        let _ = std::fs::remove_dir_all(room);
    }
    bytes
}

#[cfg(test)]
#[path = "sharing_test.rs"]
mod tests;
