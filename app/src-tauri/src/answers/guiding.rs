use std::path::PathBuf;

use tisty_core::Op;
use tisty_core::model::DocId;
use tisty_core::untouched::{GUIDE_HUE, GUIDE_ICON};

use super::settings::{ATTACHED, GUIDE_EN, GUIDE_ES, GUIDE_PAGES_EN, GUIDE_PAGES_ES, WRITTEN_BY};
use crate::{Answer, Refusal, Session};

/// One language the guide ships in: its cover, and its pages behind the markers the cover names them by.
pub struct Spoken {
    pub tongue: &'static str,
    pub told: &'static str,
    pub leaves: &'static [(&'static str, &'static str)],
}

pub const SPOKEN: [Spoken; 2] = [
    Spoken {
        tongue: "es",
        told: GUIDE_ES,
        leaves: GUIDE_PAGES_ES,
    },
    Spoken {
        tongue: "en",
        told: GUIDE_EN,
        leaves: GUIDE_PAGES_EN,
    },
];

/// The guide in every language it ships in, held as one, the person's own first and answered.
pub fn sown(
    session: &mut Session,
    first: &str,
    called: &str,
    by: &str,
    from: impl Fn(&str) -> Option<PathBuf>,
) -> Answer<tisty_core::docs::Doc> {
    let mut spoken: Vec<&Spoken> = SPOKEN.iter().collect();
    spoken.sort_by_key(|one| one.tongue != first);
    let (mut told_of, named) = pictured(session, &spoken, &from)?;

    let folder = ulid::Ulid::generate();
    let order = tisty_core::order::last_of(
        session
            .state
            .under(None)
            .iter()
            .map(|one| one.order.as_str()),
    );
    told_of.push(Op::FolderAdd {
        id: folder,
        d: tisty_core::event::FolderAdd {
            name: called.to_string(),
            order,
            parent: None,
            icon: Some(GUIDE_ICON.to_string()),
            color: Some(GUIDE_HUE.to_string()),
        },
    });
    session.commit_all(told_of)?;

    let mut covers: Vec<(DocId, tisty_core::docs::Doc)> = Vec::new();
    for (one, shots) in spoken.iter().zip(&named) {
        covers.push(one_language(session, one, shots, folder, by)?);
    }
    let Some(((anchor, cover), rest)) = covers.split_first() else {
        return Err(Refusal::of("noSuchDoc"));
    };
    let joined: Vec<Op> = rest
        .iter()
        .map(|(id, _)| Op::DocTongue {
            id: *id,
            d: tisty_core::event::Tongue { of: Some(*anchor) },
        })
        .collect();
    session.commit_all(joined)?;
    let written = cover.id.clone();
    session.keep(|c| c.guide = Some(written))?;
    Ok(cover.clone())
}

/// The guide in one more language, beside the one already here and joined to it.
pub fn sown_beside(
    session: &mut Session,
    tongue: &str,
    beside: DocId,
    by: &str,
    from: impl Fn(&str) -> Option<PathBuf>,
) -> Answer<tisty_core::docs::Doc> {
    let one = SPOKEN
        .iter()
        .find(|one| one.tongue == tongue)
        .ok_or_else(|| Refusal::of("noSuchDoc"))?;
    let folder = session
        .state
        .docs
        .get(&beside)
        .and_then(|one| one.folder)
        .ok_or_else(|| Refusal::of("noSuchDoc"))?;
    let (told_of, named) = pictured(session, &[one], &from)?;
    session.commit_all(told_of)?;
    let (cover, made) = one_language(session, one, &named[0], folder, by)?;
    let ops = session.state.joining(cover, beside);
    session.commit_all(ops)?;
    let written = made.id.clone();
    session.keep(|c| c.guide = Some(written))?;
    Ok(made)
}

// The pictures each language names, kept once when two languages show the same one.
type Pictured = (Vec<Op>, Vec<Vec<(String, String)>>);

