use std::collections::BTreeSet;
use std::sync::Mutex;

use std::path::{Path, PathBuf};

use serde::Serialize;
use tisty_carrier::folio_client;
use tisty_carrier::folio_reviewed::{self, Copy, Printed};
use tisty_contracts::Review;
use tisty_core::merge::{self, Changed};
use tisty_envelope::Translation;
use tisty_envelope::reviews::{self, Joining, Read, Signer};
use tisty_sharing::{Languages, Remembered};

use super::folio::{ToFolio, apart, kept_on, remembered_of};
use super::folio_receiving::{OTHER_LANGUAGES_AT_MOST, Opened, carried_map, opened_printed};
use super::folio_sync::owned_ids;
use crate::{Answer, Refusal, Session, held};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    document: String,
    lang: String,
    title: Option<String>,
    word: Word,
    print: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShownPicture {
    at: u64,
    path: String,
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
    copied: bool,
    changes: Option<Vec<Changed>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Word {
    Waiting,
    Kept,
    Changed,
    LeftOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageWord {
    document: String,
    lang: Option<String>,
    title: Option<String>,
    word: Word,
    mine: bool,
}

// Readers see only the version the owner kept, so one known here to differ waits for the owner again (166).
fn changed_since(known: &Prints, document: &str, kept: Option<&str>) -> bool {
    known
        .get(document)
        .is_some_and(|known| Some(known.print.as_str()) != kept)
}

type Prints = std::collections::BTreeMap<String, Printed>;

fn word_of(known: &Prints, document: &str, said: Option<&Review>) -> Word {
    match said {
        None => Word::Waiting,
        Some(one) if !one.kept => Word::LeftOut,
        Some(one) if changed_since(known, document, one.print.as_deref()) => Word::Changed,
        Some(_) => Word::Kept,
    }
}

// Someone who edits hears the owner's word on the language they added, from the reviews its anchor carries.
fn owners_word_on_mine(
    known: &Prints,
    shelf: &std::collections::BTreeMap<String, Remembered>,
    was: &Remembered,
) -> Option<LanguageWord> {
    was.admin_token.as_ref()?;
    let anchor_id = was.held_by.as_deref()?;
    let anchor = shelf
        .values()
        .find(|one| one.id == anchor_id && one.edited_through_a_link())?;
    let owner = super::folio_editing::root_of_the_block(anchor.owner.as_deref())?;
    let joining = Joining {
        anchor: anchor_id,
        owner: &owner,
        anchor_key: anchor.link_key.as_deref()?,
    };
    let reviews = tisty_sharing::reviews_carried(anchor.carried.as_deref());
    Some(LanguageWord {
        document: was.id.clone(),
        lang: None,
        title: None,
        word: word_of(
            known,
            &was.id,
            reviews::word_on(&joining, &reviews, &was.id).as_ref(),
        ),
        mine: true,
    })
}

pub(super) fn words_on_card(
    home: &Path,
    shelf: &std::collections::BTreeMap<String, Remembered>,
    was: &Remembered,
) -> Vec<LanguageWord> {
    let known = folio_reviewed::prints(home);
    if let Some(mine) = owners_word_on_mine(&known, shelf, was) {
        return vec![mine];
    }
    if was.admin_token.is_none() || was.link_key.is_none() {
        return Vec::new();
    }
    proposals_of(was, &owned_ids(shelf))
        .into_iter()
        .map(|one| LanguageWord {
            word: match one.word {
                Word::Kept if changed_since(&known, &one.document, one.print.as_deref()) => {
                    Word::Changed
                }
                word => word,
            },
            document: one.document,
            lang: Some(one.lang),
            title: one.title,
            mine: false,
        })
        .collect()
}

fn owned(session: &Session, id: &str) -> Answer<(String, Remembered)> {
    let (file, was) = remembered_of(session, id)?;
    if was.admin_token.is_none() || was.link_key.is_none() {
        return Err(Refusal::of("notOneOwnDoc"));
    }
    Ok((file, was))
}

fn latest_word<'a>(was: &Remembered, reviews: &'a [Review], document: &str) -> Option<&'a Review> {
    let of = reviews::of_language(was.link_key.as_deref().unwrap_or_default(), document)?;
    reviews
        .iter()
        .filter(|review| review.of == of)
        .max_by_key(|review| review.seq)
}

fn kept_print(was: &Remembered, reviews: &[Review], document: &str) -> Option<String> {
    latest_word(was, reviews, document)
        .filter(|review| review.kept)
        .and_then(|review| review.print.clone())
}

