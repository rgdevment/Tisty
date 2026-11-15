use std::sync::Mutex;

use jiff::civil::{Date, DateTime};
use tisty_core::Op;
use tisty_core::diary::{self, Words};
use tisty_core::witness::channel;

use crate::{Answer, Refusal, Session, blamed, held, signing};

#[derive(Debug, serde::Serialize)]
pub struct Kept {
    pub book: Option<String>,
    pub open: bool,
}

#[derive(Debug, PartialEq, Eq, serde::Serialize)]
pub struct Today {
    pub book: String,
    pub page: String,
}

fn words(session: &Session) -> &'static Words {
    diary::words_for(&tisty_core::model::spoken(session.locale.as_deref()))
}

fn now() -> DateTime {
    jiff::Zoned::now().datetime()
}

#[tauri::command(async)]
pub fn diary_of(session: tauri::State<'_, Mutex<Session>>) -> Kept {
    kept(&held(&session), now().date())
}

pub fn kept(session: &Session, today: Date) -> Kept {
    Kept {
        book: diary::book_of(&session.paths.docs(), &session.state, today.year())
            .map(|one| one.file.clone()),
        open: session.state.diary_open,
    }
}

#[tauri::command(async)]
pub fn diary_today(session: tauri::State<'_, Mutex<Session>>) -> Answer<Today> {
    opening(&mut held(&session), now().date())
}

pub fn opening(session: &mut Session, today: Date) -> Answer<Today> {
    let root = session.paths.docs();
    let maker = diary::Maker {
        root: &root,
        device: &session.config.device_id,
        by: signing(&session.state),
    };
    let opened = diary::opened(&maker, &session.state, today, words(session))
        .map_err(|e| blamed(channel::WINDOW, "the diary could not be opened", e))?;
    let diary::Opened::Ready(opening) = opened else {
        return Err(Refusal::of("documentLocked"));
    };
    let made = !opening.ops.is_empty();
    if made {
        session.commit_all(opening.ops.clone())?;
    }
    let named = diary::named(&root, session.paths.data(), &opening).map_err(|e| {
        blamed(
            channel::WINDOW,
            "the diary's book could not name its month",
            e,
        )
    })?;
    if let Some(whole) = named {
        let hand = signing(&session.state);
        session.retell(&opening.book, &whole, hand);
    }
    Ok(Today {
        book: opening.book,
        page: opening.page,
    })
}

#[tauri::command(async)]
pub fn diary_write(session: tauri::State<'_, Mutex<Session>>, text: String) -> Answer<Today> {
    writing(&mut held(&session), &text, now().date())
}

pub fn writing(session: &mut Session, text: &str, today: Date) -> Answer<Today> {
    if text.trim().is_empty() {
        return Err(Refusal::of("untitled"));
    }
    written_today(session, today, |_| text.to_string())
}

#[tauri::command(async)]
pub fn diary_stick(session: tauri::State<'_, Mutex<Session>>, note: String) -> Answer<Today> {
    sticking(
        &mut held(&session),
        &note,
        now(),
        tisty_core::brainstorm::roll(),
    )
}

pub fn sticking(session: &mut Session, note: &str, at: DateTime, roll: u32) -> Answer<Today> {
    if note.trim().is_empty() {
        return Err(Refusal::of("untitled"));
    }
    if note.trim().chars().count() > tisty_core::brainstorm::IDEA_AT_MOST {
        return Err(Refusal::about(
            "ideaTooLong",
            tisty_core::brainstorm::IDEA_AT_MOST.to_string(),
        ));
    }
    written_today(session, at.date(), |was| {
        tisty_core::brainstorm::stuck_beside(was, note, at, roll)
    })
}

fn written_today(
    session: &mut Session,
    today: Date,
    said: impl FnOnce(&str) -> String,
) -> Answer<Today> {
    let opened = opening(session, today)?;
    super::papers::writable(&session.state, &opened.page)?;
    let words = words(session);
    let whole = tisty_core::docs::amend_unkept(&session.paths.docs(), &opened.page, |was| {
        Some(diary::written_on(was, today, words, &said(was)))
    })
    .map_err(super::papers::unwritten)?
    .unwrap_or_default();
    session.corpus.forget(&opened.page);
    let hand = signing(&session.state);
    session.retell(&opened.page, &whole, hand);
    Ok(opened)
}

#[tauri::command(async)]
pub fn diary_agents(session: tauri::State<'_, Mutex<Session>>, open: bool) -> Answer<()> {
    let mut session = held(&session);
    if session.state.diary_open != open {
        session.commit(Op::DiaryAgents { open })?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn diary_drawn(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    body: String,
    today: String,
    one: String,
    many: String,
) -> tauri::ipc::Response {
    let shelf = held(&session).paths.cache().join("pages");
    let html = super::paging::paged(&shelf, &body);
    let said = tisty_core::diary_page::Said {
        today: &today,
        one: &one,
        many: &many,
    };
    let dressed = seen(&held(&session), &file, |page| {
        tisty_core::diary_page::dressed(&html, &body, page, &said)
    });
    tauri::ipc::Response::new(dressed.unwrap_or(html).into_bytes())
}

#[tauri::command(async)]
pub fn diary_closed(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    body: String,
) -> String {
    seen(&held(&session), &file, |page| {
        tisty_core::diary_page::print(&body, page)
    })
    .unwrap_or_default()
}

fn seen<T>(
    session: &Session,
    file: &str,
    look: impl FnOnce(&tisty_core::diary_page::Page) -> T,
) -> Option<T> {
    let kept = session.state.docs.values().find(|one| one.file == file)?;
    let root = session.paths.docs();
    let zone = jiff::tz::TimeZone::system();
    Some(look(&tisty_core::diary_page::Page {
        root: &root,
        state: &session.state,
        kept,
        today: now().date(),
        zone: &zone,
    }))
}

#[cfg(test)]
#[path = "diary_test.rs"]
mod tests;
