use std::path::{Path, PathBuf};

use tisty_core::event::{DocAdd, FolderAdd, Said};
use tisty_core::model::{DocId, FolderId};
use tisty_core::parcel::Along;
use tisty_core::{DeviceId, Event, Op, State, attach, docs, order, parcel};
use ulid::Ulid;

/// Opens the zip, hands the manifest over to be changed, and writes it back — which is all
/// anybody needs to forge one.
fn reworded(at: &std::path::Path, mut hand: impl FnMut(&mut serde_json::Value)) {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(at).unwrap()).unwrap();
    let mut kept: Vec<(String, Vec<u8>)> = Vec::new();
    for n in 0..zip.len() {
        let mut one = zip.by_index(n).unwrap();
        let named = one.name().to_string();
        let mut body = Vec::new();
        std::io::Read::read_to_end(&mut one, &mut body).unwrap();
        kept.push((named, body));
    }

    let out = std::fs::File::create(at).unwrap();
    let mut writing = zip::ZipWriter::new(out);
    let how = zip::write::SimpleFileOptions::default();
    for (named, body) in kept {
        writing.start_file(&named, how).unwrap();
        let body = match named == "tisty-docs.json" {
            true => {
                let mut manifest: serde_json::Value = serde_json::from_slice(&body).unwrap();
                hand(&mut manifest);
                serde_json::to_vec(&manifest).unwrap()
            }
            false => body,
        };
        std::io::Write::write_all(&mut writing, &body).unwrap();
    }
    writing.finish().unwrap();
}

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
                wrote: None,
                guest: false,
                made: None,
                by: None,
                file: made.id.clone(),
                order,
                said: Some(Said {
                    title: made.title.clone(),
                    bytes: None,
                    tags: Some(Vec::new()),
                    by: None,
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

    room.data.parent().unwrap().join("todo.tistyx")
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

    let box_at = room.path().join("una.tistyx");
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

    let box_at = room.path().join("con-video.tistyx");
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
            &here.data.join("una.tistyx"),
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

    let stray = room.path().join("cualquiera.tistyx");
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
    let box_at = room.path().join("nueva.tistyx");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let said = std::fs::read(&box_at).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(said)).unwrap();
    let ahead = room.path().join("ahead.tistyx");
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

    let box_at = room.path().join("solo-la-pagina.tistyx");
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

    let box_at = room.path().join("firmado.tistyx");
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
    let box_at = room.path().join("suyo.tistyx");
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
    let box_at = room.path().join("firmado.tistyx");
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
    assert_eq!(
        there.state.editor_of(&there.state.docs[&acta]),
        None,
        "nobody has written it here yet"
    );
    there.tell(Op::DocSaid {
        id: acta,
        d: Said {
            title: "Acta".into(),
            bytes: Some(20),
            tags: Some(Vec::new()),
            by: Some("rgdevment".into()),
        },
    });

    let acta = &there.state.docs[&acta];
    assert_eq!(there.state.author_of(acta), Some("fulanito"));
    assert_eq!(there.state.editor_of(acta), Some("rgdevment"));
}

