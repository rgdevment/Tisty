use std::collections::{BTreeSet, HashMap};
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

/// Documents one share may hold, all rounds together: what one envelope may list (decision 64).
pub(crate) const CITED_AT_MOST: usize = tisty_share_proto::envelope::CITED_AT_MOST;

/// New documents one round reserves at most, well within a server's hour; the rest go in the rounds after.
pub(crate) const NEW_A_ROUND: usize = 10;

/// A document a share leads to, as the envelopes that cite it name it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Citable {
    pub(crate) doc: DocId,
    pub(crate) file: String,
    pub(crate) title: String,
    pub(crate) page: Option<(u32, u32)>,
}

/// What a share reaches: its pages, which always go, and what else it cites, which goes when the person chose so.
#[derive(Debug, Default)]
pub(crate) struct Reach {
    pub(crate) pages: Vec<Citable>,
    pub(crate) cited: Vec<Citable>,
}

impl Reach {
    pub(crate) fn taken(self, cites: bool) -> Vec<Citable> {
        let mut all = self.pages;
        if cites {
            all.extend(self.cited);
        }
        all
    }
}

// Each document's words read once for a whole walk, however many ways lead to them.
type Texts = HashMap<DocId, Option<String>>;

/// Only the person's own go, and never one that holds others itself, since what holds is never held.
pub(crate) fn reach(session: &Session, root: DocId) -> Answer<Reach> {
    reach_among(session, root, &|kept| {
        shareable(session, kept) == Shareable::Yes
    })
}

fn reach_among(session: &Session, root: DocId, own: &dyn Fn(&Kept) -> bool) -> Answer<Reach> {
    let mut texts = Texts::new();
    let pages = walked(session, &mut texts, root, false, &BTreeSet::new(), own);
    let known: BTreeSet<DocId> = pages.iter().map(|one| one.doc).collect();
    let cited = walked(session, &mut texts, root, true, &known, own);
    if pages.len() + cited.len() > CITED_AT_MOST {
        return Err(Refusal::of("shareTooManyCited"));
    }
    Ok(Reach { pages, cited })
}

/// What a share holds: its pages always, and what it cites when the person chose so.
pub(crate) fn reached(session: &Session, root: DocId, cites: bool) -> Answer<Vec<Citable>> {
    Ok(reach(session, root)?.taken(cites))
}

fn walked(
    session: &Session,
    texts: &mut Texts,
    root: DocId,
    cites: bool,
    known: &BTreeSet<DocId>,
    own: &dyn Fn(&Kept) -> bool,
) -> Vec<Citable> {
    let mut seen = BTreeSet::from([root]);
    let mut waiting = vec![root];
    let mut found = Vec::new();
    while let Some(doc) = waiting.pop() {
        let Some(text) = text_of(session, texts, doc) else {
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
            if !(page || cites) || holds || !own(kept) {
                continue;
            }
            if !seen.insert(kept.id) {
                continue;
            }
            if !known.contains(&kept.id) {
                found.push(citable(session, texts, kept));
            }
            // Past what an envelope may list the walk stops, and `reach` refuses the share.
            if found.len() > CITED_AT_MOST {
                return found;
            }
            waiting.push(kept.id);
        }
    }
    found
}

fn citable(session: &Session, texts: &mut Texts, kept: &Kept) -> Citable {
    Citable {
        doc: kept.id,
        file: kept.file.clone(),
        // A page with no words yet has an empty title, which no envelope takes, so its file names it.
        title: kept
            .title
            .clone()
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| kept.file.clone()),
        page: kept
            .page_of
            .and_then(|parent| place(session, texts, parent, kept.id)),
    }
}

// Where a page sits among its document's, as the document reads them, from 1.
fn place(session: &Session, texts: &mut Texts, parent: DocId, page: DocId) -> Option<(u32, u32)> {
    let text = text_of(session, texts, parent).unwrap_or_default();
    let pages = session.state.pages_read(parent, &text);
    let at = pages.iter().position(|one| one.id == page)?;
    Some((
        u32::try_from(at + 1).ok()?,
        u32::try_from(pages.len()).ok()?,
    ))
}

fn text_of(session: &Session, texts: &mut Texts, doc: DocId) -> Option<String> {
    texts
        .entry(doc)
        .or_insert_with(|| {
            let kept = session.state.docs.get(&doc)?;
            tisty_core::docs::read(&session.paths.docs(), &kept.file).ok()
        })
        .clone()
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

/// Where an envelope of `doc` may lead: what its share holds, or what every share holding it holds, so no reader gets another's choice.
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
    let mut texts = Texts::new();
    docs.into_iter()
        .filter_map(|one| {
            let kept = session.state.docs.get(&one)?;
            Some((
                citable(session, &mut texts, kept),
                shares.get(&one)?.clone(),
            ))
        })
        .collect()
}

/// What someone else's share cited in the version an edit is written on, kept for the cards still there; unread, it is refused.
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

/// New links and uploads that reached the server, which its hour counts whatever came of them.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Spent {
    pub(crate) reserves: usize,
    pub(crate) uploads: usize,
}

