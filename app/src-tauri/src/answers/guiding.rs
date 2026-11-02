use std::path::PathBuf;

use tisty_core::Op;
use tisty_core::model::DocId;

use super::settings::{ATTACHED, GUIDE_EN, GUIDE_ES, GUIDE_PAGES_EN, GUIDE_PAGES_ES, WRITTEN_BY};
use crate::{Answer, Refusal, Session};

/// One language the guide ships in: its cover and its pages, each page behind the marker the
/// cover names it by until it has a file of its own.
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

/// The guide in every language it ships in, the person's own first, held together as one
/// document; `from` says where each language keeps its pictures. Answers the cover in theirs.
pub fn sown(
    session: &mut Session,
    first: &str,
    called: &str,
    from: impl Fn(&str) -> Option<PathBuf>,
) -> Answer<tisty_core::docs::Doc> {
    let mut spoken: Vec<&Spoken> = SPOKEN.iter().collect();
    spoken.sort_by_key(|one| one.tongue != first);
    let data = session.paths.data().to_path_buf();

    let mut told_of = Vec::new();
    let mut kept_at = Vec::new();
    let mut named: Vec<Vec<(String, String)>> = Vec::new();
    for one in &spoken {
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
            // The same picture in both languages is kept once and named alike.
            if !kept_at.contains(&kept.at) {
                told_of.push(kept.told(tisty_core::attach::Via::Guide));
                kept_at.push(kept.at.clone());
            }
            let leaf = tisty_core::attach::leaf_of(&kept.at).unwrap_or(&kept.at);
            here.push((shot.to_string(), leaf.to_string()));
        }
        named.push(here);
    }

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
            icon: None,
            color: None,
        },
    });
    session.commit_all(told_of)?;

    let mut covers: Vec<(DocId, tisty_core::docs::Doc)> = Vec::new();
    for (one, shots) in spoken.iter().zip(&named) {
        covers.push(one_language(session, one, shots, folder)?);
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

/// A guide sown in one language by an earlier Tisty says which, if nobody has written in it,
/// and a guide in each language joins the other. A store holding nothing else is left as it is,
/// so it still reads as new to whoever it is about to be joined with.
pub fn spoken_again(session: &mut Session) {
    let root = session.paths.docs();
    if tisty_core::untouched::untouched(&session.paths.store(), &root) {
        return;
    }
    let mut covers: Vec<(DocId, &str)> = Vec::new();
    for one in &SPOKEN {
        let title = tisty_core::docs::titled(one.told);
        let found = session.state.papers().find(|kept| {
            kept.by.as_deref() == Some(WRITTEN_BY)
                && kept.guest
                && kept.page_of.is_none()
                && kept.title.as_deref() == Some(title.as_str())
                && !session.state.held_away(kept)
        });
        if let Some(kept) = found {
            covers.push((kept.id, one.tongue));
        }
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
            by: Some(WRITTEN_BY.into()),
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

#[cfg(test)]
#[path = "guiding_test.rs"]
mod tests;
