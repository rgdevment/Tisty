use std::sync::Mutex;

use tauri::Emitter;
use tisty_core::witness::{self, Fact, channel};
use tisty_core::{Op, Paths};

use crate::{Answer, Packing, Refusal, Session, blamed, folder_open, held, signing, weighed};

#[tauri::command(async)]
pub fn doc_copy(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<tisty_core::docs::Doc> {
    held(&session).copy_doc(&id)
}

#[tauri::command(async)]
pub fn doc_export(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    into: String,
) -> Answer<Taken> {
    let mut session = held(&session);
    if let Ok(body) = tisty_core::docs::read(&session.paths.docs(), &id) {
        let _ = session.retell(&id, &body, None);
    }
    let said = tisty_core::docs::read(&session.paths.docs(), &id).unwrap_or_default();
    let pages: Vec<String> = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .map(|one| {
            session
                .state
                .pages_read(one.id, &said)
                .iter()
                .map(|page| page.file.clone())
                .collect()
        })
        .unwrap_or_default();
    let beside = session.dest();
    tisty_core::docs::with_pages(
        session.paths.data(),
        &id,
        &pages,
        std::path::Path::new(&into),
        beside.as_deref(),
    )
    .map_err(|e| {
        witness::warn(
            channel::WINDOW,
            "a document could not be taken out",
            &[
                ("id", Fact::Id(id.clone())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
        Refusal::about("cannotWrite", into)
    })
    .map(|took| {
        if !took.left.is_empty() {
            witness::warn(
                channel::WINDOW,
                "a document went out without everything it points at",
                &[
                    ("id", Fact::Id(id.clone())),
                    ("left", Fact::Why(took.left.join("; "))),
                ],
            );
        }
        Taken {
            files: took.files,
            missed: took.missed,
            left: took.left.len(),
        }
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Taken {
    files: usize,
    missed: usize,
    left: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Packed {
    docs: usize,
    pages: usize,
    folders: usize,
    files: usize,
    missed: usize,
    left: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Unpacked {
    docs: usize,
    pages: usize,
    folders: usize,
    joined: usize,
    files: usize,
    missed: usize,
}

#[tauri::command(async)]
pub fn doc_import(
    session: tauri::State<'_, Mutex<Session>>,
    from: String,
    folder: Option<String>,
) -> Answer<Arrived> {
    let folder = folder
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    if let Some(at) = folder {
        landing(&held(&session), at)?;
    }
    let (doc, _, left) = arrived(&session, std::path::Path::new(&from), folder)?;
    Ok(Arrived { doc, left })
}

fn landing(session: &Session, at: tisty_core::model::FolderId) -> Answer<()> {
    if !session.state.folders.contains_key(&at) {
        return Err(Refusal::of("noSuchFolder"));
    }
    folder_open(&session.state, at, true)
}

#[derive(serde::Serialize, Clone, Copy, Default)]
pub struct Left {
    missed: usize,
    notes: usize,
}

#[derive(serde::Serialize)]
pub struct Arrived {
    #[serde(flatten)]
    doc: tisty_core::docs::Doc,
    #[serde(flatten)]
    left: Left,
}

// Reading and copying run without the session held, so a slow drive never stalls the rest of the window.
pub(crate) fn arrived(
    session: &tauri::State<'_, Mutex<Session>>,
    from: &std::path::Path,
    folder: Option<tisty_core::model::FolderId>,
) -> Answer<(tisty_core::docs::Doc, ulid::Ulid, Left)> {
    let paths = held(session).paths.clone();
    let raw = read_outside(from)?;
    let webbed = from
        .extension()
        .is_some_and(|one| one.eq_ignore_ascii_case("html") || one.eq_ignore_ascii_case("htm"));
    let raw = match webbed {
        true => tisty_core::arriving::from_html(&raw),
        false => raw,
    };
    let body = tisty_core::arriving::tidied(&raw).body;
    let own: Vec<std::path::PathBuf> = [paths.data(), paths.config(), paths.cache()]
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .collect();
    let mut reach = |at: &std::path::Path| {
        let at = at
            .canonicalize()
            .map_err(|_| "no file is there".to_string())?;
        match own.iter().any(|one| at.starts_with(one)) {
            true => Err("it is one of Tisty's own files".to_string()),
            false => Ok(at),
        }
    };
    let (body, brought) = tisty_core::beside::brought(
        &paths,
        from,
        &body,
        &mut tisty_core::beside::Way {
            reach: &mut reach,
            outside: "",
            notes: false,
        },
    );
    let mut session = held(session);
    let (doc, id) = imported(&mut session, &body, folder)?;
    if !brought.told.is_empty() {
        session
            .commit_all(brought.told)
            .map_err(|e| blamed(channel::WINDOW, "a document's files could not be kept", e))?;
    }
    let left = Left {
        missed: brought.missed.len(),
        notes: brought.papers.len(),
    };
    Ok((doc, id, left))
}

// Past this, what was copied is a whole site rather than something a person means to paste.
const PASTED_AT_MOST: usize = 2 * 1024 * 1024;

#[tauri::command(async)]
pub fn doc_from_html(html: String) -> Answer<String> {
    if html.len() > PASTED_AT_MOST {
        return Err(Refusal::about(
            "documentTooBig",
            weighed(PASTED_AT_MOST as u64),
        ));
    }
    let said = tisty_core::arriving::tidied(&tisty_core::arriving::from_html(&html)).body;
    Ok(said.trim().to_string())
}

pub(crate) fn read_outside(from: &std::path::Path) -> Answer<String> {
    tisty_core::docs::read_outside(from).map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { limit, .. } => {
            Refusal::about("documentTooBig", weighed(limit))
        }
        _ => Refusal::about("cannotRead", from.display().to_string()),
    })
}

pub(crate) fn imported(
    session: &mut Session,
    body: &str,
    folder: Option<tisty_core::model::FolderId>,
) -> Answer<(tisty_core::docs::Doc, ulid::Ulid)> {
    if let Some(at) = folder {
        landing(session, at)?;
    }
    let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be imported", e))?;

    let order = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.folder == folder)
            .map(|one| one.order.as_str()),
    );
    let signed_as = signing(&session.state);
    let id = ulid::Ulid::generate();
    session.commit(Op::DocAdd {
        id,
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: false,
            made: None,
            by: signed_as.clone(),
            file: made.id.clone(),
            order,
            said: Some(tisty_core::event::Said::of(body)),
            folder,
            page_of: None,
        },
    })?;
    Ok((made, id))
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Afoot {
    stage: &'static str,
    far: u64,
    done: usize,
    whole: usize,
}

fn along_the_way(
    app: &tauri::AppHandle,
    stage: &'static str,
) -> impl Fn(tisty_core::parcel::Step) + use<> {
    let app = app.clone();
    let said = std::sync::atomic::AtomicU64::new(u64::MAX);
    move |step| {
        let far = match step.whole {
            0 => 0,
            whole => (step.done as u64 * 100 / whole as u64).min(100),
        };
        if said.swap(far, std::sync::atomic::Ordering::Relaxed) == far {
            return;
        }
        let _ = app.emit(
            "carrying",
            Afoot {
                stage,
                far,
                done: step.done,
                whole: step.whole,
            },
        );
    }
}

fn standing(
    session: &tauri::State<'_, Mutex<Session>>,
    which: &[String],
) -> (Paths, tisty_core::State, Option<std::path::PathBuf>) {
    let mut session = held(session);
    for one in which {
        if let Ok(body) = tisty_core::docs::read(&session.paths.docs(), one) {
            let _ = session.retell(one, &body, None);
        }
    }
    (session.paths.clone(), session.state.clone(), session.dest())
}

#[tauri::command(async)]
pub async fn docs_pack(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, Packing>,
    which: Vec<String>,
    into: String,
    number: Option<String>,
) -> Answer<Packed> {
    let _done = alone.inner().taken()?;
    let (paths, state, beside) = standing(&session, &which);
    let asked = which.clone();
    let at = into.clone();
    let telling = along_the_way(&app, "packing");
    let locking = along_the_way(&app, "locking");
    let sent = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::parcel::written(
            &paths,
            &state,
            &asked,
            std::path::Path::new(&at),
            &tisty_core::parcel::Along {
                also: beside.as_deref(),
                say: Some(&telling),
                then: Some(&locking),
            },
            number.as_deref(),
        )
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::warn(
            channel::WINDOW,
            "a parcel of documents could not be written",
            &[("why", Fact::Why(e.to_string()))],
        );
        match e {
            tisty_core::Error::TooBig => Refusal::of("tooBig"),
            tisty_core::Error::NothingToCarry => Refusal::of("nothingToCarry"),
            _ => Refusal::about("cannotWrite", into.clone()),
        }
    })?;

    if !sent.left.is_empty() {
        witness::warn(
            channel::WINDOW,
            "a parcel went out without everything it points at",
            &[("left", Fact::Why(sent.left.join("; ")))],
        );
    }
    Ok(Packed {
        docs: sent.docs,
        pages: sent.pages,
        folders: sent.folders,
        files: sent.files,
        missed: sent.missed,
        left: sent.left.len(),
    })
}

#[tauri::command(async)]
pub async fn docs_take_out(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, Packing>,
    which: Vec<String>,
    into: String,
) -> Answer<Packed> {
    let _done = alone.inner().taken()?;
    let (paths, state, beside) = standing(&session, &which);
    let asked = which.clone();
    let at = into.clone();
    let telling = along_the_way(&app, "takingOut");
    let sent = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::parcel::plainly(
            paths.data(),
            &state,
            &asked,
            std::path::Path::new(&at),
            &tisty_core::parcel::Along {
                also: beside.as_deref(),
                say: Some(&telling),
                then: None,
            },
        )
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::warn(
            channel::WINDOW,
            "the documents could not be taken out",
            &[("why", Fact::Why(e.to_string()))],
        );
        match e {
            tisty_core::Error::NothingToCarry => Refusal::of("nothingToCarry"),
            _ => Refusal::about("cannotWrite", into.clone()),
        }
    })?;

    if !sent.left.is_empty() {
        witness::warn(
            channel::WINDOW,
            "documents went out without everything they point at",
            &[("left", Fact::Why(sent.left.join("; ")))],
        );
    }
    Ok(Packed {
        docs: sent.docs,
        pages: sent.pages,
        folders: sent.folders,
        files: sent.files,
        missed: sent.missed,
        left: sent.left.len(),
    })
}

#[tauri::command(async)]
pub async fn docs_unpack(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, Packing>,
    from: String,
    number: Option<String>,
) -> Answer<Unpacked> {
    let _done = alone.inner().taken()?;
    let (paths, state, device) = {
        let session = held(&session);
        (
            session.paths.clone(),
            session.state.clone(),
            session.config.device_id.clone(),
        )
    };
    let at = from.clone();
    let telling = along_the_way(&app, "landing");
    let opening = along_the_way(&app, "opening");
    let (landed, ops) = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::parcel::taken(
            &paths,
            &state,
            &device,
            std::path::Path::new(&at),
            &tisty_core::parcel::Along {
                also: None,
                say: Some(&telling),
                then: Some(&opening),
            },
            number.as_deref(),
        )
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| match e {
        tisty_core::Error::NotAParcel(_) => Refusal::about("notAParcel", from.clone()),
        tisty_core::Error::ParcelNewer(_) => Refusal::of("parcelNewer"),
        tisty_core::Error::ParcelLocked => Refusal::of("parcelLocked"),
        tisty_core::Error::WrongNumber => Refusal::of("wrongNumber"),
        tisty_core::Error::ParcelTorn => Refusal::of("parcelTorn"),
        tisty_core::Error::NoRoom { needs, .. } => Refusal::about("noRoom", weighed(needs)),
        tisty_core::Error::TooBig => Refusal::of("tooBig"),
        other => blamed(channel::WINDOW, "a parcel could not be taken in", other),
    })?;

    // What landed is only real once the log says so: if it cannot be written, the bodies go
    // rather than sit in the folder as documents nobody knows about.
    let files: Vec<String> = ops
        .iter()
        .filter_map(|one| match one {
            tisty_core::Op::DocAdd { d, .. } => Some(d.file.clone()),
            _ => None,
        })
        .collect();
    let mut held = held(&session);
    if let Err(e) = held.commit_all(ops) {
        let papers = held.paths.docs();
        for file in files {
            let _ = tisty_core::docs::remove(&papers, &file);
        }
        return Err(blamed(
            channel::WINDOW,
            "a parcel landed but was not written",
            e,
        ));
    }
    drop(held);
    Ok(Unpacked {
        docs: landed.docs,
        pages: landed.pages,
        folders: landed.folders,
        joined: landed.joined,
        files: landed.files,
        missed: landed.missed,
    })
}
