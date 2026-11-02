use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Mutex;

use tisty_core::Op;
use tisty_core::event::Shared;
use tisty_core::model::{DocId, Kept};
use tisty_share_proto::envelope::Cited;

use super::shares::{Shareable, apart, doc_of, shareable};
use crate::handed::Voucher;
use crate::sharing::{self, Chosen, Net, Words};
use crate::{Answer, Refusal, Session, held};

/// Documents one share may hold, well within what a server lets an hour reserve and upload (decision 64).
pub(crate) const CITED_AT_MOST: usize = 20;

/// A document a share leads to, as the envelopes that cite it name it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Citable {
    pub(crate) doc: DocId,
    pub(crate) file: String,
    pub(crate) title: String,
    pub(crate) page: Option<(u32, u32)>,
}

/// What a share holds: its pages always, and what it cites when the person chose so, through every document it reaches.
/// Only the person's own go, and never one that holds others itself, since what holds is never held.
pub(crate) fn reached(session: &Session, root: DocId, cites: bool) -> Answer<Vec<Citable>> {
    let mut seen = BTreeSet::from([root]);
    let mut waiting = vec![root];
    let mut found = Vec::new();
    while let Some(doc) = waiting.pop() {
        let Some(text) = text_of(session, doc) else {
            continue;
        };
        for file in tisty_core::refs::papers(&text) {
            let Some(kept) = session.state.docs.values().find(|one| one.file == file) else {
                continue;
            };
            let page = kept.page_of == Some(doc);
            let holds = session
                .state
                .shares
                .get(&kept.id)
                .is_some_and(|one| !one.holds.is_empty());
            if !(page || cites) || holds || shareable(session, kept) != Shareable::Yes {
                continue;
            }
            if !seen.insert(kept.id) {
                continue;
            }
            found.push(citable(session, kept));
            if found.len() > CITED_AT_MOST {
                return Err(Refusal::of("shareTooManyCited"));
            }
            waiting.push(kept.id);
        }
    }
    Ok(found)
}

fn citable(session: &Session, kept: &Kept) -> Citable {
    Citable {
        doc: kept.id,
        file: kept.file.clone(),
        title: kept.title.clone().unwrap_or_else(|| kept.file.clone()),
        page: kept
            .page_of
            .and_then(|parent| place(session, parent, kept.id)),
    }
}

// Where a page sits among its document's, as the document reads them, from 1.
fn place(session: &Session, parent: DocId, page: DocId) -> Option<(u32, u32)> {
    let text = text_of(session, parent).unwrap_or_default();
    let pages = session.state.pages_read(parent, &text);
    let at = pages.iter().position(|one| one.id == page)?;
    Some((
        u32::try_from(at + 1).ok()?,
        u32::try_from(pages.len()).ok()?,
    ))
}

fn text_of(session: &Session, doc: DocId) -> Option<String> {
    let kept = session.state.docs.get(&doc)?;
    tisty_core::docs::read(&session.paths.docs(), &kept.file).ok()
}

/// What these words cite among the documents that go with them, each with the link and key that open it.
pub(crate) fn cited_in(text: &str, among: &[(Citable, Shared)]) -> Vec<Cited> {
    tisty_core::refs::papers(text)
        .into_iter()
        .filter_map(|file| {
            let (one, shared) = among.iter().find(|(one, _)| one.file == file)?;
            Some(Cited {
                doc: file,
                id: shared.link.clone(),
                key: shared.key.clone(),
                title: one.title.chars().take(200).collect(),
                page: one.page,
            })
        })
        .collect()
}

/// The documents an envelope of `doc` may lead to: those its share holds, or those held beside it by the shares holding it.
pub(crate) fn held_with(session: &Session, doc: DocId) -> Vec<(Citable, Shared)> {
    let shares = &session.state.shares;
    let mut docs: BTreeSet<DocId> = shares
        .get(&doc)
        .map(|one| one.holds.iter().copied().collect())
        .unwrap_or_default();
    for holder in shares.values().filter(|one| one.holds.contains(&doc)) {
        docs.extend(holder.holds.iter().copied());
    }
    docs.into_iter()
        .filter_map(|one| {
            let kept = session.state.docs.get(&one)?;
            Some((citable(session, kept), shares.get(&one)?.clone()))
        })
        .collect()
}

