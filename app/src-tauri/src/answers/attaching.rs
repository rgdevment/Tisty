use std::sync::Mutex;

use tisty_core::witness::{self, Fact, channel};

use crate::{Answer, Refusal, Session, Task, finding, held, herald, show, weighed};

// Two keepings at once can each miss the other and store one file twice, so one keeps at a time.
#[derive(Default)]
pub struct Keeping(std::sync::Arc<Mutex<()>>);

impl Keeping {
    fn turn(&self) -> std::sync::Arc<Mutex<()>> {
        self.0.clone()
    }
}

fn kept_alone(
    lock: &Mutex<()>,
    source: &std::path::Path,
    root: &std::path::Path,
    ceiling: u64,
    name: &str,
) -> Answer<tisty_core::attach::Kept> {
    let _alone = lock
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    kept_at(source, root, ceiling, name)
}

// Turning a photo can take seconds, so it runs where blocking work goes, as a pasted file's keeping does.
#[tauri::command]
pub async fn attach(
    session: tauri::State<'_, Mutex<Session>>,
    keeping: tauri::State<'_, Keeping>,
    path: String,
    label: Option<String>,
    shape: Option<String>,
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
    let (root, ceiling) = room_for(&session);
    let lock = keeping.turn();
    let kept =
        crate::elsewhere(move || shaped(&lock, &source, name, shape.as_deref(), &root, ceiling))
            .await??;
    noted(&session, &kept.0);
    Ok(kept.0.written(&kept.1))
}

// Asked once by the window: where nothing here can draw a HEIC, it stays a file to open elsewhere.
#[tauri::command(async)]
pub fn draws_heic() -> bool {
    crate::drawn::draws_heic()
}

fn shaped(
    lock: &Mutex<()>,
    source: &std::path::Path,
    name: String,
    shape: Option<&str>,
    root: &std::path::Path,
    ceiling: u64,
) -> Answer<(tisty_core::attach::Kept, String)> {
    let Some(how) = shape else {
        return kept_alone(lock, source, root, ceiling, &name).map(|kept| (kept, name));
    };
    let named = as_jpeg(&name);
    let made = crate::drawn::reshaped(source, &named, how == "light")
        .ok_or_else(|| Refusal::about("cannotTurn", name.clone()))?;
    let kept = kept_alone(lock, &made, root, ceiling, &named);
    if let Some(room) = made.parent() {
        let _ = std::fs::remove_dir_all(room);
    }
    kept.map(|kept| (kept, named))
}

fn as_jpeg(name: &str) -> String {
    let stem = std::path::Path::new(name)
        .file_stem()
        .and_then(|one| one.to_str())
        .unwrap_or(name);
    format!("{stem}.jpg")
}

pub(crate) fn pasted_name(raw: &str, kind: &str) -> String {
    let leaf = raw.rsplit(['/', '\\']).next().unwrap_or_default().trim();
    let stem = leaf
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    // A drive mark, a character Windows refuses or a device name would put the bytes somewhere else.
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.ends_with(|one: char| one.is_ascii_digit()));
    let refused = leaf
        .chars()
        .any(|one| one.is_control() || "<>:\"|?*".contains(one));
    if !leaf.is_empty() && leaf != "." && leaf != ".." && !reserved && !refused {
        return leaf.to_string();
    }
    let ending = match kind {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "application/pdf" => "pdf",
        "text/plain" => "txt",
        _ => "bin",
    };
    format!("pasted.{ending}")
}

static PASTED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

// Raw bytes on a thread of its own, so a large screenshot never stalls the window.
#[tauri::command]
pub async fn attach_pasted(
    session: tauri::State<'_, Mutex<Session>>,
    keeping: tauri::State<'_, Keeping>,
    request: tauri::ipc::Request<'_>,
) -> Answer<String> {
    let said = |key: &str| {
        request
            .headers()
            .get(key)
            .and_then(|one| one.to_str().ok())
            .map(|one| {
                percent_encoding::percent_decode_str(one)
                    .decode_utf8_lossy()
                    .into_owned()
            })
            .unwrap_or_default()
    };
    let name = pasted_name(&said("x-name"), &said("x-kind"));
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err(Refusal::about("cannotRead", name));
    };
    let bytes = bytes.clone();
    let (root, ceiling) = room_for(&session);
    let lock = keeping.turn();
    let kept = crate::elsewhere(move || parked(&lock, &bytes, name, &root, ceiling)).await??;
    noted(&session, &kept.0);
    Ok(kept.0.written(&kept.1))
}

