use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Refusal, Session, Task, finding, held, herald, show, weighed};

#[tauri::command]
pub fn attach(
    session: tauri::State<'_, Mutex<Session>>,
    path: String,
    label: Option<String>,
    roomy: Option<bool>,
) -> Answer<String> {
    let source = std::path::PathBuf::from(&path);
    let name = label
        .or_else(|| {
            source
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
        })
        .unwrap_or_default();
    kept_from(&session, &source, name, roomy)
}

/// What a pasted file is called: its own name without any folder, or a plain one when it brings none.
pub(crate) fn pasted_name(raw: &str) -> String {
    let leaf = raw.rsplit(['/', '\\']).next().unwrap_or_default().trim();
    if leaf.is_empty() || leaf.starts_with('.') {
        "pasted.png".to_string()
    } else {
        leaf.to_string()
    }
}

#[tauri::command]
pub fn attach_pasted(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
    bytes: Vec<u8>,
) -> Answer<String> {
    let name = pasted_name(&name);
    // A pasted file has no place on disk, so it waits in a folder of its own until it is kept.
    let room = std::env::temp_dir().join(format!(
        "tisty-pasted-{}-{}",
        std::process::id(),
        jiff::Timestamp::now().as_nanosecond()
    ));
    std::fs::create_dir_all(&room).map_err(|_| Refusal::about("cannotRead", name.clone()))?;
    let source = room.join(&name);
    let written = std::fs::write(&source, &bytes);
    let kept = written
        .map_err(|_| Refusal::about("cannotRead", name.clone()))
        .and_then(|()| kept_from(&session, &source, name, Some(true)));
    let _ = std::fs::remove_dir_all(&room);
    kept
}

