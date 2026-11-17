use std::path::Path;

use jiff::civil::{Date, DateTime, Time, Weekday};
use serde::Serialize;

use crate::State;
use crate::capture::{self, Draft, Filing, Plan, Rejected};
use crate::event::Op;
use crate::lately::Used;
use crate::model::{Cadence, DateSpec, ListId, Repeat, Tag, Task, Unit};

pub const TAGS: [&str; 2] = ["revisión-semanal", "weekly-review"];

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Waiting {
    pub unlisted: usize,
    pub ideas: usize,
    pub marked: usize,
    pub forgotten: usize,
}

pub struct Asked<'a> {
    pub title: &'a str,
    pub tag: &'a str,
    pub list: ListId,
    pub weekday: Weekday,
    pub at: Time,
    pub zone: &'a str,
}

pub fn marks_the_review(tag: &str) -> bool {
    let tag = crate::text::folded(tag);
    TAGS.iter().any(|one| crate::text::folded(one) == tag)
}

pub fn routine(state: &State) -> Option<&Task> {
    state
        .open_tasks()
        .filter(|task| !task.hidden && (task.repeat.is_some() || task.after.is_some()))
        .filter(|task| task.tags.iter().any(|tag| marks_the_review(tag.as_str())))
        .min_by_key(|task| (task.date.as_ref().map(|day| day.at), task.id))
}

pub fn due(state: &State, today: Date) -> Option<&Task> {
    routine(state).filter(|task| task.date.as_ref().is_some_and(|day| day.date() <= today))
}

pub fn unlisted(state: &State) -> impl Iterator<Item = &Task> {
    let review = routine(state).map(|task| task.id);
    state
        .inbox()
        .filter(move |task| !task.hidden && Some(task.id) != review)
}

pub fn waiting(root: &Path, state: &State, opened: &Used, now: u64) -> Waiting {
    let diaries = crate::diary::books(root, state);
    Waiting {
        unlisted: unlisted(state).count(),
        ideas: crate::brainstorm::said_sheet(root, state)
            .map_or(0, |(_, body)| crate::brainstorm::ideas(&body)),
        marked: state
            .papers()
            .filter(|kept| {
                kept.flagged.is_some()
                    && kept.page_of.is_none()
                    && !state.held_away(kept)
                    && !crate::diary::within(&diaries, kept)
            })
            .count(),
        forgotten: crate::forgotten::forgotten(root, state, opened, now).len(),
    }
}

pub fn routine_plan(state: &State, asked: &Asked, now: DateTime) -> Result<Plan, Rejected> {
    let day = next_on(asked.weekday, asked.at, now);
    let tag = match marks_the_review(asked.tag) {
        true => asked.tag,
        false => TAGS[1],
    };
    let mut plan = capture::plan(
        state,
        Draft {
            title: asked.title.to_string(),
            date: Some(DateSpec::all_day(day, asked.zone)),
            tags: Tag::new(tag).into_iter().collect(),
            filing: Some(Filing::Kept(asked.list)),
            repeat: Some(Repeat::due(Cadence {
                every: 1,
                unit: Unit::Week,
            })),
            ..Draft::default()
        },
    )?;
    for op in &mut plan.ops {
        if let Op::TaskAdd { d, .. } = op {
            d.reminders = vec![DateSpec::floating(day.to_datetime(asked.at), asked.zone)];
        }
    }
    Ok(plan)
}

fn next_on(weekday: Weekday, at: Time, now: DateTime) -> Date {
    let today = now.date();
    let ahead = (0..8)
        .filter_map(|days| today.checked_add(jiff::Span::new().days(days)).ok())
        .find(|day| day.weekday() == weekday && day.to_datetime(at) > now);
    ahead.unwrap_or(today)
}

#[cfg(test)]
#[path = "review_test.rs"]
mod tests;