#[test]
fn a_parcel_locked_with_a_number_lands_as_my_own_writing() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    here.doc(
        "# Acta

lo mio",
        None,
        None,
    );
    let box_at = room.path().join("mudanza.tistyx");
    parcel::written(
        &here.data,
        &here.state,
        &[],
        &box_at,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();

    assert!(parcel::locked(&box_at), "it went out in the clear");

    let mut fresh = Room::new(room.path(), "fresh");
    assert!(
        matches!(
            parcel::read(
                &fresh.data,
                &fresh.state,
                &fresh.dev.clone(),
                &box_at,
                &Along::default()
            ),
            Err(tisty_core::Error::ParcelLocked)
        ),
        "a locked parcel opened without the number"
    );
    assert!(
        matches!(
            parcel::taken(
                &fresh.data,
                &fresh.state,
                &fresh.dev.clone(),
                &box_at,
                &Along::default(),
                Some("000000")
            ),
            Err(tisty_core::Error::WrongNumber)
        ),
        "any number at all opened it"
    );

    let (landed, ops) = parcel::taken(
        &fresh.data,
        &fresh.state,
        &fresh.dev.clone(),
        &box_at,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();
    assert_eq!(landed.docs, 1);
    for op in ops {
        fresh.tell(op);
    }

    let acta = fresh.titled("Acta");
    assert!(!acta.guest, "my own writing came home as somebody else's");
    assert_eq!(fresh.state.author_of(acta), Some("rgdevment"));
}

#[test]
fn a_locked_parcel_cut_short_does_not_open_as_a_whole_one() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    // Long enough to be sealed in more than one block once zipped, so it has to be a body that
    // does not compress away: cutting it then leaves whole blocks behind.
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut body = String::from(
        "# Largo

",
    );
    for _ in 0..200_000 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        body.push(char::from(b'a' + ((seed >> 33) % 26) as u8));
    }
    here.doc(&body, None, None);
    let box_at = room.path().join("largo.tistyx");
    parcel::written(
        &here.data,
        &here.state,
        &[],
        &box_at,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();

    let whole = std::fs::read(&box_at).unwrap();
    assert!(whole.len() > 64 * 1024, "the parcel fit in a single block");

    let fresh = Room::new(room.path(), "fresh");
    let (landed, _) = parcel::taken(
        &fresh.data,
        &fresh.state,
        &fresh.dev.clone(),
        &box_at,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();
    assert_eq!(landed.docs, 1, "a parcel of several blocks did not open");

    std::fs::write(&box_at, &whole[..whole.len() - 4_000]).unwrap();
    let cut = Room::new(room.path(), "cut");
    assert!(
        parcel::taken(
            &cut.data,
            &cut.state,
            &cut.dev.clone(),
            &box_at,
            &Along::default(),
            Some("123456")
        )
        .is_err(),
        "a parcel cut short opened as if it were whole"
    );
}

#[test]
fn a_lock_shuts_a_document_to_everything_and_the_archive_only_to_writing() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let (shelved, away) = here.doc(
        "# Guardado

terminado",
        None,
        None,
    );
    let (bolted, shut) = here.doc(
        "# Cerrado

guardado",
        None,
        None,
    );
    here.tell(Op::DocArchive { id: shelved });
    here.tell(Op::DocLock { id: bolted });

    // Writing into either is refused, so an archived document reads and does not change.
    assert!(here.state.written_shut(shelved));
    assert!(here.state.written_shut(bolted));

    // Settling what two machines already wrote is not writing into it: only the lock bars that,
    // or an archived document caught in a conflict would have no way out of it.
    assert!(!here.state.shut_tight(&away));
    assert!(here.state.shut_tight(&shut));
}

#[test]
fn a_folder_the_archive_holds_crosses_in_a_parcel_and_lands_closed() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let gone = here.folder("Linio", None, "work");
    let under = here.folder("BOB", Some(gone), "robot");
    here.doc(
        "# Antifraude

contratos",
        Some(gone),
        None,
    );
    let (by_hand, _) = here.doc(
        "# Aparte

guardado antes",
        Some(gone),
        None,
    );
    here.doc(
        "# Despliegue

lo de bob",
        Some(under),
        None,
    );
    here.tell(Op::DocArchive { id: by_hand });
    here.tell(Op::FolderArchive { id: gone });

    let box_at = room.path().join("linio.tistybox");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    there.take_in(&box_at);

    for name in ["Antifraude", "Aparte", "Despliegue"] {
        let landed = there.titled(name).id;
        assert!(
            there.state.stowed(landed),
            "{name} came out of the parcel into the open tree"
        );
    }

    let shelf = there.shelf("Linio").id;
    assert!(
        there.shelf("Linio").archived,
        "the folder itself did not land closed"
    );
    let held = there.shelf("BOB");
    assert!(
        !held.archived && there.state.folder_away(held.id),
        "the subfolder is away through the one above it, not by a mark of its own"
    );

    // Bringing it back has to give each document what it had, not what the folder gave it.
    there.tell(Op::FolderUnarchive { id: shelf });
    assert!(!there.state.stowed(there.titled("Antifraude").id));
    assert!(!there.state.stowed(there.titled("Despliegue").id));
    assert!(
        there.state.stowed(there.titled("Aparte").id),
        "the one archived by hand before the folder was shelved lost its own mark"
    );
}

