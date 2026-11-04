use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use tauri::Manager;
use tisty_sharing::{Finished, Unfinished};

use crate::{Answer, Refusal};

const LABEL: &str = "finisher-";
// A page that never asks is a page that never loaded, and nothing else would say so.
const BEGINS_WITHIN: Duration = Duration::from_secs(30);
// A long document with many diagrams takes a while, and nobody is watching this window.
const ENDS_WITHIN: Duration = Duration::from_secs(600);

#[derive(Debug, PartialEq, Eq)]
enum Heard {
    Began,
    Ended(Option<Vec<Finished>>),
}

struct Errand {
    asked: Vec<Unfinished>,
    told: Sender<Heard>,
}

#[derive(Default)]
pub struct Finishers(Mutex<BTreeMap<String, Errand>>);

impl Finishers {
    fn held(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Errand>> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn asked_of(&self, label: &str) -> Option<Vec<Unfinished>> {
        let mut held = self.held();
        let errand = held.get_mut(label)?;
        let _ = errand.told.send(Heard::Began);
        Some(std::mem::take(&mut errand.asked))
    }

    fn answered(&self, label: &str, answers: Option<Vec<Finished>>) {
        if let Some(errand) = self.held().remove(label) {
            let _ = errand.told.send(Heard::Ended(answers));
        }
    }
}

fn waited(heard: &Receiver<Heard>, begins: Duration, ends: Duration) -> Option<Vec<Finished>> {
    match heard.recv_timeout(begins).ok()? {
        Heard::Began => match heard.recv_timeout(ends).ok()? {
            Heard::Ended(answers) => answers,
            Heard::Began => None,
        },
        Heard::Ended(answers) => answers,
    }
}

#[tauri::command]
pub fn finishing_asked(
    window: tauri::WebviewWindow,
    finishers: tauri::State<'_, Finishers>,
) -> Answer<Vec<Unfinished>> {
    finishers
        .asked_of(window.label())
        .ok_or_else(|| Refusal::of("cannotFinish"))
}

#[tauri::command]
pub fn finishing_answered(
    window: tauri::WebviewWindow,
    finishers: tauri::State<'_, Finishers>,
    answers: Option<Vec<Finished>>,
) {
    finishers.answered(window.label(), answers);
}

// Never the window the person is in: a diagram has to be laid out to be read, and sync may ask while they are writing.
pub async fn in_hiding(app: &tauri::AppHandle, asked: Vec<Unfinished>) -> Answer<Vec<Finished>> {
    let label = format!(
        "{LABEL}{}",
        ulid::Ulid::generate().to_string().to_lowercase()
    );
    let (told, heard) = channel();
    let finishers = app.state::<Finishers>();
    finishers
        .held()
        .insert(label.clone(), Errand { asked, told });
    let page = tauri::WebviewUrl::App(crate::refusing::page(false).into());
    let window = tauri::WebviewWindowBuilder::new(app, &label, page)
        .visible(false)
        .skip_taskbar(true)
        .build();
    let Ok(window) = window else {
        finishers.answered(&label, None);
        return Err(Refusal::of("cannotFinish"));
    };
    let answers =
        tauri::async_runtime::spawn_blocking(move || waited(&heard, BEGINS_WITHIN, ENDS_WITHIN))
            .await
            .ok()
            .flatten();
    finishers.answered(&label, None);
    let _ = window.destroy();
    answers.ok_or_else(|| Refusal::of("cannotFinish"))
}

#[cfg(test)]
#[path = "finishing_test.rs"]
mod tests;
