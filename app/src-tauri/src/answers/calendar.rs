use std::sync::Mutex;

use tisty_core::ics::{self, Kind, Odd, Taken, Unread};
use tisty_core::model::Repeat;
use tisty_core::{DateSpec, Op};

use crate::{Answer, Refusal, Session, held};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Found {
    at: usize,
    kind: &'static str,
    title: String,
    date: Option<DateSpec>,
    deadline: Option<DateSpec>,
    repeat: Option<Repeat>,
    reminder: Option<DateSpec>,
    place: String,
    odd: Vec<Odd>,
    done: bool,
    already: bool,
    takeable: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Brought {
    found: Vec<Found>,
    invitation: bool,
}

fn unread(why: Unread) -> Refusal {
    Refusal::of(match why {
        Unread::NotACalendar => "icsNotACalendar",
        Unread::TooBig => "icsTooBig",
        Unread::TooMany => "icsTooMany",
    })
}

fn calendar_at(path: &str) -> Result<ics::Calendar, Refusal> {
    let at = std::path::Path::new(path);
    let size = std::fs::metadata(at)
        .map_err(|_| Refusal::of("icsUnreadable"))?
        .len();
    if size > ics::BYTES_AT_MOST as u64 {
        return Err(unread(Unread::TooBig));
    }
    let bytes = std::fs::read(at).map_err(|_| Refusal::of("icsUnreadable"))?;
    ics::read_bytes(&bytes).map_err(unread)
}

fn already(session: &Session, taken: &Taken) -> bool {
    taken
        .source
        .as_ref()
        .is_some_and(|source| session.state.sourced.contains_key(source))
}

#[tauri::command(async)]
pub fn ics_read(session: tauri::State<'_, Mutex<Session>>, path: String) -> Answer<Brought> {
    let calendar = calendar_at(&path)?;
    let mut session = held(&session);
    session.reload()?;
    let now = jiff::Zoned::now();
    let found = calendar
        .items
        .iter()
        .enumerate()
        .map(|(at, item)| {
            let taken = ics::taken(item, &now);
            let already = already(&session, &taken);
            Found {
                at,
                kind: match item.kind {
                    Kind::Event => "event",
                    Kind::Todo => "todo",
                },
                takeable: taken.takeable() && !already,
                title: taken.title,
                date: taken.date,
                deadline: taken.deadline,
                repeat: taken.repeat,
                reminder: taken.reminder,
                place: taken.place,
                odd: taken.odd,
                done: taken.done,
                already,
            }
        })
        .collect();
    Ok(Brought {
        found,
        invitation: calendar.method.as_deref() == Some("REQUEST"),
    })
}

fn drafted(taken: Taken, list: Option<tisty_core::ListId>) -> tisty_core::capture::Draft {
    tisty_core::capture::Draft {
        title: taken.title,
        date: taken.date,
        deadline: taken.deadline,
        priority: taken.priority,
        filing: list.map(tisty_core::capture::Filing::Kept),
        repeat: taken.repeat,
        source: taken.source,
        ..Default::default()
    }
}

/// Each task is its own write, so two entries sharing one identity in a file still make one task.
#[tauri::command(async)]
pub fn ics_take(
    session: tauri::State<'_, Mutex<Session>>,
    path: String,
    mut picked: Vec<usize>,
    list: Option<String>,
) -> Answer<usize> {
    picked.sort_unstable();
    picked.dedup();
    let list = list
        .map(|one| one.parse().map_err(|_| Refusal::of("notAListId")))
        .transpose()?;
    let calendar = calendar_at(&path)?;
    let mut session = held(&session);
    session.reload()?;
    let now = jiff::Zoned::now();
    let mut made = 0;
    for item in picked.iter().filter_map(|at| calendar.items.get(*at)) {
        let taken = ics::taken(item, &now);
        if !taken.takeable() || already(&session, &taken) {
            continue;
        }
        let (reminder, described) = (taken.reminder.clone(), taken.described.clone());
        // One entry the core turns down leaves the others to be taken.
        let Ok(plan) = tisty_core::capture::plan(&session.state, drafted(taken, list)) else {
            continue;
        };
        let mut ops = plan.ops;
        for op in &mut ops {
            if let Op::TaskAdd { id, d } = op
                && *id == plan.task
            {
                d.reminders.extend(reminder.clone());
            }
        }
        if let Some(body) = described {
            ops.push(Op::TaskDescribe {
                id: plan.task,
                d: tisty_core::event::Body { body: Some(body) },
            });
        }
        // What was already made stays made: a write that fails partway says so only when nothing came in.
        if let Err(e) = session.commit_all(ops) {
            if made == 0 {
                return Err(e.into());
            }
            break;
        }
        made += 1;
    }
    Ok(made)
}
