use super::*;
use crate::event::{DeviceId, Held, Name, TaskAdd};
use crate::{Paths, Store};

struct Here {
    _dir: tempfile::TempDir,
    paths: Paths,
    store: Store,
}

fn here() -> Here {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::new(dir.path().join("data"), dir.path().join("config"));
    let store = Store::open(paths.store(), DeviceId("mac_a".into())).unwrap();
    Here {
        _dir: dir,
        paths,
        store,
    }
}

impl Here {
    fn untouched(&self) -> bool {
        untouched(&self.paths.store(), &self.paths.docs())
    }

    fn sow(&mut self, code: &str) {
        self.store.append_batch(crate::model::sown(code)).unwrap();
    }

    fn guide(&mut self, named: &str, said: impl Fn(&str) -> Said) -> (DocId, String) {
        self.guide_reading(named, "# Guía\n\nAsí se usa Tisty.\n", said)
    }

    fn guide_reading(
        &mut self,
        named: &str,
        body: &str,
        said: impl Fn(&str) -> Said,
    ) -> (DocId, String) {
        let folder = Ulid::generate();
        self.store
            .append_batch(vec![
                Op::AttachKept {
                    d: Held {
                        at: "attachments/ab/captura-ab12.png".into(),
                        sha256: "ab12".into(),
                        bytes: 10,
                    },
                },
                Op::FolderAdd {
                    id: folder,
                    d: FolderAdd {
                        name: named.into(),
                        order: "a0".into(),
                        parent: None,
                        icon: None,
                        color: None,
                    },
                },
            ])
            .unwrap();
        let made =
            crate::docs::create(&self.paths.docs(), &DeviceId("mac_a".into()), body).unwrap();
        let id = Ulid::generate();
        self.store
            .append(Op::DocAdd {
                id,
                d: DocAdd {
                    file: made.id.clone(),
                    order: "a0".into(),
                    by: Some(SOWN_BY.into()),
                    guest: true,
                    said: Some(said(body)),
                    folder: Some(folder),
                    ..DocAdd::default()
                },
            })
            .unwrap();
        (id, made.id)
    }

    fn sown_guide(&mut self) -> (DocId, String) {
        self.guide("Guía", Said::of)
    }
}

const SHIPPED_PAGE: &str = include_str!("../../../app/src-tauri/resources/guide/es/pagina.md");

fn as_in_1_24_4(body: &str) -> Said {
    Said {
        title: crate::docs::titled(body),
        bytes: None,
        tags: Some(Vec::new()),
        by: None,
        print: None,
    }
}

#[test]
fn a_store_that_only_opened_once_is_untouched() {
    let mut here = here();
    here.sow("es");
    here.sown_guide();

    assert!(here.untouched());
}

#[test]
fn a_store_that_never_wrote_anything_is_untouched() {
    assert!(here().untouched());
}

#[test]
fn the_guide_sown_twice_by_two_openings_is_still_untouched() {
    let mut here = here();
    here.sow("en");
    here.guide("Guide", Said::of);
    here.sow("en");
    here.guide("Guide", Said::of);

    assert!(here.untouched());
}

#[test]
fn a_guide_sown_by_1_24_4_and_read_back_since_is_untouched() {
    let mut here = here();
    here.sow("es");
    let (id, file) = here.guide_reading("Guía", SHIPPED_PAGE, as_in_1_24_4);
    let body = crate::docs::read(&here.paths.docs(), &file).unwrap();
    for _ in 0..2 {
        here.store
            .append(Op::DocSaid {
                id,
                d: Said::of(&body),
            })
            .unwrap();
    }

    assert!(here.untouched());
}