fn pictured(
    session: &Session,
    spoken: &[&Spoken],
    from: &impl Fn(&str) -> Option<PathBuf>,
) -> Answer<Pictured> {
    let data = session.paths.data().to_path_buf();
    let mut told_of = Vec::new();
    let mut kept_at = Vec::new();
    let mut named = Vec::new();
    for one in spoken {
        let mut here = Vec::new();
        for shot in ATTACHED {
            let Some(at) = from(one.tongue)
                .map(|dir| dir.join(shot))
                .filter(|at| at.is_file())
            else {
                continue;
            };
            let kept = tisty_core::attach::keep(&at, &data, tisty_core::attach::COPIED_IN_DOC)
                .map_err(|e| Refusal::about("cannotRead", e.to_string()))?;
            if !kept_at.contains(&kept.at) {
                told_of.push(kept.told(tisty_core::attach::Via::Guide));
                kept_at.push(kept.at.clone());
            }
            let leaf = tisty_core::attach::leaf_of(&kept.at).unwrap_or(&kept.at);
            here.push((shot.to_string(), leaf.to_string()));
        }
        named.push(here);
    }
    Ok((told_of, named))
}

/// Old guides nobody wrote in say their language and join; a store that still reads as new is left so.
pub fn spoken_again(session: &mut Session) {
    let root = session.paths.docs();
    let mut covers: Vec<(DocId, &str)> = Vec::new();
    for one in &SPOKEN {
        let found = session.state.papers().find(|kept| {
            kept.by.as_deref() == Some(WRITTEN_BY)
                && kept.guest
                && kept.page_of.is_none()
                && kept
                    .title
                    .as_deref()
                    .and_then(tisty_core::untouched::guide_tongue)
                    == Some(one.tongue)
                && !session.state.held_away(kept)
        });
        if let Some(kept) = found {
            covers.push((kept.id, one.tongue));
        }
    }
    let unsaid = |id: &DocId| {
        session
            .state
            .docs
            .get(id)
            .is_some_and(|one| one.lang.is_none() && one.wrote == one.made)
    };
    let saying = |id: &DocId| {
        unsaid(id)
            || session
                .state
                .docs
                .get(id)
                .is_some_and(|one| one.lang.is_some())
    };
    // Two that can never say their language are never joined, so they are no reason to look.
    let apart = covers.len() > 1
        && session.state.tongues_of(covers[0].0).is_empty()
        && covers.iter().all(|(id, _)| saying(id));
    // Reading the whole log to know the store is new is paid only when an old guide is there.
    if !covers.iter().any(|(id, _)| unsaid(id)) && !apart
        || tisty_core::untouched::untouched(&session.paths.store(), &root)
    {
        return;
    }
    for (id, tongue) in &covers {
        let Some(kept) = session.state.docs.get(id) else {
            continue;
        };
        // Only words nobody wrote in since they were sown: a guide someone made their own stays theirs.
        if kept.lang.is_some() || kept.wrote != kept.made {
            continue;
        }
        let file = kept.file.clone();
        let written = tisty_core::docs::amend(&root, session.paths.data(), &file, |was| {
            Some(tisty_markdown::front::with_field(was, "lang", tongue))
        });
        if let Ok(Some(body)) = written {
            session.retell(&file, &body, None);
        }
    }
    let saying: Vec<(DocId, tisty_core::model::Kept)> = covers
        .iter()
        .filter_map(|(id, _)| session.state.docs.get(id).cloned().map(|one| (*id, one)))
        .filter(|(_, one)| one.lang.is_some())
        .collect();
    if let [(first, _), (other, _)] = saying.as_slice()
        && session.state.tongues_of(*first).is_empty()
    {
        let ops = session.state.joining(*other, *first);
        let _ = session.commit_all(ops);
    }
}

// A title after the path, as in `](walks.html "file")`, keeps the attachment drawn as a file.
fn pointed(text: &str, shots: &[(String, String)]) -> String {
    shots.iter().fold(text.to_string(), |text, (shot, leaf)| {
        text.replace(&format!("]({shot})"), &format!("]({leaf})"))
            .replace(&format!("]({shot} \""), &format!("]({leaf} \""))
    })
}

