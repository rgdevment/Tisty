use std::io::Write;

use super::*;

fn tmp() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}
use crate::event::{DeviceId, TaskAdd};
use crate::{Op, Store};
use ulid::Ulid;

fn quarters(dir: &tempfile::TempDir) -> Paths {
    Paths::new(dir.path().join("data"), dir.path().join("config"))
}

fn filled(named: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let mut store = Store::open(data.join("store"), DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new(named, "a0"),
        })
        .unwrap();

    let shelf = data.join("attachments").join("ab");
    std::fs::create_dir_all(&shelf).unwrap();
    std::fs::write(shelf.join("foto-a1b2c3d4.png"), b"a picture").unwrap();

    let papers = data.join("docs");
    std::fs::create_dir_all(&papers).unwrap();
    std::fs::write(papers.join("a3f1-0001.md"), b"# Minuta\n\nlo que dije").unwrap();

    let before = data.join("originals");
    std::fs::create_dir_all(&before).unwrap();
    std::fs::write(before.join("a3f1-0001.md"), b"---\nx: 1\n---\n\n# Minuta").unwrap();
    (dir, data)
}

#[test]
fn taking_a_folder_over_leaves_it_empty_and_the_backup_holding_what_it_had() {
    let (_src, folder) = filled("lo que guardaba la carpeta");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("carpeta.zip");

    let made = take_over(&folder, "01KEPT00000000000000000000", &file, tmp().path()).unwrap();

    assert!(file.exists());
    assert!(made.files >= 2, "{made:?}");
    for folder_named in CARRIED {
        if folder_named == "store" {
            continue;
        }
        assert!(
            !folder.join(folder_named).exists(),
            "{folder_named} sigue ahi"
        );
    }
    assert!(
        crate::store::segments_in(&folder.join("store"))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_folder_is_never_emptied_when_the_backup_could_not_be_written() {
    let (_src, folder) = filled("lo que no se puede perder");
    let out = tempfile::tempdir().unwrap();
    let taken = out.path().join("carpeta.zip");
    std::fs::create_dir_all(&taken).unwrap();

    let outcome = take_over(&folder, "01KEPT00000000000000000000", &taken, tmp().path());

    assert!(outcome.is_err(), "dijo que si con el destino ocupado");
    assert!(
        folder.join("store").exists(),
        "vacio la carpeta sin respaldo"
    );
    assert!(folder.join("docs").exists());
    assert!(folder.join("attachments").exists());
}

#[test]
fn what_was_taken_over_reads_back_as_a_store_of_its_own() {
    let (_src, folder) = filled("una tarea que estaba alli");
    let was = crate::store::identity(folder.join("store")).unwrap();
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("carpeta.zip");

    let made = take_over(&folder, "01KEPT00000000000000000000", &file, tmp().path()).unwrap();

    assert_eq!(made.store_id, was);
}

#[test]
fn a_folder_taken_over_no_longer_claims_the_history_it_had() {
    let (_src, folder) = filled("algo");
    let out = tempfile::tempdir().unwrap();

    take_over(
        &folder,
        "01KEPT00000000000000000000",
        &out.path().join("carpeta.zip"),
        tmp().path(),
    )
    .unwrap();

    assert_eq!(
        crate::store::peek_identity(folder.join("store")).as_deref(),
        Some("01KEPT00000000000000000000"),
        "la carpeta quedo sin dueno y cualquiera la reclama"
    );
    assert!(!crate::store::inhabited(folder.join("store")));
}

#[test]
fn a_backup_carries_the_store_and_the_attachments() {
    let (_src, data) = filled("comprar pan");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");

    let made = write(&data, &file, tmp().path()).unwrap();
    assert!(made.files >= 2, "{made:?}");
    assert!(made.bytes > 0);
    assert!(file.exists());
}

#[test]
fn restoring_forgets_what_this_machine_had_carried_instead_of_pushing_it_back() {
    let (_src, data) = filled("comprar pan");
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::new(data.clone(), dir.path().join("config"));
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let mut said = crate::docs::Carried::default();
    said.keep(
        "a3f1-0001",
        "a print of a body that is about to be replaced",
    );
    said.save(&data).unwrap();

    read(&paths, &file).unwrap();

    assert_eq!(
        crate::docs::Carried::read(&data).of("a3f1-0001"),
        None,
        "it kept describing a body the restore threw away, which pushes it back out"
    );
}

#[test]
fn joining_forgets_what_this_machine_had_carried() {
    let (_src, data) = filled("comprar pan");
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::new(data.clone(), dir.path().join("config"));
    let mut said = crate::docs::Carried::default();
    said.keep("a3f1-0001", "a print from the store it is leaving");
    said.save(&data).unwrap();
    let out = tempfile::tempdir().unwrap();

    reset(&paths, &out.path().join("before.zip"), tmp().path()).unwrap();

    assert_eq!(crate::docs::Carried::read(&data).of("a3f1-0001"), None);
}

#[test]
fn a_backup_carries_the_documents_too_or_it_does_not_carry_your_work() {
    let (_src, data) = filled("comprar pan");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let fresh = tempfile::tempdir().unwrap();
    read(&quarters(&fresh), &file).unwrap();

    assert_eq!(
        std::fs::read_to_string(quarters(&fresh).data().join("docs/a3f1-0001.md")).unwrap(),
        "# Minuta\n\nlo que dije",
        "the documents did not travel"
    );
}

#[test]
fn a_reset_leaves_nothing_of_what_was_here() {
    let dir = tempfile::tempdir().unwrap();
    let paths = quarters(&dir);
    std::fs::create_dir_all(paths.data()).unwrap();
    let mut store = Store::open(paths.store(), DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("comprar pan", "a0"),
        })
        .unwrap();
    let was = store::identity(paths.store()).unwrap();
    std::fs::create_dir_all(paths.data().join("docs")).unwrap();
    std::fs::write(paths.data().join("docs/a3f1-0001.md"), b"# Minuta").unwrap();

    std::fs::create_dir_all(paths.private()).unwrap();
    std::fs::write(store::kept_at(&paths, &was).unwrap(), [6u8; 32]).unwrap();

    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("before-joining.zip");
    reset(&paths, &file, tmp().path()).unwrap();

    assert!(store::read_all(paths.store()).unwrap().is_empty());
    assert!(!paths.data().join("docs/a3f1-0001.md").exists());
    assert!(
        store::secret_kept(&paths).is_none(),
        "starting over kept the secret of the store it replaced"
    );
}

#[test]
fn a_key_left_inside_a_store_is_not_lost_when_the_store_is_replaced() {
    let (_src, data) = filled("comprar pan");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let mut paths = quarters(&dir);
    paths.unpaired_for_test();
    std::fs::create_dir_all(paths.store()).unwrap();
    std::fs::write(paths.store().join(store::KEEP), [6u8; 32]).unwrap();

    read(&paths, &file).unwrap();

    let kept: Vec<Vec<u8>> = store::displaced(&paths)
        .iter()
        .map(|one| std::fs::read(one).unwrap())
        .collect();
    assert_eq!(
        kept,
        vec![vec![6u8; 32]],
        "restoring wiped the store and the only copy of its key with it"
    );
}

#[test]
fn starting_over_keeps_what_the_zip_it_writes_cannot_carry() {
    let dir = tempfile::tempdir().unwrap();
    let paths = quarters(&dir);
    std::fs::create_dir_all(paths.data()).unwrap();
    Store::open(paths.store(), DeviceId("dev_a".into())).unwrap();
    let was = store::identity(paths.store()).unwrap();
    std::fs::create_dir_all(paths.private()).unwrap();
    std::fs::write(store::kept_at(&paths, &was).unwrap(), [6u8; 32]).unwrap();

    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("before-joining.zip");
    reset(&paths, &file, tmp().path()).unwrap();

    assert_ne!(
        store::identity(paths.store()).unwrap(),
        was,
        "starting over kept the name of the store it replaced"
    );
    assert_eq!(
        std::fs::read(store::kept_at(&paths, &was).unwrap()).unwrap(),
        [6u8; 32],
        "the zip cannot carry the key, so starting over has to leave it under its own name"
    );
    assert!(
        store::secret_kept(&paths).is_none(),
        "the store that starts over inherited the seal of the one it replaced"
    );
}

#[test]
fn a_machine_that_rejoins_comes_back_under_a_new_name() {
    let (_src, data) = filled("comprar pan");
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::new(data.clone(), dir.path().join("config"));
    let was = Config::load_or_init(&paths).unwrap().device_id;
    let out = tempfile::tempdir().unwrap();

    reset(&paths, &out.path().join("before.zip"), tmp().path()).unwrap();

    let now = Config::load_or_init(&paths).unwrap().device_id;
    assert_ne!(now, was, "it came back carrying its own tombstone");
}

#[test]
fn a_reset_cannot_happen_without_the_backup_landing_first() {
    let dir = tempfile::tempdir().unwrap();
    let paths = quarters(&dir);
    std::fs::create_dir_all(paths.data()).unwrap();
    let mut store = Store::open(paths.store(), DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("comprar pan", "a0"),
        })
        .unwrap();

    let nowhere = dir.path().join("no/such/place/before-joining.zip");
    let why = reset(&paths, &nowhere, tmp().path());

    assert!(why.is_err(), "it reset with nowhere to put the backup");
    assert_eq!(
        store::read_all(paths.store()).unwrap().len(),
        1,
        "it threw away what it could not back up"
    );
}

