use std::collections::BTreeMap;
use std::io::IsTerminal;

use serde::{Deserialize, Serialize};
use tisty_core::witness::{self, Fact, channel};
use tisty_core::{Paths, Task, TaskId, store::write_atomic};
use ulid::Ulid;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Selection {
    by_number: BTreeMap<usize, TaskId>,
}

const MIN_ID_FRAGMENT: usize = 4;

#[derive(Debug, PartialEq, Eq)]
pub enum Resolved {
    One(TaskId),
    None,
    Many(Vec<TaskId>),
}

impl Selection {
    pub fn load(paths: &Paths) -> Self {
        let Ok(text) = std::fs::read_to_string(paths.selection_file()) else {
            return Self::default();
        };
        serde_json::from_str(&text).unwrap_or_else(|why| {
            witness::warn(
                channel::TERMINAL,
                "the last listing could not be read",
                &[("why", Fact::Why(why.to_string()))],
            );
            Self::default()
        })
    }

    pub fn save(paths: &Paths, tasks: &[&Task]) -> std::io::Result<()> {
        let selection = Self {
            by_number: tasks
                .iter()
                .enumerate()
                .map(|(i, t)| (i + 1, t.id))
                .collect(),
        };
        std::fs::create_dir_all(paths.cache())?;
        let json = serde_json::to_vec(&selection)?;
        write_atomic(&paths.selection_file(), &json).map_err(std::io::Error::other)
    }

    pub fn number(&self, n: usize) -> Option<TaskId> {
        self.by_number.get(&n).copied()
    }

    pub fn len(&self) -> usize {
        self.by_number.len()
    }
}

pub fn resolve(selector: &str, selection: &Selection, tasks: &[&Task]) -> Resolved {
    if let Ok(n) = selector.parse::<usize>() {
        return match selection.number(n) {
            Some(id) if tasks.iter().any(|t| t.id == id) => Resolved::One(id),
            _ => Resolved::None,
        };
    }

    if let Ok(id) = selector.parse::<Ulid>() {
        return match tasks.iter().find(|t| t.id == id) {
            Some(t) => Resolved::One(t.id),
            None => Resolved::None,
        };
    }

    let upper = selector.to_uppercase();
    let by_id: Vec<_> = if selector.len() >= MIN_ID_FRAGMENT {
        tasks
            .iter()
            .filter(|t| t.id.to_string().ends_with(&upper))
            .map(|t| t.id)
            .collect()
    } else {
        Vec::new()
    };
    if by_id.len() == 1 {
        return Resolved::One(by_id[0]);
    }

    let needle = selector.to_lowercase();
    let by_title: Vec<_> = tasks
        .iter()
        .filter(|t| t.title.to_lowercase().contains(&needle))
        .map(|t| t.id)
        .collect();

    match by_title.len() {
        0 if by_id.is_empty() => Resolved::None,
        0 => Resolved::Many(by_id),
        1 => Resolved::One(by_title[0]),
        _ => Resolved::Many(by_title),
    }
}

pub fn prompt(tasks: &[&Task], lang: crate::i18n::Lang) -> anyhow::Result<Option<TaskId>> {
    if tasks.is_empty() {
        return Ok(None);
    }
    if !std::io::stdin().is_terminal() {
        anyhow::bail!("{}", lang.get("needs-terminal"));
    }

    let labels: Vec<&str> = tasks.iter().map(|t| t.title.as_str()).collect();
    let chosen = dialoguer::FuzzySelect::new()
        .with_prompt(lang.get("which-task"))
        .items(&labels)
        .default(0)
        .interact_opt()?;

    Ok(chosen.map(|i| tasks[i].id))
}

#[cfg(test)]
#[path = "select_test.rs"]
mod tests;
