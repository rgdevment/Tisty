use std::path::{Path, PathBuf};

use tisty_core::event::{DocAdd, FolderAdd, Said};
use tisty_core::model::{DocId, FolderId};
use tisty_core::parcel::Along;
use tisty_core::{DeviceId, Event, Op, State, attach, docs, order, parcel};
use ulid::Ulid;

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn device(name: &str) -> DeviceId {
    DeviceId(name.into())
}

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

struct Room {
    data: PathBuf,
    state: State,
    dev: DeviceId,
    seq: i64,
}

impl Room {
    fn new(under: &Path, named: &str) -> Self {
        let data = under.join(named);
        std::fs::create_dir_all(data.join("docs")).unwrap();
        Self {
            data,
            state: State::default(),
            dev: device(named),
            seq: 0,
        }
    }

    fn tell(&mut self, op: Op) {
        self.seq += 1;
        let event = Event::new(self.dev.clone(), at(self.seq), op);
        self.state.apply(&event);
    }

    fn folder(&mut self, name: &str, parent: Option<FolderId>, icon: &str) -> FolderId {
        let id = Ulid::generate();
        let order = order::last_of(
            self.state
                .under(parent)
                .iter()
                .map(|one| one.order.as_str()),
        );
        self.tell(Op::FolderAdd {
            id,
            d: FolderAdd {
                name: name.into(),
                order,
                parent,
                icon: Some(icon.into()),
                color: Some("teal".into()),
            },
        });
        id
    }

    fn doc(
        &mut self,
        body: &str,
        folder: Option<FolderId>,
        page_of: Option<DocId>,
    ) -> (DocId, String) {
        let made = docs::create(&self.data.join("docs"), &self.dev, body).unwrap();
        let id = Ulid::generate();
        let order = order::last_of(
            self.state
                .docs
                .values()
                .filter(|one| one.folder == folder)
                .map(|one| one.order.as_str()),
        );
        self.tell(Op::DocAdd {
            id,
            d: DocAdd {
                made: None,
                by: None,
                file: made.id.clone(),
                order,
                said: Some(Said {
                    title: made.title.clone(),
                    bytes: None,
                    tags: Some(Vec::new()),
                }),
                folder,
                page_of,
            },
        });
        (id, made.id)
    }

    fn take_in(&mut self, from: &Path) -> parcel::Landed {
        let (landed, ops) = parcel::read(
            &self.data,
            &self.state,
            &self.dev.clone(),
            from,
            &Along::default(),
        )
        .unwrap();
        for op in ops {
            self.tell(op);
        }
        landed
    }

    fn titled(&self, name: &str) -> &tisty_core::model::Kept {
        self.state
            .docs
            .values()
            .find(|one| one.title.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("no document called {name}"))
    }

    fn shelf(&self, name: &str) -> &tisty_core::model::Folder {
        self.state
            .folders
            .values()
            .find(|one| one.name == name)
            .unwrap_or_else(|| panic!("no folder called {name}"))
    }

    fn body(&self, file: &str) -> String {
        docs::read(&self.data.join("docs"), file).unwrap()
    }
}

fn filled(room: &mut Room) -> PathBuf {
    let shed = room.data.join("attachments").join("ab");
    std::fs::create_dir_all(&shed).unwrap();
    std::fs::write(shed.join("plano-91f2ab00.png"), b"a picture").unwrap();

    let personal = room.folder("Personal", None, "home");
    let casa = room.folder("Casa", Some(personal), "build");

    let (book, book_file) = room.doc("# Obra\n\ntexto", Some(casa), None);
    let (_, page_file) = room.doc(
        "# Plano\n\n![el plano](<attachments/ab/plano-91f2ab00.png>)",
        None,
        Some(book),
    );
    docs::write(
        &room.data.join("docs"),
        &book_file,
        &format!("# Obra\n\ntexto\n\n![Plano](tisty:doc/{page_file})"),
    )
    .unwrap();

    let (locked, _) = room.doc("# Guardado\n\nno se toca", Some(personal), None);
    room.tell(Op::DocLock { id: locked });
    let (away, _) = room.doc("# Terminado\n\nya esta", Some(personal), None);
    room.tell(Op::DocArchive { id: away });

    room.data.parent().unwrap().join("todo.tistydoc")
}

