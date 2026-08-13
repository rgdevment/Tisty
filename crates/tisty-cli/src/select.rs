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
mod tests {
    use super::*;

    fn task(title: &str) -> Task {
        Task::new(Ulid::generate(), title, "a0")
    }

    fn selection_of(tasks: &[&Task]) -> Selection {
        Selection {
            by_number: tasks
                .iter()
                .enumerate()
                .map(|(i, t)| (i + 1, t.id))
                .collect(),
        }
    }

    #[test]
    fn a_number_refers_to_the_last_listing() {
        let (a, b) = (task("first"), task("second"));
        let tasks = [&a, &b];
        let selection = selection_of(&tasks);

        assert_eq!(resolve("2", &selection, &tasks), Resolved::One(b.id));
    }

    #[test]
    fn text_matches_the_title_case_insensitively() {
        let a = task("Deploy The Release");
        let tasks = [&a];

        assert_eq!(
            resolve("release", &Selection::default(), &tasks),
            Resolved::One(a.id)
        );
    }

    #[test]
    fn an_ambiguous_text_returns_every_candidate() {
        let (a, b) = (task("validar pagos"), task("validar certificados"));
        let tasks = [&a, &b];

        match resolve("validar", &Selection::default(), &tasks) {
            Resolved::Many(ids) => assert_eq!(ids.len(), 2),
            other => panic!("expected Many, got {other:?}"),
        }
    }

    #[test]
    fn a_short_id_matches_on_the_tail() {
        let mut a = task("first");
        let mut b = task("second");
        a.id = "01J8F2K3XQ0000000000000ABC".parse().unwrap();
        b.id = "01J8F2K3XQ0000000000000XYZ".parse().unwrap();
        let tasks = [&a, &b];

        assert_eq!(
            resolve("000abc", &Selection::default(), &tasks),
            Resolved::One(a.id)
        );
    }

    #[test]
    fn nothing_matching_is_not_a_silent_success() {
        let a = task("ship it");
        assert_eq!(
            resolve("nonexistent", &Selection::default(), &[&a]),
            Resolved::None
        );
    }

    #[test]
    fn a_number_beyond_the_last_listing_resolves_to_nothing() {
        let a = task("ship it");
        let tasks = [&a];
        let selection = selection_of(&tasks);

        assert_eq!(resolve("9", &selection, &tasks), Resolved::None);
    }

    #[test]
    fn a_number_never_falls_through_to_an_id_ending_in_it() {
        let mut a = task("ship it");
        a.id = "01J8F2K3XQ0000000000000009".parse().unwrap();
        let tasks = [&a];

        assert_eq!(resolve("9", &Selection::default(), &tasks), Resolved::None);
    }

    #[test]
    fn a_number_never_matches_a_title_that_contains_it() {
        let a = task("migrate to v9");
        let tasks = [&a];

        assert_eq!(resolve("9", &Selection::default(), &tasks), Resolved::None);
    }

    #[test]
    fn a_fragment_too_short_is_not_treated_as_an_id() {
        let mut a = task("ship it");
        a.id = "01J8F2K3XQ00000000000000AB".parse().unwrap();
        let tasks = [&a];

        assert_eq!(resolve("ab", &Selection::default(), &tasks), Resolved::None);
        assert_eq!(
            resolve("000ab", &Selection::default(), &tasks),
            Resolved::One(a.id)
        );
    }

    #[test]
    fn selection_survives_a_round_trip_to_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        let (a, b) = (task("first"), task("second"));

        Selection::save(&paths, &[&a, &b]).unwrap();
        let loaded = Selection::load(&paths);

        assert_eq!(loaded.number(1), Some(a.id));
        assert_eq!(loaded.number(2), Some(b.id));
    }

    #[test]
    fn a_missing_selection_file_is_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        assert_eq!(Selection::load(&paths).number(1), None);
    }
}
