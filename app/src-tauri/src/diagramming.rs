use std::sync::Mutex;

use tauri::{AppHandle, Emitter};
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
    paper: Paper,
    at: usize,
) -> Answer<()> {
    // Opened only where it can be saved, so no work is done that a lock would then refuse.
    writable(&held(&session).state, &paper.file)?;
    let body = viewing::doc_body(&app, &paper.file)?;
    let Some((nth, found)) = diagrams::diagrams(&body)
        .into_iter()
        .enumerate()
        .find(|(_, one)| one.at.start == at)
    else {
        return Err(Refusal::about("diagramMoved", paper.title));
    };
    viewing::open(
        &app,
        Shown::Diagram(Drawn {
            paper,
            place: Place { at, nth },
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