#[test]
fn everything_written_travels_to_another_tisty_and_lands_as_its_own() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let box_at = filled(&mut here);

    let sent = parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();
    assert_eq!(
        (sent.docs, sent.pages, sent.folders, sent.files),
        (3, 1, 2, 1)
    );

    let mut there = Room::new(room.path(), "theirs");
    let landed = there.take_in(&box_at);

    assert_eq!((landed.docs, landed.pages, landed.folders), (3, 1, 2));
    assert_eq!(landed.files, 1);
    assert_eq!(landed.missed, 0);

    let casa = there.shelf("Casa");
    assert_eq!(casa.parent, Some(there.shelf("Personal").id));
    assert_eq!(casa.icon.as_deref(), Some("build"));
    assert_eq!(casa.color.as_deref(), Some("teal"));

    let obra = there.titled("Obra");
    assert_eq!(obra.folder, Some(casa.id));
    let plano = there.titled("Plano");
    assert_eq!(plano.page_of, Some(obra.id));
    assert!(there.titled("Guardado").locked);
    assert!(there.titled("Terminado").archived);
}

#[test]
fn what_a_document_points_at_still_points_at_it_under_its_new_name() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let box_at = filled(&mut here);
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    there.take_in(&box_at);

    let obra = there.titled("Obra");
    let plano = there.titled("Plano");
    let said = there.body(&obra.file);
    assert!(
        said.contains(&format!("tisty:doc/{}", plano.file)),
        "the page card was left pointing at a name this store never had: {said}"
    );
    assert!(!said.contains("mine-"), "{said}");

    let carried = there.body(&plano.file);
    let at = carried
        .split_once("](<")
        .and_then(|(_, rest)| rest.split_once(">)"))
        .map(|(at, _)| at)
        .unwrap();
    assert_eq!(
        std::fs::read(attach::resolve(at, &there.data).unwrap()).unwrap(),
        b"a picture"
    );
}

#[test]
fn a_parcel_carries_the_writing_and_not_one_line_of_the_log() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let box_at = filled(&mut here);
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let file = std::fs::File::open(&box_at).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let inside: Vec<String> = (0..zip.len())
        .map(|i| zip.by_index(i).unwrap().name().to_string())
        .collect();

    assert!(
        !inside.iter().any(|one| one.starts_with("store/")),
        "{inside:?}"
    );
    assert!(inside.iter().any(|one| one == "tisty-docs.json"));
    assert!(inside.iter().filter(|one| one.starts_with("docs/")).count() == 4);
}

#[test]
fn a_folder_that_is_already_there_takes_the_documents_in_rather_than_standing_beside_itself() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let box_at = filled(&mut here);
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    let personal = there.folder("personal ", None, "home");
    there.doc("# Suyo\n\nya estaba", Some(personal), None);

    let landed = there.take_in(&box_at);

    assert_eq!(landed.joined, 1, "it did not recognise the folder by name");
    assert_eq!(landed.folders, 1);
    assert_eq!(
        there
            .state
            .folders
            .values()
            .filter(|one| one.parent.is_none())
            .count(),
        1,
        "a second Personal was made beside the first"
    );
    assert_eq!(there.titled("Guardado").folder, Some(personal));
    assert_eq!(there.shelf("Casa").parent, Some(personal));
}

#[test]
fn one_document_can_travel_alone_and_its_pages_go_with_it() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let box_at = filled(&mut here);
    let obra = here.titled("Obra").file.clone();

    let sent = parcel::write(&here.data, &here.state, &[obra], &box_at, &Along::default()).unwrap();

    assert_eq!((sent.docs, sent.pages), (1, 1));
    let mut there = Room::new(room.path(), "theirs");
    let landed = there.take_in(&box_at);
    assert_eq!((landed.docs, landed.pages), (1, 1));
    assert_eq!(there.state.docs.len(), 2);
}

#[test]
fn a_file_that_is_not_in_the_store_is_named_rather_than_carried_in_silence() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc(
        "# Sola\n\n![un video](<attachments/6d/clip-da1d77da.mov>)",
        None,
        None,
    );

    let box_at = room.path().join("una.tistydoc");
    let sent = parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    assert_eq!(sent.files, 0);
    assert_eq!(sent.left, ["attachments/6d/clip-da1d77da.mov"]);
}

