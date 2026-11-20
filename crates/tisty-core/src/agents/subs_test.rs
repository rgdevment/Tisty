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