#[test]
fn a_parcel_read_by_a_build_that_knows_nothing_of_shelved_folders_still_lands_closed() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let gone = here.folder("Linio", None, "work");
    here.doc(
        "# Antifraude

contratos",
        Some(gone),
        None,
    );
    here.tell(Op::FolderArchive { id: gone });

    let box_at = room.path().join("linio.tistybox");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    // What an older reader does with the two fields it has never heard of: drops them. Every
    // document still says it is in the archive on its own, which is the safe side to land on.
    reworded(&box_at, |manifest| {
        for shelf in manifest["folders"].as_array_mut().unwrap() {
            shelf.as_object_mut().unwrap().remove("archived");
        }
        for paper in manifest["docs"].as_array_mut().unwrap() {
            paper.as_object_mut().unwrap().remove("by_folder");
        }
    });

    let mut there = Room::new(room.path(), "theirs");
    there.take_in(&box_at);

    let landed = there.titled("Antifraude").id;
    assert!(
        there.state.stowed(landed),
        "an older build would have poured the archive into the open tree"
    );
}

#[test]
fn two_folders_that_only_differ_in_case_do_not_pour_into_one() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    for (named, title) in [("Casa", "Uno"), ("CASA", "Dos"), ("casa", "Tres")] {
        let shelf = Ulid::generate();
        here.tell(Op::FolderAdd {
            id: shelf,
            d: tisty_core::event::FolderAdd {
                name: named.into(),
                order: format!("a{title}"),
                parent: None,
                icon: None,
                color: None,
            },
        });
        here.doc(
            &format!(
                "# {title}

lo suyo"
            ),
            Some(shelf),
            None,
        );
    }

    let out = room.path().join("plano");
    let sent = parcel::plainly(&here.data, &here.state, &[], &out, &Along::default()).unwrap();
    assert_eq!(sent.folders, 3);

    // Windows and macOS hand back one directory for «Casa» and «CASA», so three folders that
    // spell the same must reach disk under three names of their own.
    let mut stood: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .filter_map(|one| one.ok())
        .filter(|one| one.path().is_dir())
        .map(|one| one.file_name().to_string_lossy().to_string())
        .collect();
    stood.sort();
    assert_eq!(stood.len(), 3, "folders poured into one another: {stood:?}");
    for at in &stood {
        let held = std::fs::read_dir(out.join(at)).unwrap().count();
        assert_eq!(held, 1, "{at} holds what belongs to another folder");
    }
}

#[test]
fn a_parcel_that_wears_my_stores_name_without_its_seal_is_a_strangers() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let mine = tisty_core::store::identity(here.data.join("store")).unwrap();
    here.doc(
        "# Acta

lo mio",
        None,
        None,
    );
    let box_at = room.path().join("mio.tistyx");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    // Anybody who was ever handed a parcel of mine knows that name.
    let mut there = Room::new(room.path(), "theirs");
    there.doc(
        "# Suyo

lo suyo",
        None,
        None,
    );
    let forged = room.path().join("forjado.tistyx");
    parcel::write(&there.data, &there.state, &[], &forged, &Along::default()).unwrap();
    reworded(&forged, |manifest| {
        manifest["from"] = serde_json::Value::String(mine.clone());
    });

    here.take_in(&forged);
    assert!(
        here.titled("Suyo").guest,
        "a forged name was taken for writing born in this store"
    );

    // And what this store really wrote still comes home as its own.
    here.take_in(&box_at);
    for one in here
        .state
        .docs
        .values()
        .filter(|one| one.title.as_deref() == Some("Acta"))
    {
        assert!(!one.guest, "my own parcel stopped being mine");
    }
}

#[test]
fn a_parcel_cannot_ask_us_to_grind_whatever_it_likes() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc(
        "# Acta

lo mio",
        None,
        None,
    );
    let box_at = room.path().join("hostil.tistyx");
    parcel::written(
        &here.data,
        &here.state,
        &[],
        &box_at,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();

    // The ninth byte is what the file says its key cost to make.
    let mut whole = std::fs::read(&box_at).unwrap();
    whole[8] = 30;
    std::fs::write(&box_at, &whole).unwrap();

    let fresh = Room::new(room.path(), "fresh");
    assert!(
        matches!(
            parcel::taken(
                &fresh.data,
                &fresh.state,
                &fresh.dev.clone(),
                &box_at,
                &Along::default(),
                Some("123456")
            ),
            Err(tisty_core::Error::NotAParcel(_))
        ),
        "a file talked us into grinding a gigabyte on its say-so"
    );
}

