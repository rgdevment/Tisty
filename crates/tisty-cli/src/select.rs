use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tisty_core::{Paths, Task, TaskId, store::write_atomic};
use ulid::Ulid;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Selection {
    by_number: BTreeMap<usize, TaskId>,
}

/// A ULID is for scripts, not for fingers. Numbers refer to the last listing
/// and only have to survive the seconds between reading it and typing.
#[derive(Debug, PartialEq, Eq)]
pub enum Resolved {
    One(TaskId),
    None,
    Many(Vec<TaskId>),
}

impl Selection {
    pub fn load(paths: &Paths) -> Self {
        std::fs::read_to_string(paths.selection_file())
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
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
}

/// Tries, in order: the number from the last listing, a ULID prefix, then a
/// case-insensitive match on the title.
pub fn resolve(selector: &str, selection: &Selection, tasks: &[&Task]) -> Resolved {
    if let Ok(n) = selector.parse::<usize>()
        && let Some(id) = selection.number(n)
    {
        return Resolved::One(id);
    }

    if let Ok(id) = selector.parse::<Ulid>() {
        return match tasks.iter().find(|t| t.id == id) {
            Some(t) => Resolved::One(t.id),
            None => Resolved::None,
        };
    }

    // Match on the tail: a ULID leads with its timestamp, so tasks created in
    // the same millisecond share a prefix and only differ at the end.
    let upper = selector.to_uppercase();
    let by_id: Vec<_> = tasks
        .iter()
        .filter(|t| t.id.to_string().ends_with(&upper))
        .map(|t| t.id)
        .collect();
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
        let a = task("enviar SOBR a producción");
        let tasks = [&a];

        assert_eq!(
            resolve("sobr", &Selection::default(), &tasks),
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

    /// The tail, not the head: two tasks created in the same millisecond share
    /// a ULID prefix, so a git-style prefix match would be ambiguous for both.
    #[test]
    fn a_short_id_matches_on_the_tail() {
        let (a, b) = (task("first"), task("second"));
        let tasks = [&a, &b];
        let id = a.id.to_string();
        let tail = &id[id.len() - 6..];

        assert_eq!(
            resolve(tail, &Selection::default(), &tasks),
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

    /// A stale number must not silently point at whatever occupies that slot
    /// now: the listing it came from no longer exists.
    #[test]
    fn a_number_beyond_the_last_listing_resolves_to_nothing() {
        let a = task("ship it");
        let tasks = [&a];
        let selection = selection_of(&tasks);

        assert_eq!(resolve("9", &selection, &tasks), Resolved::None);
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
