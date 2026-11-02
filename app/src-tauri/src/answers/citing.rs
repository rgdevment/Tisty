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

/// Documents one share may hold: with the share itself, well within the reserves a server allows an hour (decision 64).
pub(crate) const CITED_AT_MOST: usize = 10;

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

/// The documents an envelope of `doc` may lead to: those its share holds, or those every share holding it holds too,
/// so a document two shares hold never hands one's readers what only the other chose to share.
pub(crate) fn held_with(session: &Session, doc: DocId) -> Vec<(Citable, Shared)> {
    let shares = &session.state.shares;
    let holders: Vec<&Shared> = shares
        .values()
        .filter(|one| one.holds.contains(&doc))
        .collect();
    let docs: Vec<DocId> = match (shares.get(&doc), holders.split_first()) {
        (Some(own), _) if !own.holds.is_empty() => own.holds.clone(),
        (_, Some((first, rest))) => first
            .holds
            .iter()
            .filter(|one| rest.iter().all(|other| other.holds.contains(one)))
            .copied()
            .collect(),
        _ => Vec::new(),
    };
    docs.into_iter()
        .filter_map(|one| {
            let kept = session.state.docs.get(&one)?;
            Some((citable(session, kept), shares.get(&one)?.clone()))
        })
        .collect()
}

