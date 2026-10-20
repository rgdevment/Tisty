use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};
use tisty_core::{Op, Reading, State};

use crate::{
    Answer, Refusal, Session, Task, blamed, doc_out, elsewhere, folder_open, held, placed,
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
    read_minded(&session, id)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Opened {
    body: String,
    print: String,
}

/// The print rides back with the save, so only the words the window actually holds are written over.
#[tauri::command(async)]
pub fn doc_opened(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Opened> {
    let body = read_minded(&session, id)?;
    let print = tisty_core::docs::print_of_text(&body);
    Ok(Opened { body, print })
}

fn read_minded(session: &tauri::State<'_, Mutex<Session>>, id: String) -> Answer<String> {
    let root = held(session).paths.docs();
    let read = tisty_core::docs::read(&root, &id);
    if let Ok(body) = &read {
        let mut session = held(session);
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
        _ if !root.join(format!("{id}.md")).exists() && still_coming(session, &id) => {
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

/// A file from outside Tisty tells only what the disk knows of it; no log names its author.
pub(crate) fn outside_facts(at: &std::path::Path) -> Answer<Facts> {
    let about =
        std::fs::metadata(at).map_err(|_| Refusal::about("noSuchDoc", at.display().to_string()))?;
    Ok(Facts {
        made: seconds(about.created()),
        wrote: seconds(about.modified()),
        bytes: about.len(),
        pages: 0,
        author: None,
        editor: None,
        born: None,
    })
}

fn seconds(at: std::io::Result<std::time::SystemTime>) -> Option<i64> {
    at.ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|gone| gone.as_secs() as i64)
}

#[tauri::command(async)]
pub fn keep_file(at: String, bytes: Vec<u8>) -> Answer<()> {
    std::fs::write(&at, bytes).map_err(|e| {
        blamed(
            channel::WINDOW,
            "a file the window made could not be written",
            tisty_core::Error::Io(e),
        )
    })
}

// The page goes through a window nobody sees, whose engine lays it on paper the way it draws it.
#[tauri::command]
pub async fn page_pdf(app: tauri::AppHandle, html: String, paper: String) -> Answer<Vec<u8>> {
    let room = super::carrying::Room(
        std::env::temp_dir().join(format!("tisty-pdf-{}", ulid::Ulid::generate())),
    );
    let page = room.0.join("page.html");
    let to = room.0.join("page.pdf");
    let cannot = || Refusal::about("cannotWrite", to.display().to_string());
    let (folder, written) = (room.0.clone(), page.clone());
    let laid = tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&folder).and_then(|()| std::fs::write(&written, html))
    })
    .await;
    if !matches!(laid, Ok(Ok(()))) {
        return Err(cannot());
    }
    let url = url::Url::from_file_path(&page).map_err(|()| cannot())?;
    let (told, heard) = std::sync::mpsc::channel();
    let told = std::sync::Mutex::new(Some(told));
    let sheet = crate::pdf::leaf(&paper);
    let printing = to.clone();
    let label = format!(
        "paper-{}",
        ulid::Ulid::generate().to_string().to_lowercase()
    );
    let window = tauri::WebviewWindowBuilder::new(&app, label, tauri::WebviewUrl::External(url))
        .visible(false)
        .on_page_load(move |window, loaded| {
            if loaded.event() != tauri::webview::PageLoadEvent::Finished {
                return;
            }
            let Some(once) = told.lock().ok().and_then(|mut one| one.take()) else {
                return;
            };
            let back = once.clone();
            if crate::pdf::print(&window, printing.clone(), sheet, once).is_err() {
                let _ = back.send(false);
            }
        })
        .build()
        .map_err(|_| cannot())?;
    let made = to.clone();
    let printed = tauri::async_runtime::spawn_blocking(move || {
        let done = heard
            .recv_timeout(std::time::Duration::from_secs(60))
            .unwrap_or(false);
        done.then(|| std::fs::read(&made).ok()).flatten()
    })
    .await
    .ok()
    .flatten();
    let _ = window.destroy();
    tauri::async_runtime::spawn_blocking(move || drop(room));
    printed.ok_or_else(cannot)
}

// The pictures come from the store and its sync folder alike, read whole since Word carries them inside.
#[tauri::command(async)]
pub fn doc_word(
    session: tauri::State<'_, Mutex<Session>>,
    bodies: Vec<String>,
    at: String,
) -> Answer<()> {
    let (data, beside) = {
        let session = held(&session);
        (session.paths.data().to_path_buf(), session.dest())
    };
    let mut picture = |src: &str| {
        let found = tisty_core::attach::found(src, &data, beside.as_deref()).ok()?;
        let file = std::fs::File::open(found).ok()?;
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(
            &mut std::io::Read::take(file, tisty_core::arriving::WORD_AT_MOST),
            &mut bytes,
        )
        .ok()?;
        Some(bytes)
    };
    let made = tisty_core::leaving_word::to_word(&bodies, &mut picture).map_err(|e| {
        blamed(
            channel::WINDOW,
            "a Word file could not be made",
            tisty_core::Error::Io(e),
        )
    })?;
    keep_file(at, made)
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
    unmarked(
        &session,
        &id,
        |one| one.flagged.is_some(),
        |id| Op::DocUnflag { id },
    )
}

#[tauri::command]
pub fn doc_keep_compacted(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    unmarked(
        &session,
        &id,
        |one| one.compacted.is_some(),
        |id| Op::DocUncompacted { id },
    )
}

#[tauri::command(async)]
pub fn doc_compacted_last(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<bool> {
    Ok(compacted_last(&held(&session), &id))
}

pub fn compacted_last(session: &Session, id: &str) -> bool {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .and_then(|one| one.compacted.as_ref())
        .is_some_and(|done| {
            tisty_core::docs::read(&session.paths.docs(), id).is_ok_and(|now| done.still_last(&now))
        })
}

fn unmarked(
    session: &tauri::State<'_, Mutex<Session>>,
    id: &str,
    marked: impl Fn(&tisty_core::model::Kept) -> bool,
    settle: impl Fn(tisty_core::model::DocId) -> Op,
) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let mut session = held(session);
    match session.state.docs.get(&id) {
        None => Err(Refusal::of("noSuchDoc")),
        Some(one) if !marked(one) => Ok(()),
        Some(_) => {
            session.commit(settle(id))?;
            Ok(())
        }
    }
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
            said: Some(tisty_core::event::Said::of("")),
            folder,
            page_of,
            task_of: None,
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
pub async fn settle_paper(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    keep: String,
    marked: Option<String>,
) -> Answer<Option<String>> {
    let (carrier, here, shown) = {
        let session = held(&session);
        // Settling what two machines already wrote is not writing into it, so being archived is no
        // reason to leave the rift with no way out. Only a lock guards the text itself.
        if session.state.shut_tight(&id) {
            return Err(Refusal::of("documentLocked"));
        }
        (
            session.carrying()?,
            session.here(),
            session.asked.get(&id).cloned(),
        )
    };
    let (keep, kept) = match keep.as_str() {
        "mine" => (tisty_carrier::Keep::Mine, "mine"),
        "theirs" => (tisty_carrier::Keep::Theirs, "theirs"),
        _ => (tisty_carrier::Keep::Both, "both"),
    };
    witness::note(
        channel::SYNC,
        "a document two machines wrote was settled",
        &[("at", Fact::Id(id.clone())), ("kept", Fact::Word(kept))],
    );
    let (asking, there, at) = (carrier.clone(), here.clone(), id.clone());
    let brought = elsewhere(move || {
        if !matches!(keep, tisty_carrier::Keep::Theirs)
            && let Some((theirs, _)) = &shown
            && asking.paper_print(&at).as_ref() != Some(theirs)
        {
            return Err(Refusal::of("movedUnderfoot"));
        }
        let seen = shown.as_ref().and_then(|(_, mine)| mine.as_deref());
        asking.settle(&there, &at, keep, seen).map_err(said)
    })
    .await??;

    let answer = {
        let mut session = held(&session);
        session.mind(&id);
        session.asked.remove(&id);
        let Some(body) = brought else {
            told_as_settled(&mut session, &id);
            return Ok(None);
        };
        other_version_kept(&mut session, &id, body, marked.as_deref())?
    };

    let at = id.clone();
    elsewhere(move || {
        carrier
            .settle(&here, &at, tisty_carrier::Keep::Mine, None)
            .map_err(said)
    })
    .await??;
    let mut session = held(&session);
    session.mind(&id);
    told_as_settled(&mut session, &id);
    Ok(answer)
}

// Asked again about the same arrival, keeping both must not leave one more copy each time.
fn other_version_kept(
    session: &mut Session,
    id: &str,
    body: String,
    marked: Option<&str>,
) -> Answer<Option<String>> {
    let paged = session
        .state
        .docs
        .values()
        .any(|one| one.file == id && one.task_of.is_some());
    if paged {
        return kept_on_the_page(session, id, &body, marked).map(|()| None);
    }
    let beside = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .map(|one| (one.folder, one.page_of, one.order.clone()));
    let same_as_mine = tisty_core::docs::read(&session.paths.docs(), id)
        .is_ok_and(|mine| tisty_core::docs::unchanged(&mine, &body));
    let body = match marked {
        Some(said) => tisty_core::docs::marked(&body, said),
        None => body,
    };
    let twin = match same_as_mine {
        true => Some(None),
        false => twin_of(session, beside.as_ref(), &body).map(Some),
    };
    if let Some(twin) = twin {
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
                said: Some(tisty_core::event::Said::of(&body)),
                page_of,
                task_of: None,
            },
        })
        .map_err(|e| blamed(channel::SYNC, "the other version was not written down", e))?;
    Ok(Some(file))
}

