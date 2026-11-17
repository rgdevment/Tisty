use std::sync::Mutex;

use jiff::civil::{Date, DateTime};
use tisty_core::Op;
use tisty_core::diary::{self, Words};
use tisty_core::diary_day::{self, DayAt};
use tisty_core::witness::channel;

use crate::{Answer, Refusal, Session, blamed, held, signing};

#[derive(Debug, serde::Serialize)]
pub struct Kept {
    pub book: Option<String>,
    pub page: Option<String>,
    pub day: String,
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
    let book = diary::book_of(&session.paths.docs(), &session.state, today.year());
    Kept {
        page: book
            .and_then(|one| diary::page_of(&session.state, one.id, today.month()))
            .map(|one| one.file.clone()),
        book: book.map(|one| one.file.clone()),
        day: diary::day_title(today, words(session)),
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
    let opening = match opened {
        diary::Opened::Ready(opening) => opening,
        diary::Opened::Locked { .. } => return Err(Refusal::of("documentLocked")),
        diary::Opened::Waiting { behind } => {
            let named: Vec<String> = behind
                .iter()
                .map(|device| format!("«{}»", session.state.called(device)))
                .collect();
            return Err(Refusal::about("diaryBehind", named.join(", ")));
        }
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

#[tauri::command(async)]
pub fn diary_file_idea(
    session: tauri::State<'_, Mutex<Session>>,
    idea: String,
    heading: String,
) -> Answer<Today> {
    filing(&mut held(&session), &idea, &heading, now().date())
}

pub fn filing(session: &mut Session, idea: &str, heading: &str, today: Date) -> Answer<Today> {
    if idea.trim().is_empty() {
        return Err(Refusal::of("untitled"));
    }
    written_today(session, today, |_| {
        tisty_core::brainstorm::filed(idea, heading)
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

#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub struct Opened {
    day: i8,
    nth: usize,
}

impl From<Opened> for DayAt {
    fn from(open: Opened) -> Self {
        DayAt {
            day: open.day,
            nth: open.nth,
        }
    }
}

#[tauri::command(async)]
pub fn diary_laid(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    body: String,
    said: Labels,
    open: Option<Opened>,
) -> tauri::ipc::Response {
    let days = tisty_core::diary_page::days(&body);
    let (strips, words) = {
        let session = held(&session);
        let strips = seen(&session, &file, |page| {
            tisty_core::diary_page::strips(&days, page)
        });
        (strips.unwrap_or_default(), words(&session))
    };
    let said = tisty_core::diary_page::Said {
        one: &said.one,
        many: &said.many,
        empty: &said.empty,
        writing: &said.writing,
        keep: &said.keep,
        cancel: &said.cancel,
    };
    let laid = tisty_core::diary_page::laid(&days, &strips, words, &said, open.map(DayAt::from));
    tauri::ipc::Response::new(laid.into_bytes())
}

#[tauri::command(async)]
pub fn diary_day_words(body: String, at: Opened) -> Option<String> {
    diary_day::words(&body, at.into())
}

#[tauri::command(async)]
pub fn diary_rewrite(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    at: Opened,
    was: String,
    words: String,
) -> Answer<Today> {
    rewriting(&mut held(&session), &file, at.into(), &was, &words)
}

pub fn rewriting(
    session: &mut Session,
    file: &str,
    at: DayAt,
    was: &str,
    words: &str,
) -> Answer<Today> {
    day_amended(session, file, |body| {
        diary_day::rewritten(body, at, was, words)
    })
}

#[tauri::command(async)]
pub fn diary_unstick(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    at: Opened,
    note: usize,
    shown: String,
) -> Answer<Today> {
    unsticking(&mut held(&session), &file, at.into(), note, &shown)
}

pub fn unsticking(
    session: &mut Session,
    file: &str,
    at: DayAt,
    note: usize,
    shown: &str,
) -> Answer<Today> {
    day_amended(session, file, |body| {
        diary_day::unstuck(body, at, shown, note)
    })
}

fn day_amended(
    session: &mut Session,
    file: &str,
    change: impl FnOnce(&str) -> Option<String>,
) -> Answer<Today> {
    let root = session.paths.docs();
    let page = session.state.docs.values().find(|one| one.file == file);
    let book = page
        .filter(|one| diary::holds(&root, &session.state, one))
        .and_then(|one| session.state.docs.get(&one.page_of?))
        .map(|one| one.file.clone())
        .ok_or_else(|| Refusal::of("diaryDayChanged"))?;
    super::papers::writable(&session.state, file)?;
    let whole = tisty_core::docs::amend_unkept(&root, file, change)
        .map_err(super::papers::unwritten)?
        .ok_or_else(|| Refusal::of("diaryDayChanged"))?;
    session.corpus.forget(file);
    let hand = signing(&session.state);
    session.retell(file, &whole, hand);
    Ok(Today {
        book,
        page: file.to_string(),
    })
}

#[tauri::command(async)]
pub fn diary_newest(session: tauri::State<'_, Mutex<Session>>, file: String) -> Option<String> {
    newest(&held(&session), &file)
}

pub fn newest(session: &Session, file: &str) -> Option<String> {
    let book = session
        .state
        .docs
        .values()
        .find(|one| one.file == file && one.diary && one.page_of.is_none())?;
    diary::newest_page(&session.state, book.id).map(|one| one.file.clone())
}

#[derive(Debug, serde::Deserialize)]
pub struct Labels {
    one: String,
    many: String,
    empty: String,
    writing: String,
    keep: String,
    cancel: String,
}

#[tauri::command(async)]
pub fn diary_closed(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    body: String,
) -> String {
    let days = tisty_core::diary_page::days(&body);
    seen(&held(&session), &file, |page| {
        tisty_core::diary_page::print(&tisty_core::diary_page::strips(&days, page))
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