/// Whether one of this store's shares holds `doc`, which then never holds anything itself.
pub(crate) fn held_here(session: &Session, doc: DocId) -> bool {
    session
        .state
        .shares
        .values()
        .any(|one| one.holds.contains(&doc))
}

/// The print a share keeps of what it last put up: the words, and what they cite when they cite anything.
pub(crate) fn printed(text: &str, cited: &[Cited]) -> String {
    match cited.is_empty() {
        true => sharing::print_of(text),
        false => sharing::print_of(&format!(
            "{text}\0{}",
            serde_json::to_string(cited).unwrap_or_default()
        )),
    }
}

/// A document about to go with a share: shared already, or with the words its first version will carry.
pub(crate) struct Going {
    pub(crate) citable: Citable,
    pub(crate) was: Option<Shared>,
    pub(crate) text: String,
}

/// Who signs what goes up, and where its pictures are read from.
pub(crate) struct Signing {
    pub(crate) voucher: Voucher,
    pub(crate) name: Option<String>,
    pub(crate) data: PathBuf,
}

/// Shares each document not shared yet, then has the holder hold them all; on a failure the new ones go again.
/// Hands back every one with its share, the new ones marked as held only through it.
pub(crate) fn took_up(
    wire: &impl sharing::Wire,
    holder: &Shared,
    going: &[Going],
    signing: &Signing,
) -> Answer<Vec<(Citable, Shared)>> {
    let mut members: Vec<(Citable, Shared)> = Vec::with_capacity(going.len());
    for one in going {
        let shared = match &one.was {
            Some(was) => was.clone(),
            None => match sharing::reserve(wire, &holder.server, &chosen_for(holder)) {
                Ok(reserved) => Shared {
                    held: true,
                    ..reserved
                },
                Err(refused) => return Err(let_go(wire, going, &members, refused)),
            },
        };
        members.push((one.citable.clone(), shared));
    }
    match put_all(wire, holder, going, &mut members, signing) {
        Ok(()) => Ok(members),
        Err(refused) => Err(let_go(wire, going, &members, refused)),
    }
}

// What goes only through its holder is signed and follows saves as its holder does, with nothing else asked of it.
fn chosen_for(holder: &Shared) -> Chosen {
    Chosen {
        signs: holder.signs,
        still: holder.still,
        ..Chosen::default()
    }
}

fn put_all(
    wire: &impl sharing::Wire,
    holder: &Shared,
    going: &[Going],
    members: &mut [(Citable, Shared)],
    signing: &Signing,
) -> Answer<()> {
    let listed = members.to_vec();
    for (one, (_, shared)) in going.iter().zip(members.iter_mut()) {
        if one.was.is_some() {
            continue;
        }
        let cited = cited_in(&one.text, &listed);
        let (text, pictures) = sharing::pictured(&one.text, |target| {
            sharing::attachment(target, &signing.data)
        });
        let words = Words {
            text: &text,
            cited: &cited,
            pictures: &pictures,
            voucher: &signing.voucher,
            name: signing.name.as_deref(),
        };
        sharing::first_up(wire, shared, &words, &chosen_for(holder))?;
        shared.print = printed(&one.text, &cited);
    }
    for (one, (_, shared)) in going.iter().zip(members.iter()) {
        // One the person shared on its own keeps its own life too once let go (decision 64).
        let lives = one.was.as_ref().is_some_and(|was| !was.held);
        sharing::hold(wire, holder, shared, lives)?;
    }
    Ok(())
}

// A link nobody keeps is a link nobody can stop, so what was reserved here goes when the rest fails.
fn let_go(
    wire: &impl sharing::Wire,
    going: &[Going],
    members: &[(Citable, Shared)],
    refused: Refusal,
) -> Refusal {
    for (one, (_, shared)) in going.iter().zip(members) {
        if one.was.is_none() {
            let _ = sharing::stop(wire, shared);
        }
    }
    refused
}

