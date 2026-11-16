use std::sync::Mutex;

use jiff::civil::{Time, Weekday};
use tisty_core::{Task, review};

use crate::{Answer, Refusal, Session, held};

#[derive(Debug, serde::Serialize)]
pub struct Reviewing {
    pub routine: Option<Task>,
    pub unlisted: Vec<Task>,
}

#[tauri::command(async)]
pub fn review_of(session: tauri::State<'_, Mutex<Session>>) -> Answer<Reviewing> {
    let mut session = held(&session);
    session.reload()?;
    Ok(reviewing(&session))
}

pub fn reviewing(session: &Session) -> Reviewing {
    Reviewing {
        routine: review::routine(&session.state).cloned(),
        unlisted: review::unlisted(&session.state).cloned().collect(),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Nudge {
    pub title: String,
    pub tag: String,
    pub list: String,
    pub weekday: i8,
    pub hour: i8,
    pub minute: i8,
}

#[tauri::command(async)]
pub fn review_remind(session: tauri::State<'_, Mutex<Session>>, nudge: Nudge) -> Answer<Task> {
    reminded(&mut held(&session), &nudge, jiff::Zoned::now().datetime())
}

pub fn reminded(session: &mut Session, nudge: &Nudge, now: jiff::civil::DateTime) -> Answer<Task> {
    let list = nudge
        .list
        .parse()
        .map_err(|_| Refusal::about("noSuchList", &nudge.list))?;
    let weekday = Weekday::from_monday_one_offset(nudge.weekday)
        .map_err(|_| Refusal::about("notADate", nudge.weekday.to_string()))?;
    let at = Time::new(nudge.hour, nudge.minute, 0, 0)
        .map_err(|_| Refusal::about("notADate", format!("{}:{}", nudge.hour, nudge.minute)))?;
    if let Some(set) = review::routine(&session.state) {
        return Ok(set.clone());
    }
    let zone = crate::zone();
    let asked = review::Asked {
        title: &nudge.title,
        tag: &nudge.tag,
        list,
        weekday,
        at,
        zone: &zone,
    };
    let plan = review::routine_plan(&session.state, &asked, now)?;
    session.commit_all(plan.ops)?;
    session
        .state
        .tasks
        .get(&plan.task)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[cfg(test)]
#[path = "reviewing_test.rs"]
mod tests;
