use std::sync::Mutex;

use tisty_core::witness::{self, channel};
use tisty_core::{Op, Reading, State};

use crate::{
    Answer, Refusal, Session, Task, blamed, doc_out, folder_open, held, leave, noted, placed,
    reading_as, said, signing, stop, weighed,
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
    let dest = match &held(session).config.sync {
        Some(tisty_core::config::Sync::Folder(dest)) => dest.clone(),
        _ => return false,
    };
    tisty_sync::paper_waiting(&dest, id)
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
pub fn parted(app: tauri::AppHandle) {
    leave(&app);
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
    let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
        return Err(Refusal::of("noRemote"));
    };
    let keep = match keep.as_str() {
        "mine" => tisty_sync::Keep::Mine,
        "theirs" => tisty_sync::Keep::Theirs,
        _ => tisty_sync::Keep::Both,
    };

    let data = session.paths.data().to_path_buf();
    let brought = tisty_sync::settle(&data, &dest, &id, keep).map_err(said)?;
    session.mind(&id);

    let Some(body) = brought else { return Ok(None) };
    let beside = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .map(|one| (one.folder, one.page_of, one.order.clone()));
    let body = match &marked {
        Some(said) => tisty_core::docs::marked(&body, said),
        None => body,
    };
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
                }),
                page_of,
            },
        })
        .map_err(|e| blamed(channel::SYNC, "the other version was not written down", e))?;

    tisty_sync::settle(&data, &dest, &id, tisty_sync::Keep::Mine).map_err(said)?;
    session.mind(&id);
    Ok(Some(file))
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
