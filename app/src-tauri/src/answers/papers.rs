use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};
use tisty_core::{Op, Reading, State};

use crate::{
    Answer, Refusal, Session, Task, blamed, doc_out, folder_open, held, placed, reading_as, said,
    signing, stop, weighed,
};

#[tauri::command]
pub fn doc_file(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    folder: Option<String>,
    before: Option<String>,
) -> Answer<()> {
    let folder = folder
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let mut session = held(&session);

    let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let before: Option<tisty_core::model::DocId> = before
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchDoc")))
        .transpose()?;
    match session.state.docs.get(&id) {
        None => return Err(Refusal::of("noSuchDoc")),
        Some(one) if one.page_of.is_some() => return Err(Refusal::of("pageStaysPut")),
        Some(_) => {}
    }
    doc_out(&session.state, id)?;
    if let Some(at) = folder {
        if !session.state.folders.contains_key(&at) {
            return Err(Refusal::of("noSuchFolder"));
        }
        folder_open(&session.state, at, true)?;
    }

    if before.is_some_and(|at| at == id) {
        return Err(Refusal::of("intoItself"));
    }
    let ops = beside_docs(&session.state, id, folder, before);
    session.commit_all(ops)?;
    Ok(())
}

pub fn beside_docs(
    state: &State,
    id: tisty_core::model::DocId,
    folder: Option<tisty_core::model::FolderId>,
    before: Option<tisty_core::model::DocId>,
) -> Vec<Op> {
    let mut sitting: Vec<&tisty_core::model::Kept> = state
        .docs
        .values()
        .filter(|one| {
            one.page_of.is_none() && !state.held_away(one) && one.folder == folder && one.id != id
        })
        .collect();
    sitting.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));
    let keys: Vec<&str> = sitting.iter().map(|one| one.order.as_str()).collect();
    let (mine, fresh) = tisty_core::order::dealt(
        &keys,
        stop(before.map(|at| sitting.iter().position(|one| one.id == at))),
    );
    let mut ops: Vec<Op> = sitting
        .iter()
        .zip(fresh)
        .filter_map(|(one, order)| {
            order.map(|order| Op::DocMove {
                id: one.id,
                d: tisty_core::event::Filed {
                    folder: None,
                    page_of: None,
                    order: Some(order),
                },
            })
        })
        .collect();
    ops.push(Op::DocMove {
        id,
        d: tisty_core::event::Filed {
            folder: Some(folder),
            page_of: None,
            order: Some(mine),
        },
    });
    ops
}

#[tauri::command(async)]
pub fn doc_read(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<String> {
    let root = held(&session).paths.docs();
    let read = tisty_core::docs::read(&root, &id);
    if let Ok(body) = &read {
        let mut session = held(&session);
        session.mind_body(&id, body);
        noted(&mut session, &id, body);
    }
    read.map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { bytes, limit } => {
            witness::warn(
                channel::WINDOW,
                "a document too big to hold was not opened",
                &[
                    ("id", witness::Fact::Id(id)),
                    ("bytes", witness::Fact::Bytes(bytes)),
                ],
            );
            Refusal::about("documentTooBig", weighed(limit))
        }
        _ if !root.join(format!("{id}.md")).exists() && still_coming(&session, &id) => {
            Refusal::about("docComing", id)
        }
        _ => Refusal::about("noSuchDoc", id),
    })
}

fn still_coming(session: &tauri::State<'_, Mutex<Session>>, id: &str) -> bool {
    let carrier = held(session).carrier();
    carrier.is_some_and(|carrier| carrier.paper_waiting(id))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Facts {
    made: Option<i64>,
    wrote: Option<i64>,
    bytes: u64,
    pages: usize,
    author: Option<String>,
    editor: Option<String>,
    born: Option<String>,
}

fn seconds(at: std::io::Result<std::time::SystemTime>) -> Option<i64> {
    at.ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|gone| gone.as_secs() as i64)
}

