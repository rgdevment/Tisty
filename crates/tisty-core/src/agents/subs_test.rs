use super::*;
use crate::DeviceId;
use crate::event::{Event, FolderAdd};

fn root(state: &mut State) -> FolderId {
    let id = ulid::Ulid::generate();
    apply(
        state,
        "dev_desk",
        Op::FolderAdd {
            id,
            d: FolderAdd {
                name: super::super::FOLDER.into(),
                order: "a0".into(),
                parent: None,
                icon: Some(super::super::ICON.into()),
                color: None,
                computer: None,
            },
        },
    );
    id
}

fn apply(state: &mut State, who: &str, op: Op) {
    let at = jiff::Timestamp::from_millisecond(state.folders.len() as i64 + 1).unwrap();
    state.apply(&Event::new(DeviceId(who.into()), at, op));
}

fn names(state: &State, root: FolderId) -> Vec<String> {
    let mut all: Vec<String> = state
        .folders
        .values()
        .filter(|one| one.parent == Some(root))
        .map(|one| one.name.clone())
        .collect();
    all.sort();
    all
}

#[test]
fn each_name_missing_gets_its_folder_and_one_already_there_is_left_alone() {
    let mut state = State::default();
    let root = root(&mut state);
    for op in sub_folders(&state, root, &["Global"]) {
        apply(&mut state, "dev_desk", op);
    }
    let ops = sub_folders(&state, root, &["Global", "Gemini", "Claude"]);
    assert_eq!(ops.len(), 2);
    for op in ops {
        apply(&mut state, "dev_desk", op);
    }
    assert_eq!(names(&state, root), ["Claude", "Gemini", "Global"]);
    assert!(sub_folders(&state, root, &["Global", "Gemini", "Claude"]).is_empty());
}

#[test]
fn two_computers_making_the_same_folder_make_one() {
    let mut state = State::default();
    let root = root(&mut state);
    let (desk, laptop) = (
        sub_folders(&state, root, &["Gemini"]),
        sub_folders(&state, root, &["Gemini"]),
    );
    assert_eq!(desk, laptop);
    for op in desk {
        apply(&mut state, "dev_desk", op);
    }
    for op in laptop {
        apply(&mut state, "dev_laptop", op);
    }
    assert_eq!(names(&state, root), ["Gemini"]);
}

#[test]
fn a_folder_the_person_deleted_is_never_made_again() {
    let mut state = State::default();
    let root = root(&mut state);
    for op in sub_folders(&state, root, &["Gemini"]) {
        apply(&mut state, "dev_desk", op);
    }
    let id = sub_id(root, "Gemini");
    apply(&mut state, "dev_desk", Op::FolderDelete { id });
    assert!(sub_folders(&state, root, &["Gemini"]).is_empty());
    assert!(names(&state, root).is_empty());
}

fn named_as(state: &mut State, who: &str, name: &str) {
    apply(
        state,
        who,
        Op::DeviceNamed {
            d: DeviceId(who.into()),
            name: name.into(),
            os: None,
        },
    );
}

fn made(state: &mut State, who: &str, root: FolderId) -> FolderId {
    let (id, op) = computer_folder(state, root, &DeviceId(who.into()));
    apply(state, who, op.unwrap());
    id
}

#[test]
fn a_computers_folder_name_is_short_has_no_slash_and_is_told_apart_by_its_device() {
    let mut state = State::default();
    let root = root(&mut state);
    named_as(
        &mut state,
        "dev_ab12cd34",
        &format!("a/b {}", "long ".repeat(20)),
    );
    let (_, op) = computer_folder(&state, root, &DeviceId("dev_ab12cd34".into()));
    let Some(Op::FolderAdd { d, .. }) = op else {
        panic!("no folder made");
    };
    assert!(d.name.chars().count() <= crate::model::FOLDER_NAME_AT_MOST);
    assert!(!d.name.contains('/'));
    assert!(d.name.ends_with("ab12cd34"));
    assert_eq!(d.computer, Some(DeviceId("dev_ab12cd34".into())));

    named_as(&mut state, "dev_ef56ab78", "Desk");
    named_as(&mut state, "dev_ab12cd34", "Desk");
    let (_, other) = computer_folder(&state, root, &DeviceId("dev_ef56ab78".into()));
    let Some(Op::FolderAdd { d: other, .. }) = other else {
        panic!("no folder made");
    };
    assert_ne!(other.name, "Desk");
    assert!(other.name.starts_with("Desk"));
}

#[test]
fn two_computers_with_one_name_get_two_different_folders() {
    let mut state = State::default();
    let root = root(&mut state);
    named_as(&mut state, "dev_aaaa1111", "Desk");
    named_as(&mut state, "dev_bbbb2222", "Desk");
    let first = made(&mut state, "dev_aaaa1111", root);
    let second = made(&mut state, "dev_bbbb2222", root);
    assert_ne!(first, second);
    let mut found = names(&state, root);
    found.dedup();
    assert_eq!(found.len(), 2);
}

#[test]
fn a_computer_that_has_its_folder_makes_none_and_gets_the_same_one_back() {
    let mut state = State::default();
    let root = root(&mut state);
    let first = made(&mut state, "dev_aaaa1111", root);
    let (again, op) = computer_folder(&state, root, &DeviceId("dev_aaaa1111".into()));
    assert_eq!(again, first);
    assert!(op.is_none());
}

#[test]
fn a_folder_by_name_is_reused_before_a_new_one_is_planned() {
    let mut state = State::default();
    let root = root(&mut state);
    let mut ops = Vec::new();
    let made = room(&state, &mut ops, root, "Notes");
    assert_eq!(ops.len(), 1);
    assert_eq!(made, sub_id(root, "Notes"));
    assert_eq!(room(&state, &mut ops, root, "Notes"), made);
    assert_eq!(ops.len(), 1);

    apply(&mut state, "dev_desk", ops.remove(0));
    let mut none = Vec::new();
    assert_eq!(room(&state, &mut none, root, "Notes"), made);
    assert!(none.is_empty());
}

#[test]
fn a_folder_whose_derived_id_was_deleted_is_made_with_a_fresh_one() {
    let mut state = State::default();
    let root = root(&mut state);
    let mut ops = Vec::new();
    let first = room(&state, &mut ops, root, "Notes");
    apply(&mut state, "dev_desk", ops.remove(0));
    apply(&mut state, "dev_desk", Op::FolderDelete { id: first });

    let mut again = Vec::new();
    let fresh = room(&state, &mut again, root, "Notes");
    assert_ne!(fresh, first);
    assert_eq!(again.len(), 1);
}

#[test]
fn a_folder_planned_beside_a_fresh_id_keeps_its_place_in_the_order() {
    let mut state = State::default();
    let root = root(&mut state);
    let mut ops = Vec::new();
    let first = room(&state, &mut ops, root, "Notes");
    apply(&mut state, "dev_desk", ops.remove(0));
    apply(&mut state, "dev_desk", Op::FolderDelete { id: first });

    let mut planned = Vec::new();
    let again = room(&state, &mut planned, root, "Notes");
    room(&state, &mut planned, root, "Other");
    assert_ne!(again, first);
    assert_eq!(room(&state, &mut planned, root, "Notes"), again);
    assert_eq!(planned.len(), 2);
    let orders: Vec<&str> = planned
        .iter()
        .filter_map(|op| match op {
            Op::FolderAdd { d, .. } => Some(d.order.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(orders.len(), 2);
    assert_ne!(
        orders[0], orders[1],
        "each planned folder takes its own place"
    );
}