#[test]
fn a_body_that_fills_its_blocks_exactly_comes_back_whole() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    // Sealed in blocks of 64 KiB, so a zip that lands on the boundary ends with a block of no
    // bytes at all — the one place the loop could drop the ending and not notice.
    for n in 0..40 {
        let mut seed = 0x2545_f491_4f6c_dd1du64 ^ n;
        let mut body = format!(
            "# Uno {n}

"
        );
        for _ in 0..4_000 {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            body.push(char::from(b'a' + ((seed >> 33) % 26) as u8));
        }
        here.doc(&body, None, None);
    }
    let box_at = room.path().join("justo.tistyx");
    parcel::written(
        &here.data,
        &here.state,
        &[],
        &box_at,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();

    let fresh = Room::new(room.path(), "fresh");
    let (landed, _) = parcel::taken(
        &fresh.data,
        &fresh.state,
        &fresh.dev.clone(),
        &box_at,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();
    assert_eq!(landed.docs, 40);
    assert_eq!(landed.missed, 0);
}

#[test]
fn a_number_that_is_no_number_locks_nothing() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc(
        "# Acta

lo mio",
        None,
        None,
    );
    let box_at = room.path().join("vacio.tistyx");

    assert!(
        matches!(
            parcel::written(
                &here.data,
                &here.state,
                &[],
                &box_at,
                &Along::default(),
                Some("")
            ),
            Err(tisty_core::Error::WrongNumber)
        ),
        "a parcel locked with nothing at all says it is locked"
    );
    assert!(!box_at.exists(), "it left the parcel behind anyway");
}

#[test]
fn nothing_of_a_locked_parcel_is_left_in_the_clear_where_it_was_written() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc(
        "# Acta

lo mio",
        None,
        None,
    );
    let out = room.path().join("salida");
    std::fs::create_dir_all(&out).unwrap();

    parcel::written(
        &here.data,
        &here.state,
        &[],
        &out.join("mudanza.tistyx"),
        &Along::default(),
        Some("123456"),
    )
    .unwrap();

    let left: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .filter_map(|one| one.ok())
        .map(|one| one.file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(
        left,
        ["mudanza.tistyx"],
        "something other than the locked parcel stayed at the destination"
    );
    for at in std::fs::read_dir(&here.data)
        .unwrap()
        .filter_map(|one| one.ok())
    {
        assert!(
            !at.file_name().to_string_lossy().starts_with(".packing-"),
            "the clear copy stayed inside the store"
        );
    }
}

#[test]
fn a_parcel_that_opens_and_then_comes_apart_is_not_a_wrong_number() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let mut seed = 0x9e37_79b9_7f4a_7c15u64;
    let mut body = String::from(
        "# Largo

",
    );
    for _ in 0..200_000 {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        body.push(char::from(b'a' + ((seed >> 33) % 26) as u8));
    }
    here.doc(&body, None, None);
    let box_at = room.path().join("largo.tistyx");
    parcel::written(
        &here.data,
        &here.state,
        &[],
        &box_at,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();

    let mut whole = std::fs::read(&box_at).unwrap();
    let end = whole.len() - 1;
    whole[end] ^= 0x40;
    std::fs::write(&box_at, &whole).unwrap();

    let fresh = Room::new(room.path(), "fresh");
    assert!(
        matches!(
            parcel::taken(
                &fresh.data,
                &fresh.state,
                &fresh.dev.clone(),
                &box_at,
                &Along::default(),
                Some("123456")
            ),
            Err(tisty_core::Error::ParcelTorn)
        ),
        "a parcel that came apart was blamed on the number"
    );
}