#[tauri::command(async)]
pub fn keep_pdf(at: String, bytes: Vec<u8>) -> Answer<()> {
    std::fs::write(&at, bytes).map_err(|e| {
        blamed(
            channel::WINDOW,
            "a pdf could not be written",
            tisty_core::Error::Io(e),
        )
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Signed {
    alias: Option<String>,
    before: Vec<String>,
    mine: usize,
}

#[tauri::command]
pub fn signed(session: tauri::State<'_, Mutex<Session>>) -> Answer<Signed> {
    let session = held(&session);
    Ok(as_signed(&session))
}

fn as_signed(session: &Session) -> Signed {
    Signed {
        alias: session.state.signed.alias.clone(),
        before: {
            let mut seen: Vec<String> = Vec::new();
            for was in session.state.signed_before.iter().rev() {
                if !seen
                    .iter()
                    .any(|one| tisty_core::state::same_name(one, was))
                {
                    seen.push(was.clone());
                }
            }
            seen
        },
        mine: match session.state.signed.alias.is_some() {
            true => session.state.mine_to_sign().len(),
            false => 0,
        },
    }
}

#[tauri::command]
pub fn sign_the_rest(session: tauri::State<'_, Mutex<Session>>) -> Answer<usize> {
    let mut session = held(&session);
    let Some(alias) = session.state.signed.alias.clone() else {
        return Ok(0);
    };
    let ops: Vec<Op> = session
        .state
        .mine_to_sign()
        .into_iter()
        .map(|id| Op::DocSigned {
            id,
            d: alias.clone(),
        })
        .collect();
    let many = ops.len();
    if many > 0 {
        session
            .commit_all(ops)
            .map_err(|e| blamed(channel::WINDOW, "the documents could not be signed", e))?;
    }
    Ok(many)
}

#[tauri::command]
pub fn sign(session: tauri::State<'_, Mutex<Session>>, alias: Option<String>) -> Answer<Signed> {
    let said = alias
        .map(|one| tisty_core::text::plainly(&one).trim().to_string())
        .filter(|one| !one.is_empty());
    if said
        .as_ref()
        .is_some_and(|one| one.chars().count() > tisty_core::event::ALIAS_AT_MOST)
    {
        return Err(Refusal::about(
            "aliasTooLong",
            tisty_core::event::ALIAS_AT_MOST.to_string(),
        ));
    }

    let mut session = held(&session);
    let same = match (said.as_deref(), session.state.signed.alias.as_deref()) {
        (Some(one), Some(was)) => tisty_core::state::same_name(one, was),
        (one, was) => one == was,
    };
    if same {
        return Ok(as_signed(&session));
    }
    let mut signature = session.state.signed.clone();
    signature.alias = said;
    session
        .commit(Op::Signed {
            d: signature.clone(),
        })
        .map_err(|e| blamed(channel::WINDOW, "the signature could not be written", e))?;
    Ok(as_signed(&session))
}

#[tauri::command]
pub fn doc_facts(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Facts> {
    let session = held(&session);
    let root = session.paths.docs();
    let kept = session.state.docs.values().find(|one| one.file == id);
    let made = kept.map(|one| match one.made {
        Some(at) => at.as_second(),
        None => (one.id.timestamp_ms() / 1000) as i64,
    });
    let pages = kept.map_or(0, |one| session.state.pages_of(one.id).len());
    let author = kept
        .and_then(|one| session.state.author_of(one))
        .map(str::to_string);
    let editor = kept
        .and_then(|one| session.state.editor_of(one))
        .map(str::to_string);
    let born = kept
        .and_then(|one| session.state.born_of(one))
        .map(str::to_string);
    let at = tisty_core::docs::resolve(&root, &id)
        .map_err(|_| Refusal::about("noSuchDoc", id.clone()))?;
    let about = std::fs::metadata(&at).map_err(|_| Refusal::about("noSuchDoc", id))?;
    let wrote = kept
        .and_then(|one| one.wrote)
        .map(|at| at.as_second())
        .or_else(|| seconds(about.modified()));
    Ok(Facts {
        made,
        wrote,
        bytes: about.len(),
        pages,
        author,
        editor,
        born,
    })
}

#[tauri::command]
pub fn doc_lock(session: tauri::State<'_, Mutex<Session>>, id: String, shut: bool) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let mut session = held(&session);
    match session.state.docs.get(&id) {
        None => return Err(Refusal::of("noSuchDoc")),
        Some(one) if shut && one.page_of.is_some() => return Err(Refusal::of("lockIsTheDocs")),
        Some(_) => {}
    }
    doc_out(&session.state, id)?;
    session.commit(if shut {
        Op::DocLock { id }
    } else {
        Op::DocUnlock { id }
    })?;
    Ok(())
}

#[tauri::command]
pub fn doc_away(session: tauri::State<'_, Mutex<Session>>, id: String, away: bool) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let mut session = held(&session);
    if !session.state.docs.contains_key(&id) {
        return Err(Refusal::of("noSuchDoc"));
    }
    doc_out(&session.state, id)?;
    session.commit(if away {
        Op::DocArchive { id }
    } else {
        Op::DocUnarchive { id }
    })?;
    Ok(())
}

#[tauri::command]
pub fn doc_unflag(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let mut session = held(&session);
    match session.state.docs.get(&id) {
        None => return Err(Refusal::of("noSuchDoc")),
        Some(one) if one.flagged.is_none() => return Ok(()),
        Some(_) => {}
    }
    session.commit(Op::DocUnflag { id })?;
    Ok(())
}

#[tauri::command]
pub fn spelled(said: String) -> String {
    tisty_core::docs::spelled(&said)
}

#[tauri::command]
pub fn doc_adopt(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
) -> Answer<tisty_core::docs::Doc> {
    held(&session).take_in(&file)
}

#[tauri::command]
pub fn doc_let_go(session: tauri::State<'_, Mutex<Session>>, file: String) -> Answer<()> {
    held(&session).let_go_of(&file)
}

#[tauri::command]
pub fn doc_new(
    session: tauri::State<'_, Mutex<Session>>,
    folder: Option<String>,
    page_of: Option<String>,
) -> Answer<tisty_core::docs::Doc> {
    let folder = folder
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let page_of = page_of
        .map(|up| up.parse().map_err(|_| Refusal::of("noSuchDoc")))
        .transpose()?;
    let mut session = held(&session);
    if let Some(at) = folder {
        if !session.state.folders.contains_key(&at) {
            return Err(Refusal::of("noSuchFolder"));
        }
        folder_open(&session.state, at, true)?;
    }
    let under = match page_of {
        Some(up) => match session.state.docs.get(&up) {
            None => return Err(Refusal::of("noSuchDoc")),
            Some(one) if one.page_of.is_some() => return Err(Refusal::of("pageOfPage")),
            Some(one) if session.state.held_away(one) => {
                return Err(Refusal::of("pageOfAway"));
            }
            Some(one) if one.locked => return Err(Refusal::of("pageOfLocked")),
            Some(one) => Some(one.folder),
        },
        None => None,
    };
    let folder = under.unwrap_or(folder);
    let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, "")
        .map_err(|e| blamed(channel::WINDOW, "a document could not be made", e))?;

    let order = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.page_of == page_of && (page_of.is_some() || one.folder == folder))
            .map(|one| one.order.as_str()),
    );
    let signed_as = signing(&session.state);
    session.commit(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: false,
            made: None,
            by: signed_as.clone(),
            file: made.id.clone(),
            order,
            said: Some(tisty_core::event::Said {
                title: made.title.clone(),
                bytes: None,
                tags: Some(Vec::new()),
                by: None,
                print: None,
            }),
            folder,
            page_of,
        },
    })?;
    Ok(made)
}

