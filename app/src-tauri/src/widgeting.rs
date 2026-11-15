use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};
use tisty_markdown::diagrams::{self, Fence, Place};

use crate::answers::papers::{unwritten, writable};
use crate::diagramming::brought;
use crate::viewing::{self, Paper, Shown, Views};
use crate::{Answer, Refusal, Session, held, signing};

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub fn view_widget(
    app: AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    views: tauri::State<'_, Views>,
    source: String,
    from: Option<Paper>,
    at: Option<usize>,
    nth: Option<usize>,
    code: Option<bool>,
) {
    let code = code.unwrap_or(false);
    let found = from
        .as_ref()
        .zip(at.zip(nth))
        .and_then(|(paper, (at, nth))| {
            writable(&held(&session).state, &paper.file).ok()?;
            let body = viewing::doc_body(&app, &paper.file).ok()?;
            diagrams::still_in(
                &body,
                Fence::Widget,
                Place { at, nth },
                &diagrams::as_read(&source),
            )
        });
    let Some((place, sitting)) = found else {
        viewing::open(
            &app,
            Shown::Widget {
                source,
                from,
                place: None,
                code,
            },
        );
        return;
    };
    // Two windows on one widget would refuse each other's saves.
    let file = from
        .as_ref()
        .map(|paper| paper.file.clone())
        .unwrap_or_default();
    let open = viewing::showing(
        &views,
        |one| matches!(one, Shown::Widget { from: Some(own), place: Some(here), .. } if own.file == file && here.at == place.at),
    );
    if let Some(window) = open.and_then(|label| app.get_webview_window(&label)) {
        brought(&window);
        return;
    }
    viewing::open(
        &app,
        Shown::Widget {
            source: sitting.source,
            from,
            place: Some(place),
            code,
        },
    );
}

#[tauri::command(async)]
pub fn widget_save(
    app: AppHandle,
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
    session: tauri::State<'_, Mutex<Session>>,
    now: String,
) -> Answer<()> {
    let Shown::Widget {
        source,
        from: Some(paper),
        place: Some(place),
        code,
    } = viewing::shown_in(&views, window.label())?
    else {
        return Err(Refusal::of("notAllowed"));
    };
    let file = paper.file.clone();
    {
        let mut session = held(&session);
        writable(&session.state, &file)?;
        let mut moved = None;
        let body =
            tisty_core::docs::amend(&session.paths.docs(), session.paths.data(), &file, |was| {
                diagrams::redrawn_in(was, Fence::Widget, place, &source, &now).map(|(body, at)| {
                    moved = Some(at);
                    body
                })
            })
            .map_err(unwritten)?;
        let (Some(body), Some(place)) = (body, moved) else {
            return Err(Refusal::about("widgetMoved", paper.title));
        };
        session.corpus.forget(&file);
        let hand = signing(&session.state);
        session.retell(&file, &body, hand);
        viewing::reshow(
            &views,
            window.label(),
            Shown::Widget {
                source: diagrams::as_read(&now),
                from: Some(paper),
                place: Some(place),
                code,
            },
        );
    }
    let _ = app.emit_to("main", "stirred", ());
    Ok(())
}
