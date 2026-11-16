use std::sync::Mutex;

use tauri::{AppHandle, Emitter, Manager};
use tisty_markdown::diagrams::{self, Held, Place, Unwritten};

use crate::answers::papers::{unwritten, writable};
use crate::diagramming::brought;
use crate::viewing::{self, Paper, Shown, Views};
use crate::{Answer, Refusal, Session, held, signing};

#[tauri::command(async)]
#[allow(clippy::too_many_arguments)]
pub fn view_piece(
    app: AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    views: tauri::State<'_, Views>,
    held: Held,
    source: Option<String>,
    from: Option<Paper>,
    at: Option<usize>,
    nth: Option<usize>,
    code: Option<bool>,
) {
    let code = code.unwrap_or(false);
    let sitting = from
        .as_ref()
        .zip(at.zip(nth))
        .and_then(|(paper, (at, nth))| {
            let body = viewing::doc_body(&app, &paper.file).ok()?;
            let place = Place { at, nth };
            match &source {
                Some(said) => diagrams::still_in(&body, held, place, &held.as_read(said))
                    .map(|(place, one)| (place, one.source)),
                None => diagrams::at_place_in(&body, held, place),
            }
        });
    let mine = from
        .as_ref()
        .is_some_and(|paper| writable(&self::held(&session).state, &paper.file).is_ok());
    let (place, source) = match sitting {
        Some((place, said)) if mine => (place, said),
        found => {
            let said = found.map(|(_, said)| said).or(source).unwrap_or_default();
            viewing::open(
                &app,
                Shown::Piece {
                    held,
                    source: said,
                    from,
                    place: None,
                    code,
                },
            );
            return;
        }
    };
    // Two windows on one piece would refuse each other's saves.
    let file = from
        .as_ref()
        .map(|paper| paper.file.clone())
        .unwrap_or_default();
    let open = viewing::showing(&views, |one| {
        matches!(one, Shown::Piece { held: kind, from: Some(own), place: Some(here), .. }
            if *kind == held && own.file == file && here.at == place.at)
    });
    if let Some(window) = open.and_then(|label| app.get_webview_window(&label)) {
        brought(&window);
        return;
    }
    viewing::open(
        &app,
        Shown::Piece {
            held,
            source,
            from,
            place: Some(place),
            code,
        },
    );
}

#[tauri::command(async)]
pub fn piece_save(
    app: AppHandle,
    window: tauri::WebviewWindow,
    views: tauri::State<'_, Views>,
    session: tauri::State<'_, Mutex<Session>>,
    now: String,
) -> Answer<()> {
    let Shown::Piece {
        held: kind,
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
        let mut written = Err(Unwritten::Moved);
        let body =
            tisty_core::docs::amend(&session.paths.docs(), session.paths.data(), &file, |was| {
                written = diagrams::redrawn_in(was, kind, place, &source, &now);
                written.as_ref().ok().map(|(body, _)| body.clone())
            })
            .map_err(unwritten)?;
        let (body, place) = match (body, written) {
            (Some(body), Ok((_, place))) => (body, place),
            (_, Err(Unwritten::Unread)) => return Err(Refusal::about("pieceUnread", paper.title)),
            _ => return Err(Refusal::about("pieceMoved", paper.title)),
        };
        session.corpus.forget(&file);
        let hand = signing(&session.state);
        session.retell(&file, &body, hand);
        viewing::reshow(
            &views,
            window.label(),
            Shown::Piece {
                held: kind,
                source: kind.as_read(&now),
                from: Some(paper),
                place: Some(place),
                code,
            },
        );
    }
    let _ = app.emit_to("main", "stirred", ());
    Ok(())
}