#[test]
fn a_guest_in_a_locked_parcel_is_still_a_guest_at_the_other_end() {
    let room = tmp();
    let mut theirs = Room::new(room.path(), "theirs");
    theirs.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("fulanito".into()),
            ..Default::default()
        },
    });
    theirs.doc(
        "# Suyo

lo suyo",
        None,
        None,
    );
    let handed = room.path().join("suyo.tistyx");
    parcel::write(&theirs.data, &theirs.state, &[], &handed, &Along::default()).unwrap();

    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    here.take_in(&handed);
    here.doc(
        "# Mio

lo mio",
        None,
        None,
    );

    let moving = room.path().join("mudanza.tistyx");
    parcel::written(
        &here.data,
        &here.state,
        &[],
        &moving,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();

    let mut fresh = Room::new(room.path(), "fresh");
    let (_, ops) = parcel::taken(
        &fresh.data,
        &fresh.state,
        &fresh.dev.clone(),
        &moving,
        &Along::default(),
        Some("123456"),
    )
    .unwrap();
    for op in ops {
        fresh.tell(op);
    }

    assert!(
        !fresh.titled("Mio").guest,
        "what I wrote did not come home as mine"
    );
    assert!(
        fresh.titled("Suyo").guest,
        "somebody else's writing turned into mine by riding along"
    );
    assert_eq!(
        fresh.state.author_of(fresh.titled("Suyo")),
        Some("fulanito")
    );
}

#[test]
fn a_guest_stays_a_guest_after_a_trip_out_and_back_through_my_own_store() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("fulanito".into()),
            ..Default::default()
        },
    });
    here.doc(
        "# Acta

lo suyo",
        None,
        None,
    );
    let theirs = room.path().join("suyo.tistyx");
    parcel::write(&here.data, &here.state, &[], &theirs, &Along::default()).unwrap();

    let mut mine = Room::new(room.path(), "theirs");
    std::fs::create_dir_all(mine.data.join("store")).unwrap();
    std::fs::write(
        mine.data.join("store").join(tisty_core::store::MARKER),
        "store-of-mine",
    )
    .unwrap();
    mine.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    mine.take_in(&theirs);
    assert!(mine.titled("Acta").guest, "it arrived as my own writing");

    let round = room.path().join("vuelta.tistyx");
    parcel::write(&mine.data, &mine.state, &[], &round, &Along::default()).unwrap();
    mine.take_in(&round);

    for one in mine.state.docs.values() {
        assert!(
            one.guest,
            "a trip out of my own store and back made somebody else's writing mine"
        );
    }
    assert!(
        mine.state.mine_to_sign().is_empty(),
        "it offered to sign writing that is not mine"
    );
}

#[test]
fn what_was_never_signed_leaves_unsigned_however_i_sign_today() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc(
        "# Antes

sin firmar",
        None,
        None,
    );
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });

    let box_at = room.path().join("sin-firma.tistyx");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    there.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("fulanito".into()),
            ..Default::default()
        },
    });
    there.take_in(&box_at);

    assert_eq!(
        there.state.author_of(there.titled("Antes")),
        None,
        "what its writer chose to leave unsigned went out under their alias"
    );
}

#[test]
fn a_hand_of_mine_reads_as_the_alias_i_sign_with_now() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("mario".into()),
            ..Default::default()
        },
    });
    here.doc(
        "# Acta

lo suyo",
        None,
        None,
    );
    let acta = here.titled("Acta").id;
    here.tell(Op::DocSaid {
        id: acta,
        d: Said {
            title: "Acta".into(),
            bytes: Some(20),
            tags: Some(Vec::new()),
            by: Some("mario".into()),
        },
    });
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });

    let kept = &here.state.docs[&acta];
    assert_eq!(
        here.state.editor_of(kept),
        None,
        "the author signs it as well, so there is nothing to add"
    );
    assert_eq!(
        here.state.docs[&acta].edited_by.as_deref(),
        Some("mario"),
        "the log keeps the hand it was written with"
    );
}