fn kept_from(
    session: &tauri::State<'_, Mutex<Session>>,
    source: &std::path::Path,
    name: String,
    roomy: Option<bool>,
) -> Answer<String> {
    let (root, ceiling) = {
        let session = held(session);
        let ceiling = if roomy.unwrap_or(false) {
            tisty_core::attach::COPIED_IN_DOC
        } else {
            session.config.copies_up_to()
        };
        (session.paths.data().to_path_buf(), ceiling)
    };
    let kept = tisty_core::attach::keep(source, &root, ceiling).map_err(|e| {
        witness::warn(channel::ATTACH, "the file could not be kept", &e.told());
        match e {
            tisty_core::Error::AttachmentTooBig { limit, .. } => Refusal::about(
                if roomy.unwrap_or(false) {
                    "attachmentTooBigHere"
                } else {
                    "attachmentTooBig"
                },
                weighed(limit),
            ),
            _ => Refusal::about("cannotRead", name.clone()),
        }
    })?;

    if let Err(e) = held(session).commit(kept.told()) {
        witness::warn(
            channel::ATTACH,
            "what an attachment holds could not be written down",
            &[
                ("at", Fact::Id(kept.at.clone())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
    }
    Ok(kept.written(&name))
}

#[tauri::command]
pub fn owed(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<String>> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let session = held(&session);
    let today = jiff::Zoned::now().date();
    Ok(session
        .state
        .owed_since(id, today)
        .iter()
        .map(ToString::to_string)
        .collect())
}

#[tauri::command]
pub fn complete(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    also: Option<Vec<String>>,
) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let also = also
        .unwrap_or_default()
        .iter()
        .map(|day| {
            day.parse::<jiff::civil::Date>()
                .map_err(|_| Refusal::about("notADate", day))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut session = held(&session);
    let ops = if also.is_empty() {
        session.state.completing(id, jiff::Zoned::now())
    } else {
        session.state.covering(id, jiff::Zoned::now(), &also)
    };
    session.commit_all(ops)?;
    let task = session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    drop(session);
    if task.status == tisty_core::Status::Done {
        let _ = herald::told(
            &app,
            tisty_core::herald::Happening::Done {
                title: task.title.clone(),
            },
        );
    }
    Ok(task)
}

fn refused(found: &std::path::Path, reference: &str) -> Refusal {
    match tisty_core::holes::a_hole(found) {
        true => Refusal::about("heldAway", reference.to_string()),
        false => Refusal::about("cannotRead", reference.to_string()),
    }
}

pub(crate) fn read_out(reference: String, at: finding::Where) -> Answer<Vec<u8>> {
    let found = finding::handed_over(&reference, &at)?;
    let body = std::fs::read(&found).map_err(|_| refused(&found, &reference))?;
    tisty_core::lately::used(&at.reached, &reference);
    Ok(body)
}

#[tauri::command]
pub async fn attached(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
) -> Answer<Vec<u8>> {
    finding::fetched(&session, reference, read_out).await
}

fn pointed_at(reference: String, at: finding::Where) -> Answer<String> {
    let found = finding::handed_over(&reference, &at)?;
    tisty_core::lately::used(&at.reached, &reference);
    Ok(found.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn served(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
) -> Answer<String> {
    finding::fetched(&session, reference, pointed_at).await
}

fn taken_out(reference: String, into: String, at: finding::Where) -> Answer<()> {
    let from = finding::handed_over(&reference, &at)?;
    std::fs::copy(&from, &into).map_err(|e| {
        witness::warn(
            channel::ATTACH,
            "an attachment could not be taken out",
            &[
                ("at", Fact::Id(reference.clone())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
        Refusal::about("cannotWrite", into)
    })?;
    tisty_core::lately::used(&at.reached, &reference);
    Ok(())
}

#[tauri::command]
pub async fn attach_export(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
    into: String,
) -> Answer<()> {
    finding::fetched(&session, reference, move |reference, at| {
        taken_out(reference, into, at)
    })
    .await
}

#[tauri::command(async)]
pub fn weighs(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<u64> {
    let looking = finding::where_to(&session, &reference);
    finding::weight_of(
        &reference,
        &looking.data,
        looking.shared.as_deref(),
        looking.weight,
    )
    .ok_or_else(|| Refusal::about("cannotRead", reference))
}

fn looked_up(reference: String, at: finding::Where) -> Answer<std::path::PathBuf> {
    let found = finding::handed_over(&reference, &at)?;
    tisty_core::lately::used(&at.reached, &reference);
    Ok(found)
}

#[tauri::command]
pub async fn opened(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
) -> Answer<()> {
    let at = finding::fetched(&session, reference.clone(), looked_up).await?;
    if !safe_to_open(&at) {
        return show(&at, &reference);
    }
    handed(&at).map_err(|_| Refusal::about("cannotOpen", reference))?;
    let _ = app;
    Ok(())
}

fn handed(at: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    tauri_plugin_opener::open_path(at, None::<&str>)?;
    Ok(())
}

fn safe_to_open(at: &std::path::Path) -> bool {
    let name = at
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .trim_end_matches(['.', ' '])
        .to_lowercase();
    let ext = name.rsplit_once('.').map(|(_, e)| e).unwrap_or_default();
    matches!(
        ext,
        "pdf"
            | "txt"
            | "md"
            | "markdown"
            | "rtf"
            | "csv"
            | "tsv"
            | "json"
            | "xml"
            | "yaml"
            | "yml"
            | "toml"
            | "log"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "avif"
            | "bmp"
            | "tiff"
            | "tif"
            | "heic"
            | "svg"
            | "ico"
            | "mp3"
            | "wav"
            | "flac"
            | "aac"
            | "ogg"
            | "opus"
            | "m4a"
            | "mp4"
            | "m4v"
            | "mov"
            | "webm"
            | "mkv"
            | "avi"
            | "doc"
            | "docx"
            | "xls"
            | "xlsx"
            | "ppt"
            | "pptx"
            | "odt"
            | "ods"
            | "odp"
            | "pages"
            | "numbers"
            | "key"
            | "epub"
            | "zip"
            | "gz"
            | "tar"
            | "bz2"
            | "xz"
            | "7z"
    )
}

#[cfg(test)]
#[path = "attaching_test.rs"]
mod tests;
