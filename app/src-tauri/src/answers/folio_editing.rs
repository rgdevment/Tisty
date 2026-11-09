use std::sync::Mutex;

use serde::Serialize;
use tisty_core::event::Signature;
use tisty_sharing::Remembered;

use super::folio::{Card, card_of, kept_on, remembered_of};
use crate::{Answer, Refusal, Session, held};

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
    card_of(&was, &session.state.signed).ok_or_else(|| Refusal::of("folioGarbled"))
}

#[cfg(test)]
#[path = "folio_editing_test.rs"]
mod tests;
