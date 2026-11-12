use std::sync::Mutex;

use tisty_core::Op;
use tisty_core::witness::channel;

use crate::{Answer, Refusal, Session, blamed, held, signing};

#[derive(Debug, serde::Serialize)]
pub struct Jotted {
    pub file: String,
    pub made: bool,
}

#[tauri::command(async)]
pub fn jot(
    session: tauri::State<'_, Mutex<Session>>,
    idea: String,
    title: String,
) -> Answer<Jotted> {
    jotting(
        &mut held(&session),
        &idea,
        &title,
        jiff::Zoned::now().datetime(),
        tisty_core::brainstorm::roll(),
    )
}

#[derive(Debug, serde::Serialize)]
pub struct Pinned {
    pub file: String,
    pub ideas: usize,
    pub marked: usize,
}

#[tauri::command(async)]
pub fn brainstorm_of(session: tauri::State<'_, Mutex<Session>>) -> Option<Pinned> {
    let mut session = held(&session);
    let (file, body) = found(&mut session)?;
    let marked = session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .map_or(0, |kept| tisty_core::brainstorm::marked(kept, &body).len());
    Some(Pinned {
        ideas: tisty_core::brainstorm::ideas(&body),
        marked,
        file,
    })
}

#[tauri::command(async)]
pub fn idea_unflag(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    heading: String,
    text: String,
) -> Answer<()> {
    unflagging(
        &mut held(&session),
        &file,
        tisty_core::event::Idea { heading, text },
    )
}

pub fn unflagging(session: &mut Session, file: &str, idea: tisty_core::event::Idea) -> Answer<()> {
    let Some(kept) = session.state.docs.values().find(|one| one.file == file) else {
        return Err(Refusal::of("noSuchDoc"));
    };
    if !kept.ideas_flagged.iter().any(|one| one.idea == idea) {
        return Ok(());
    }
    let id = kept.id;
    session.commit(Op::IdeaUnflag { id, idea })?;
    Ok(())
}

#[tauri::command(async)]
pub fn idea_to_doc(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    idea: String,
    from: String,
) -> Answer<()> {
    filing(
        &mut held(&session),
        &file,
        &idea,
        &from,
        jiff::Zoned::now().date(),
    )
}

pub fn filing(
    session: &mut Session,
    file: &str,
    idea: &str,
    from: &str,
    on: jiff::civil::Date,
) -> Answer<()> {
    if idea.trim().is_empty() {
        return Err(Refusal::of("untitled"));
    }
    if !session.state.docs.values().any(|one| one.file == file) {
        return Err(Refusal::of("noSuchDoc"));
    }
    super::papers::writable(&session.state, file)?;
    let whole = tisty_core::docs::append(
        &session.paths.docs(),
        file,
        &tisty_core::brainstorm::filed(idea, from, on),
    )
    .map_err(super::papers::unwritten)?;
    session.corpus.forget(file);
    let hand = signing(&session.state);
    session.retell(file, &whole, hand);
    Ok(())
}

fn found(session: &mut Session) -> Option<(String, String)> {
    let kept = tisty_core::brainstorm::find(
        &session.paths.docs(),
        &session.state,
        session.brainstorm.as_deref(),
    )
    .map(|(one, body)| (one.file.clone(), body));
    session.brainstorm = kept.as_ref().map(|(file, _)| file.clone());
    kept
}

pub fn jotting(
    session: &mut Session,
    idea: &str,
    title: &str,
    at: jiff::civil::DateTime,
    roll: u32,
) -> Answer<Jotted> {
    if idea.trim().is_empty() {
        return Err(Refusal::of("untitled"));
    }
    if idea.trim().chars().count() > tisty_core::brainstorm::IDEA_AT_MOST {
        return Err(Refusal::about(
            "ideaTooLong",
            tisty_core::brainstorm::IDEA_AT_MOST.to_string(),
        ));
    }
    let root = session.paths.docs();
    let (file, was, made) = match found(session) {
        Some((file, body)) => (file, body, false),
        None => {
            let file = made_anew(session, title)?;
            (file, tisty_core::brainstorm::opening(title), true)
        }
    };
    super::papers::writable(&session.state, &file)?;
    session.brainstorm = Some(file.clone());
    let colour = tisty_core::brainstorm::coloured(&was, roll);
    let whole = tisty_core::docs::append(
        &root,
        &file,
        &tisty_core::brainstorm::jotted(idea, at, colour),
    )
    .map_err(super::papers::unwritten)?;
    session.corpus.forget(&file);
    let hand = signing(&session.state);
    session.retell(&file, &whole, hand);
    Ok(Jotted { file, made })
}

fn made_anew(session: &mut Session, title: &str) -> Answer<String> {
    let body = tisty_core::brainstorm::opening(title);
    let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, &body)
        .map_err(|e| blamed(channel::WINDOW, "a brainstorm could not be made", e))?;
    let order = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.page_of.is_none() && one.folder.is_none())
            .map(|one| one.order.as_str()),
    );
    let by = signing(&session.state);
    session.commit(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: false,
            sown: false,
            made: None,
            by,
            file: made.id.clone(),
            order,
            said: Some(tisty_core::event::Said::of(&body)),
            folder: None,
            page_of: None,
            task_of: None,
            edited: None,
        },
    })?;
    Ok(made.id)
}

#[cfg(test)]
#[path = "brainstorming_test.rs"]
mod tests;