#[test]
fn what_a_reset_backed_up_can_be_restored_afterwards() {
    let (_src, data) = filled("comprar pan");
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::new(data.clone(), dir.path().join("config"));
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("before-joining.zip");

    reset(&paths, &file, tmp().path()).unwrap();
    let fresh = tempfile::tempdir().unwrap();
    read(&quarters(&fresh), &file).unwrap();

    assert_eq!(
        std::fs::read_to_string(quarters(&fresh).data().join("docs/a3f1-0001.md")).unwrap(),
        "# Minuta\n\nlo que dije",
        "the reset backup did not hold the documents"
    );
}

#[test]
fn a_backup_carries_what_a_document_looked_like_before_it_was_converted() {
    let (_src, data) = filled("comprar pan");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let fresh = tempfile::tempdir().unwrap();
    read(&quarters(&fresh), &file).unwrap();

    assert_eq!(
        std::fs::read_to_string(quarters(&fresh).data().join("originals/a3f1-0001.md")).unwrap(),
        "---\nx: 1\n---\n\n# Minuta",
        "the only copy of what was lost did not travel"
    );
}

#[test]
fn a_destination_that_cannot_be_written_leaves_it_alone() {
    let (_src, data) = filled("comprar pan");
    let out = tempfile::tempdir().unwrap();

    let taken = out.path().join("tisty.zip");
    std::fs::create_dir(&taken).unwrap();
    std::fs::write(taken.join("inside"), b"still here").unwrap();

    assert!(write(&data, &taken, tmp().path()).is_err());
    assert!(taken.join("inside").exists());
}

