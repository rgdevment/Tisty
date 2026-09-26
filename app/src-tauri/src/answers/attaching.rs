use std::sync::Mutex;

use tisty_core::witness::{self, channel};

use crate::{Answer, Refusal, Session, Task, held, herald, weighed};

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
