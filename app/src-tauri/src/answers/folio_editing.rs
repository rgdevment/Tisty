use std::sync::Mutex;

use serde::Serialize;
use tisty_carrier::folio_limits;
use tisty_core::event::Signature;
use tisty_sharing::Remembered;

use super::folio::{Card, card_of, kept_on, remembered_of};
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

#[cfg(test)]
#[path = "folio_editing_test.rs"]
mod tests;