#[test]
fn nothing_is_left_beside_the_backup() {
    let (_src, data) = filled("comprar pan");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");

    write(&data, &file, tmp().path()).unwrap();

    let left: Vec<String> = std::fs::read_dir(out.path())
        .unwrap()
        .filter_map(|one| one.ok())
        .map(|one| one.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left, vec!["tisty.zip".to_string()], "{left:?}");
}

#[test]
fn what_comes_back_is_what_went_in() {
    let (_src, data) = filled("comprar pan");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let fresh = tempfile::tempdir().unwrap();
    let restored = read(&quarters(&fresh), &file).unwrap();

    assert!(restored.files >= 2);
    assert_eq!(restored.devices, 1);
    let events = store::read_all(quarters(&fresh).store()).unwrap();
    assert_eq!(events.len(), 1);
    assert!(
        quarters(&fresh)
            .data()
            .join("attachments/ab/foto-a1b2c3d4.png")
            .exists()
    );
}

#[test]
fn what_proves_the_store_is_its_own_never_enters_a_copy() {
    let (_src, data) = filled("lo de siempre");
    std::fs::write(data.join("store").join(store::KEEP), [9u8; 32]).unwrap();

    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let held = std::fs::read(&file).unwrap();
    let mut zip = zip::ZipArchive::new(std::fs::File::open(&file).unwrap()).unwrap();
    let named: Vec<String> = (0..zip.len())
        .map(|n| zip.by_index(n).unwrap().name().to_string())
        .collect();
    assert!(
        !named.iter().any(|one| one.contains(store::KEEP)),
        "the key went out under its own name: {named:?}"
    );
    assert!(
        !held.windows(32).any(|run| run == [9u8; 32]),
        "the key's bytes went out in the copy under some other name"
    );
}