// What the owner left out is still its own to look at again, so it is named beside the map, once.
fn foreign_of(carried: &Languages, ours: &BTreeSet<&str>) -> Vec<Translation> {
    let mut seen = BTreeSet::new();
    carried
        .map
        .iter()
        .chain(&carried.set_aside)
        .filter(|one| !ours.contains(one.document.as_str()))
        .filter(|one| seen.insert(one.document.as_str()))
        .cloned()
        .collect()
}

fn proposals_in(was: &Remembered, carried: &Languages, ours: &BTreeSet<&str>) -> Vec<Proposal> {
    foreign_of(carried, ours)
        .into_iter()
        .map(|one| Proposal {
            word: match latest_word(was, &carried.reviews, &one.document) {
                None => Word::Waiting,
                Some(review) if review.kept => Word::Kept,
                Some(_) => Word::LeftOut,
            },
            print: kept_print(was, &carried.reviews, &one.document),
            document: one.document,
            lang: one.lang,
            title: one.title,
        })
        .collect()
}

pub(super) fn proposals_of(was: &Remembered, ours: &BTreeSet<&str>) -> Vec<Proposal> {
    let carried = tisty_sharing::languages_carried(was.carried.as_deref());
    proposals_in(was, &carried, ours)
}

fn kept_entries(was: &Remembered, ours: &BTreeSet<&str>) -> Vec<Translation> {
    let carried = tisty_sharing::languages_carried(was.carried.as_deref());
    let kept: BTreeSet<String> = proposals_in(was, &carried, ours)
        .into_iter()
        .filter(|one| one.word == Word::Kept)
        .map(|one| one.document)
        .collect();
    carried
        .map
        .into_iter()
        .filter(|one| kept.contains(&one.document) && one.key.is_some())
        .collect()
}

// A language brought back takes its place in the map again, so a full map refuses it before the owner's word is signed.
pub(super) fn room_for(was: &Remembered, document: &str, top: Option<usize>) -> Answer<()> {
    let map = carried_map(was);
    let most = top.map_or(OTHER_LANGUAGES_AT_MOST, |top| {
        top.saturating_sub(1).min(OTHER_LANGUAGES_AT_MOST)
    });
    match map.iter().any(|one| one.document == document) || map.len() < most {
        true => Ok(()),
        false => Err(Refusal::of("folioLanguagesFull")),
    }
}

// Asking after a version Folio already showed here would only bring the same print back.
pub(super) fn looked_again(
    door: &dyn folio_client::Door,
    home: &Path,
    entry: &Translation,
) -> bool {
    let Ok(latest) = folio_client::latest(door, &entry.document) else {
        return false;
    };
    let was = folio_reviewed::printed(home, &entry.document);
    if was
        .as_ref()
        .is_some_and(|one| one.version >= latest.version)
    {
        return false;
    }
    let key = entry.key.as_deref().unwrap_or_default();
    let Some((version, print)) = opened_printed(door, &entry.document, key)
        .ok()
        .and_then(|opened| Some((opened.version, opened.print?)))
    else {
        return false;
    };
    let changed = was.is_none_or(|one| one.print != print);
    folio_reviewed::print_known(home, &entry.document, Printed { version, print });
    changed
}

#[tauri::command]
pub async fn doc_languages_looked(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<bool> {
    let (home, anchor, kept) = {
        let session = held(&session);
        let (_, was) = owned(&session, &id)?;
        let shelf = tisty_sharing::remembered(&session.state);
        let kept = kept_entries(&was, &owned_ids(shelf));
        (session.paths.private(), was.id, kept)
    };
    let now = tisty_carrier::policy::now();
    if kept.is_empty() || !folio_reviewed::prints_due(&home, &anchor, now) {
        return Ok(false);
    }
    apart(move || {
        let door = ToFolio::new(&home)?;
        let looked: Vec<bool> = kept
            .iter()
            .map(|entry| looked_again(&door, &home, entry))
            .collect();
        folio_reviewed::prints_asked(&home, &anchor, now);
        Ok(looked.contains(&true))
    })
    .await
}

fn told_whole(copy: &Copy) -> String {
    match &copy.title {
        Some(title) => format!("{title}\n\n{}", copy.body),
        None => copy.body.clone(),
    }
}

// What changed is told against the copy kept with the owner's word, and only while that word still stands.
pub(super) fn copy_of_word(home: &Path, was: &Remembered, document: &str) -> Option<Copy> {
    let reviews = tisty_sharing::reviews_carried(was.carried.as_deref());
    let kept = kept_print(was, &reviews, document)?;
    folio_reviewed::kept_copy(home, document).filter(|one| one.print == kept)
}

pub(super) fn changes_since(copy: Option<&Copy>, now: &Copy) -> Option<Vec<Changed>> {
    let copy = copy.filter(|one| one.print != now.print)?;
    merge::changed(&told_whole(copy), &told_whole(now))
}

// Someone else's words stay here only while the share that brought them lasts.
pub(super) fn forgettable<'a>(
    gone: impl IntoIterator<Item = &'a Remembered>,
    ours: &BTreeSet<&str>,
) -> Vec<String> {
    let mut ids = Vec::new();
    for was in gone {
        ids.push(was.id.clone());
        let carried = tisty_sharing::languages_carried(was.carried.as_deref());
        ids.extend(
            foreign_of(&carried, ours)
                .into_iter()
                .map(|one| one.document),
        );
    }
    ids
}

