use super::*;
use crate::event::{DeviceId, FolderAdd, ListAdd};
use crate::{Paths, Store};
use ulid::Ulid;

struct Here {
    _dir: tempfile::TempDir,
    store: Store,
}

fn here() -> Here {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::new(dir.path().join("data"), dir.path().join("config"));
    let store = Store::open(paths.store(), DeviceId("mac_a".into())).unwrap();
    Here { _dir: dir, store }
}

impl Here {
    fn state(&self) -> State {
        State::replay(&self.store.read_all().unwrap())
    }

    fn list(&mut self, name: &str, icon: Option<&str>, color: Option<&str>) -> Ulid {
        let id = Ulid::generate();
        self.store
            .append_batch(vec![
                Op::ListAdd {
                    id,
                    d: ListAdd {
                        name: name.into(),
                        order: "a0".into(),
                        color: None,
                    },
                },
                Op::ListLook {
                    id,
                    d: Look {
                        icon: Some(icon.map(str::to_string)),
                        color: Some(color.map(str::to_string)),
                    },
                },
            ])
            .unwrap();
        id
    }

    fn folder(&mut self, name: &str, parent: Option<Ulid>) -> Ulid {
        let id = Ulid::generate();
        self.store
            .append(Op::FolderAdd {
                id,
                d: FolderAdd {
                    name: name.into(),
                    order: "a0".into(),
                    parent,
                    icon: None,
                    color: None,
                },
            })
            .unwrap();
        id
    }
}

fn looks(ops: &[Op]) -> Vec<(Ulid, Option<String>, Option<String>)> {
    ops.iter()
        .map(|op| match op {
            Op::ListLook { id, d } | Op::FolderLook { id, d } => {
                (*id, d.icon.clone().flatten(), d.color.clone().flatten())
            }
            other => panic!("only looks are dressed, not {other:?}"),
        })
        .collect()
}

#[test]
fn what_tisty_made_and_nobody_touched_takes_the_look_it_is_made_with_now() {
    let mut here = here();
    let work = here.list("Trabajo", Some("work"), None);
    let guide = here.folder("Guía", None);
    let diagrams = here.folder("Diagrams", None);

    let mut dressed = looks(&dressed(&here.state()));
    dressed.sort();
    let mut wanted = vec![
        (work, Some("work".into()), Some("blue".into())),
        (guide, Some(GUIDE_ICON.into()), Some(GUIDE_HUE.into())),
        (
            diagrams,
            Some(DIAGRAMS_ICON.into()),
            Some(DIAGRAMS_HUE.into()),
        ),
    ];
    wanted.sort();
    assert_eq!(dressed, wanted);
}

#[test]
fn what_the_person_chose_or_made_is_left_as_it_is() {
    let mut here = here();
    here.list("Trabajo", Some("work"), Some("red"));
    here.list("Personal", Some("heart"), None);
    here.list("Mi lista", Some("work"), None);
    let mine = here.folder("Proyectos", None);
    here.folder("Guía", Some(mine));

    assert!(dressed(&here.state()).is_empty());
}

#[test]
fn dressed_once_there_is_nothing_left_to_dress() {
    let mut here = here();
    here.list("Health", Some("health"), None);
    here.folder("Guide", None);
    let first = dressed(&here.state());
    here.store.append_batch(first).unwrap();

    assert!(dressed(&here.state()).is_empty());
}

#[test]
fn the_look_is_one_the_catalogue_and_the_palette_know() {
    for (icon, hue) in [(GUIDE_ICON, GUIDE_HUE), (DIAGRAMS_ICON, DIAGRAMS_HUE)] {
        assert!(crate::model::icon::known(icon), "unknown icon: {icon}");
        assert!(
            crate::model::hue::kept(hue).is_some(),
            "unknown colour: {hue}"
        );
    }
}