/// What someone else's share cited in the version an edit through its link is written on, for the cards still in the words:
/// its owner chose what goes with it, and an editor keeps that rather than dropping every card.
/// One that cannot be read is refused rather than taken as citing nothing, so a blip never drops every card for every reader.
pub(crate) fn kept_from(
    wire: &impl sharing::Wire,
    shared: &Shared,
    text: &str,
) -> Answer<Vec<Cited>> {
    let cards = tisty_core::refs::papers(text);
    if cards.is_empty() {
        return Ok(Vec::new());
    }
    let said = sharing::version(wire, shared, shared.version)?;
    let got = crate::received::opened_from(said, &shared.key)?;
    Ok(got
        .cited
        .into_iter()
        .filter(|one| cards.contains(&one.doc))
        .collect())
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

/// Has the holder hold what is shared already, then shares and holds each document not shared yet.
/// Hands back every one held with its share, the new ones marked as held only through it; on a failure the new ones go again.
/// `room` is how many new links the hour still allows it, so what the server let go of never reserves past it.
pub(crate) fn took_up(
    wire: &impl sharing::Wire,
    holder: &Shared,
    going: &[Going],
    signing: &Signing,
    room: usize,
) -> Answer<Vec<(Citable, Shared)>> {
    let mut members: Vec<(Citable, Shared)> = Vec::with_capacity(going.len());
    let mut fresh: Vec<&Going> = Vec::new();
    // What is shared already is held first: a hold reserves nothing, so a refusal never costs a reserve.
    for one in going {
        let Some(was) = &one.was else {
            fresh.push(one);
            continue;
        };
        match sharing::hold(wire, holder, was, !was.held) {
            Ok(()) => members.push((one.citable.clone(), was.clone())),
            // The server let it go while this store still names it, so it is shared anew.
            Err(refused) if sharing::refused_as(&refused, &["cited_gone"]) => fresh.push(one),
            // Blocked or held by too many: its card reads as not shared rather than stopping the rest.
            Err(refused)
                if sharing::refused_as(&refused, &["blocked", "too_long", "forbidden"]) => {}
            Err(refused) => return Err(refused),
        }
    }
    if fresh.len() > room {
        return Err(Refusal::of("shareSlow"));
    }
    let mut reserved: Vec<(Citable, Shared)> = Vec::with_capacity(fresh.len());
    for one in &fresh {
        match sharing::reserve(wire, &holder.server, &chosen_for(holder)) {
            Ok(shared) => reserved.push((
                one.citable.clone(),
                Shared {
                    held: true,
                    ..shared
                },
            )),
            Err(refused) => return Err(stopped(wire, holder, &reserved, refused)),
        }
    }
    let listed: Vec<(Citable, Shared)> = members.iter().chain(&reserved).cloned().collect();
    if let Err(refused) = put_new(wire, holder, &fresh, &mut reserved, &listed, signing) {
        return Err(stopped(wire, holder, &reserved, refused));
    }
    members.extend(reserved);
    Ok(members)
}

// What goes only through its holder is signed and follows saves as its holder does, with nothing else asked of it.
fn chosen_for(holder: &Shared) -> Chosen {
    Chosen {
        signs: holder.signs,
        still: holder.still,
        ..Chosen::default()
    }
}

fn put_new(
    wire: &impl sharing::Wire,
    holder: &Shared,
    fresh: &[&Going],
    reserved: &mut [(Citable, Shared)],
    listed: &[(Citable, Shared)],
    signing: &Signing,
) -> Answer<()> {
    for (one, (_, shared)) in fresh.iter().zip(reserved.iter_mut()) {
        let cited = cited_in(&one.text, listed);
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
    for (_, shared) in reserved.iter() {
        sharing::hold(wire, holder, shared, false)?;
    }
    Ok(())
}

// A link nobody keeps is a link nobody can stop, so what was reserved here goes when the rest fails;
// let go of first, since one its holder still holds would only lose its own life and stay up unseen.
fn stopped(
    wire: &impl sharing::Wire,
    holder: &Shared,
    reserved: &[(Citable, Shared)],
    refused: Refusal,
) -> Refusal {
    for (_, shared) in reserved {
        let _ = sharing::release(wire, holder, &shared.link);
        let _ = sharing::stop(wire, shared);
    }
    refused
}

/// How many of these must be reserved anew, which a server counts against its hour.
pub(crate) fn fresh_in(going: &[Going]) -> usize {
    going.iter().filter(|one| one.was.is_none()).count()
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
    // A document shared anew, here for the first time or after the server let its old link go, is written down.
    let mut ops: Vec<Op> = members
        .into_iter()
        .filter(|(one, shared)| {
            session.state.shares.get(&one.doc).map(|was| &was.link) != Some(&shared.link)
        })
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
        let going = going_of(&session, new);
        let rewritten: Vec<String> = going
            .iter()
            .filter(|one| one.was.is_some())
            .map(|one| one.citable.file.clone())
            .chain(
                shared
                    .holds
                    .iter()
                    .filter(|one| !holds.contains(one))
                    .filter_map(|one| session.state.docs.get(one).map(|it| it.file.clone())),
            )
            .collect();
        (doc, shared, holds, going, dropped, signing, rewritten)
    };
    let (doc, shared, holds, going, dropped, signing, rewritten) = gathered;
    let before = shared.holds.clone();
    let following = app.state::<super::versions::Following>();
    // One round at a time across every share, so two never reserve the same new document twice.
    let Some(_alone) = following.alone(super::versions::CITING) else {
        return Err(Refusal::of("shareSlow"));
    };
    // Reserves and first versions count against the server's hour as any upload does, so a burst waits for the next round.
    let fresh = fresh_in(&going);
    if !following.spend(fresh, fresh) {
        return Err(Refusal::of("shareSlow"));
    }
    let members = apart(move || {
        let wire = Net::to(&shared.server)?;
        // Let go of first, so a failure here leaves nothing new up that this store would not know of.
        for link in &dropped {
            match sharing::release(&wire, &shared, link) {
                Err(refused) if !sharing::already_gone(&refused) => return Err(refused),
                _ => {}
            }
        }
        took_up(&wire, &shared, &going, &signing, fresh)
    })
    .await
    .inspect_err(|_| following.refund(fresh, fresh))?;
    // What the server would not let it hold stays out, and is asked for again with the next save.
    let holds: Vec<DocId> = holds
        .into_iter()
        .filter(|one| before.contains(one) || members.iter().any(|(it, _)| it.doc == *one))
        .collect();
    {
        let mut session = held(&session);
        let ops = settled(&session, doc, holds, members);
        session.commit_all(ops).map_err(Refusal::from)?;
    }
    // What each one held beside it leads to changed, so its own envelope goes up again with the list it now has.
    for file in rewritten {
        super::versions::follow(app, &file);
    }
    Ok(())
}

#[cfg(test)]
#[path = "citing_test.rs"]
mod tests;