#[test]
fn everything_written_plainly_stands_in_the_folders_it_was_kept_in() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    filled(&mut here);

    let out = room.path().join("plano");
    let sent = parcel::plainly(&here.data, &here.state, &[], &out, &Along::default()).unwrap();

    assert_eq!((sent.docs, sent.pages, sent.files), (3, 1, 1));
    assert_eq!(sent.folders, 2);
    let obra = out.join("Personal").join("Casa").join("Obra");
    assert!(obra.join("Obra.md").is_file(), "{obra:?}");
    assert!(obra.join("01 Plano.md").is_file());
    assert!(
        obra.join("attachments")
            .join("ab")
            .join("plano-91f2ab00.png")
            .is_file()
    );
    assert!(
        out.join("Personal")
            .join("Guardado")
            .join("Guardado.md")
            .is_file()
    );
    assert!(
        out.join("Personal")
            .join("Terminado")
            .join("Terminado.md")
            .is_file(),
        "an archived document was left out of what says it takes everything"
    );
}

#[test]
fn two_documents_called_the_same_thing_do_not_write_over_each_other() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc("# Acta\n\nla primera", None, None);
    here.doc("# Acta\n\nla segunda", None, None);

    let out = room.path().join("plano");
    let sent = parcel::plainly(&here.data, &here.state, &[], &out, &Along::default()).unwrap();

    assert_eq!(sent.docs, 2);
    assert_eq!(
        std::fs::read_to_string(out.join("Acta").join("Acta.md")).unwrap(),
        "# Acta\n\nla primera\n"
    );
    assert_eq!(
        std::fs::read_to_string(out.join("Acta 2").join("Acta 2.md")).unwrap(),
        "# Acta\n\nla segunda\n"
    );
}

#[test]
fn what_is_too_heavy_to_keep_here_is_carried_from_the_folder_everyone_shares() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc(
        "# Charla\n\n![el video](<attachments/6d/clip-da1d77da.mov>)",
        None,
        None,
    );
    let shared = room.path().join("drive");
    let shelf = shared.join("attachments").join("6d");
    std::fs::create_dir_all(&shelf).unwrap();
    std::fs::write(shelf.join("clip-da1d77da.mov"), b"a heavy video").unwrap();

    let box_at = room.path().join("con-video.tistydoc");
    let sent = parcel::write(
        &here.data,
        &here.state,
        &[],
        &box_at,
        &Along {
            also: Some(&shared),
            ..Along::default()
        },
    )
    .unwrap();

    assert_eq!(sent.files, 1, "the video was left behind: {:?}", sent.left);
    assert!(sent.left.is_empty());

    let mut there = Room::new(room.path(), "theirs");
    there.take_in(&box_at);
    let charla = there.titled("Charla");
    let said = there.body(&charla.file);
    let at = said
        .split_once("](<")
        .and_then(|(_, rest)| rest.split_once(">)"))
        .map(|(at, _)| at)
        .unwrap();
    assert_eq!(
        std::fs::read(attach::resolve(at, &there.data).unwrap()).unwrap(),
        b"a heavy video"
    );
}

#[test]
fn a_long_carry_says_how_far_along_it_is_rather_than_going_quiet() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let box_at = filled(&mut here);

    let steps = std::cell::RefCell::new(Vec::new());
    let sent = parcel::write(
        &here.data,
        &here.state,
        &[],
        &box_at,
        &Along {
            say: Some(&|step| steps.borrow_mut().push((step.done, step.whole))),
            ..Along::default()
        },
    )
    .unwrap();

    let told = steps.into_inner();
    assert_eq!(told.len(), sent.docs + sent.pages + sent.files);
    assert!(told.iter().all(|(done, whole)| done <= whole), "{told:?}");
    assert_eq!(told.last(), Some(&(5, 5)), "{told:?}");
}

#[test]
fn a_parcel_is_never_written_into_the_store_it_came_from() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc("# Sola\n\nnada mas", None, None);

    assert!(
        parcel::write(
            &here.data,
            &here.state,
            &[],
            &here.data.join("una.tistydoc"),
            &Along::default()
        )
        .is_err()
    );
}

#[test]
fn what_is_not_a_parcel_is_turned_away_rather_than_half_read() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc("# Sola\n\nnada mas", None, None);

    let stray = room.path().join("cualquiera.tistydoc");
    std::fs::write(&stray, b"not a zip at all").unwrap();
    assert!(
        parcel::read(
            &here.data,
            &here.state,
            &here.dev.clone(),
            &stray,
            &Along::default()
        )
        .is_err()
    );
}