#[tauri::command]
pub fn doc_page(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    page_of: Option<String>,
) -> Answer<()> {
    let id: tisty_core::model::DocId = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let page_of = page_of
        .map(|up| up.parse().map_err(|_| Refusal::of("noSuchDoc")))
        .transpose()?;
    let mut session = held(&session);

    match session.state.docs.get(&id) {
        None => return Err(Refusal::of("noSuchDoc")),
        Some(one) if one.page_of == page_of => return Ok(()),
        Some(_) => {}
    }
    if session.state.shut(id) {
        return Err(Refusal::of("lockedStaysPut"));
    }
    doc_out(&session.state, id)?;
    if let Some(up) = page_of {
        if up == id {
            return Err(Refusal::of("pageOfPage"));
        }
        doc_out(&session.state, up)?;
        match session.state.docs.get(&up) {
            None => return Err(Refusal::of("noSuchDoc")),
            Some(one) if one.page_of.is_some() => return Err(Refusal::of("pageOfPage")),
            Some(one) if session.state.held_away(one) => {
                return Err(Refusal::of("pageOfAway"));
            }
            Some(one) if one.locked => return Err(Refusal::of("pageOfLocked")),
            Some(_) => {}
        }
        // Hanging carries the parent's archived state over, and the inverse cannot carry it back.
        if session.state.docs.get(&id).is_some_and(|one| one.archived) {
            return Err(Refusal::of("awayStaysAway"));
        }
        if session
            .state
            .docs
            .values()
            .any(|one| one.page_of == Some(id))
        {
            return Err(Refusal::of("holdsPages"));
        }
    }

    let d = match page_of {
        Some(_) => tisty_core::event::Filed {
            folder: None,
            page_of: Some(page_of),
            order: None,
        },
        None => session
            .unhang(id)
            .map_err(|e| blamed(channel::WINDOW, "the log would not be read back", e))?,
    };
    let over = page_of
        .and_then(|up| session.state.docs.get(&up))
        .map(|up| up.file.clone());
    session.commit(Op::DocMove { id, d })?;
    // The page takes a key of its own on the way in, and where it is read comes from the text, so
    // a book that already names it has to be settled from what it says or the two drift apart.
    if let Some(up) = over
        && let Ok(body) = tisty_core::docs::read(&session.paths.docs(), &up)
    {
        session.retell(&up, &body, None);
    }
    Ok(())
}

