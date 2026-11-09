use std::sync::Mutex;

use serde::Serialize;
use tisty_carrier::folio_client::{self, Door};
use tisty_carrier::folio_limits;
use tisty_core::event::Signature;
use tisty_envelope::sealing::{link_key, permit_shown};
use tisty_sharing::{Remembered, Told};

use super::folio::{Card, ToFolio, apart, card_of, kept_on, remembered_of, said};
use super::folio_sync::{Synced, doc_share_sync};
use crate::{Answer, Refusal, Session, held};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Named {
    name: String,
    checked: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct Who {
    pub owner: Option<Named>,
    pub signer: Option<Named>,
    pub editors: Vec<Named>,
}

fn editors_carried(was: &Remembered) -> Vec<tisty_envelope::Editor> {
    was.carried
        .as_deref()
        .and_then(|said| serde_json::from_str::<serde_json::Value>(said).ok())
        .and_then(|said| serde_json::from_value(said["editors"].clone()).ok())
        .unwrap_or_default()
}

// Only the maker's root reads as checked for now: a name alone proves nothing (144).
pub(super) fn who_of(was: &Remembered, maker: Option<&str>) -> Who {
    let editors = editors_carried(was);
    let checked = |root: Option<&str>| root.is_some_and(|one| Some(one) == maker);
    let owner_root = was
        .owner
        .as_deref()
        .and_then(|block| serde_json::from_str::<serde_json::Value>(block).ok())
        .and_then(|block| block["root"].as_str().map(str::to_owned));
    let owner = was
        .owner_name
        .clone()
        .filter(|_| was.admin_token.is_none())
        .map(|name| Named {
            checked: checked(owner_root.as_deref()),
            name,
        });
    let signer = was.version_by.clone().map(|name| Named {
        checked: checked(was.version_root.as_deref()),
        name,
    });
    let editors = editors
        .iter()
        .filter(|one| one.r#in == was.id && one.owned != Some(true))
        .map(|one| Named {
            name: one.name.clone(),
            checked: checked(Some(&one.root)),
        })
        .collect();
    Who {
        owner,
        signer,
        editors,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignsAs {
    now: Option<String>,
    other: Option<String>,
}

fn names_of(signature: &Signature) -> Vec<String> {
    let mut names: Vec<String> = [&signature.alias, &signature.name]
        .into_iter()
        .flatten()
        .filter(|one| !one.trim().is_empty())
        .cloned()
        .collect();
    names.dedup();
    names
}

pub(super) fn signs_as(was: &Remembered, signature: &Signature) -> Option<SignsAs> {
    if !was.edited_through_a_link() {
        return None;
    }
    // The same name sealing falls back to, so what the row says is what travels.
    let now = was
        .signed_as
        .clone()
        .filter(|one| !one.trim().is_empty())
        .or_else(|| signature.alias.clone());
    let other = names_of(signature)
        .into_iter()
        .find(|one| Some(one) != now.as_ref());
    Some(SignsAs { now, other })
}

#[tauri::command]
pub fn doc_share_sign_as(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    name: String,
) -> Answer<Card> {
    let maker = folio_limits::maker_root(&held(&session).paths.private());
    let mut session = held(&session);
    let (file, mut was) = remembered_of(&session, &id)?;
    if signs_as(&was, &session.state.signed).is_none() {
        return Err(Refusal::of("notOneOwnDoc"));
    }
    // A name that stopped being one's own since the row was drawn changes nothing, and the row is drawn again.
    if names_of(&session.state.signed).contains(&name) {
        was.signed_as = Some(name);
        kept_on(&mut session, &[(file, was.clone())])?;
    }
    card_of(&was, &session.state.signed, maker.as_deref())
        .ok_or_else(|| Refusal::of("folioGarbled"))
}

// Said before sending and answered once a press: what follows the upload is no time to ask (140).
pub(super) fn others_asked(told: &[Told]) -> Vec<String> {
    told.iter()
        .find_map(|one| match one {
            Told::OthersEdited { editors } => Some(editors.clone()),
            _ => None,
        })
        .unwrap_or_default()
}

pub(super) fn said_after_sending(told: Vec<Told>) -> Vec<Told> {
    told.into_iter()
        .filter(|one| !matches!(one, Told::OthersEdited { .. }))
        .collect()
}

// Folio keeps the permit its owner last set, and this Tisty holds the token: its word puts them back in step (148).
pub(super) fn permit_put_back(door: &dyn Door, was: &Remembered) -> bool {
    let (Some(admin), Some(token)) = (was.admin_token.as_deref(), was.edit_token.as_deref()) else {
        return false;
    };
    let Ok(permit) = permit_shown(token) else {
        return false;
    };
    folio_client::permit(door, &was.id, admin, Some(&permit)).is_ok()
}

// The new token is kept before Folio hears of it: if anything stops halfway, the next Sync finishes the change.
#[tauri::command]
pub async fn doc_share_edit_link_renewed(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Synced> {
    let (file, was) = remembered_of(&held(&session), &id)?;
    if was.admin_token.is_none() || was.edit_token.is_none() {
        return Err(Refusal::of("notOneOwnDoc"));
    }
    let token = link_key().map_err(|_| Refusal::of("cannotSeal"))?;
    let permit = permit_shown(&token).map_err(|_| Refusal::of("cannotSeal"))?;
    let admin = was.admin_token.clone().unwrap_or_default();
    let shared = was.id.clone();
    let mut now = was;
    now.edit_token = Some(token);
    now.words_print = None;
    kept_on(&mut held(&session), &[(file, now)])?;
    apart(move || {
        let door = ToFolio::new()?;
        folio_client::permit(&door, &shared, &admin, Some(&permit)).map_err(|one| said(&one))
    })
    .await?;
    doc_share_sync(app, session, id, None).await
}

#[cfg(test)]
#[path = "folio_editing_test.rs"]
mod tests;