// A task's page is the task's alone, so the other version waits on it rather than as a loose document.
fn kept_on_the_page(
    session: &mut Session,
    id: &str,
    theirs: &str,
    marked: Option<&str>,
) -> Answer<()> {
    let root = session.paths.docs();
    let mine = tisty_core::docs::read(&root, id)
        .map_err(|e| blamed(channel::SYNC, "a task's page could not be read", e))?;
    let heading = marked
        .map(str::trim)
        .filter(|one| !one.is_empty())
        .map_or_else(
            || "Other version".to_string(),
            |said| {
                let mut letters = said.chars();
                letters
                    .next()
                    .map(|first| first.to_uppercase().chain(letters).collect())
                    .unwrap_or_default()
            },
        );
    let both = tisty_core::task_docs::kept_beside(&mine, theirs, &heading);
    if both == mine {
        return Ok(());
    }
    tisty_core::docs::write(&root, id, &both)
        .map_err(|e| blamed(channel::SYNC, "the other version could not be kept", e))?;
    witness::note(
        channel::SYNC,
        "the other version was kept on the task's page",
        &[("at", Fact::Id(id.to_string()))],
    );
    session.mind_body(id, &tisty_core::docs::settled(&both));
    session.corpus.forget(id);
    let hand = signing(&session.state);
    session.retell(id, &both, hand);
    Ok(())
}