/// Holds what is shared already, then shares and holds as many new ones as `room` allows; the rest wait for the next round.
pub(crate) fn took_up(
    wire: &impl sharing::Wire,
    holder: &Shared,
    going: &[Going],
    signing: &Signing,
    room: usize,
    spent: &mut Spent,
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
    fresh.truncate(room);
    let mut reserved: Vec<(Citable, Shared)> = Vec::with_capacity(fresh.len());
    for one in &fresh {
        match sharing::reserve(wire, &holder.server, &chosen_for(holder)) {
            Ok(shared) => {
                spent.reserves += 1;
                reserved.push((
                    one.citable.clone(),
                    Shared {
                        held: true,
                        ..shared
                    },
                ));
            }
            Err(refused) => return Err(stopped(wire, holder, &reserved, refused)),
        }
    }
    let listed: Vec<(Citable, Shared)> = members.iter().chain(&reserved).cloned().collect();
    if let Err(refused) = put_new(wire, holder, &fresh, &mut reserved, &listed, signing, spent) {
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
    spent: &mut Spent,
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
        spent.uploads += 1;
        sharing::first_up(wire, shared, &words, &chosen_for(holder))?;
        shared.print = printed(&one.text, &cited);
    }
    for (_, shared) in reserved.iter() {
        sharing::hold(wire, holder, shared, false)?;
    }
    Ok(())
}

// Let go of before stopped, since one its holder still holds would only lose its own life and stay up unseen.
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

/// Each document about to go, with its share when it has one and the words it would carry otherwise.
pub(crate) fn going_of(session: &Session, citables: Vec<Citable>) -> Vec<Going> {
    let mut texts = Texts::new();
    citables
        .into_iter()
        .map(|citable| Going {
            was: session.state.shares.get(&citable.doc).cloned(),
            text: text_of(session, &mut texts, citable.doc).unwrap_or_default(),
            citable,
        })
        .collect()
}

/// The documents shared anew, here for the first time or after the server let their old link go, as the log writes them.
pub(crate) fn written(session: &Session, members: Vec<(Citable, Shared)>) -> Vec<Op> {
    members
        .into_iter()
        .filter(|(one, shared)| {
            session.state.shares.get(&one.doc).map(|was| &was.link) != Some(&shared.link)
        })
        .map(|(one, shared)| Op::ShareDoc {
            id: one.doc,
            d: shared,
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
    let mut ops = written(session, members);
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
    let following = app.state::<super::versions::Following>();
    let gathered = {
        let session = held(&session);
        let doc = doc_of(&session, file)?;
        let Some(shared) = session.state.shares.get(&doc).cloned() else {
            return Ok(());
        };
        // What is held never holds; its own cards may reach something new, so every share walks again.
        if held_here(&session, doc) {
            following.unsettled();
            return Ok(());
        }
        // Someone else's share is theirs to keep up, and one kept as it is stays so.
        if shared.taken || shared.still {
            return Ok(());
        }
        let text = tisty_core::docs::read(&session.paths.docs(), file).unwrap_or_default();
        let cards = tisty_core::refs::papers(&text).join(
            "
",
        );
        // The same cards as when its holds last matched them reach the same documents, so nothing is walked again.
        if following.unchanged(file, &cards) {
            return Ok(());
        }
        let now = reached(&session, doc, shared.cites)?;
        let holds: Vec<DocId> = now.iter().map(|one| one.doc).collect();
        if holds == shared.holds {
            following.settled(file, Some(cards));
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
        (
            doc, shared, holds, going, dropped, signing, rewritten, cards,
        )
    };
    let (doc, shared, holds, going, dropped, signing, rewritten, cards) = gathered;
    let before = shared.holds.clone();
    // One round at a time across every share, so two never reserve the same new document twice.
    let Some(_alone) = following.alone(super::versions::CITING) else {
        return Err(Refusal::of("shareSlow"));
    };
    // As many new ones as the hour has room for go now, those the server let go of among them; the rest wait for the next round.
    let room = following.take(0, going.len().min(NEW_A_ROUND)).unwrap_or(0);
    let (spent, members) = apart(move || {
        let mut spent = Spent::default();
        let members = (|| {
            let wire = Net::to(&shared.server)?;
            // Let go of first, so a failure here leaves nothing new up that this store would not know of.
            for link in &dropped {
                match sharing::release(&wire, &shared, link) {
                    Err(refused) if !sharing::already_gone(&refused) => return Err(refused),
                    _ => {}
                }
            }
            took_up(&wire, &shared, &going, &signing, room, &mut spent)
        })();
        Ok((spent, members))
    })
    .await?;
    following.refund(
        room - spent.reserves.min(room),
        room - spent.uploads.min(room),
    );
    let members = members?;
    // What the server would not let it hold, or the hour had no room for, stays out and is asked for again.
    let held: Vec<DocId> = holds
        .iter()
        .filter(|one| before.contains(one) || members.iter().any(|(it, _)| it.doc == **one))
        .copied()
        .collect();
    following.settled(file, (held == holds).then_some(cards));
    {
        let mut session = crate::held(&session);
        let ops = settled(&session, doc, held, members);
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
