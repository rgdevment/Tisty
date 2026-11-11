use std::collections::BTreeSet;
use std::sync::Mutex;

use serde::Serialize;
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
pub struct Shown {
    document: String,
    lang: Option<String>,
    title: Option<String>,
    by: Option<String>,
    body: String,
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

fn entry_of(was: &Remembered, document: &str) -> Answer<Translation> {
    carried_map(was)
        .into_iter()
        .find(|one| one.document == document && one.key.is_some())
        .ok_or_else(|| Refusal::of("folioGone"))
}

// A map could name a document that is not this anchor's language, and the owner's word goes only on one that is.
fn its_language(
    door_home: std::path::PathBuf,
    anchor: String,
    entry: Translation,
) -> Answer<Opened> {
    let door = ToFolio::new(&door_home)?;
    let key = entry.key.unwrap_or_default();
    let opened = opened_printed(&door, &entry.document, &key)?;
    match opened.document["anchor"].as_str() == Some(anchor.as_str()) {
        true => Ok(opened),
        false => Err(Refusal::of("folioUnopened")),
    }
}

fn text(opened: &Opened, field: &str) -> Option<String> {
    opened.document[field].as_str().map(str::to_owned)
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

#[tauri::command]
pub async fn doc_language_shown(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    document: String,
) -> Answer<Shown> {
    let (home, was) = {
        let session = held(&session);
        (session.paths.private(), owned(&session, &id)?.1)
    };
    let entry = entry_of(&was, &document)?;
    let anchor = was.id.clone();
    let opened = apart(move || its_language(home, anchor, entry)).await?;
    Ok(Shown {
        document,
        lang: text(&opened, "lang"),
        title: text(&opened, "title"),
        by: text(&opened, "signed_as").or_else(|| text(&opened, "owner_name")),
        body: text(&opened, "body").unwrap_or_default(),
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

pub(super) fn reviewed_now(
    signer: &Signer,
    was: &Remembered,
    document: &str,
    decided: Option<&Decided>,
) -> Answer<Remembered> {
    let anchor_key = was.link_key.as_deref().unwrap_or_default();
    let of =
        reviews::of_language(anchor_key, document).ok_or_else(|| Refusal::of("folioGarbled"))?;
    let seq = reviews::next_seq(&tisty_sharing::reviews_carried(was.carried.as_deref()), &of);
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

// What the owner keeps is the version it was shown, so one that changed since is shown again before anything is signed.
#[tauri::command]
pub async fn doc_language_reviewed(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    document: String,
    keep: bool,
    print: Option<String>,
) -> Answer<()> {
    let (home, was) = {
        let session = held(&session);
        (session.paths.private(), owned(&session, &id)?.1)
    };
    let decided = match keep {
        false => None,
        true => {
            let entry = entry_of(&was, &document)?;
            let key = entry.key.clone().unwrap_or_default();
            let anchor = was.id.clone();
            let opened = apart(move || its_language(home, anchor, entry)).await?;
            if opened.print.is_none() || opened.print != print {
                return Err(Refusal::of("folioLanguageChanged"));
            }
            Some(Decided {
                lang: text(&opened, "lang").ok_or_else(|| Refusal::of("folioUnopened"))?,
                title: text(&opened, "title"),
                key,
                print: opened.print.unwrap_or_default(),
            })
        }
    };
    let mut session = held(&session);
    let (file, now) = owned(&session, &id)?;
    let device = session.config.device_id.clone();
    let (key, certificate) =
        tisty_core::person::signer(&session.paths, &session.state.persons, &device)
            .ok_or_else(|| Refusal::of("noIdentityToSign"))?;
    let signer = Signer {
        key: &key,
        certificate: &certificate,
    };
    let after = reviewed_now(&signer, &now, &document, decided.as_ref())?;
    kept_on(&mut session, &[(file, after)])
}

#[cfg(test)]
#[path = "folio_reviewing_test.rs"]
mod tests;
