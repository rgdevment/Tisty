use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager, Runtime};
use tisty_markdown::diagrams::{self, Place};

use crate::answers::papers::{unwritten, writable};
use crate::viewing::{self, Paper, Shown, Views};
use crate::{Answer, Refusal, Session, held, signing};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawn {
    pub paper: Paper,
    pub place: Place,
    pub was: String,
}

#[tauri::command(async)]
pub fn view_diagram(
    app: AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    views: tauri::State<'_, Views>,
    paper: Paper,
    at: usize,
    nth: usize,
    source: String,
) -> Answer<()> {
    // Opened only where it can be saved, so no work is done that a lock would then refuse.
    writable(&held(&session).state, &paper.file)?;
    let body = viewing::doc_body(&app, &paper.file)?;
    // The page may be a save behind the disk, so the diagram is the one that says what the page drew.
    let found = diagrams::still(&body, Place { at, nth }, &source);
    let sitting = found.as_ref().map_or(at, |(place, _)| place.at);
    // Two windows on one diagram would refuse each other's saves.
    let open = viewing::showing(
        &views,
        |one| matches!(one, Shown::Diagram(drawn) if drawn.paper.file == paper.file && drawn.place.at == sitting),
    );
    if let Some(window) = open.and_then(|label| app.get_webview_window(&label)) {
        brought(&window);
        return Ok(());
    }
    let Some((place, found)) = found else {
        return Err(Refusal::about("diagramMoved", paper.title));
    };
    viewing::open(
        &app,
        Shown::Diagram(Drawn {
            paper,
            place,
            was: found.source,
        }),
    );
    Ok(())
}

#[tauri::command(async)]
pub fn diagram_save(
    app: AppHandle,
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
    session: tauri::State<'_, Mutex<Session>>,
    now: String,
) -> Answer<()> {
    let Shown::Diagram(drawn) = viewing::shown_in(&views, window.label())? else {
        return Err(Refusal::of("notAllowed"));
    };
    let file = drawn.paper.file.clone();
    {
        let mut session = held(&session);
        writable(&session.state, &file)?;
        let mut place = None;
        let body =
            tisty_core::docs::amend(&session.paths.docs(), session.paths.data(), &file, |was| {
                diagrams::redrawn(was, drawn.place, &drawn.was, &now).map(|(body, now_at)| {
                    place = Some(now_at);
                    body
                })
            })
            .map_err(unwritten)?;
        let (Some(body), Some(place)) = (body, place) else {
            return Err(Refusal::about("diagramMoved", drawn.paper.title));
        };
        session.corpus.forget(&file);
        let hand = signing(&session.state);
        session.retell(&file, &body, hand);
        viewing::reshow(
            &views,
            window.label(),
            Shown::Diagram(Drawn {
                place,
                was: diagrams::as_read(&now),
                ..drawn
            }),
        );
    }
    // The main window learns of it as of any writer from outside, and draws the page again.
    let _ = app.emit_to("main", "stirred", ());
    Ok(())
}

/// The diagram as the document says it now, held from here on as what this window opened.
#[tauri::command(async)]
pub fn diagram_again(
    app: AppHandle,
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
) -> Answer<String> {
    let Shown::Diagram(drawn) = viewing::shown_in(&views, window.label())? else {
        return Err(Refusal::of("notAllowed"));
    };
    let body = viewing::doc_body(&app, &drawn.paper.file)?;
    let Some((place, was)) = diagrams::at_place(&body, drawn.place) else {
        return Err(Refusal::about("diagramGone", drawn.paper.title));
    };
    viewing::reshow(
        &views,
        window.label(),
        Shown::Diagram(Drawn {
            place,
            was: was.clone(),
            ..drawn
        }),
    );
    Ok(was)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Found {
    file: String,
    title: String,
    section: String,
    nth: usize,
    source: String,
}

// Read after the lock is let go: a store of many documents must not hold every other command back.
#[tauri::command(async)]
pub fn diagrams_everywhere(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<Found>> {
    let (root, papers) = {
        let session = held(&session);
        let papers: Vec<(String, String)> = session
            .state
            .papers()
            .filter(|kept| !session.state.held_away(kept))
            .map(|kept| (kept.file.clone(), kept.title.clone().unwrap_or_default()))
            .collect();
        (session.paths.docs(), papers)
    };
    Ok(papers
        .into_iter()
        .flat_map(|(file, title)| {
            let body = tisty_core::docs::read(&root, &file).unwrap_or_default();
            found_in(&file, &title, &body)
        })
        .collect())
}

// A diagram is named by the heading it sits under; the document's own title says nothing new.
fn found_in(file: &str, title: &str, body: &str) -> Vec<Found> {
    let heads = tisty_markdown::outline(&tisty_markdown::read(body));
    diagrams::diagrams(body)
        .into_iter()
        .enumerate()
        .map(|(nth, one)| Found {
            file: file.to_string(),
            title: title.to_string(),
            section: heads
                .iter()
                .rfind(|head| head.at.start < one.at.start && head.said != title)
                .map(|head| head.said.clone())
                .unwrap_or_default(),
            nth,
            source: one.source,
        })
        .collect()
}

#[tauri::command(async)]
pub fn diagram_order(
    app: AppHandle,
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
) -> Answer<usize> {
    let Shown::Diagram(drawn) = viewing::shown_in(&views, window.label())? else {
        return Err(Refusal::of("notAllowed"));
    };
    let body = viewing::doc_body(&app, &drawn.paper.file)?;
    Ok(diagrams::order_of(&body, drawn.place, &drawn.was))
}

#[tauri::command]
pub fn diagram_unsaved(
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
    unsaved: bool,
) {
    viewing::mark_unsaved(&views, window.label(), unsaved);
}

/// The person chose to leave without saving: the quit asked for goes on.
#[tauri::command]
pub fn diagram_leave(app: AppHandle, window: tauri::WebviewWindow, views: tauri::State<'_, Views>) {
    viewing::mark_unsaved(&views, window.label(), false);
    crate::desktop::parting(&app);
}

/// Quitting waits while a diagram window holds unsaved work, and that window asks first.
pub(crate) fn holds_back<R: Runtime>(app: &tauri::AppHandle<R>) -> bool {
    let Some(views) = app.try_state::<Views>() else {
        return false;
    };
    let Some(window) =
        viewing::first_unsaved(&views).and_then(|label| app.get_webview_window(&label))
    else {
        return false;
    };
    brought(&window);
    let _ = window.emit_to(window.label(), "diagramLeaving", ());
    true
}

fn brought<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
}

#[cfg(test)]
#[path = "diagramming_test.rs"]
mod tests;