// Unsigned, as settling writes nothing new, yet another machine asks about a body no history answers for.
fn told_as_settled(session: &mut Session, id: &str) {
    if let Ok(body) = tisty_core::docs::read(&session.paths.docs(), id) {
        session.retell(id, &body, None);
    }
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
    compacted: Option<Compaction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_was: Option<Vec<String>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Compaction {
    at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    via: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Marked {
    at: String,
    said: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    via: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instead: Option<String>,
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
    let mut kept_in_order: Vec<&tisty_core::model::Kept> = session.state.papers().collect();
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
                reason: mark.reason.clone(),
                instead: mark.instead.clone(),
            }),
            compacted: kept.compacted.as_ref().map(|done| Compaction {
                at: done.at.to_string(),
                via: done.via.clone(),
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

pub(crate) fn writable(state: &State, id: &str) -> Result<(), Refusal> {
    match (state.bolted(id), state.away(id)) {
        (false, _) => Ok(()),
        (true, true) => Err(Refusal::of("documentAway")),
        (true, false) => Err(Refusal::of("documentLocked")),
    }
}

pub fn went_back(session: &mut Session, id: &str) -> Answer<tisty_core::docs::Doc> {
    writable(&session.state, id)?;
    let Some(was) = one_step_back(session, id) else {
        return Err(Refusal::of("nothingKeptBeside"));
    };
    let root = session.paths.docs();
    let now = tisty_core::docs::read(&root, id)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be read", e))?;
    let print = tisty_core::attach::printed(now.as_bytes());
    let undone = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .filter(|one| {
            one.compacted
                .as_ref()
                .is_some_and(|done| done.undone_by(&now, &was))
        })
        .map(|one| one.id);
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
    session.unbin_named(&whole);
    // The body is back already; a mark left standing is a lesser wrong than reporting the undo as failed.
    if let Some(doc) = undone
        && let Err(e) = session.commit(Op::DocUncompacted { id: doc })
    {
        tisty_core::witness::warn(
            tisty_core::witness::channel::STORE,
            "a document went back past its compaction but the mark stayed",
            &[("why", tisty_core::witness::Fact::Why(e.to_string()))],
        );
    }
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Kept {
    id: String,
    title: String,
    print: String,
}

#[tauri::command(async)]
pub fn doc_write(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    body: String,
    anyway: Option<bool>,
    since: Option<String>,
) -> Answer<Kept> {
    let mut session = held(&session);
    writable(&session.state, &id)?;
    let since = since.filter(|_| !anyway.unwrap_or(false));
    if since.is_none() && !anyway.unwrap_or(false) && session.moved(&id) {
        return Err(Refusal::about("documentMoved", id));
    }
    let root = session.paths.docs();
    let title = tisty_core::docs::titled(&body);
    if let Ok(was) = tisty_core::docs::read(&root, &id)
        && tisty_core::docs::unchanged(&was, &body)
    {
        let print = tisty_core::docs::print_of_text(&was);
        return Ok(Kept { id, title, print });
    }
    let print = tisty_core::docs::print_of_text(&body);
    let wrote = match &since {
        Some(seen) => tisty_core::docs::write_over(&root, &id, &body, seen),
        None => tisty_core::docs::write(&root, &id, &body).map(|()| true),
    };
    if !wrote.map_err(unwritten)? {
        return Err(Refusal::about("documentMoved", id));
    }
    session.mind_body(&id, &tisty_core::docs::settled(&body));
    session.corpus.forget(&id);
    let hand = signing(&session.state);
    session.retell(&id, &body, hand);
    Ok(Kept { id, title, print })
}

fn unwritten(e: tisty_core::Error) -> Refusal {
    match e {
        tisty_core::Error::DocumentTooBig { limit, .. } => {
            Refusal::about("documentTooLong", weighed(limit))
        }
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        _ => blamed(channel::WINDOW, "a document could not be written", e),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Woven {
    body: String,
    print: String,
    title: String,
}

/// `None` when the two touched the same place, which only the person can choose between.
#[tauri::command(async)]
pub fn doc_merge(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    base: String,
    mine: String,
) -> Answer<Option<Woven>> {
    let mut session = held(&session);
    writable(&session.state, &id)?;
    let root = session.paths.docs();
    let theirs = tisty_core::docs::read(&root, &id)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be read", e))?;
    let Some(whole) = tisty_core::merge::merged(&base, &mine, &theirs) else {
        return Ok(None);
    };
    // The merge joins lines with `\n`; a body kept with `\r\n` stays so, or every line would read as changed.
    let whole = match theirs.contains("\r\n") && !theirs.replace("\r\n", "").contains('\n') {
        true => whole.replace("\r\n", "\n").replace('\n', "\r\n"),
        false => whole,
    };
    let seen = tisty_core::docs::print_of_text(&theirs);
    if !tisty_core::docs::write_over(&root, &id, &whole, &seen).map_err(unwritten)? {
        return Ok(None);
    }
    let body = tisty_core::docs::settled(&whole);
    session.mind_body(&id, &body);
    session.corpus.forget(&id);
    let hand = signing(&session.state);
    session.retell(&id, &body, hand);
    let print = tisty_core::docs::print_of_text(&body);
    let title = tisty_core::docs::titled(&body);
    Ok(Some(Woven { body, print, title }))
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

type Seen = (Option<String>, Vec<tisty_core::model::Tag>);

fn seen(kept: &tisty_core::model::Kept) -> Seen {
    (kept.title.clone(), kept.tags.clone())
}

/// Front matter tags only count since 1.31, so what an older build filed learns them once, off the window's way.
pub fn tags_once(app: tauri::AppHandle) {
    use tauri::{Emitter, Manager};
    std::thread::spawn(move || {
        let session = app.state::<Mutex<Session>>();
        let (root, owing) = {
            let held = held(&session);
            if held.config.tags_read == Some(true) {
                return;
            }
            let owing: Vec<(tisty_core::model::DocId, String, Seen)> = held
                .state
                .docs
                .values()
                .map(|one| (one.id, one.file.clone(), seen(one)))
                .collect();
            (held.paths.docs(), owing)
        };
        for some in owing.chunks(CATCHING_UP_AT_ONCE) {
            if !tags_read_in(&session, &root, some) {
                return;
            }
        }
        let _ = held(&session).keep(|config| config.tags_read = Some(true));
        let _ = app.emit("stirred", ());
    });
}

// A file the folder has not brought down is left for when it is opened, never pulled down here.
fn tags_read_in(
    session: &tauri::State<'_, Mutex<Session>>,
    root: &std::path::Path,
    some: &[(tisty_core::model::DocId, String, Seen)],
) -> bool {
    let read: Vec<_> = some
        .iter()
        .filter_map(|(id, file, was)| {
            let at = tisty_core::docs::resolve(root, file).ok()?;
            if tisty_core::holes::a_hole(&at) {
                return None;
            }
            let body = tisty_core::docs::read(root, file).ok()?;
            Some((*id, was, tisty_core::event::Said::of(&body).by(None)))
        })
        .collect();
    let mut held = held(session);
    if held.paths.docs() != root {
        return false;
    }
    // A document saved while this read it already holds what it says now.
    let ops: Vec<Op> = read
        .into_iter()
        .filter(|(id, was, said)| {
            held.state
                .docs
                .get(id)
                .is_some_and(|kept| seen(kept) == **was && said.news_for(kept))
        })
        .map(|(id, _, said)| Op::DocSaid { id, d: said })
        .collect();
    if ops.is_empty() {
        return true;
    }
    if let Err(e) = held.commit_all(ops) {
        witness::warn(
            channel::WINDOW,
            "the tags of older documents could not be written down",
            &[("why", Fact::Why(e.to_string()))],
        );
        return false;
    }
    true
}