#[test]
fn somebody_elses_hand_stays_theirs_however_i_sign() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("rgdevment".into()),
            ..Default::default()
        },
    });
    here.doc(
        "# Acta

lo mio",
        None,
        None,
    );
    let acta = here.titled("Acta").id;
    here.tell(Op::DocSaid {
        id: acta,
        d: Said {
            title: "Acta".into(),
            bytes: Some(20),
            tags: Some(Vec::new()),
            by: Some("fulanito".into()),
        },
    });

    assert_eq!(
        here.state.editor_of(&here.state.docs[&acta]),
        Some("fulanito")
    );
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
    let box_at = room.path().join("suyo.tistyx");
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

    assert_eq!(
        there.state.signed_before,
        ["rgdevment", "mario", "rgdevment"],
        "going back to a name it had signed with before was not written down"
    );
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
    let box_at = room.path().join("suyo.tistyx");
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
    let box_at = room.path().join("mio.tistyx");
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
        ["rgdevment"],
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
    let box_at = room.path().join("respaldo.tistyx");
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
    assert_eq!(fresh.state.author_of(acta), Some("rgdevment"));
    assert_eq!(fresh.state.editor_of(acta), None);
    assert!(
        fresh.state.mine_to_sign().is_empty(),
        "it asked to be signed with the name it already carries"
    );
}

#[test]
fn a_parcel_from_a_store_that_never_signed_is_not_yours_to_claim() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc("# Acta\n\nlo escribio alguien sin alias", None, None);
    let box_at = room.path().join("sin-firma.tistyx");
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
    assert_eq!(
        there.state.author_of(&there.state.docs[&acta]),
        None,
        "an unsigned document was credited to whoever took it in"
    );
    assert_eq!(
        there.state.mine_to_sign(),
        [acta],
        "writing nobody ever signed was not there to be claimed"
    );

    there.tell(Op::DocSigned {
        id: acta,
        d: "rgdevment".into(),
    });
    assert_eq!(
        there.state.author_of(&there.state.docs[&acta]),
        Some("rgdevment"),
        "the first to sign it did not become its author"
    );
    assert_eq!(
        there.state.born_of(&there.state.docs[&acta]),
        None,
        "it claimed a name it never had before"
    );
    assert!(
        !there.state.docs[&acta].guest,
        "claiming it left it flagged as somebody else's for ever"
    );
}

#[test]
fn a_parcel_that_carries_nothing_leaves_the_one_already_there_alone() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let (_, gone) = here.doc(
        "# Solo

se va",
        None,
        None,
    );
    std::fs::remove_file(here.data.join("docs").join(format!("{gone}.md"))).unwrap();

    let box_at = room.path().join("copia.tistyx");
    std::fs::write(&box_at, b"lo de la semana pasada").unwrap();

    assert!(
        parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).is_err(),
        "it found something to carry where there was nothing"
    );
    assert_eq!(
        std::fs::read(&box_at).unwrap(),
        b"lo de la semana pasada",
        "a parcel that carried nothing took the last one down with it"
    );
}

#[test]
fn no_event_can_sign_over_what_somebody_else_wrote() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.tell(Op::Signed {
        d: tisty_core::event::Signature {
            alias: Some("fulanito".into()),
            ..Default::default()
        },
    });
    here.doc(
        "# Acta

lo suyo",
        None,
        None,
    );
    let box_at = room.path().join("suyo.tistyx");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    let mut there = Room::new(room.path(), "theirs");
    there.take_in(&box_at);
    let acta = there.titled("Acta").id;

    there.tell(Op::DocSigned {
        id: acta,
        d: "rgdevment".into(),
    });

    assert_eq!(
        there.state.author_of(&there.state.docs[&acta]),
        Some("fulanito"),
        "an event signed over somebody else's writing"
    );
}

#[test]
fn a_body_that_cannot_be_read_is_counted_rather_than_dropped_from_the_parcel() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    here.doc("# Uno\n\nvivo", None, None);
    let (_, gone) = here.doc("# Dos\n\nse va", None, None);
    std::fs::remove_file(here.data.join("docs").join(format!("{gone}.md"))).unwrap();

    let box_at = room.path().join("corto.tistyx");
    let sent = parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();

    assert_eq!(sent.docs, 1);
    assert_eq!(sent.missed, 1, "a document left the parcel in silence");
}

#[test]
fn two_folders_that_spell_the_same_do_not_pour_into_one() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let one = here.folder("Casa", None, "home");
    let other = here.folder("Casa?", None, "home");
    here.doc("# Primero\n\nen la una", Some(one), None);
    here.doc("# Segundo\n\nen la otra", Some(other), None);

    let out = room.path().join("plano");
    parcel::plainly(&here.data, &here.state, &[], &out, &Along::default()).unwrap();

    let made: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .filter_map(|one| one.ok())
        .map(|one| one.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        made.len(),
        2,
        "both folders wrote into the same place: {made:?}"
    );
}