#[test]
fn a_single_task_makes_it_the_persons() {
    let mut here = here();
    here.sow("es");
    here.store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("Llamar al banco", "a0"),
        })
        .unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_task_made_and_deleted_still_counts() {
    let mut here = here();
    let id = Ulid::generate();
    here.store
        .append(Op::TaskAdd {
            id,
            d: TaskAdd::new("borrador", "a0"),
        })
        .unwrap();
    here.store.append(Op::TaskDelete { id }).unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_list_of_the_persons_own_or_one_renamed_makes_it_the_persons() {
    let mut own = here();
    own.store
        .append(Op::ListAdd {
            id: Ulid::generate(),
            d: ListAdd {
                name: "Casa".into(),
                order: "a0".into(),
                color: None,
            },
        })
        .unwrap();
    assert!(!own.untouched());

    let mut renamed = here();
    renamed.sow("es");
    let first = crate::State::replay(&renamed.store.read_all().unwrap())
        .lists
        .keys()
        .copied()
        .next()
        .unwrap();
    renamed
        .store
        .append(Op::ListRename {
            id: first,
            d: Name {
                name: "Trabajo".into(),
            },
        })
        .unwrap();
    assert!(!renamed.untouched());
}

#[test]
fn a_starter_list_painted_another_colour_makes_it_the_persons() {
    let mut here = here();
    here.sow("es");
    let first = crate::State::replay(&here.store.read_all().unwrap())
        .lists
        .keys()
        .copied()
        .next()
        .unwrap();
    here.store
        .append(Op::ListLook {
            id: first,
            d: Look {
                icon: None,
                color: Some(Some("red".into())),
            },
        })
        .unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_document_the_person_wrote_makes_it_the_persons() {
    let mut here = here();
    here.sown_guide();
    let made = crate::docs::create(
        &here.paths.docs(),
        &DeviceId("mac_a".into()),
        "# Ideas\n\nmías\n",
    )
    .unwrap();
    here.store
        .append(Op::DocAdd {
            id: Ulid::generate(),
            d: DocAdd {
                file: made.id,
                order: "a1".into(),
                ..DocAdd::default()
            },
        })
        .unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_guide_written_into_makes_it_the_persons() {
    let mut here = here();
    let (id, _) = here.sown_guide();
    here.store
        .append(Op::DocSaid {
            id,
            d: Said::of("# Guía\n\nAsí se usa Tisty.\n\nMi nota.\n"),
        })
        .unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_guide_signed_by_a_hand_makes_it_the_persons() {
    let mut here = here();
    let (id, file) = here.sown_guide();
    let body = crate::docs::read(&here.paths.docs(), &file).unwrap();
    here.store
        .append(Op::DocSaid {
            id,
            d: Said::of(&body).by(Some("Mario".into())),
        })
        .unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_guide_changed_on_disk_with_nothing_in_the_log_makes_it_the_persons() {
    let mut here = here();
    let (_, file) = here.sown_guide();
    crate::docs::write(
        &here.paths.docs(),
        &file,
        "# Guía\n\nAsí se usa Tisty.\n\nalgo mío\n",
    )
    .unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_document_on_disk_the_log_never_named_makes_it_the_persons() {
    let mut here = here();
    here.sown_guide();
    crate::docs::create(&here.paths.docs(), &DeviceId("mac_a".into()), "# Suelto\n").unwrap();

    assert!(!here.untouched());
}

#[test]
fn an_undo_is_the_person_doing_something() {
    let mut here = here();
    here.store
        .append_batch_marked(crate::model::sown("es"), true)
        .unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_guide_an_agent_filed_is_not_a_sown_one() {
    let mut here = here();
    let agent = DeviceId("agent_claude".into());
    here.store
        .append(Op::DeviceJoin {
            d: agent.clone(),
            k: Some(DeviceKind::Agent),
            p: None,
        })
        .unwrap();
    let mut events = here.store.read_all().unwrap();
    let mut sown = Event::new(
        agent,
        jiff::Timestamp::now(),
        Op::FolderAdd {
            id: Ulid::generate(),
            d: FolderAdd {
                name: "Guía".into(),
                order: "a0".into(),
                parent: None,
                icon: None,
                color: None,
            },
        },
    );
    sown.seq = 1;
    events.push(sown);

    assert!(sown_only(&events).is_none());
}

#[test]
fn an_attachment_kept_outside_the_guide_makes_it_the_persons() {
    let mut here = here();
    here.store
        .append(Op::AttachKept {
            d: Held {
                at: "attachments/cd/foto-cd34.png".into(),
                sha256: "cd34".into(),
                bytes: 10,
            },
        })
        .unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_store_that_cannot_be_read_is_never_taken_for_new() {
    let here = here();
    let device = here.paths.store().join("pc_b");
    std::fs::create_dir_all(&device).unwrap();
    std::fs::write(device.join("active.tisty"), b"\x00\x01 not a log").unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_guide_sown_by_1_24_4_and_never_read_back_is_untouched() {
    let mut here = here();
    here.sow("es");
    here.guide_reading("Guía", SHIPPED_PAGE, as_in_1_24_4);

    assert!(here.untouched());
}

#[test]
fn a_guide_sown_by_1_24_4_keeps_its_untouched_reading_wherever_its_links_point() {
    let mut here = here();
    here.sow("es");
    let moved = SHIPPED_PAGE.replacen("](", "](tisty:doc/mac_a-0002", 1);
    here.guide_reading("Guía", &moved, as_in_1_24_4);

    assert!(here.untouched());
}

#[test]
fn a_guide_sown_by_1_24_4_and_edited_outside_with_its_title_kept_is_the_persons() {
    let mut here = here();
    here.sow("es");
    let (_, file) = here.guide_reading("Guía", SHIPPED_PAGE, as_in_1_24_4);
    let edited = format!("{SHIPPED_PAGE}\nalgo mío\n");
    crate::docs::write(&here.paths.docs(), &file, &edited).unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_guide_sown_by_1_24_4_edited_and_read_back_is_the_persons() {
    let mut here = here();
    here.sow("es");
    let (id, file) = here.guide_reading("Guía", SHIPPED_PAGE, as_in_1_24_4);
    let edited = format!("{SHIPPED_PAGE}\nalgo mío\n");
    crate::docs::write(&here.paths.docs(), &file, &edited).unwrap();
    here.store
        .append(Op::DocSaid {
            id,
            d: Said::of(&edited),
        })
        .unwrap();

    assert!(!here.untouched());
}

#[test]
fn a_guide_sown_without_a_print_that_no_release_shipped_is_the_persons() {
    let mut here = here();
    here.sow("es");
    here.guide("Guía", as_in_1_24_4);

    assert!(!here.untouched());
}