/// Each document about to go, with its share when it has one and the words it would carry otherwise.
pub(crate) fn going_of(session: &Session, citables: Vec<Citable>) -> Vec<Going> {
    citables
        .into_iter()
        .map(|citable| Going {
            was: session.state.shares.get(&citable.doc).cloned(),
            text: text_of(session, citable.doc).unwrap_or_default(),
            citable,
        })
        .collect()
}

/// What a share now holds, and the shares of what was held only through it and nothing else holds any longer.
pub(crate) fn settled(
    session: &Session,
    doc: DocId,
    holds: Vec<DocId>,
    members: Vec<(Citable, Shared)>,
) -> Vec<Op> {
    let mut ops: Vec<Op> = members
        .into_iter()
        .filter(|(one, _)| !session.state.shares.contains_key(&one.doc))
        .map(|(one, shared)| Op::ShareDoc {
            id: one.doc,
            d: shared,
        })
        .collect();
    let Some(was) = session.state.shares.get(&doc) else {
        return ops;
    };
    ops.extend(let_go_of(session, doc, was, &holds));
    ops.push(Op::ShareDoc {
        id: doc,
        d: Shared {
            holds,
            ..was.clone()
        },
    });
    ops
}

/// The shares of what `holder` held, no longer `kept`, that went only through it and that no other share holds.
pub(crate) fn let_go_of(session: &Session, doc: DocId, holder: &Shared, kept: &[DocId]) -> Vec<Op> {
    holder
        .holds
        .iter()
        .filter(|one| !kept.contains(one))
        .filter_map(|one| {
            let shared = session.state.shares.get(one)?;
            let elsewhere = session
                .state
                .shares
                .iter()
                .any(|(other, share)| *other != doc && share.holds.contains(one));
            (shared.held && !elsewhere).then(|| Op::ShareStop {
                id: *one,
                link: shared.link.clone(),
            })
        })
        .collect()
}

/// Keeps what a share holds in step with what its words reach now: what is new goes and is held, what is not is let go.
pub(crate) async fn kept_up(app: &tauri::AppHandle, file: &str) -> Answer<()> {
    use tauri::Manager;
    let session = app.state::<Mutex<Session>>();
    let gathered = {
        let session = held(&session);
        let doc = doc_of(&session, file)?;
        let Some(shared) = session.state.shares.get(&doc).cloned() else {
            return Ok(());
        };
        // What is held never holds, someone else's share is theirs to keep up, and one kept as it is stays so.
        if shared.taken || shared.still || held_here(&session, doc) {
            return Ok(());
        }
        let now = reached(&session, doc, shared.cites)?;
        let holds: Vec<DocId> = now.iter().map(|one| one.doc).collect();
        if holds == shared.holds {
            return Ok(());
        }
        let dropped: Vec<String> = shared
            .holds
            .iter()
            .filter(|one| !holds.contains(one))
            .filter_map(|one| session.state.shares.get(one).map(|it| it.link.clone()))
            .collect();
        let new: Vec<Citable> = now
            .into_iter()
            .filter(|one| !shared.holds.contains(&one.doc))
            .collect();
        let (voucher, _) = super::shares::sharer(&session).ok_or(Refusal::of("shareNoKey"))?;
        let signing = Signing {
            voucher,
            name: super::shares::signing_name(&session.state.signed, shared.signs),
            data: session.paths.data().to_path_buf(),
        };
        (
            doc,
            shared,
            holds,
            going_of(&session, new),
            dropped,
            signing,
        )
    };
    let (doc, shared, holds, going, dropped, signing) = gathered;
    let members = apart(move || {
        let wire = Net::to(&shared.server)?;
        let members = took_up(&wire, &shared, &going, &signing)?;
        for link in &dropped {
            sharing::release(&wire, &shared, link)?;
        }
        Ok(members)
    })
    .await?;
    let mut session = held(&session);
    let ops = settled(&session, doc, holds, members);
    session.commit_all(ops).map_err(Refusal::from)
}

#[cfg(test)]
#[path = "citing_test.rs"]
mod tests;