#[test]
fn a_copy_carries_everything_a_working_store_holds() {
    let (_src, data) = filled("lo de siempre");
    std::fs::write(data.join("store").join(store::KEEP), [9u8; 32]).unwrap();
    std::fs::write(data.join("docs/.spent-dev_a"), b"7").unwrap();
    std::fs::write(
        data.join("store/dev_a/active (conflicted copy).tisty"),
        b"{}
",
    )
    .unwrap();
    std::fs::write(data.join("store/dev_a/.lock"), b"").unwrap();

    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let mut zip = zip::ZipArchive::new(std::fs::File::open(&file).unwrap()).unwrap();
    let inside: Vec<String> = (0..zip.len())
        .map(|n| zip.by_index(n).unwrap().name().to_string())
        .collect();

    for one in walk(&data) {
        let rest = one.strip_prefix(&data).unwrap();
        let named_as = rest.to_string_lossy().replace('\\', "/");
        if kept_out(&named(rest).unwrap()) {
            assert!(
                !inside.contains(&named_as),
                "{named_as} should never travel"
            );
            continue;
        }
        assert!(
            inside.contains(&named_as),
            "{named_as} was left out of the copy"
        );
        assert!(
            carried(rest),
            "{named_as} travels in a copy and could not come back from one"
        );
    }
}

#[test]
fn a_copy_that_carries_a_key_cannot_put_it_back() {
    assert_eq!(safe(&format!("store/{}", store::KEEP)), None);
    assert_eq!(safe(&format!("store/dev_a/{}", store::KEEP)), None);
}

#[test]
fn restoring_onto_an_empty_machine_keeps_the_old_devices_history() {
    let (_src, data) = filled("lo de antes");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let fresh = tempfile::tempdir().unwrap();
    read(&quarters(&fresh), &file).unwrap();

    let mut store = Store::open(quarters(&fresh).store(), DeviceId("dev_b".into())).unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("lo de ahora", "a1"),
        })
        .unwrap();

    let events = store::read_all(quarters(&fresh).store()).unwrap();
    assert_eq!(events.len(), 2, "the new machine writes beside the old one");
    assert!(quarters(&fresh).store().join("dev_a").is_dir());
    assert!(quarters(&fresh).store().join("dev_b").is_dir());
}

#[test]
fn restoring_goes_back_to_the_moment_and_loses_what_came_after() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::new(dir.path().join("data"), dir.path().join("config"));
    let was = Config::load_or_init(&paths).unwrap().device_id.0;

    let mut store = Store::open(paths.store(), DeviceId(was.clone())).unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("lo de antes", "a0"),
        })
        .unwrap();

    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(paths.data(), &file, tmp().path()).unwrap();

    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("lo de después", "a1"),
        })
        .unwrap();
    assert_eq!(store::read_all(paths.store()).unwrap().len(), 2);

    read(&paths, &file).unwrap();

    assert_eq!(
        store::read_all(paths.store()).unwrap().len(),
        1,
        "a photograph does not keep what happened after it"
    );
    let now = Config::load(&paths.config_file())
        .unwrap()
        .unwrap()
        .device_id
        .0;
    assert_ne!(now, was, "a restored machine writes under a new name");
}