pub(super) fn forgotten_here(home: &Path, ids: &[String]) {
    for id in ids {
        folio_reviewed::forgotten(home, id);
    }
}

fn copy_of(opened: &Opened) -> Copy {
    Copy {
        print: opened.print.clone().unwrap_or_default(),
        title: text(opened, "title"),
        body: text(opened, "body").unwrap_or_default(),
    }
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
    let carried = tisty_sharing::languages_carried(was.carried.as_deref());
    foreign_of(&carried, ours)
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

pub fn folder(cache: &Path) -> PathBuf {
    cache.join("reviewing")
}

// Only the language being reviewed is kept, and its pictures go to the window as files it reads, never inside an answer.
pub(super) fn pictures_kept(opened: &Opened, folder: &Path) -> Answer<Vec<ShownPicture>> {
    let _ = std::fs::remove_dir_all(folder);
    std::fs::create_dir_all(folder).map_err(|_| Refusal::of("folioGarbled"))?;
    let said = opened.document["pictures"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut kept = Vec::with_capacity(said.len());
    for (one, bytes) in said.iter().zip(&opened.pictures) {
        let at = one["at"].as_u64().unwrap_or_default();
        let ending = match one["kind"].as_str() {
            Some("image/jpeg") => "jpg",
            _ => "png",
        };
        let path = folder.join(format!("{at}.{ending}"));
        std::fs::write(&path, bytes).map_err(|_| Refusal::of("folioGarbled"))?;
        kept.push(ShownPicture {
            at,
            path: path.to_string_lossy().into_owned(),
        });
    }
    Ok(kept)
}

// The print the owner signs answers for the pictures too, so they are shown with the words.
#[tauri::command]
pub async fn doc_language_shown(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    document: String,
) -> Answer<Shown> {
    let (home, cache, was, entry) = {
        let session = held(&session);
        let (_, was, entry) = entry_here(&session, &id, &document)?;
        (
            session.paths.private(),
            session.paths.cache().to_path_buf(),
            was,
            entry,
        )
    };
    let (anchor, private) = (was.id.clone(), home.clone());
    let (opened, pictures) = apart(move || {
        let opened = its_language(private, anchor, entry)?;
        let pictures = pictures_kept(&opened, &folder(&cache))?;
        Ok((opened, pictures))
    })
    .await?;
    let now = copy_of(&opened);
    let version = opened.version;
    folio_reviewed::print_known(
        &home,
        &document,
        Printed {
            version,
            print: now.print.clone(),
        },
    );
    let copy = copy_of_word(&home, &was, &document);
    Ok(Shown {
        copied: copy.is_some(),
        changes: changes_since(copy.as_ref(), &now),
        lang: text(&opened, "lang"),
        title: now.title,
        by: text(&opened, "signed_as").or_else(|| text(&opened, "owner_name")),
        body: now.body,
        pictures,
        print: now.print,
        document,
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
        true => tisty_sharing::brought_back(carried.as_deref(), document),
        false => tisty_sharing::set_aside(carried.as_deref(), document),
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
        let home = session.paths.private();
        if keep {
            room_for(&was, &document, super::folio::languages_top(&home))?;
        }
        (home, was, entry)
    };
    let (decided, copy) = match keep {
        false => (None, None),
        true => {
            let key = entry.key.clone().unwrap_or_default();
            let (home, anchor) = (home.clone(), was.id.clone());
            let opened = apart(move || its_language(home, anchor, entry)).await?;
            (
                Some(as_shown(&opened, key, print.as_deref())?),
                Some(copy_of(&opened)),
            )
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
    kept_on(&mut session, &[(file, after)])?;
    match copy {
        Some(copy) => folio_reviewed::copy_kept(&home, &document, &copy),
        None => folio_reviewed::copy_dropped(&home, &document),
    }
    Ok(())
}

#[cfg(test)]
#[path = "folio_reviewing_test.rs"]
mod tests;
