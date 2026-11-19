use super::{Group, groups_of};

#[test]
fn every_folder_and_list_gives_its_name_and_colour_to_the_network() {
    let mut state = tisty_core::State::default();
    let folder = ulid::Ulid::generate();
    let list = ulid::Ulid::generate();
    let mut shelf = tisty_core::model::Folder::new(folder, "Proyectos", "a0");
    shelf.color = Some("teal".into());
    state.folders.insert(folder, shelf);
    state.lists.insert(
        list,
        tisty_core::List {
            id: list,
            name: "Casa".into(),
            order: "a0".into(),
            color: None,
            icon: None,
            archived: false,
        },
    );

    assert_eq!(
        groups_of(&state),
        vec![
            Group {
                id: folder.to_string(),
                name: "Proyectos".into(),
                color: Some("teal".into()),
            },
            Group {
                id: list.to_string(),
                name: "Casa".into(),
                color: None,
            },
        ]
    );
}