#[test]
fn a_backup_of_another_store_is_refused_rather_than_merged() {
    let (_a, one) = filled("lo mío");
    let (_b, other) = filled("lo de otro");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&one, &file, tmp().path()).unwrap();

    let other_paths = Paths::new(&other, other.parent().unwrap().join("config"));
    let Err(Error::OtherStore { theirs }) = read(&other_paths, &file) else {
        panic!("merging two histories cannot be undone");
    };
    assert!(!theirs.is_empty());
}

#[test]
fn the_configuration_never_travels() {
    let (_src, data) = filled("comprar pan");
    std::fs::create_dir_all(data.join("config")).unwrap();
    std::fs::write(data.join("config/config.toml"), b"device_id = 'dev_a'").unwrap();

    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let held = std::fs::File::open(&file).unwrap();
    let mut zip = zip::ZipArchive::new(held).unwrap();
    for i in 0..zip.len() {
        let named = zip.by_index(i).unwrap().name().to_string();
        assert!(
            !named.contains("config"),
            "the device id travelled: {named}"
        );
    }
}

#[test]
fn a_zip_that_is_not_a_backup_leaves_everything_where_it_was() {
    let (_src, data) = filled("lo mio");
    let paths = Paths::new(&data, data.parent().unwrap().join("config"));
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("holiday-photos.zip");
    {
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&file).unwrap());
        zip.start_file("photos/beach.jpg", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"not a backup").unwrap();
        zip.finish().unwrap();
    }

    assert!(read(&paths, &file).is_err(), "it accepted a stranger's zip");
    assert_eq!(
        store::read_all(paths.store()).unwrap().len(),
        1,
        "the store was emptied by a zip full of photographs"
    );
    assert!(data.join("attachments/ab/foto-a1b2c3d4.png").exists());
}

#[test]
fn a_corrupt_backup_costs_nothing() {
    let (_src, data) = filled("lo mio");
    let paths = Paths::new(&data, data.parent().unwrap().join("config"));
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&data, &file, tmp().path()).unwrap();

    let mut bytes = std::fs::read(&file).unwrap();
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0xff;
    bytes[middle + 1] ^= 0xff;
    std::fs::write(&file, &bytes).unwrap();

    let _ = read(&paths, &file);
    assert_eq!(
        store::read_all(paths.store()).unwrap().len(),
        1,
        "a damaged backup took the store with it"
    );
}

#[test]
fn a_backup_with_no_marker_is_refused_onto_a_store_that_has_one() {
    let (_src, data) = filled("lo mio");
    let paths = Paths::new(&data, data.parent().unwrap().join("config"));
    let (_b, other) = filled("lo de otro");
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(&other, &file, tmp().path()).unwrap();

    let stripped = out.path().join("stripped.zip");
    {
        let held = std::fs::File::open(&file).unwrap();
        let mut from = zip::ZipArchive::new(held).unwrap();
        let mut to = zip::ZipWriter::new(std::fs::File::create(&stripped).unwrap());
        for i in 0..from.len() {
            let mut one = from.by_index(i).unwrap();
            let named = one.name().to_string();
            if named.ends_with(store::MARKER) {
                continue;
            }
            let mut body = Vec::new();
            one.read_to_end(&mut body).unwrap();
            to.start_file(named, zip::write::SimpleFileOptions::default())
                .unwrap();
            to.write_all(&body).unwrap();
        }
        to.finish().unwrap();
    }

    assert!(matches!(
        read(&paths, &stripped),
        Err(Error::OtherStore { .. })
    ));
    assert_eq!(store::read_all(paths.store()).unwrap().len(), 1);
}