#[test]
fn a_title_that_names_a_device_or_a_path_becomes_a_folder_both_systems_can_hold() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    for said in [
        "# CON\n\nen Windows esto es un dispositivo",
        "# ../../fuera\n\nsubir por el arbol",
        "# C:\\Windows\\System32\n\nuna ruta entera",
        "# nombre.\n\ntermina en punto",
        "# año 2026: qué tal ✅\n\nacentos y emoji",
    ] {
        here.doc(said, None, None);
    }

    let out = room.path().join("plano");
    let sent = parcel::plainly(&here.data, &here.state, &[], &out, &Along::default()).unwrap();

    assert_eq!(sent.docs, 5, "left behind: {:?}", sent.left);
    let made: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .filter_map(|one| one.ok())
        .map(|one| one.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(made.len(), 5, "{made:?}");
    for one in &made {
        assert!(!one.contains(['/', '\\', ':']), "{one}");
        assert!(!one.starts_with('.'), "{one}");
        assert!(!one.ends_with('.') && !one.ends_with(' '), "{one}");
    }
    assert!(
        std::fs::read_dir(room.path())
            .unwrap()
            .filter_map(|one| one.ok())
            .all(|one| one.file_name() != "fuera"),
        "a title climbed out of the folder it was given"
    );
}

#[test]
fn a_parcel_from_a_newer_tisty_is_turned_away_rather_than_half_understood() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc("# Sola\n\nnada mas", None, None);
    let box_at = room.path().join("nueva.tistydoc");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let said = std::fs::read(&box_at).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(said)).unwrap();
    let ahead = room.path().join("ahead.tistydoc");
    let mut out = zip::ZipWriter::new(std::fs::File::create(&ahead).unwrap());
    for i in 0..zip.len() {
        let mut held = zip.by_index(i).unwrap();
        let named = held.name().to_string();
        let mut body = Vec::new();
        std::io::Read::read_to_end(&mut held, &mut body).unwrap();
        if named == "tisty-docs.json" {
            let said = String::from_utf8(body)
                .unwrap()
                .replace("\"version\": 1", "\"version\": 99");
            body = said.into_bytes();
        }
        out.start_file(named, zip::write::SimpleFileOptions::default())
            .unwrap();
        std::io::Write::write_all(&mut out, &body).unwrap();
    }
    out.finish().unwrap();

    let mut there = Room::new(room.path(), "theirs");
    let refused = parcel::read(
        &there.data,
        &there.state,
        &there.dev.clone(),
        &ahead,
        &Along::default(),
    );
    assert!(refused.is_err());
    assert!(there.state.docs.is_empty());
    there.seq += 1;
}

#[test]
fn a_landing_that_never_finished_is_swept_by_the_next_one() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let box_at = filled(&mut here);
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    let stale = there.data.join(".landing-999999");
    std::fs::create_dir_all(stale.join("attachments")).unwrap();
    std::fs::write(stale.join("attachments").join("big.mp4"), b"left over").unwrap();

    there.take_in(&box_at);

    assert!(
        !stale.exists(),
        "the leftovers of an interrupted landing stayed"
    );
    assert!(
        std::fs::read_dir(&there.data)
            .unwrap()
            .filter_map(|one| one.ok())
            .all(|one| !one.file_name().to_string_lossy().starts_with(".landing-")),
        "a landing folder outlived the landing"
    );
}

#[test]
fn a_page_whose_document_never_arrived_is_counted_rather_than_hung_from_nothing() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let (book, _) = here.doc("# Libro\n\ntexto", None, None);
    here.doc("# Capitulo\n\nuno", None, Some(book));

    let box_at = room.path().join("solo-la-pagina.tistydoc");
    let page = here
        .state
        .docs
        .values()
        .find(|one| one.page_of.is_some())
        .unwrap()
        .file
        .clone();
    parcel::write(&here.data, &here.state, &[page], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    let landed = there.take_in(&box_at);

    assert_eq!((landed.docs, landed.pages), (1, 0));
    assert!(there.titled("Capitulo").page_of.is_none());
}