// A pasted file has no place on disk, so it waits in a folder of its own until it is kept.
fn parked(
    lock: &Mutex<()>,
    bytes: &[u8],
    name: String,
    root: &std::path::Path,
    ceiling: u64,
) -> Answer<(tisty_core::attach::Kept, String)> {
    let room = std::env::temp_dir().join(format!(
        "tisty-pasted-{}-{}",
        std::process::id(),
        PASTED.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let source = room.join(&name);
    let kept = std::fs::create_dir_all(&room)
        .and_then(|()| std::fs::write(&source, bytes))
        .map_err(|_| Refusal::about("cannotRead", name.clone()))
        .and_then(|()| kept_alone(lock, &source, root, ceiling, &name))
        .map(|kept| (kept, name));
    let _ = std::fs::remove_dir_all(&room);
    kept
}

fn room_for(session: &tauri::State<'_, Mutex<Session>>) -> (std::path::PathBuf, u64) {
    let session = held(session);
    (
        session.paths.data().to_path_buf(),
        session.config.copies_up_to(),
    )
}

fn kept_at(
    source: &std::path::Path,
    root: &std::path::Path,
    ceiling: u64,
    name: &str,
) -> Answer<tisty_core::attach::Kept> {
    tisty_core::attach::keep(source, root, ceiling).map_err(|e| {
        witness::warn(channel::ATTACH, "the file could not be kept", &e.told());
        match e {
            tisty_core::Error::AttachmentTooBig { limit, .. } => {
                Refusal::about("attachmentTooBig", weighed(limit))
            }
            _ => Refusal::about("cannotRead", name.to_string()),
        }
    })
}

fn noted(session: &tauri::State<'_, Mutex<Session>>, kept: &tisty_core::attach::Kept) {
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
    summary: Option<String>,
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
    let closing = super::task_docs::closing_words(&mut session, id, summary.as_deref());
    if !closing.ops.is_empty() {
        session.commit_all(closing.ops)?;
    }
    let ops = match also.is_empty() {
        true => session.state.completing(id, jiff::Zoned::now()),
        false => session.state.covering(id, jiff::Zoned::now(), &also),
    };
    session.commit_all(ops)?;
    if let Some(said) = &closing.said {
        super::task_docs::round_cleared(&mut session, id, said);
    }
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
    let cache = held(&session).paths.cache().to_path_buf();
    finding::fetched(&session, reference, move |reference, at| {
        let named = reference.clone();
        let found = pointed_at(reference, at)?;
        let seen = crate::drawn::as_seen(std::path::PathBuf::from(found), &cache)
            .ok_or_else(|| Refusal::about("cannotDraw", named))?;
        Ok(seen.to_string_lossy().into_owned())
    })
    .await
}

pub fn copied_out(reference: String, at: finding::Where, to: &str) -> Answer<()> {
    let found = finding::handed_over(&reference, &at)?;
    std::fs::copy(&found, to).map_err(|_| refused(&found, &reference))?;
    tisty_core::lately::used(&at.reached, &reference);
    Ok(())
}

#[tauri::command]
pub async fn copied(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
    to: String,
) -> Answer<()> {
    finding::fetched(&session, reference, move |reference, at| {
        copied_out(reference, at, &to)
    })
    .await
}

// Only what this computer already holds counts: asking the cloud is what a player waits to be told to do.
#[tauri::command(async)]
pub fn kept_here(session: tauri::State<'_, Mutex<Session>>, reference: String) -> bool {
    let looking = finding::where_to(&session, &reference);
    finding::where_it_lies(&reference, &looking.data, looking.shared.as_deref())
        .is_some_and(|at| !tisty_core::holes::a_hole(&at))
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

#[tauri::command]
pub fn roomy() -> u64 {
    tisty_core::docs::BODY_ROOMY
}
