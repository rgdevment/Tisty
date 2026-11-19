use tisty_core::{DeviceId, Paths, State};

use super::brain::project::progress;
use super::jsonrpc::speaking_as;
use super::looking::named_task;
use super::pressing::{pressed, pressing, tallied};
use super::resuming::{
    Asked, Since, left_off, moved_since, open_tasks_left, own_events_newest_first,
};
use super::{seen_by, when};

pub(crate) const SAID_AT_MOST: usize = 2_000;
const OPEN_TASKS_SHOWN: usize = 3;
const MOVED_NAMED: usize = 5;

// A door the person has not opened says nothing, as the MCP lets nothing through it.
pub(crate) fn opening(paths: &Paths, client: &str) -> Option<String> {
    speaking_as(client);
    let agent = registered(paths)?;
    let (state, agent) = seen_by(paths, agent).ok().flatten()?;
    let today = jiff::Zoned::now().date();
    let mut lines = vec![
        "Tisty holds the person's tasks and documents; you reach them through its MCP tools."
            .to_string(),
    ];

    let left = open_tasks_left(
        &state,
        own_events_newest_first(paths, &agent),
        OPEN_TASKS_SHOWN,
    );
    if !left.is_empty() {
        lines.push("Your last open tasks here, newest first:".to_string());
        for (id, at) in &left {
            if let Some(task) = state.tasks.get(id) {
                lines.push(format!(
                    "- {}, last written {}: {}",
                    named_task(&state, task),
                    when(*at),
                    progress(paths, &state, task).told()
                ));
            }
        }
    }
    lines.extend(moved(paths, &state, &agent, today));
    let tally = tallied(&pressing(&state, today));
    let pressed = pressed(&tally);
    if !pressed.is_empty() {
        lines.push(format!(
            "Pressing for the person:{}.",
            pressed.trim_end_matches(')').replacen(" (", " ", 1)
        ));
    }
    lines.push(
        "Before working on one of these tasks, `read` it; as you finish steps, `tick` them, and \
         `say_done` with the proof when it is done. `catch_up` gives the whole picture."
            .to_string(),
    );
    Some(capped(lines))
}

fn registered(paths: &Paths) -> Option<DeviceId> {
    let text = std::fs::read_to_string(paths.config_file()).ok()?;
    let table = toml::from_str::<toml::Table>(&text).ok()?;
    let agent = table.get("agent_id")?.as_str()?.trim();
    (!agent.is_empty()).then(|| DeviceId(agent.to_string()))
}

fn moved(paths: &Paths, state: &State, agent: &DeviceId, today: jiff::civil::Date) -> Vec<String> {
    let left = left_off(paths, state, own_events_newest_first(paths, agent));
    let since = left
        .seen
        .clone()
        .map(Since::Seen)
        .or(left.at.map(Since::At));
    let Some(since) = since else {
        return Vec::new();
    };
    let Ok(log) = tisty_core::store::read_all(paths.store()) else {
        return Vec::new();
    };
    let moved = moved_since(
        state,
        &log,
        Asked {
            since: &since,
            mine_too: false,
            seen: left.seen.clone().unwrap_or_default(),
        },
        agent,
        today,
    );
    if moved.tasks_all + moved.docs_all == 0 {
        return Vec::new();
    }
    let from = left
        .looked_at
        .or_else(|| since.latest())
        .map(|at| format!(", {}", when(at)))
        .unwrap_or_default();
    let by = match moved.by.is_empty() {
        true => String::new(),
        false => format!(
            ", by {}",
            moved.by.iter().cloned().collect::<Vec<_>>().join(" and ")
        ),
    };
    let mut lines = vec![format!(
        "Since you were last here{from}, {} task(s) and {} document(s) moved{by}:",
        moved.tasks_all, moved.docs_all
    )];
    let tasks = moved
        .tasks
        .iter()
        .filter_map(|one| one["id"].as_str()?.parse().ok())
        .filter_map(|id| state.tasks.get(&id))
        .map(|task| format!("- task {}", named_task(state, task)));
    let docs = moved.docs.iter().filter_map(|one| {
        let file = one["doc"].as_str()?;
        Some(format!(
            "- document {:?} [{file}]",
            one["title"].as_str().unwrap_or(file)
        ))
    });
    lines.extend(tasks.chain(docs).take(MOVED_NAMED));
    lines
}

fn capped(mut lines: Vec<String>) -> String {
    let size = |all: &[String]| all.iter().map(|one| one.chars().count() + 1).sum::<usize>();
    while size(&lines) > SAID_AT_MOST {
        match lines.iter().rposition(|one| one.starts_with("- ")) {
            Some(at) => {
                lines.remove(at);
            }
            None => break,
        }
    }
    let mut said = lines.join("\n");
    if said.chars().count() > SAID_AT_MOST {
        said = said.chars().take(SAID_AT_MOST - 1).collect::<String>() + "…";
    }
    said
}

#[cfg(test)]
#[path = "opening_test.rs"]
mod tests;