#[test]
fn a_deep_tree_with_long_names_still_lands_on_a_system_that_counts_its_path_characters() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let long = "carpeta-de-nombre-larguisimo-para-medir";
    let mut at = None;
    for _ in 0..4 {
        at = Some(here.folder(long, at, "home"));
    }
    let titled = "titulo tan largo como Tisty permite antes de cortarlo por lo sano y algo mas";
    let shed = here.data.join("attachments").join("ab");
    std::fs::create_dir_all(&shed).unwrap();
    let named = "un-nombre-de-adjunto-francamente-larguisimo-91f2ab00.png";
    std::fs::write(shed.join(named), b"a picture").unwrap();
    here.doc(
        &format!("# {titled}\n\n![una foto](<attachments/ab/{named}>)"),
        at,
        None,
    );

    let out = room
        .path()
        .join("una carpeta de salida con su propio nombre largo")
        .join("y otra dentro");
    let sent = parcel::plainly(&here.data, &here.state, &[], &out, &Along::default()).unwrap();

    assert_eq!(sent.docs, 1, "left behind: {:?}", sent.left);
    assert_eq!(sent.files, 1);
    let deep = (0..4).fold(out, |at, _| at.join(long));
    let folder = std::fs::read_dir(&deep)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(
        folder.join("attachments").join("ab").join(named).is_file(),
        "{folder:?}"
    );
}

#[test]
fn what_somebody_else_wrote_keeps_their_name_on_it_after_it_lands() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("fulanito".into()),
            ..Default::default()
        },
    });
    here.doc("# Acta\n\nlo que escribi", None, None);

    let box_at = room.path().join("firmado.tistydoc");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    there.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    there.take_in(&box_at);

    let acta = there.titled("Acta");
    assert_eq!(there.state.author_of(acta), Some("fulanito"));
    assert_eq!(
        there.state.editor_of(acta),
        None,
        "nobody has touched it yet"
    );
    assert_eq!(
        acta.made,
        here.titled("Acta").made,
        "the day it was written changed"
    );
}

#[test]
fn signing_the_store_signs_what_comes_next_and_leaves_what_was_already_written() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc(
        "# Antes

sin firmar",
        None,
        None,
    );
    let before = here.titled("Antes").id;

    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("  rgdevment  ".into()),
            name: Some("Mario".into()),
            ..Default::default()
        },
    });
    here.doc(
        "# Despues

ya firmado",
        None,
        None,
    );

    assert_eq!(here.state.author_of(&here.state.docs[&before]), None);
    assert_eq!(
        here.state.author_of(here.titled("Despues")),
        Some("rgdevment")
    );
    assert_eq!(here.state.signed.name.as_deref(), Some("Mario"));

    assert_eq!(here.state.mine_to_sign(), [before]);
    here.tell(Op::DocSigned {
        id: before,
        d: "rgdevment".into(),
    });
    assert_eq!(
        here.state.author_of(&here.state.docs[&before]),
        Some("rgdevment")
    );
    assert!(here.state.mine_to_sign().is_empty());
}

#[test]
fn signing_again_offers_only_what_this_store_signed_before_and_never_a_guest() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("fulanito".into()),
            ..Default::default()
        },
    });
    here.doc(
        "# Suyo

lo suyo",
        None,
        None,
    );
    let box_at = room.path().join("suyo.tistydoc");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    there.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("mario".into()),
            ..Default::default()
        },
    });
    there.doc(
        "# Mio

lo mio",
        None,
        None,
    );
    there.take_in(&box_at);
    there.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });

    let mine = there.titled("Mio").id;
    assert_eq!(
        there.state.mine_to_sign(),
        [mine],
        "a document somebody else signed was offered up for re-signing"
    );
    assert_eq!(
        there.state.author_of(there.titled("Suyo")),
        Some("fulanito")
    );
}

#[test]
fn taking_in_and_then_writing_says_who_wrote_last_without_taking_the_name_away() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("fulanito".into()),
            ..Default::default()
        },
    });
    here.doc("# Acta\n\nlo suyo", None, None);
    let box_at = room.path().join("firmado.tistydoc");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    there.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    there.take_in(&box_at);

    let acta = there.titled("Acta").id;
    there.tell(Op::DocSaid {
        id: acta,
        d: Said {
            title: "Acta".into(),
            bytes: Some(20),
            tags: Some(Vec::new()),
        },
    });

    let acta = &there.state.docs[&acta];
    assert_eq!(there.state.author_of(acta), Some("fulanito"));
    assert_eq!(there.state.editor_of(acta), Some("rgdevment"));
}

