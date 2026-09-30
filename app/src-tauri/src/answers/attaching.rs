use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Refusal, Session, Task, elsewhere, finding, held, herald, show, weighed};

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

    let (root, ceiling) = {
        let session = held(&session);
        let ceiling = if roomy.unwrap_or(false) {
            tisty_core::attach::COPIED_IN_DOC
        } else {
            session.config.copies_up_to()
        };
        (session.paths.data().to_path_buf(), ceiling)
    };
    let kept = tisty_core::attach::keep(&source, &root, ceiling).map_err(|e| {
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

#[tauri::command]
pub async fn attached(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
) -> Answer<Vec<u8>> {
    let (data, shared) = finding::where_to(&session);
    elsewhere(move || {
        let at = match finding::found_in(&reference, &data, shared.as_deref()) {
            finding::Sought::At(at) => at,
            other => return Err(finding::unreachable(other, reference)),
        };
        std::fs::read(&at).map_err(|_| Refusal::about("cannotRead", reference))
    })
    .await?
}

#[tauri::command]
pub async fn served(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
) -> Answer<String> {
    let (data, shared) = finding::where_to(&session);
    elsewhere(move || {
        let at = match finding::found_in(&reference, &data, shared.as_deref()) {
            finding::Sought::At(at) => at,
            other => return Err(finding::unreachable(other, reference)),
        };
        Ok(at.to_string_lossy().into_owned())
    })
    .await?
}

#[tauri::command]
pub async fn attach_export(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
    into: String,
) -> Answer<()> {
    let (data, shared) = finding::where_to(&session);
    elsewhere(move || {
        let from = match finding::found_in(&reference, &data, shared.as_deref()) {
            finding::Sought::At(at) => at,
            other => return Err(finding::unreachable(other, reference)),
        };
        std::fs::copy(&from, &into).map_err(|e| {
            witness::warn(
                channel::ATTACH,
                "an attachment could not be taken out",
                &[
                    ("at", Fact::Id(reference)),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            Refusal::about("cannotWrite", into)
        })?;
        Ok(())
    })
    .await?
}

#[tauri::command(async)]
pub fn weighs(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<u64> {
    let (data, shared) = finding::where_to(&session);
    let at = finding::where_it_lies(&reference, &data, shared.as_deref())
        .ok_or_else(|| Refusal::about("cannotRead", reference.clone()))?;
    let told = std::fs::metadata(&at).map_err(|_| Refusal::about("cannotRead", reference))?;
    Ok(told.len())
}

#[tauri::command]
pub async fn opened(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
) -> Answer<()> {
    let (data, shared) = finding::where_to(&session);
    let asked = reference.clone();
    let at = elsewhere(
        move || match finding::found_in(&asked, &data, shared.as_deref()) {
            finding::Sought::At(at) => Ok(at),
            other => Err(finding::unreachable(other, asked)),
        },
    )
    .await??;
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
