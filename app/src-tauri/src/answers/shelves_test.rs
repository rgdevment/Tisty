use super::*;
use tisty_core::event::{Event, FolderAdd, Name};

fn with_agents() -> (State, ulid::Ulid) {
    let mut state = State::default();
    let root = ulid::Ulid::generate();
    state.apply(&Event::new(
        tisty_core::DeviceId("dev_desk".into()),
        jiff::Timestamp::from_millisecond(1).unwrap(),
        Op::FolderAdd {
            id: root,
            d: FolderAdd {
                name: tisty_core::agents::FOLDER.into(),
                order: "a0".into(),
                parent: None,
                icon: Some(tisty_core::agents::ICON.into()),
                color: None,
            },
        },
    ));
    (state, root)
}

#[test]
fn the_window_says_agents_stays_instead_of_seeming_to_rename_it() {
    let (state, root) = with_agents();
    let renamed = Op::FolderRename {
        id: root,
        d: Name {
            name: "Notes".into(),
        },
    };
    assert_eq!(
        stays_hooked(&state, &[renamed]).unwrap_err().code,
        "agentsRootStays"
    );
    let named_again = Op::FolderRename {
        id: root,
        d: Name {
            name: tisty_core::agents::FOLDER.into(),
        },
    };
    assert!(stays_hooked(&state, &[named_again]).is_ok());
}
