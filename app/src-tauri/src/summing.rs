use tisty_core::model::Reading;
use tisty_core::{State, Task};

use crate::{Filter, Scope, Window, today};

pub const AHEAD: i64 = 7;

pub const BEADS: usize = 5;

pub fn a_board(state: &State, most: usize, today: jiff::civil::Date) -> (Vec<Task>, usize) {
    let whole = Filter::default();
    let mut by: std::collections::BTreeMap<tisty_core::model::Priority, Vec<&Task>> =
        Default::default();
    let mut loose: Vec<&Task> = Vec::new();
    let mut all = 0;
    for task in state.ordered_open() {
        if !whole.matches(task, today) {
            continue;
        }
        if task.priority.set() {
            all += 1;
            let mine = by.entry(task.priority).or_default();
            if mine.len() < most {
                mine.push(task);
            }
        } else if task.repeat.is_none() {
            all += 1;
            if loose.len() < most {
                loose.push(task);
            }
        }
    }
    let dealt = by.into_values().flatten().chain(loose).cloned().collect();
    (dealt, all)
}

pub fn waits_in_the_tray(task: &Task) -> bool {
    task.date.is_none() && task.deadline.is_none() && task.repeat.is_none()
}

pub fn a_spread(state: &State, most: usize, today: jiff::civil::Date) -> (Vec<Task>, usize) {
    let whole = Filter::default();
    let mut dated: Vec<&Task> = Vec::new();
    let mut waiting: Vec<&Task> = Vec::new();
    let mut all = 0;
    for task in state.ordered_open() {
        if !whole.matches(task, today) || task.repeat.is_some() {
            continue;
        }
        all += 1;
        if task.date.is_some() || task.deadline.is_some() {
            dated.push(task);
        } else if waiting.len() < most {
            waiting.push(task);
        }
    }
    let drawn = dated.into_iter().chain(waiting).cloned().collect();
    (drawn, all)
}

pub fn a_column(
    state: &State,
    view: Option<crate::asked::View>,
    today: jiff::civil::Date,
) -> Result<(Vec<Task>, usize), crate::Refusal> {
    let most = view.as_ref().and_then(|one| one.most);
    if view.as_ref().is_some_and(|one| one.spread) {
        return Ok(a_spread(state, most.unwrap_or(usize::MAX), today));
    }
    if view.as_ref().is_some_and(|one| one.board) {
        return Ok(a_board(state, most.unwrap_or(usize::MAX), today));
    }
    let filter = match view {
        Some(view) => view.resolve()?,
        None => Filter::default(),
    };
    let found = state.matching(&filter, today);
    let total = found.len();
    Ok((
        found
            .into_iter()
            .take(most.unwrap_or(usize::MAX))
            .cloned()
            .collect(),
        total,
    ))
}

pub const SOONEST: usize = 3;

pub fn soonest_in(
    state: &State,
    today: jiff::civil::Date,
) -> std::collections::BTreeMap<String, Vec<Task>> {
    let when = |task: &Task| {
        task.date
            .as_ref()
            .or(task.deadline.as_ref())
            .map(|one| one.at)
    };
    let whole = Filter::default();
    let mut by: std::collections::BTreeMap<tisty_core::model::ListId, Vec<&Task>> =
        Default::default();
    for task in state.ordered_open() {
        if !whole.matches(task, today) {
            continue;
        }
        if let Some(list) = task.list {
            by.entry(list).or_default().push(task);
        }
    }
    by.into_iter()
        .map(|(id, mut held)| {
            held.sort_by(|a, b| match (when(a), when(b)) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            });
            (
                id.to_string(),
                held.into_iter().take(SOONEST).cloned().collect(),
            )
        })
        .collect()
}

pub fn asked_about() -> Vec<(&'static str, Filter)> {
    let mut asked: Vec<(&'static str, Filter)> = vec![
        (
            "tasks",
            Filter {
                window: Some(Window::Today),
                ..Default::default()
            },
        ),
        (
            "upcoming",
            Filter {
                window: Some(Window::After(today())),
                ..Default::default()
            },
        ),
        (
            "repeating",
            Filter {
                repeating: true,
                ..Default::default()
            },
        ),
        ("all", Filter::default()),
        (
            "archive",
            Filter {
                scope: Scope::Archived,
                ..Default::default()
            },
        ),
        (
            "folded",
            Filter {
                scope: Scope::Archived,
                hidden: true,
                ..Default::default()
            },
        ),
        (
            "tracesHidden",
            Filter {
                scope: Scope::Archived,
                hidden: true,
                reading: Some(Reading::Trace),
                ..Default::default()
            },
        ),
        (
            "overdue",
            Filter {
                window: Some(Window::Overdue),
                ..Default::default()
            },
        ),
        (
            "dueToday",
            Filter {
                window: Some(Window::On(today())),
                ..Default::default()
            },
        ),
        (
            "undated",
            Filter {
                window: Some(Window::Undated),
                ..Default::default()
            },
        ),
        (
            "inbox",
            Filter {
                inbox: true,
                ..Default::default()
            },
        ),
    ];
    for (key, how) in [("stories", Reading::Story), ("traces", Reading::Trace)] {
        asked.push((
            key,
            Filter {
                scope: Scope::Archived,
                reading: Some(how),
                ..Default::default()
            },
        ));
    }
    for (key, wanted) in [
        ("do", tisty_core::model::Priority::Do),
        ("decide", tisty_core::model::Priority::Decide),
        ("delegate", tisty_core::model::Priority::Delegate),
        ("minor", tisty_core::model::Priority::Minor),
    ] {
        asked.push((
            key,
            Filter {
                priority: Some(wanted),
                ..Default::default()
            },
        ));
    }
    asked
}