fn one_language(
    session: &mut Session,
    one: &Spoken,
    shots: &[(String, String)],
    folder: ulid::Ulid,
    by: &str,
) -> Answer<(DocId, tisty_core::docs::Doc)> {
    let root = session.paths.docs();
    let device = session.store.device().clone();
    let mut body = pointed(one.told, shots);
    let mut leafed = Vec::new();
    for (marker, leaf) in one.leaves {
        let leaf = pointed(leaf, shots);
        let made = tisty_core::docs::create(&root, &device, &leaf)
            .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
        body = body.replace(
            &format!("]({marker})"),
            &format!("]({}{})", tisty_core::refs::DOC, made.id),
        );
        leafed.push((made.id, leaf));
    }
    let made = tisty_core::docs::create(&root, &device, &body)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    let sorted = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.folder == Some(folder))
            .map(|one| one.order.as_str()),
    );
    let cover = ulid::Ulid::generate();
    let added = |file: String, body: &str, order: String, page_of: Option<DocId>| Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            guest: true,
            sown: true,
            by: Some(by.into()),
            file,
            order,
            said: Some(tisty_core::event::Said::of(body)),
            folder: Some(folder),
            page_of,
            ..Default::default()
        },
    };
    let mut first = added(made.id.clone(), &body, sorted, None);
    if let Op::DocAdd { id, .. } = &mut first {
        *id = cover;
    }
    session.commit(first)?;
    let mut order = tisty_core::order::first();
    for (file, leaf) in &leafed {
        session.commit(added(file.clone(), leaf, order.clone(), Some(cover)))?;
        order = tisty_core::order::after(&order);
    }
    Ok((cover, made))
}

pub fn signed_author(from: &impl Fn(&str) -> Option<PathBuf>) -> Option<String> {
    let guide = from("es")?.parent()?.to_path_buf();
    tisty_core::guide_seal::author_of_guide(&guide, tisty_core::guide_seal::MAKER_ROOT)
}

fn a_cover(kept: &tisty_core::model::Kept) -> bool {
    kept.page_of.is_none() && kept.task_of.is_none() && !kept.archived && kept.guest
}

// A guide its author signed is opened as it is, without reading the files it was sealed over.
pub fn signed_guide_here(
    session: &mut Session,
    tongue: &str,
) -> Answer<Option<tisty_core::docs::Doc>> {
    let mut signed: Vec<&tisty_core::model::Kept> = session
        .state
        .docs
        .values()
        .filter(|one| {
            one.page_of.is_none()
                && one.task_of.is_none()
                && one.guest
                && one.sown
                && one.by.as_deref() != Some(WRITTEN_BY)
        })
        .collect();
    signed.sort_by_key(|one| (one.archived, one.lang.as_deref() != Some(tongue)));
    let Some(found) = signed.first().map(|one| tisty_core::docs::Doc {
        id: one.file.clone(),
        title: one.title.clone().unwrap_or_default(),
    }) else {
        return Ok(None);
    };
    let kept = found.id.clone();
    session.keep(|c| c.guide = Some(kept))?;
    Ok(Some(found))
}

// A copy or another version is never joined, never the guide Tisty opened, and its title says it is a copy.
fn the_guide_itself(session: &Session, kept: &tisty_core::model::Kept) -> bool {
    !session.state.tongues_of(kept.id).is_empty()
        || session.config.guide.as_deref() == Some(kept.file.as_str())
        || kept
            .title
            .as_deref()
            .and_then(tisty_core::untouched::guide_tongue)
            .is_some()
}

// Only the guide as it was sown is put away: one someone wrote in, a copy or another version stays theirs.
fn an_untouched_unsigned_guide(session: &Session, kept: &tisty_core::model::Kept) -> bool {
    a_cover(kept)
        && kept.by.as_deref() == Some(WRITTEN_BY)
        && the_guide_itself(session, kept)
        && tisty_core::docs::read(&session.paths.docs(), &kept.file)
            .is_ok_and(|body| tisty_core::untouched::reads_as_sown(kept, &body))
}

pub fn signed_guide(
    session: &mut Session,
    tongue: &str,
    called: &str,
    author: &str,
    from: impl Fn(&str) -> Option<PathBuf>,
) -> Answer<tisty_core::docs::Doc> {
    let unsigned: Vec<Op> = session
        .state
        .docs
        .values()
        .filter(|one| an_untouched_unsigned_guide(session, one))
        .map(|one| Op::DocArchive { id: one.id })
        .collect();
    session.commit_all(unsigned)?;
    sown(session, tongue, called, author, from)
}

#[cfg(test)]
#[path = "guiding_test.rs"]
mod tests;