#[test]
fn the_machine_is_renamed_before_anything_is_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let paths = Paths::new(dir.path().join("data"), dir.path().join("config"));
    let was = Config::load_or_init(&paths).unwrap().device_id.0;
    let mut store = Store::open(paths.store(), DeviceId(was.clone())).unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("lo de antes", "a0"),
        })
        .unwrap();

    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("tisty.zip");
    write(paths.data(), &file, tmp().path()).unwrap();
    read(&paths, &file).unwrap();

    let now = Config::load(&paths.config_file()).unwrap().unwrap();
    assert_ne!(now.device_id.0, was);
    assert!(
        now.synced_at.is_none(),
        "it claimed a sync that predates it"
    );
}

#[test]
fn a_zip_bomb_is_refused_and_costs_nothing() {
    let (_src, data) = filled("lo mio");
    let paths = Paths::new(&data, data.parent().unwrap().join("config"));
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("bomb.zip");
    {
        let mut zip = zip::ZipWriter::new(std::fs::File::create(&file).unwrap());
        zip.start_file(
            "store/dev_a/000001.tisty",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        let chunk = vec![b'0'; 1024 * 1024];
        for _ in 0..4 {
            zip.write_all(&chunk).unwrap();
        }
        zip.finish().unwrap();
    }
    assert!(
        std::fs::metadata(&file).unwrap().len() < 64 * 1024,
        "not a bomb"
    );

    assert!(
        within(&paths, &file, 1024 * 1024).is_err(),
        "it swallowed the bomb"
    );
    assert_eq!(
        store::read_all(paths.store()).unwrap().len(),
        1,
        "the store went with it"
    );
}

#[test]
fn a_swap_that_cannot_finish_puts_everything_back() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    let staged = dir.path().join("staged");
    let old = dir.path().join("old");
    for at in [&data, &staged, &old] {
        std::fs::create_dir_all(at).unwrap();
    }
    for root in [&data, &staged] {
        for folder in CARRIED {
            std::fs::create_dir_all(root.join(folder)).unwrap();
        }
    }
    std::fs::write(data.join("store/mine.txt"), b"what was here").unwrap();
    std::fs::write(staged.join("store/theirs.txt"), b"the photograph").unwrap();

    std::fs::create_dir_all(old.join("attachments/busy")).unwrap();
    std::fs::write(old.join("attachments/busy/x"), b"in the way").unwrap();

    assert!(swap(&data, &staged, &old).is_err(), "it swapped anyway");

    assert!(
        data.join("store/mine.txt").exists(),
        "the old store never came back"
    );
    assert!(
        !data.join("store/theirs.txt").exists(),
        "half the photograph stayed"
    );
    assert!(data.join("attachments").is_dir());
}

#[test]
fn what_could_not_be_put_back_is_kept_where_it_is_instead_of_destroyed() {
    let dest = tempfile::tempdir().unwrap();
    let stale = dest.path().join(".taking-over-333");
    std::fs::create_dir_all(stale.join("docs")).unwrap();
    std::fs::write(stale.join("docs").join("importante.md"), "no me borres").unwrap();
    std::fs::write(dest.path().join("docs"), "algo ocupa el sitio").unwrap();

    super::rescued(dest.path());

    assert_eq!(
        std::fs::read_to_string(stale.join("docs").join("importante.md")).unwrap(),
        "no me borres",
        "se destruyo lo que no pudo devolver"
    );
}

#[test]
fn two_take_overs_at_once_never_walk_over_each_other() {
    let dest = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dest.path().join("store")).unwrap();
    let held = super::only_one_taking_over(dest.path()).unwrap();

    let second = super::only_one_taking_over(dest.path());

    assert!(second.is_err(), "dos tomas de control a la vez");
    drop(held);
    assert!(super::only_one_taking_over(dest.path()).is_ok());
}

#[test]
fn a_take_over_cut_short_leaves_nothing_stranded_in_the_shared_folder() {
    let dest = tempfile::tempdir().unwrap();
    let stale = dest.path().join(".taking-over-999");
    std::fs::create_dir_all(stale.join("docs")).unwrap();
    std::fs::write(stale.join("docs").join("uno.md"), "un cuerpo").unwrap();
    std::fs::create_dir_all(dest.path().join("store")).unwrap();

    super::rescued(dest.path());

    assert!(!stale.exists(), "el resto quedo ahi para siempre");
    assert_eq!(
        std::fs::read_to_string(dest.path().join("docs").join("uno.md")).unwrap(),
        "un cuerpo",
        "los documentos no volvieron a su sitio"
    );
}

