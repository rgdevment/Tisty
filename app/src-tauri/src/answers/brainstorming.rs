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

#[tauri::command(async)]
pub fn brainstorm_of(session: tauri::State<'_, Mutex<Session>>) -> Option<String> {
    found(&mut held(&session))
}

fn found(session: &mut Session) -> Option<String> {
    let file = tisty_core::brainstorm::find(
        &session.paths.docs(),
        &session.state,
        session.brainstorm.as_deref(),
    )
    .map(|one| one.file.clone());
    session.brainstorm.clone_from(&file);
    file
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
    let (file, made) = match found(session) {
        Some(file) => (file, false),
        None => (made_anew(session, title)?, true),
    };
    super::papers::writable(&session.state, &file)?;
    session.brainstorm = Some(file.clone());
    let was = tisty_core::docs::read(&root, &file).unwrap_or_default();
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