#[test]
fn the_aliases_this_store_signed_with_are_kept_apart_from_the_ones_that_arrived() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("fulanito".into()),
            ..Default::default()
        },
    });
    here.doc("# Suyo\n\nlo que escribio", None, None);
    let box_at = room.path().join("suyo.tistydoc");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    for alias in ["rgdevment", "mario", "rgdevment"] {
        there.tell(Op::Signed {
            d: tisty_core::event::Signature {
                alias: Some(alias.into()),
                ..Default::default()
            },
        });
    }
    there.take_in(&box_at);

    assert_eq!(there.state.signed_before, ["mario", "rgdevment"]);
    assert_eq!(there.state.signed.alias.as_deref(), Some("rgdevment"));
    assert_eq!(
        there.state.author_of(there.titled("Suyo")),
        Some("fulanito")
    );
}

#[test]
fn signing_with_the_same_name_as_a_guest_never_makes_their_writing_yours() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("fulanito".into()),
            ..Default::default()
        },
    });
    here.doc("# Suyo\n\nlo suyo", None, None);
    let box_at = room.path().join("suyo.tistydoc");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    there.take_in(&box_at);
    for alias in ["fulanito", "rgdevment"] {
        there.tell(Op::Signed {
            d: tisty_core::event::Signature {
                alias: Some(alias.into()),
                ..Default::default()
            },
        });
    }

    assert!(
        there.state.mine_to_sign().is_empty(),
        "taking somebody's alias handed you their writing"
    );
    for id in there.state.mine_to_sign() {
        there.tell(Op::DocSigned {
            id,
            d: "rgdevment".into(),
        });
    }
    assert_eq!(
        there.state.author_of(there.titled("Suyo")),
        Some("fulanito")
    );
}

#[test]
fn a_document_remembers_the_name_it_was_born_under_after_it_is_signed_again() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("mario".into()),
            ..Default::default()
        },
    });
    here.doc("# Acta\n\nlo mio", None, None);
    let acta = here.titled("Acta").id;

    assert_eq!(here.state.born_of(&here.state.docs[&acta]), None);

    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    here.tell(Op::DocSigned {
        id: acta,
        d: "rgdevment".into(),
    });

    let acta = &here.state.docs[&acta];
    assert_eq!(here.state.author_of(acta), Some("rgdevment"));
    assert_eq!(here.state.born_of(acta), Some("mario"));
}

#[test]
fn coming_home_under_the_same_name_leaves_no_mark_however_it_was_typed() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    here.doc("# Acta\n\nlo mio", None, None);
    let box_at = room.path().join("mio.tistydoc");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();
    here.take_in(&box_at);

    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("RGDEVMENT".into()),
            ..Default::default()
        },
    });

    assert!(
        here.state.mine_to_sign().is_empty(),
        "the same name in another case asked to be signed again"
    );
    assert_eq!(
        here.state.signed_before,
        ["RGDEVMENT"],
        "the history kept the same name twice"
    );
    for one in here.state.docs.values() {
        assert_eq!(
            here.state.born_of(one),
            None,
            "it claimed a change that never happened"
        );
    }
}

#[test]
fn whoever_signs_first_owns_what_nobody_had_signed() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc("# Huerfano\n\nsin dueño", None, None);
    let lone = here.titled("Huerfano").id;

    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    here.tell(Op::DocSigned {
        id: lone,
        d: "rgdevment".into(),
    });

    let lone = &here.state.docs[&lone];
    assert_eq!(here.state.author_of(lone), Some("rgdevment"));
    assert_eq!(
        lone.born_by.as_deref(),
        Some("rgdevment"),
        "nobody took it as their own"
    );
    assert_eq!(
        here.state.born_of(lone),
        None,
        "it says it changed hands when it never had any"
    );
}

#[test]
fn what_you_wrote_yourself_comes_home_as_yours_and_not_as_a_guest() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    here.doc("# Acta\n\nlo mio", None, None);
    let box_at = room.path().join("respaldo.tistydoc");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut fresh = Room::new(room.path(), "fresh");
    fresh.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("RgDevMent".into()),
            ..Default::default()
        },
    });
    fresh.take_in(&box_at);

    let acta = fresh.titled("Acta");
    assert!(!acta.guest, "your own writing came home as somebody else's");
    assert_eq!(fresh.state.author_of(acta), Some("rgdevment"));
    assert_eq!(fresh.state.editor_of(acta), None);
    assert!(
        fresh.state.mine_to_sign().is_empty(),
        "it asked to be signed with the name it already carries"
    );
}
