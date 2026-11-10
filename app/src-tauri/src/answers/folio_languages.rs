use std::collections::BTreeSet;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tisty_carrier::folio_client::Door;
use tisty_core::State;
use tisty_envelope::Translation;
use tisty_sharing::Remembered;

use super::folio::{ToFolio, apart, kept_on, remembered_of};
use super::folio_receiving::{
    Opened, all_kept_as, kept_here, opened_from, reviewed, saying_its_language,
};
use super::folio_sync::{Synced, doc_share_sync};
use super::sharing::doc_id;
use crate::{Answer, Refusal, Session, held};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Proposed {
    lang: String,
    document: String,
    key: String,
    by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedShown {
    lang: String,
    by: Option<String>,
}

pub(super) fn proposed_of(was: &Remembered) -> Vec<Proposed> {
    was.carried
        .as_deref()
        .and_then(|said| serde_json::from_str::<Value>(said).ok())
        .and_then(|said| serde_json::from_value(said["proposed"].clone()).ok())
        .unwrap_or_default()
}

pub(super) fn shown_of(was: &Remembered) -> Vec<ProposedShown> {
    proposed_of(was)
        .into_iter()
        .map(|one| ProposedShown {
            lang: one.lang,
            by: one.by,
        })
        .collect()
}

fn with_proposed(carried: Option<&str>, proposed: &[Proposed]) -> Option<String> {
    let mut kept = carried
        .and_then(|said| serde_json::from_str::<Value>(said).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| json!({}));
    let fields = kept.as_object_mut()?;
    match proposed.is_empty() {
        true => fields.remove("proposed"),
        false => fields.insert("proposed".into(), serde_json::to_value(proposed).ok()?),
    };
    (!fields.is_empty()).then(|| kept.to_string())
}

pub(super) fn proposals_merged(was: &Remembered, brought: Vec<Proposed>) -> Option<String> {
    let mut all = proposed_of(was);
    all.retain(|one| brought.iter().all(|now| now.lang != one.lang));
    all.extend(brought);
    with_proposed(was.carried.as_deref(), &all)
}

pub(super) fn held_ids(state: &State, was: &Remembered) -> BTreeSet<String> {
    tisty_sharing::remembered(state)
        .values()
        .filter(|one| one.held_by.as_deref() == Some(&was.id))
        .map(|one| one.id.clone())
        .chain(std::iter::once(was.id.clone()))
        .collect()
}

// A language the owner never sent and never reviewed waits for them: someone who edits added it (166).
pub(super) fn proposals_in(
    door: &dyn Door,
    opened: &Opened,
    held: &BTreeSet<String>,
) -> Vec<Proposed> {
    let map: Vec<Translation> =
        serde_json::from_value(opened.document["languages"].clone()).unwrap_or_default();
    map.into_iter()
        .filter(|one| !held.contains(&one.document))
        .filter_map(|one| {
            let key = one.key.clone()?;
            let its = opened_from(door, &one.document, &key).ok()?;
            let by = its.document["signed_as"]
                .as_str()
                .or(its.document["owner_name"].as_str())
                .map(str::to_owned);
            (!reviewed(opened, &its)).then_some(Proposed {
                lang: one.lang,
                document: one.document,
                key,
                by,
            })
        })
        .collect()
}

fn proposal(session: &Session, id: &str, lang: &str) -> Answer<(String, Remembered, Proposed)> {
    let (file, was) = remembered_of(session, id)?;
    if was.admin_token.is_none() {
        return Err(Refusal::of("notOneOwnDoc"));
    }
    let one = proposed_of(&was)
        .into_iter()
        .find(|one| one.lang == lang)
        .ok_or_else(|| Refusal::of("folioGone"))?;
    Ok((file, was, one))
}

fn without(was: &Remembered, lang: &str) -> Remembered {
    let left: Vec<Proposed> = proposed_of(was)
        .into_iter()
        .filter(|one| one.lang != lang)
        .collect();
    Remembered {
        carried: with_proposed(was.carried.as_deref(), &left),
        ..was.clone()
    }
}

#[tauri::command]
pub async fn doc_share_language_adopted(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    lang: String,
) -> Answer<Synced> {
    let (file, was, one) = proposal(&held(&session), &id, &lang)?;
    let (home, data) = {
        let session = held(&session);
        (session.paths.private(), session.paths.data().to_path_buf())
    };
    let its = apart(move || {
        let door = ToFolio::new(&home)?;
        opened_from(&door, &one.document, &one.key)
    })
    .await?;
    let (received, told) = apart(move || Ok(kept_here(&data, &its))).await?;
    let received = saying_its_language(received, Some(&lang));
    {
        let mut session = held(&session);
        session.commit_all(told)?;
        let doc = doc_id(&id)?;
        let anchor = session
            .state
            .docs
            .get(&doc)
            .map(|kept| session.state.anchor_of(kept))
            .ok_or_else(|| Refusal::of("noSuchDoc"))?;
        let by = session.state.signed.alias.clone();
        let (adopted, _) = all_kept_as(&mut session, &received, (false, by))?;
        let joined = session.state.joining(adopted, anchor);
        session.commit_all(joined)?;
        kept_on(&mut session, &[(file, without(&was, &lang))])?;
    }
    doc_share_sync(app, session, id, None).await
}

#[tauri::command]
pub fn doc_share_language_dropped(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    lang: String,
) -> Answer<()> {
    let mut session = held(&session);
    let (file, was, _) = proposal(&session, &id, &lang)?;
    kept_on(&mut session, &[(file, without(&was, &lang))])
}

#[cfg(test)]
#[path = "folio_languages_test.rs"]
mod tests;