#[test]
fn a_name_that_is_a_prefix_of_another_is_not_rewritten_in_the_middle() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    // The store hands out fixed-width names, so a prefix pair has to be written by hand.
    for (file, body) in [
        (
            "mine-0001",
            "# Corto

soy el corto",
        ),
        (
            "mine-00011",
            "# Largo

soy el largo",
        ),
    ] {
        std::fs::write(
            here.data.join("docs").join(format!("{file}.md")),
            docs::settled(body),
        )
        .unwrap();
        let id = Ulid::generate();
        here.tell(Op::DocAdd {
            id,
            d: DocAdd {
                file: file.into(),
                order: order::last_of(here.state.docs.values().map(|one| one.order.as_str())),
                said: Some(Said {
                    title: docs::titled(body),
                    bytes: None,
                    tags: Some(Vec::new()),
                    by: None,
                }),
                ..Default::default()
            },
        });
    }
    here.doc(
        "# Libro

[a](tisty:doc/mine-0001) y [b](tisty:doc/mine-00011)",
        None,
        None,
    );

    let box_at = room.path().join("prefijos.tistyx");
    parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();
    let mut there = Room::new(room.path(), "theirs");
    there.take_in(&box_at);

    let said = there.body(&there.titled("Libro").file);
    let corto = there.titled("Corto").file.clone();
    let largo = there.titled("Largo").file.clone();
    assert!(said.contains(&format!("tisty:doc/{corto})")), "{said}");
    assert!(
        !said.contains("mine-0001)"),
        "the old name survived: {said}"
    );
    assert!(said.contains(&format!("tisty:doc/{largo})")), "{said}");
}

#[test]
fn a_landing_in_flight_survives_a_sweep_from_the_same_process() {
    let room = tmp();
    let data = room.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let mine = data.join(format!(".landing-{}", std::process::id()));
    let stale = data.join(".landing-999999");
    std::fs::create_dir_all(&mine).unwrap();
    std::fs::create_dir_all(&stale).unwrap();

    parcel::swept(&data);

    assert!(mine.is_dir(), "a landing still in flight was swept away");
    assert!(!stale.exists(), "the leftovers of another one stayed");
}

#[test]
fn a_wide_tree_keeps_every_folder_it_was_kept_in() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    for n in 0..20 {
        let up = here.folder(&format!("Raiz {n}"), None, "home");
        let down = here.folder(&format!("Rama {n}"), Some(up), "home");
        here.doc(&format!("# Doc {n}\n\ntexto"), Some(down), None);
    }

    let out = room.path().join("plano");
    let sent = parcel::plainly(&here.data, &here.state, &[], &out, &Along::default()).unwrap();

    assert_eq!(sent.docs, 20);
    assert_eq!(sent.folders, 40, "some folders never made it into the tree");
    for n in 0..20 {
        let at = out.join(format!("Raiz-{n}")).join(format!("Rama-{n}"));
        assert!(at.is_dir(), "{at:?} was flattened into the root");
    }
}

#[test]
fn an_attachment_named_with_an_anchor_still_lands() {
    let room = tmp();
    let mut here = Room::new(room.path(), "mine");
    let shed = here.data.join("attachments").join("ab");
    std::fs::create_dir_all(&shed).unwrap();
    std::fs::write(shed.join("foto-91f2ab00.png"), b"a picture").unwrap();
    here.doc(
        "# Con ancla\n\n![x](<attachments/ab/foto-91f2ab00.png#arriba>)",
        None,
        None,
    );

    let box_at = room.path().join("ancla.tistyx");
    let sent = parcel::write(&here.data, &here.state, &[], &box_at, &Along::default()).unwrap();
    assert_eq!(sent.files, 1);

    let mut there = Room::new(room.path(), "theirs");
    let landed = there.take_in(&box_at);
    assert_eq!(landed.files, 1, "the picture was left behind: {landed:?}");
    assert_eq!(landed.missed, 0);
}
