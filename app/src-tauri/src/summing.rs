use tisty_core::model::Reading;
use tisty_core::{State, Task};

use crate::{Filter, Scope, Window, today};

pub const AHEAD: i64 = 7;

pub const BEADS: usize = 5;

pub fn tally(state: &State) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    let mut count = |key: &str, filter: Filter| {
        counts.insert(key.to_string(), state.matching(&filter, today()).len());
    };

    count(
        "tasks",
        Filter {
            window: Some(Window::Today),
            ..Default::default()
        },
    );
    count(
        "upcoming",
        Filter {
            window: Some(Window::After(today())),
            ..Default::default()
        },
    );
    count(
        "repeating",
        Filter {
            repeating: true,
            ..Default::default()
        },
    );
    count("all", Filter::default());
    count(
        "archive",
        Filter {
            scope: Scope::Archived,
            ..Default::default()
        },
    );
    count(
        "folded",
        Filter {
            scope: Scope::Archived,
            hidden: true,
            ..Default::default()
        },
    );
    for (key, how) in [("stories", Reading::Story), ("traces", Reading::Trace)] {
        count(
            key,
            Filter {
                scope: Scope::Archived,
                reading: Some(how),
                ..Default::default()
            },
        );
    }
    // The empty trace layer says where the trace went: hidden traces, not every hidden task.
    count(
        "tracesHidden",
        Filter {
            scope: Scope::Archived,
            hidden: true,
            reading: Some(Reading::Trace),
            ..Default::default()
        },
    );
    count(
        "overdue",
        Filter {
            window: Some(Window::Overdue),
            ..Default::default()
        },
    );
    count(
        "dueToday",
        Filter {
            window: Some(Window::On(today())),
            ..Default::default()
        },
    );
    count(
        "undated",
        Filter {
            window: Some(Window::Undated),
            ..Default::default()
        },
    );
    count(
        "inbox",
        Filter {
            inbox: true,
            ..Default::default()
        },
    );
    for (key, wanted) in [
        ("do", tisty_core::model::Priority::Do),
        ("decide", tisty_core::model::Priority::Decide),
        ("delegate", tisty_core::model::Priority::Delegate),
        ("minor", tisty_core::model::Priority::Minor),
    ] {
        count(
            key,
            Filter {
                priority: Some(wanted),
                ..Default::default()
            },
        );
    }

    counts.insert("routines".to_string(), tisty_core::series::how_many(state));

    counts.insert("tags".to_string(), state.tags().len());
    counts.insert(
        "quadrants".to_string(),
        state
            .matching(&Filter::default(), today())
            .iter()
            .filter(|task| !task.priority.set())
            .count(),
    );

    for list in state.ordered_lists() {
        counts.insert(list.id.to_string(), state.tasks_in(list.id).count());
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