#[tauri::command]
pub fn doc_drop(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    held(&session).drop_doc(&id)
}

#[tauri::command]
pub fn convert_paper(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    body: String,
) -> Answer<()> {
    let mut session = held(&session);
    if session.state.bolted(&id) {
        return Err(Refusal::of(match session.state.away(&id) {
            true => "documentAway",
            false => "documentLocked",
        }));
    }
    let papers = session.paths.docs();
    let was = tisty_core::docs::read(&papers, &id)
        .map_err(|_| Refusal::about("cannotRead", id.clone()))?;

    tisty_core::docs::kept_before(session.paths.data(), &id, &was, &body)
        .map_err(|e| blamed(channel::SYNC, "what it was could not be kept", e))?;
    tisty_core::docs::write(&papers, &id, &body).map_err(|e| match e {
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        e => blamed(
            channel::SYNC,
            "the converted document could not be written",
            e,
        ),
    })?;
    session.mind(&id);
    let hand = signing(&session.state);
    session.retell(&id, &body, hand);
    Ok(())
}

#[tauri::command]
pub fn settle_paper(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    keep: String,
    marked: Option<String>,
) -> Answer<Option<String>> {
    let mut session = held(&session);
    // Settling what two machines already wrote is not writing into it, so being archived is no
    // reason to leave the rift with no way out. Only a lock guards the text itself.
    if session.state.shut_tight(&id) {
        return Err(Refusal::of("documentLocked"));
    }
    let carrier = session.carrying()?;
    let here = session.here();
    let keep = match keep.as_str() {
        "mine" => tisty_carrier::Keep::Mine,
        "theirs" => tisty_carrier::Keep::Theirs,
        _ => tisty_carrier::Keep::Both,
    };

    if !matches!(keep, tisty_carrier::Keep::Theirs)
        && let Some(shown) = session.asked.get(&id)
        && carrier.paper_print(&id).as_ref() != Some(shown)
    {
        return Err(Refusal::of("movedUnderfoot"));
    }
    let brought = carrier.settle(&here, &id, keep).map_err(said)?;
    session.mind(&id);
    session.asked.remove(&id);

    let Some(body) = brought else { return Ok(None) };
    let beside = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .map(|one| (one.folder, one.page_of, one.order.clone()));
    let same_as_mine = tisty_core::docs::read(&session.paths.docs(), &id)
        .is_ok_and(|mine| tisty_core::docs::unchanged(&mine, &body));
    let body = match &marked {
        Some(said) => tisty_core::docs::marked(&body, said),
        None => body,
    };
    // Asked again about the same arrival, keeping both must not leave one more copy each time.
    let twin = match same_as_mine {
        true => Some(None),
        false => twin_of(&session, beside.as_ref(), &body).map(Some),
    };
    if let Some(twin) = twin {
        carrier
            .settle(&here, &id, tisty_carrier::Keep::Mine)
            .map_err(said)?;
        session.mind(&id);
        return Ok(twin);
    }
    let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, &body)
        .map_err(|e| blamed(channel::SYNC, "the other version could not be kept", e))?;
    let file = made.id.clone();
    let (folder, page_of, order) = placed(beside, &made.id);
    let signed_as = signing(&session.state);
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: signed_as.clone(),
                file: file.clone(),
                folder,
                order,
                said: Some(tisty_core::event::Said {
                    title: made.title.clone(),
                    bytes: None,
                    tags: Some(Vec::new()),
                    by: None,
                    print: None,
                }),
                page_of,
            },
        })
        .map_err(|e| blamed(channel::SYNC, "the other version was not written down", e))?;

    carrier
        .settle(&here, &id, tisty_carrier::Keep::Mine)
        .map_err(said)?;
    session.mind(&id);
    Ok(Some(file))
}

