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