#[test]
fn what_a_dead_restore_left_behind_can_be_found() {
    let (_src, data) = filled("lo mio");
    std::fs::create_dir_all(data.join(".replaced-4242/store")).unwrap();
    std::fs::create_dir_all(data.join(".restoring-4242")).unwrap();

    let found = leftovers(&data);
    assert_eq!(found.len(), 2, "{found:?}");
}

#[test]
fn a_zip_full_of_refused_entries_does_not_roll_the_diary_away() {
    let _alone = crate::witness::ALONE
        .lock()
        .unwrap_or_else(|e| e.into_inner());

    let (_src, data) = filled("comprar pan");
    let out = tempfile::tempdir().unwrap();
    let good = out.path().join("tisty.zip");
    write(&data, &good, tmp().path()).unwrap();

    let hostile = out.path().join("hostile.zip");
    {
        let held = std::fs::File::open(&good).unwrap();
        let mut from = zip::ZipArchive::new(held).unwrap();
        let mut to = zip::ZipWriter::new(std::fs::File::create(&hostile).unwrap());
        for i in 0..from.len() {
            let mut one = from.by_index(i).unwrap();
            let named = one.name().to_string();
            let mut body = Vec::new();
            one.read_to_end(&mut body).unwrap();
            to.start_file(named, zip::write::SimpleFileOptions::default())
                .unwrap();
            to.write_all(&body).unwrap();
        }
        for i in 0..3_000 {
            to.start_file(
                format!("junk/{i:05}.bin"),
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            to.write_all(b"x").unwrap();
        }
        to.finish().unwrap();
    }

    let fresh = tempfile::tempdir().unwrap();
    let paths = quarters(&fresh);
    crate::witness::keeps(crate::witness::file(&paths), false);
    crate::witness::warn(channel::BACKUP, "the mark that has to survive", &[]);

    read(&paths, &hostile).unwrap();

    let diary = std::fs::read_to_string(crate::witness::file(&paths)).unwrap();
    crate::witness::stops();

    assert!(
        diary.contains("the mark that has to survive"),
        "restoring rolled the diary away and took the earlier diagnosis with it"
    );
    assert_eq!(
        diary.matches("does not put back").count(),
        1,
        "3000 refused entries wrote more than the one line that says so"
    );
    assert!(
        (diary.len() as u64) < crate::witness::ROLLS_AT,
        "the diary is {} bytes after one restore",
        diary.len()
    );
}

#[test]
fn a_zip_cannot_name_its_way_out_of_the_data_directory() {
    assert_eq!(
        safe("store/dev_a/active.tisty"),
        Some(PathBuf::from("store/dev_a/active.tisty"))
    );
    assert_eq!(
        safe("attachments/ab/foto-a1b2c3d4.png"),
        Some(PathBuf::from("attachments/ab/foto-a1b2c3d4.png"))
    );

    for climbing in [
        "../secrets",
        "store/../../etc/passwd",
        "/etc/passwd",
        "config/config.toml",
        "",
        "store",
        "store/.store-key",
        "store/dev_a/.store-key",
        "store/dev_a/.lock",
        "attachments/ab/.4812.7.part",
        "originals/../../etc/passwd",
    ] {
        assert_eq!(safe(climbing), None, "«{climbing}» got out");
    }

    for held in [
        "store/Dev_A/active.tisty",
        "store/dev_a657da33 2/000001.tisty",
        "store/dev_a/notes.txt",
        "store/dev_a/active.torn",
        "docs/.spent-dev_a",
        "attachments/ab/installer-a1b2c3d4.part",
    ] {
        assert!(
            safe(held).is_some(),
            "«{held}» goes into a copy and cannot come back from one"
        );
    }
}