pub fn tally(state: &State) -> std::collections::BTreeMap<String, usize> {
    let today = today();
    let asked = asked_about();
    let whole = Filter::default();
    let mut counts: std::collections::BTreeMap<String, usize> = asked
        .iter()
        .map(|(key, _)| ((*key).to_string(), 0))
        .collect();
    let mut quadrants = 0;
    let mut tray = 0;
    let mut in_list: std::collections::BTreeMap<tisty_core::model::ListId, usize> =
        Default::default();

    for task in state.tasks.values() {
        let open = task.is_open();
        for (key, filter) in &asked {
            let fits = match filter.scope {
                Scope::Open => open,
                Scope::Archived => !open,
                Scope::Either => true,
            };
            if fits && filter.matches(task, today) {
                *counts.entry((*key).to_string()).or_default() += 1;
            }
        }
        if !open {
            continue;
        }
        if let Some(list) = task.list {
            *in_list.entry(list).or_default() += 1;
        }
        if !whole.matches(task, today) {
            continue;
        }
        if !task.priority.set() && task.repeat.is_none() {
            quadrants += 1;
        }
        if waits_in_the_tray(task) {
            tray += 1;
        }
    }

    counts.insert("routines".to_string(), tisty_core::series::how_many(state));
    counts.insert("tags".to_string(), state.tags().len());
    counts.insert("quadrants".to_string(), quadrants);
    counts.insert("tray".to_string(), tray);
    for list in state.ordered_lists() {
        counts.insert(
            list.id.to_string(),
            in_list.get(&list.id).copied().unwrap_or(0),
        );
    }
    counts
}

#[derive(serde::Serialize)]
pub struct Counted {
    pub tag: String,
    pub tasks: usize,
    pub docs: usize,
}

pub fn tags_in_use(state: &State) -> Vec<Counted> {
    state
        .tags()
        .into_iter()
        .map(|tag| Counted {
            tag: tag.to_string(),
            tasks: state.tasks_tagged(tag).filter(|t| !t.hidden).count(),
            docs: state.docs_tagged(tag).count(),
        })
        .collect()
}

#[derive(serde::Serialize)]
pub struct Coming {
    pub task: Task,
    pub on: jiff::civil::Date,
    pub due: bool,
}

pub fn horizon(from: jiff::civil::Date) -> Option<jiff::civil::Date> {
    jiff::Span::new()
        .try_days(AHEAD)
        .ok()
        .and_then(|span| from.checked_add(span).ok())
}

pub fn coming(state: &State, from: jiff::civil::Date) -> Vec<Coming> {
    let Some(until) = horizon(from) else {
        return Vec::new();
    };

    let within = |on: jiff::civil::Date| on > from && on <= until;
    let mut out: Vec<Coming> = state
        .matching(&Filter::default(), from)
        .into_iter()
        .filter(|task| task.repeat.is_none())
        .flat_map(|task| {
            let held = task
                .date
                .as_ref()
                .map(|d| d.date())
                .filter(|on| within(*on))
                .map(|on| Coming {
                    task: task.clone(),
                    on,
                    due: false,
                });
            let own = task.date.as_ref().map(|d| d.date());
            let owed = task
                .deadline
                .as_ref()
                .map(|d| d.date())
                .filter(|on| within(*on) && Some(*on) != own)
                .map(|on| Coming {
                    task: task.clone(),
                    on,
                    due: true,
                });
            held.into_iter().chain(owed)
        })
        .collect();
    out.sort_by_key(|one| one.on);
    out
}

#[derive(serde::Serialize)]
pub struct Habit {
    pub task: Task,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<tisty_core::series::Series>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<jiff::civil::Date>,
}

pub fn recurring(state: &State, from: jiff::civil::Date) -> Vec<Habit> {
    let Some(until) = horizon(from) else {
        return Vec::new();
    };

    state
        .matching(&Filter::default(), from)
        .into_iter()
        .filter_map(|task| {
            let (Some(repeat), Some(spec)) = (task.repeat, task.date.as_ref()) else {
                return None;
            };

            let mut turns: Vec<jiff::civil::Date> = Vec::new();
            let own = spec.date();
            if own > from && own <= until {
                turns.push(own);
            }
            if repeat.cadence().every > 0 {
                let mut walked = repeat.cadence().beyond(spec.at, from);
                for _ in 0..AHEAD {
                    let Some(at) = walked else {
                        break;
                    };
                    let on = at.date();
                    if on > until || repeat.ended(on) {
                        break;
                    }
                    if !turns.contains(&on) {
                        turns.push(on);
                    }
                    walked = repeat.cadence().after(at);
                }
            }

            (!turns.is_empty()).then(|| Habit {
                task: task.clone(),
                series: tisty_core::series::series(state, task.id).map(|mut told| {
                    told.turns.drain(..told.turns.len().saturating_sub(BEADS));
                    told
                }),
                on: (turns.len() == 1).then(|| turns[0]),
            })
        })
        .collect()
}
