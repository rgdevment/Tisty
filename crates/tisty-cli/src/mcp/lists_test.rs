use super::*;
use ulid::Ulid;

fn kept(names: &[&str]) -> State {
    let mut state = State::default();
    for name in names {
        let list = List::new(Ulid::generate(), *name, "a0");
        state.lists.insert(list.id, list);
    }
    state
}

fn called_names(state: &State, said: &str) -> Vec<String> {
    called(state, said)
        .unwrap()
        .iter()
        .map(|one| one.name.clone())
        .collect()
}

#[test]
fn a_list_is_found_whole_before_in_part_and_whatever_its_capitals() {
    let state = kept(&["Trabajo", "Trabajo en casa", "Viajes"]);

    assert_eq!(called_names(&state, "trabajo"), ["Trabajo"]);
    assert_eq!(called_names(&state, "en casa"), ["Trabajo en casa"]);
    assert_eq!(called_names(&state, "viaje"), ["Viajes"]);
}

#[test]
fn a_list_that_does_not_exist_is_refused_naming_the_live_ones() {
    let mut state = kept(&["Viajes", "Casa", "Mudanza"]);
    state
        .lists
        .values_mut()
        .filter(|one| one.name == "Mudanza")
        .for_each(|one| one.archived = true);

    assert_eq!(
        called(&state, "Oficina").unwrap_err(),
        "there is no list called \"Oficina\", and you cannot make one. These exist: Casa, Viajes."
    );
    assert_eq!(called_names(&state, "mudanza"), ["Mudanza"]);
    assert_eq!(
        called(&kept(&[]), "Casa").unwrap_err(),
        "there is no list called \"Casa\", and there are none yet."
    );
}

#[test]
fn a_piece_of_several_names_is_refused_naming_them_and_one_name_held_twice_is_not() {
    let why = called(&kept(&["Casa nueva", "Casa vieja"]), "casa").unwrap_err();
    assert!(
        why.starts_with("\"casa\" matches more than one list."),
        "{why}"
    );
    assert!(
        why.contains("Casa nueva") && why.contains("Casa vieja"),
        "{why}"
    );

    assert_eq!(
        called_names(&kept(&["Casa", "casa"]), "Casa").len(),
        2,
        "one put away and one live may share a name"
    );
}

#[test]
fn the_inbox_goes_by_its_names_in_both_languages_unless_a_list_is_called_so() {
    let state = kept(&["Casa"]);
    for said in [
        "inbox",
        "The Inbox",
        "@inbox",
        "Bandeja de entrada",
        "la bandeja",
        "Sin lista",
        "No list",
    ] {
        assert!(is_the_inbox(&state, said), "{said}");
    }
    assert!(!is_the_inbox(&state, "Casa"));
    assert!(!is_the_inbox(&kept(&["Inbox"]), "inbox"));
    assert!(is_the_inbox(&kept(&["Inbox zero"]), "inbox"));
    assert_eq!(inbox_named(&state), "inbox");
    assert_eq!(inbox_named(&kept(&["Inbox"])), "the inbox");
}

#[test]
fn only_the_open_tasks_in_sight_count_for_a_list_and_for_the_inbox() {
    let mut state = kept(&["Casa"]);
    let casa = *state.lists.keys().next().unwrap();
    for (title, list, hidden) in [
        ("loose", None, false),
        ("loose and out of sight", None, true),
        ("paint", Some(casa), false),
        ("mow", Some(casa), false),
    ] {
        let mut task = Task::new(Ulid::generate(), title, "a0");
        task.list = list;
        task.hidden = hidden;
        state.tasks.insert(task.id, task);
    }

    assert_eq!(open_in_the_inbox(&state), 1);
    assert_eq!(open_by_name(&state), [("Casa", 2)]);
}

#[test]
fn lists_that_share_a_name_are_named_once_and_their_open_tasks_add_up() {
    let mut state = kept(&["Casa", "Casa", "casa", "Viajes"]);
    let all: Vec<ListId> = state.lists.values().map(|one| one.id).collect();
    for at in &all {
        let mut task = Task::new(Ulid::generate(), "paint", "a0");
        task.list = Some(*at);
        state.tasks.insert(task.id, task);
    }

    assert_eq!(open_by_name(&state), [("Casa", 3), ("Viajes", 1)]);
    assert_eq!(names(&state), ["Casa", "Viajes"]);

    let put_away = state
        .lists
        .values()
        .find(|one| one.name == "casa")
        .unwrap()
        .id;
    state.lists.get_mut(&put_away).unwrap().archived = true;
    assert_eq!(
        open_by_name(&state),
        [("Casa", 3), ("Viajes", 1)],
        "`brain` with that name brings the one put away too"
    );
}