fn twin_of(session: &Session, beside: Option<&crate::Placing>, body: &str) -> Option<String> {
    let (folder, page_of, _) = beside?;
    let docs = session.paths.docs();
    let weighs = tisty_core::docs::settled(body).len() as u64;
    // The size is read before any body, so a crowded folder costs one look per document.
    let alike = |file: &str| {
        tisty_core::docs::resolve(&docs, file)
            .ok()
            .and_then(|at| std::fs::metadata(at).ok())
            .is_some_and(|told| told.len() == weighs)
    };
    session
        .state
        .docs
        .values()
        .filter(|one| !one.archived && one.folder == *folder && one.page_of == *page_of)
        .map(|one| one.file.clone())
        .filter(|file| alike(file))
        .find(|file| {
            tisty_core::docs::read(&docs, file)
                .is_ok_and(|kept| tisty_core::docs::unchanged(&kept, body))
        })
}

/// The person's reading of a closed task, story or trace; a routine reads as a routine and
/// an open task is not read yet.
#[tauri::command]
pub fn read_as(session: tauri::State<'_, Mutex<Session>>, id: String, how: String) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let how = match how.as_str() {
        "story" => Reading::Story,
        "trace" => Reading::Trace,
        _ => return Err(Refusal::of("notAReading")),
    };
    reading_as(&mut held(&session), id, how)
}

