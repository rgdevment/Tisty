use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

use tauri::Manager;
use tisty_sharing::{Finished, Unfinished};

use crate::{Answer, Refusal};

const LABEL: &str = "finisher-";
// A long document with many diagrams takes a while, and nobody is watching this window.
const WITHIN: Duration = Duration::from_secs(600);

struct Errand {
    asked: Vec<Unfinished>,
    answer: Sender<Option<Vec<Finished>>>,
}

/// What each window that nobody sees was opened to finish.
#[derive(Default)]
pub struct Finishers(Mutex<BTreeMap<String, Errand>>);

impl Finishers {
    fn held(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Errand>> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn asked_of(&self, label: &str) -> Option<Vec<Unfinished>> {
        self.held()
            .get_mut(label)
            .map(|one| std::mem::take(&mut one.asked))
    }

    fn answered(&self, label: &str, answers: Option<Vec<Finished>>) {
        if let Some(errand) = self.held().remove(label) {
            let _ = errand.answer.send(answers);
        }
    }
}

/// What a window that was opened to finish something asks for, and only that window is answered.
#[tauri::command]
pub fn finishing_asked(
    window: tauri::WebviewWindow,
    finishers: tauri::State<'_, Finishers>,
) -> Answer<Vec<Unfinished>> {
    finishers
        .asked_of(window.label())
        .ok_or_else(|| Refusal::of("cannotFinish"))
}

/// What it finished, or `None` when it could not, which ends the wait at once.
#[tauri::command]
pub fn finishing_answered(
    window: tauri::WebviewWindow,
    finishers: tauri::State<'_, Finishers>,
    answers: Option<Vec<Finished>>,
) {
    finishers.answered(window.label(), answers);
}

/// Colours and draws what is asked in a window nobody sees, never the one the person is in: a diagram has to be laid out to be read, and sync may ask while they are writing.
pub async fn in_hiding(app: &tauri::AppHandle, asked: Vec<Unfinished>) -> Answer<Vec<Finished>> {
    let label = format!(
        "{LABEL}{}",
        ulid::Ulid::generate().to_string().to_lowercase()
    );
    let (told, heard) = channel();
    let finishers = app.state::<Finishers>();
    finishers.held().insert(
        label.clone(),
        Errand {
            asked,
            answer: told,
        },
    );
    let page = tauri::WebviewUrl::App(crate::refusing::page(false).into());
    let window = tauri::WebviewWindowBuilder::new(app, &label, page)
        .visible(false)
        .skip_taskbar(true)
        .build();
    let Ok(window) = window else {
        finishers.answered(&label, None);
        return Err(Refusal::of("cannotFinish"));
    };
    let answers = tauri::async_runtime::spawn_blocking(move || heard.recv_timeout(WITHIN))
        .await
        .ok()
        .and_then(Result::ok)
        .flatten();
    finishers.answered(&label, None);
    let _ = window.destroy();
    answers.ok_or_else(|| Refusal::of("cannotFinish"))
}

#[cfg(test)]
#[path = "finishing_test.rs"]
mod tests;
