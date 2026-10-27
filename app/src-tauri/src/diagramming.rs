use std::sync::Mutex;

use tauri::{AppHandle, Emitter};
use tisty_markdown::diagrams;

use crate::answers::papers::{unwritten, writable};
use crate::viewing::{self, Paper, Shown, Views};
use crate::{Answer, Refusal, Session, held, signing};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drawn {
    pub paper: Paper,
    pub at: usize,
    pub was: String,
}

#[tauri::command(async)]
pub fn view_diagram(app: AppHandle, paper: Paper, at: usize) -> Answer<()> {
    let body = viewing::doc_body(&app, &paper.file)?;
    let Some(found) = diagrams::diagrams(&body)
        .into_iter()
        .find(|one| one.at.start == at)
    else {
        return Err(Refusal::about("diagramMoved", paper.title));
    };
    viewing::open(
        &app,
        Shown::Diagram(Drawn {
            paper,
            at,
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
                diagrams::redrawn(was, drawn.at, &drawn.was, &now).map(|(body, at)| {
                    place = Some(at);
                    body
                })
            })
            .map_err(unwritten)?;
        let (Some(body), Some(at)) = (body, place) else {
            return Err(Refusal::about("diagramMoved", drawn.paper.title));
        };
        session.corpus.forget(&file);
        let hand = signing(&session.state);
        session.retell(&file, &body, hand);
        viewing::reshow(
            &views,
            window.label(),
            Shown::Diagram(Drawn {
                at,
                was: diagrams::as_read(&now),
                ..drawn
            }),
        );
    }
    // The main window learns of it as of any writer from outside, and draws the page again.
    let _ = app.emit_to("main", "stirred", ());
    Ok(())
}