const CATCHING_UP_AT_ONCE: usize = 500;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Papers {
    folders: Vec<Folded>,
    docs: Vec<Filed>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Folded {
    id: String,
    name: String,
    parent: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    holds: usize,
    /// The folder's own mark, so only the one that was shelved offers to come back.
    archived: bool,
    /// What the archive holds, the folders above it counted in.
    away: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Filed {
    id: String,
    file: String,
    tags: Vec<String>,
    title: String,
    told: bool,
    bytes: Option<u64>,
    wrote: Option<String>,
    folder: Option<String>,
    /// The document's own mark, so the menu offers what the document itself can answer for.
    archived: bool,
    /// What the archive holds, the folder above it counted in.
    away: bool,
    locked: bool,
    gone: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    guest: Option<String>,
    page_of: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flagged: Option<Marked>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_was: Option<Vec<String>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Marked {
    at: String,
    said: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    via: Option<String>,
}

#[tauri::command(async)]
pub fn docs_catch_up(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<Filed>> {
    let (root, owing) = {
        let held = held(&session);
        let owing: std::collections::BTreeSet<String> = held
            .state
            .docs
            .values()
            .filter(|one| one.title.is_none())
            .map(|one| one.file.clone())
            .collect();
        (held.paths.docs(), owing)
    };

    let found: Vec<tisty_core::docs::Doc> = match owing.is_empty() {
        true => Vec::new(),
        false => tisty_core::docs::all(&root)
            .into_iter()
            .filter(|one| owing.contains(&one.id))
            .collect(),
    };

    for some in found.chunks(CATCHING_UP_AT_ONCE) {
        // Read the bodies before taking the session: five hundred files off a synced folder is a
        // second of disk, and nothing else can be asked of Tisty while the guard is held.
        let read: Vec<(&tisty_core::docs::Doc, Option<tisty_core::event::Said>)> = some
            .iter()
            .map(|one| {
                let said = tisty_core::docs::read(&root, &one.id)
                    .ok()
                    .map(|body| tisty_core::event::Said::of(&body));
                (one, said)
            })
            .collect();

        let mut held = held(&session);
        let ops: Vec<Op> = read
            .into_iter()
            .filter_map(|(one, said)| {
                let kept = held.state.docs.values().find(|kept| kept.file == one.id)?;
                let said = said
                    .unwrap_or_else(|| tisty_core::event::Said {
                        title: one.title.clone(),
                        bytes: None,
                        tags: Some(kept.tags.clone()),
                        by: None,
                        print: None,
                    })
                    .by(None);
                said.news_for(kept).then_some(Op::DocSaid {
                    id: kept.id,
                    d: said,
                })
            })
            .collect();
        if !ops.is_empty() {
            held.commit_all(ops)
                .map_err(|e| blamed(channel::WINDOW, "the titles could not be written down", e))?;
        }
    }
    Ok(gathered(&held(&session)))
}

#[tauri::command(async)]
pub fn docs(session: tauri::State<'_, Mutex<Session>>) -> Answer<Papers> {
    let mut session = held(&session);
    session.reload()?;
    Ok(Papers {
        folders: hanging(&session.state, None),
        docs: gathered(&session),
    })
}

fn gathered(session: &Session) -> Vec<Filed> {
    let on_disk = tisty_core::docs::names(&session.paths.docs());
    let mut kept_in_order: Vec<&tisty_core::model::Kept> = session.state.docs.values().collect();
    kept_in_order.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));

    kept_in_order
        .into_iter()
        .map(|kept| Filed {
            id: kept.id.to_string(),
            file: kept.file.clone(),
            title: kept.title.clone().unwrap_or_default(),
            told: kept.title.is_some(),
            bytes: kept.bytes,
            wrote: kept.wrote.map(|at| at.to_string()),
            folder: kept.folder.map(|at| at.to_string()),
            archived: kept.archived,
            away: session.state.held_away(kept),
            locked: session.state.shut(kept.id),
            gone: !on_disk.contains(&kept.file),
            // Empty means it came from elsewhere under nobody's name: the list still has to
            // say so before anybody signs it as their own.
            guest: kept.guest.then(|| kept.by.clone().unwrap_or_default()),
            page_of: kept.page_of.map(|up| up.to_string()),
            tags: kept.tags.iter().map(|one| one.to_string()).collect(),
            flagged: kept.flagged.as_ref().map(|mark| Marked {
                at: mark.at.to_string(),
                said: mark.body.clone(),
                via: mark.via.clone(),
            }),
            folder_was: kept.folder_was.clone(),
        })
        .collect()
}

fn hanging(state: &State, parent: Option<tisty_core::model::FolderId>) -> Vec<Folded> {
    state
        .under(parent)
        .into_iter()
        .flat_map(|one| {
            let mut branch = vec![Folded {
                id: one.id.to_string(),
                name: one.name.clone(),
                parent: one.parent.map(|at| at.to_string()),
                icon: one.icon.clone(),
                color: one.color.clone(),
                holds: state.held_by(one.id),
                archived: one.archived,
                away: state.folder_away(one.id),
            }];
            branch.append(&mut hanging(state, Some(one.id)));
            branch
        })
        .collect()
}

pub fn one_step_back(session: &Session, id: &str) -> Option<String> {
    let was = tisty_core::docs::read_before(session.paths.data(), id)?;
    let now = tisty_core::docs::read(&session.paths.docs(), id).ok()?;
    if tisty_core::docs::unchanged(&now, &was) {
        return None;
    }
    let print = tisty_core::attach::printed(now.as_bytes());
    match tisty_core::docs::before_left_at(session.paths.data(), id).as_deref() == Some(&print) {
        true => Some(was),
        false => None,
    }
}

pub fn went_back(session: &mut Session, id: &str) -> Answer<tisty_core::docs::Doc> {
    if session.state.bolted(id) {
        return Err(Refusal::of(match session.state.away(id) {
            true => "documentAway",
            false => "documentLocked",
        }));
    }
    let Some(was) = one_step_back(session, id) else {
        return Err(Refusal::of("nothingKeptBeside"));
    };
    let root = session.paths.docs();
    let now = tisty_core::docs::read(&root, id)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be read", e))?;
    let print = tisty_core::attach::printed(now.as_bytes());
    tisty_core::docs::kept_before(session.paths.data(), id, &now, &was)
        .map_err(|e| blamed(channel::WINDOW, "what a document said could not be kept", e))?;
    let made =
        tisty_core::docs::rewrite(&root, session.paths.data(), id, &was, &print).map_err(|e| {
            match e {
                tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
                _ => blamed(channel::WINDOW, "a document could not be written", e),
            }
        })?;
    let whole = match made {
        tisty_core::docs::Rewrite::Moved => return Err(Refusal::about("documentMoved", id)),
        tisty_core::docs::Rewrite::Made { whole, .. } => whole,
    };
    session.mind_body(id, &tisty_core::docs::settled(&whole));
    session.corpus.forget(id);
    let hand = signing(&session.state);
    session.retell(id, &whole, hand);
    Ok(tisty_core::docs::Doc {
        title: tisty_core::docs::titled(&whole),
        id: id.to_string(),
    })
}

#[tauri::command(async)]
pub fn doc_back(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<tisty_core::docs::Doc> {
    let mut session = held(&session);
    went_back(&mut session, &id)
}

#[tauri::command(async)]
pub fn doc_backable(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<bool> {
    let session = held(&session);
    Ok(!session.state.bolted(&id) && one_step_back(&session, &id).is_some())
}

#[tauri::command(async)]
pub fn doc_write(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    body: String,
    anyway: Option<bool>,
) -> Answer<tisty_core::docs::Doc> {
    let mut session = held(&session);
    if session.state.bolted(&id) {
        return Err(Refusal::of(match session.state.away(&id) {
            true => "documentAway",
            false => "documentLocked",
        }));
    }
    if !anyway.unwrap_or(false) && session.moved(&id) {
        return Err(Refusal::about("documentMoved", id));
    }
    let root = session.paths.docs();
    if tisty_core::docs::read(&root, &id).is_ok_and(|was| tisty_core::docs::unchanged(&was, &body))
    {
        return Ok(tisty_core::docs::Doc {
            title: tisty_core::docs::titled(&body),
            id,
        });
    }
    tisty_core::docs::write(&root, &id, &body).map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { limit, .. } => {
            Refusal::about("documentTooLong", weighed(limit))
        }
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        _ => blamed(channel::WINDOW, "a document could not be written", e),
    })?;
    session.mind_body(&id, &tisty_core::docs::settled(&body));
    session.corpus.forget(&id);
    let hand = signing(&session.state);
    session.retell(&id, &body, hand);
    let title = tisty_core::docs::titled(&body);
    Ok(tisty_core::docs::Doc { title, id })
}

/// What reading a body catches up with: the title and the tags both come out of it, and neither
/// is worth a line in the log unless it changed. Size alone is not news — it moves with every
/// keystroke — but where there is news anyway, the note carries the size it was read at.
fn noted(session: &mut Session, file: &str, body: &str) {
    let Some(kept) = session.state.docs.values().find(|one| one.file == file) else {
        return;
    };
    let told = tisty_core::event::Said {
        bytes: kept.bytes,
        by: None,
        ..tisty_core::event::Said::of(body)
    };
    if !told.news_for(kept) {
        return;
    }
    let id = kept.id;
    let said = tisty_core::event::Said {
        bytes: Some(tisty_core::docs::settled(body).len() as u64),
        // Reading a document is not writing into it, whoever happens to be signing.
        by: None,
        ..told
    };
    if let Err(e) = session.commit(Op::DocSaid { id, d: said }) {
        witness::warn(
            channel::STORE,
            "what a document says of itself could not be written down",
            &[
                ("at", Fact::Id(id.to_string())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
    }
}

/// Read as a file, ordered from the log: a body that arrived from elsewhere may say otherwise.
#[tauri::command(async)]
pub fn doc_order(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    body: String,
) -> Answer<bool> {
    Ok(held(&session).retell(&id, &body, None))
}

#[cfg(test)]
#[path = "papers_test.rs"]
mod tests;
