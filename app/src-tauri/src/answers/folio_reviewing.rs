use std::collections::BTreeSet;
use std::sync::Mutex;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::Serialize;
use tisty_carrier::folio_reviewed;
use tisty_contracts::Review;
use tisty_envelope::Translation;
use tisty_envelope::reviews::{self, Read, Signer};
use tisty_sharing::Remembered;

use super::folio::{ToFolio, apart, kept_on, remembered_of};
use super::folio_receiving::{Opened, carried_map, opened_printed};
use super::folio_sync::owned_ids;
use crate::{Answer, Refusal, Session, held};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proposal {
    document: String,
    lang: String,
    title: Option<String>,
    kept: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShownPicture {
    at: u64,
    kind: String,
    data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Shown {
    document: String,
    lang: Option<String>,
    title: Option<String>,
    by: Option<String>,
    body: String,
    pictures: Vec<ShownPicture>,
    print: String,
}

fn owned(session: &Session, id: &str) -> Answer<(String, Remembered)> {
    let (file, was) = remembered_of(session, id)?;
    if was.admin_token.is_none() || was.link_key.is_none() {
        return Err(Refusal::of("notOneOwnDoc"));
    }
    Ok((file, was))
}

pub(super) fn proposals_of(was: &Remembered, ours: &BTreeSet<&str>) -> Vec<Proposal> {
    let key = was.link_key.as_deref().unwrap_or_default();
    let reviews = tisty_sharing::reviews_carried(was.carried.as_deref());
    carried_map(was)
        .into_iter()
        .filter(|one| !ours.contains(one.document.as_str()))
        .map(|one| {
            let of = reviews::of_language(key, &one.document);
            let latest = reviews
                .iter()
                .filter(|review| Some(&review.of) == of.as_ref())
                .max_by_key(|review| review.seq);
            Proposal {
                kept: latest.is_some_and(|review| review.kept),
                document: one.document,
                lang: one.lang,
                title: one.title,
            }
        })
        .collect()
}

// The owner's word goes only on a language others added that the map names, never on one of its own.
pub(super) fn foreign_entry(
    was: &Remembered,
    ours: &BTreeSet<&str>,
    document: &str,
) -> Answer<Translation> {
    if ours.contains(document) {
        return Err(Refusal::of("notOneOwnDoc"));
    }
    carried_map(was)
        .into_iter()
        .find(|one| one.document == document && one.key.is_some())
        .ok_or_else(|| Refusal::of("folioGone"))
}

fn entry_here(
    session: &Session,
    id: &str,
    document: &str,
) -> Answer<(String, Remembered, Translation)> {
    let (file, was) = owned(session, id)?;
    let shelf = tisty_sharing::remembered(&session.state);
    let entry = foreign_entry(&was, &owned_ids(shelf), document)?;
    Ok((file, was, entry))
}

// A map could name a document that is not this anchor's language, and the owner's word goes only on one that is.
pub(super) fn named_here(opened: Opened, anchor: &str) -> Answer<Opened> {
    match opened.document["anchor"].as_str() == Some(anchor) {
        true => Ok(opened),
        false => Err(Refusal::of("folioUnopened")),
    }
}

fn its_language(home: std::path::PathBuf, anchor: String, entry: Translation) -> Answer<Opened> {
    let door = ToFolio::new(&home)?;
    let key = entry.key.unwrap_or_default();
    named_here(opened_printed(&door, &entry.document, &key)?, &anchor)
}

fn text(opened: &Opened, field: &str) -> Option<String> {
    opened.document[field].as_str().map(str::to_owned)
}

pub(super) fn pictures_of(opened: &Opened) -> Vec<ShownPicture> {
    let said = opened.document["pictures"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    said.iter()
        .zip(&opened.pictures)
        .map(|(one, bytes)| ShownPicture {
            at: one["at"].as_u64().unwrap_or_default(),
            kind: one["kind"].as_str().unwrap_or_default().to_owned(),
            data: STANDARD.encode(bytes),
        })
        .collect()
}

#[tauri::command]
pub fn doc_language_proposals(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Vec<Proposal>> {
    let session = held(&session);
    let (_, was) = owned(&session, &id)?;
    let shelf = tisty_sharing::remembered(&session.state);
    Ok(proposals_of(&was, &owned_ids(shelf)))
}

// The print the owner signs answers for the pictures too, so they are shown with the words.
#[tauri::command]
pub async fn doc_language_shown(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    document: String,
) -> Answer<Shown> {
    let (home, was, entry) = {
        let session = held(&session);
        let (_, was, entry) = entry_here(&session, &id, &document)?;
        (session.paths.private(), was, entry)
    };
    let anchor = was.id.clone();
    let opened = apart(move || its_language(home, anchor, entry)).await?;
    Ok(Shown {
        document,
        lang: text(&opened, "lang"),
        title: text(&opened, "title"),
        by: text(&opened, "signed_as").or_else(|| text(&opened, "owner_name")),
        body: text(&opened, "body").unwrap_or_default(),
        pictures: pictures_of(&opened),
        print: opened.print.clone().unwrap_or_default(),
    })
}

pub(super) fn carried_after(was: &Remembered, document: &str, review: Review) -> Option<String> {
    let kept = review.kept;
    let reviews = reviews::with_review(
        &tisty_sharing::reviews_carried(was.carried.as_deref()),
        review,
    );
    let carried = tisty_sharing::with_reviews(was.carried.as_deref(), &reviews);
    match kept {
        true => carried,
        false => tisty_sharing::languages_gone(
            carried.as_deref(),
            &BTreeSet::from([document.to_owned()]),
            &BTreeSet::new(),
        ),
    }
}

pub(super) struct Decided {
    pub lang: String,
    pub title: Option<String>,
    pub key: String,
    pub print: String,
}

// What the owner keeps is the version it was shown, so one that changed since is shown again before anything is signed.
pub(super) fn as_shown(opened: &Opened, key: String, shown: Option<&str>) -> Answer<Decided> {
    let print = opened
        .print
        .clone()
        .filter(|print| Some(print.as_str()) == shown)
        .ok_or_else(|| Refusal::of("folioLanguageChanged"))?;
    Ok(Decided {
        lang: text(opened, "lang").ok_or_else(|| Refusal::of("folioUnopened"))?,
        title: text(opened, "title"),
        key,
        print,
    })
}

// Another of the owner's machines may have said more of this language than this one holds, so the word counts above every word seen.
pub(super) fn reviewed_now(
    signer: &Signer,
    was: &Remembered,
    document: &str,
    decided: Option<&Decided>,
    at_least: u64,
) -> Answer<Remembered> {
    let anchor_key = was.link_key.as_deref().unwrap_or_default();
    let of =
        reviews::of_language(anchor_key, document).ok_or_else(|| Refusal::of("folioGarbled"))?;
    let reviews = tisty_sharing::reviews_carried(was.carried.as_deref());
    let seq = reviews::next_seq(&reviews, &of).max(at_least);
    let read = decided.map(|one| Read {
        lang: &one.lang,
        title: one.title.as_deref(),
        key: &one.key,
        print: &one.print,
    });
    let review = reviews::review_signed(signer, &was.id, anchor_key, document, read.as_ref(), seq)
        .ok_or_else(|| Refusal::of("cannotSeal"))?;
    Ok(Remembered {
        carried: carried_after(was, document, review),
        ..was.clone()
    })
}

#[tauri::command]
pub async fn doc_language_reviewed(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    document: String,
    keep: bool,
    print: Option<String>,
) -> Answer<()> {
    let (home, was, entry) = {
        let session = held(&session);
        let (_, was, entry) = entry_here(&session, &id, &document)?;
        (session.paths.private(), was, entry)
    };
    let decided = match keep {
        false => None,
        true => {
            let key = entry.key.clone().unwrap_or_default();
            let (home, anchor) = (home.clone(), was.id.clone());
            let opened = apart(move || its_language(home, anchor, entry)).await?;
            Some(as_shown(&opened, key, print.as_deref())?)
        }
    };
    let mut session = held(&session);
    let (file, now, _) = entry_here(&session, &id, &document)?;
    let device = session.config.device_id.clone();
    let (key, certificate) =
        tisty_core::person::signer(&session.paths, &session.state.persons, &device)
            .ok_or_else(|| Refusal::of("noIdentityToSign"))?;
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };
    let at_least = folio_reviewed::seen(&home, &now.id) + 1;
    let after = reviewed_now(&signer, &now, &document, decided.as_ref(), at_least)?;
    kept_on(&mut session, &[(file, after)])
}

#[cfg(test)]
#[path = "folio_reviewing_test.rs"]
mod tests;
