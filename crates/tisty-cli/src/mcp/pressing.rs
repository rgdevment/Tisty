use serde_json::{Value, json};
use tisty_core::State;
use tisty_core::model::Task;
use tisty_core::standing::{DUE_SOON_DAYS, Standing};

use super::looking::{list_of, named_task, standing_of};

// Late has no deadline and piles up, so it comes after today's: a stale backlog never hides the day's own.
const PRESSING: [Standing; 4] = [
    Standing::Overdue,
    Standing::DueSoon,
    Standing::Today,
    Standing::Late,
];
const PRESSING_SHOWN: usize = 5;

/// What an agent said done or not doing waits for the person, not for work, so it is left out.
pub(super) fn pressing(state: &State, today: jiff::civil::Date) -> Vec<(Standing, &Task)> {
    let mut all: Vec<(usize, &Task)> = state
        .tasks
        .values()
        .filter(|one| !one.folded() && one.resolved.is_none())
        .filter_map(|one| {
            let how = standing_of(one, today)?;
            Some((PRESSING.iter().position(|was| *was == how)?, one))
        })
        .collect();
    let day = |task: &Task| {
        task.deadline
            .as_ref()
            .or(task.date.as_ref())
            .map(|d| d.date())
    };
    all.sort_by(|(rank, one), (other, two)| {
        (rank, day(one), &one.title, one.id).cmp(&(other, day(two), &two.title, two.id))
    });
    all.into_iter()
        .map(|(rank, one)| (PRESSING[rank], one))
        .collect()
}

fn word(how: Standing) -> String {
    json!(how).as_str().unwrap_or_default().to_string()
}

/// Every count goes out even at zero, so a missing key never has to be read as «none».
pub(super) fn tallied(pressing: &[(Standing, &Task)]) -> Vec<(String, usize)> {
    PRESSING
        .iter()
        .map(|how| {
            let many = pressing.iter().filter(|(was, _)| was == how).count();
            (word(*how), many)
        })
        .collect()
}

pub(super) fn pressed(tally: &[(String, usize)]) -> String {
    let told: Vec<String> = tally
        .iter()
        .filter(|(_, many)| *many > 0)
        .map(|(how, many)| format!("{many} {}", how.replace('_', " ")))
        .collect();
    if told.is_empty() {
        return String::new();
    }
    format!(" ({})", told.join(", "))
}

pub(super) fn name_pressing(
    state: &State,
    pressing: &[(Standing, &Task)],
    today: jiff::civil::Date,
    kept: &mut serde_json::Map<String, Value>,
) -> Option<String> {
    if pressing.is_empty() {
        return None;
    }
    let shown = &pressing[..pressing.len().min(PRESSING_SHOWN)];
    kept.insert(
        "pressing".into(),
        json!(
            shown
                .iter()
                .map(|(how, task)| {
                    let mut one =
                        json!({ "id": task.id.to_string(), "title": task.title, "standing": how });
                    if let Some(list) = list_of(state, task) {
                        one["list"] = json!(list);
                    }
                    one
                })
                .collect::<Vec<_>>()
        ),
    );
    let named: Vec<String> = shown
        .iter()
        .map(|(how, task)| {
            format!(
                "{} ({})",
                named_task(state, task),
                word(*how).replace('_', " ")
            )
        })
        .collect();
    let more = pressing.len() - shown.len();
    let rest = match more {
        0 => String::new(),
        _ => {
            kept.insert("pressing_more".into(), json!(more));
            let near = today.saturating_add(jiff::Span::new().days(DUE_SOON_DAYS));
            format!(
                " {more} more: `find` with `scope` «open» and `to` {near} brings them, each with \
                 its `standing`."
            )
        }
    };
    Some(format!(
        "Pressing, most first: {}. Tell the person, and `read` one before you take it up.{rest}",
        named.join(", ")
    ))
}

#[cfg(test)]
#[path = "pressing_test.rs"]
mod tests;
