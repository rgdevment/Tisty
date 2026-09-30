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
    let at = finding::where_to(&session);
    elsewhere(move || read_out(reference, at)).await?
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
    let at = finding::where_to(&session);
    elsewhere(move || pointed_at(reference, at)).await?
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
    let at = finding::where_to(&session);
    elsewhere(move || taken_out(reference, into, at)).await?
}

#[tauri::command(async)]
pub fn weighs(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<u64> {
    let looking = finding::where_to(&session);
    let at = finding::where_it_lies(&reference, &looking.data, looking.shared.as_deref())
        .ok_or_else(|| Refusal::about("cannotRead", reference.clone()))?;
    let told = std::fs::metadata(&at).map_err(|_| Refusal::about("cannotRead", reference))?;
    Ok(told.len())
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
    let looking = finding::where_to(&session);
    let asked = reference.clone();
    let at = elsewhere(move || looked_up(asked, looking)).await??;
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
