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
    let root_named = |name: &str| {
        editors
            .iter()
            .find(|one| one.name == name)
            .map(|one| one.root.as_str())
    };
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
        checked: checked(root_named(&name)),
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
    now: String,
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

// Whoever edits through a link chooses the name its versions go under, as the owner did when sharing (132).
pub(super) fn signs_as(was: &Remembered, signature: &Signature) -> Option<SignsAs> {
    if !was.edited_through_a_link() {
        return None;
    }
    let names = names_of(signature);
    let now = was
        .signed_as
        .clone()
        .filter(|one| names.contains(one))
        .or_else(|| names.first().cloned())?;
    let other = names.into_iter().find(|one| *one != now);
    Some(SignsAs { now, other })
}

#[tauri::command]
pub fn doc_share_sign_as(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    name: String,
) -> Answer<Card> {
    let mut session = held(&session);
    let (file, mut was) = remembered_of(&session, &id)?;
    if signs_as(&was, &session.state.signed).is_none()
        || !names_of(&session.state.signed).contains(&name)
    {
        return Err(Refusal::of("folioGarbled"));
    }
    was.signed_as = Some(name);
    kept_on(&mut session, &[(file, was.clone())])?;
    let maker = folio_limits::maker_root(&session.paths.private());
    card_of(&was, &session.state.signed, maker.as_deref())
        .ok_or_else(|| Refusal::of("folioGarbled"))
}

#[cfg(test)]
#[path = "folio_editing_test.rs"]
mod tests;
