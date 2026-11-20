use super::*;
use crate::DeviceId;
use crate::event::{DocAdd, Event, Filed, FolderAdd, Look, Name};
use ulid::Ulid;

struct World {
    state: State,
    root: FolderId,
    outside: FolderId,
    ms: i64,
}

impl World {
    fn apply(&mut self, op: Op) {
        self.ms += 1;
        let at = jiff::Timestamp::from_millisecond(self.ms).unwrap();
        self.state
            .apply(&Event::new(DeviceId("dev_desk".into()), at, op));
    }

    fn folder(&mut self, name: &str, icon: Option<&str>) -> FolderId {
        let id = Ulid::generate();
        self.apply(Op::FolderAdd {
            id,
            d: FolderAdd {
                name: name.into(),
                order: "a0".into(),
                parent: None,
                icon: icon.map(str::to_string),
                color: None,
            },
        });
        id
    }
}

fn world() -> World {
    let mut world = World {
        state: State::default(),
        root: Ulid::nil(),
        outside: Ulid::nil(),
        ms: 0,
    };
    world.root = world.folder(FOLDER, Some(ICON));
    world.outside = world.folder("Notes", None);
    world.apply(Op::DocAdd {
        id: Ulid::generate(),
        d: DocAdd {
            file: "rule".into(),
            order: "a0".into(),
            folder: Some(world.root),
            ..Default::default()
        },
    });
    world
}

#[test]
fn nothing_takes_agents_off_the_top_renames_it_or_archives_it() {
    let mut world = world();
    let root = world.root;
    let into = Filed {
        folder: Some(Some(world.outside)),
        page_of: None,
        order: None,
    };
    for op in [
        Op::FolderMove { id: root, d: into },
        Op::FolderRename {
            id: root,
            d: Name {
                name: "Notes".into(),
            },
        },
        Op::FolderLook {
            id: root,
            d: Look {
                icon: Some(Some("home".into())),
                color: None,
            },
        },
        Op::FolderLook {
            id: root,
            d: Look {
                icon: Some(None),
                color: None,
            },
        },
        Op::FolderArchive { id: root },
        Op::FolderDelete { id: root },
    ] {
        world.apply(op);
        assert_eq!(root_of(&world.state), Some(root));
    }
}

#[test]
fn a_colour_or_a_new_order_is_still_the_persons_to_give() {
    let mut world = world();
    let root = world.root;
    world.apply(Op::FolderLook {
        id: root,
        d: Look {
            icon: Some(Some(ICON.into())),
            color: Some(Some("purple".into())),
        },
    });
    world.apply(Op::FolderMove {
        id: root,
        d: Filed {
            folder: None,
            page_of: None,
            order: Some("b0".into()),
        },
    });
    let folder = &world.state.folders[&root];
    assert_eq!(folder.color.as_deref(), Some("purple"));
    assert_eq!(folder.order, "b0");
    assert_eq!(root_of(&world.state), Some(root));
}

#[test]
fn an_agents_left_empty_still_goes_and_any_other_folder_moves_freely() {
    let mut world = world();
    let (root, outside) = (world.root, world.outside);
    world.apply(Op::FolderRename {
        id: outside,
        d: Name {
            name: "Elsewhere".into(),
        },
    });
    assert_eq!(world.state.folders[&outside].name, "Elsewhere");
    let rule = world.state.docs.values().next().unwrap().id;
    world.apply(Op::DocMove {
        id: rule,
        d: Filed {
            folder: Some(Some(outside)),
            page_of: None,
            order: None,
        },
    });
    world.apply(Op::FolderDelete { id: root });
    assert_eq!(root_of(&world.state), None);
}

#[test]
fn an_agents_not_yet_joined_still_holds_its_documents_and_their_pages() {
    let mut world = world();
    let second = Ulid::from_parts((1 << 48) - 1, 0);
    world.apply(Op::FolderAdd {
        id: second,
        d: FolderAdd {
            name: FOLDER.into(),
            order: "b0".into(),
            parent: None,
            icon: Some(ICON.into()),
            color: None,
        },
    });
    assert_eq!(roots(&world.state), vec![world.root, second]);
    let claude = Ulid::generate();
    world.apply(Op::FolderAdd {
        id: claude,
        d: FolderAdd {
            name: "Claude".into(),
            order: "a0".into(),
            parent: Some(second),
            icon: None,
            color: None,
        },
    });
    let rule = Ulid::generate();
    world.apply(Op::DocAdd {
        id: rule,
        d: DocAdd {
            file: "second-rule".into(),
            order: "a0".into(),
            folder: Some(claude),
            ..Default::default()
        },
    });
    let page = Ulid::generate();
    world.apply(Op::DocAdd {
        id: page,
        d: DocAdd {
            file: "second-page".into(),
            order: "a0".into(),
            page_of: Some(rule),
            ..Default::default()
        },
    });
    let state = &world.state;
    assert!(holds_folder(state, claude));
    assert!(holds(state, &state.docs[&rule]));
    assert!(holds(state, &state.docs[&page]));
    let held = holding(state);
    assert!(held(&state.docs[&rule]) && held(&state.docs[&page]));
}
