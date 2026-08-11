//! One file you can keep anywhere. A plain zip, so it opens without Tisty.

use std::io::{Read, Seek, Write};
use std::path::{Component, Path, PathBuf};

use crate::{
    Config, Error, Paths, Result, store,
    witness::{self, Fact, channel},
};

/// Never the configuration: a shared `device_id` puts two machines in one file.
const CARRIED: [&str; 2] = ["store", "attachments"];
/// A backup is history, not a filesystem: past this it is somebody's zip bomb.
const AT_MOST: u64 = 8 * 1024 * 1024 * 1024;
const AT_MOST_FILES: usize = 200_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Made {
    pub files: usize,
    pub bytes: u64,
    pub store_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Restored {
    pub files: usize,
    pub devices: usize,
}

/// Written aside and renamed at the end: overwriting the previous backup in
/// place would trade a good one for a well-formed zip that is missing files.
pub fn write(data: &Path, into: &Path) -> Result<Made> {
    let store_id = store::identity(data.join("store"))?;
    store::read_all(data.join("store"))?;

    let part = into.with_extension(format!("{}.part", std::process::id()));
    let made = fill(data, &part, store_id);
    match made {
        Ok(made) => {
            std::fs::rename(&part, into)?;
            Ok(made)
        }
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            Err(e)
        }
    }
}

fn fill(data: &Path, into: &Path, store_id: String) -> Result<Made> {
    let file = std::fs::File::create(into)?;
    // The zip is the whole history in one file, and it often lands somewhere
    // shared like /tmp.
    let _ = crate::paths::ours_alone(into);
    let mut zip = zip::ZipWriter::new(file);
    let mut made = Made {
        files: 0,
        bytes: 0,
        store_id,
    };

    for folder in CARRIED {
        let root = data.join(folder);
        for at in walk(&root) {
            let Ok(rest) = at.strip_prefix(data) else {
                continue;
            };
            // A lock is a live claim on this machine, not history to carry.
            if rest.file_name().is_some_and(|n| n == ".lock") {
                continue;
            }
            let named = rest.to_string_lossy().replace('\\', "/");
            let body = std::fs::read(&at)?;

            // The same ceiling as reading: a backup nobody can restore is worse
            // than none, because it is only found out on the bad day.
            made.files += 1;
            made.bytes += body.len() as u64;
            if made.bytes > AT_MOST || made.files > AT_MOST_FILES {
                return Err(Error::TooBig);
            }

            zip.start_file(named, zip::write::SimpleFileOptions::default())
                .map_err(zipped)?;
            zip.write_all(&body)?;
        }
    }
    zip.finish().map_err(zipped)?;
    Ok(made)
}

/// A photograph: back to that moment, and what came after is lost on purpose.
/// The machine takes a new identity so it can never shrink what others hold.
///
/// Nothing of yours is touched until the whole backup has been unpacked beside
/// it and read back: a zip that turns out to be corrupt, truncated or somebody
/// else's must cost you nothing, and the only way to be sure is to try it first.
pub fn read(paths: &Paths, from: &Path) -> Result<Restored> {
    within(paths, from, AT_MOST)
}

pub(crate) fn within(paths: &Paths, from: &Path, at_most: u64) -> Result<Restored> {
    let data = paths.data();
    let file = std::fs::File::open(from)?;
    let mut zip = zip::ZipArchive::new(file).map_err(zipped)?;

    let root = data.join("store");
    let theirs = named_in(&mut zip)?;
    let ours = store::peek_identity(&root);
    let free = ours.is_none()
        && store::read_all(&root)
            .map(|all| all.is_empty())
            .unwrap_or(false);
    // An unmarked backup is only safe onto an unmarked, empty machine: taken
    // the other way round it is a stranger's history replacing your own.
    match (&ours, theirs.is_empty()) {
        (_, false) if ours.as_ref() == Some(&theirs) || free => {}
        (None, true) if free => {}
        _ => {
            return Err(Error::OtherStore {
                theirs: if theirs.is_empty() {
                    from.display().to_string()
                } else {
                    theirs
                },
            });
        }
    }

    let staged = data.join(format!(".restoring-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staged);
    let done = unpack(&mut zip, &staged, at_most).and_then(|files| {
        store::read_all(staged.join("store"))?;
        if files == 0 || !staged.join("store").is_dir() {
            return Err(Error::OtherStore {
                theirs: from.display().to_string(),
            });
        }
        Ok(files)
    });
    let files = match done {
        Ok(files) => files,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&staged);
            return Err(e);
        }
    };

    // Named before anything moves: writing under the old one into a store that
    // just went back in time is what shrinks a directory other machines hold.
    let was = Config::load_or_init(paths)?;
    let mut config = was.clone();
    config.device_id = crate::DeviceId(crate::config::new_device_id());
    config.synced_at = None;
    config.save(paths)?;

    let old = data.join(format!(".replaced-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&old);
    std::fs::create_dir_all(&old)?;
    if let Err(e) = swap(data, &staged, &old) {
        // Half a restore is the one outcome worth more than either whole: put
        // everything back, including the name, and let the person try again.
        if was.save(paths).is_err() {
            witness::error(channel::BACKUP, "device name not put back", &[]);
        }
        let _ = std::fs::remove_dir_all(&staged);
        return Err(e);
    }
    let _ = std::fs::remove_dir_all(&staged);
    let _ = std::fs::remove_dir_all(&old);
    let _ = std::fs::remove_dir_all(paths.cache());

    Ok(Restored {
        files,
        devices: std::fs::read_dir(data.join("store"))
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                    .count()
            })
            .unwrap_or(0),
    })
}

/// Every old folder steps aside before a single new one steps in, and a failure
/// walks all of them back: a machine left with half a photograph and half of
/// what it had is worse off than with either one whole.
fn swap(data: &Path, staged: &Path, old: &Path) -> Result<()> {
    let mut moved: Vec<&str> = Vec::new();
    for folder in CARRIED {
        let at = data.join(folder);
        if !at.exists() {
            continue;
        }
        if let Err(e) = std::fs::rename(&at, old.join(folder)) {
            undo(data, old, &moved);
            return Err(Error::Io(e));
        }
        moved.push(folder);
    }

    for folder in CARRIED {
        let fresh = staged.join(folder);
        if !fresh.exists() {
            continue;
        }
        if let Err(e) = std::fs::rename(&fresh, data.join(folder)) {
            for done in CARRIED {
                let at = data.join(done);
                if at.exists() && staged.join(done).exists() {
                    continue;
                }
                let _ = std::fs::rename(&at, staged.join(done));
            }
            undo(data, old, &moved);
            return Err(Error::Io(e));
        }
    }
    Ok(())
}

fn undo(data: &Path, old: &Path, moved: &[&str]) {
    for folder in moved {
        let at = data.join(folder);
        if at.exists() {
            let _ = std::fs::remove_dir_all(&at);
        }
        if let Err(e) = std::fs::rename(old.join(folder), &at) {
            witness::error(
                channel::BACKUP,
                "folder not put back",
                &[
                    ("at", Fact::Path(at.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
    }
}

/// Left by a restore that died: `.replaced-*` is the only copy of what was
/// swapped out, so a machine that crashed mid-swap still has its history here.
pub fn leftovers(data: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(data) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|at| {
            at.is_dir()
                && at
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(".restoring-") || n.starts_with(".replaced-"))
        })
        .collect()
}

fn unpack<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    into: &Path,
    at_most: u64,
) -> Result<usize> {
    if zip.len() > AT_MOST_FILES {
        return Err(Error::TooBig);
    }
    let mut files = 0;
    let mut bytes = 0u64;

    for i in 0..zip.len() {
        let mut held = zip.by_index(i).map_err(zipped)?;
        if held.is_dir() {
            continue;
        }
        let Some(rest) = safe(held.name()) else {
            continue;
        };
        if held.size() > at_most.saturating_sub(bytes) {
            return Err(Error::TooBig);
        }

        let at = into.join(&rest);
        if let Some(parent) = at.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // Streamed, and capped: a declared size is what the zip claims, not
        // what it holds, so the ceiling has to hold on the way out too.
        let mut file = std::fs::File::create(&at)?;
        let _ = crate::paths::ours_alone(&at);
        let room = at_most.saturating_sub(bytes).saturating_add(1);
        let written = std::io::copy(&mut held.by_ref().take(room), &mut file)?;
        if written >= room {
            return Err(Error::TooBig);
        }
        bytes = bytes.saturating_add(written);
        files += 1;
    }
    Ok(files)
}

fn named_in<R: Read + Seek>(zip: &mut zip::ZipArchive<R>) -> Result<String> {
    let at = format!("store/{}", store::MARKER);
    match zip.by_name(&at) {
        Ok(mut held) => {
            let mut said = String::new();
            held.read_to_string(&mut said)?;
            Ok(said.trim().to_string())
        }
        Err(_) => Ok(String::new()),
    }
}

fn safe(named: &str) -> Option<PathBuf> {
    let at = Path::new(named);
    if !at
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        return None;
    }
    let head = at.components().next()?.as_os_str().to_str()?;
    CARRIED.contains(&head).then(|| at.to_path_buf())
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::BACKUP,
                    "folder left out of the copy",
                    &[
                        ("at", Fact::Path(root.to_path_buf())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
            return found;
        }
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let at = entry.path();
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => found.extend(walk(&at)),
            Ok(kind) if kind.is_file() => found.push(at),
            _ => {}
        }
    }
    found.sort();
    found
}

fn zipped(e: zip::result::ZipError) -> Error {
    Error::Io(std::io::Error::other(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
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
        std::fs::write(shelf.join("cd.png"), b"a picture").unwrap();
        (dir, data)
    }

    #[test]
    fn a_backup_carries_the_store_and_the_attachments() {
        let (_src, data) = filled("comprar pan");
        let out = tempfile::tempdir().unwrap();
        let file = out.path().join("tisty.zip");

        let made = write(&data, &file).unwrap();
        assert!(made.files >= 2, "{made:?}");
        assert!(made.bytes > 0);
        assert!(file.exists());
    }

    #[test]
    fn what_comes_back_is_what_went_in() {
        let (_src, data) = filled("comprar pan");
        let out = tempfile::tempdir().unwrap();
        let file = out.path().join("tisty.zip");
        write(&data, &file).unwrap();

        let fresh = tempfile::tempdir().unwrap();
        let restored = read(&quarters(&fresh), &file).unwrap();

        assert!(restored.files >= 2);
        assert_eq!(restored.devices, 1);
        let events = store::read_all(quarters(&fresh).store()).unwrap();
        assert_eq!(events.len(), 1);
        assert!(
            quarters(&fresh)
                .data()
                .join("attachments/ab/cd.png")
                .exists()
        );
    }

    /// After a format the machine is new and the history is old, and that is
    /// exactly what one directory per device already means.
    #[test]
    fn restoring_onto_an_empty_machine_keeps_the_old_devices_history() {
        let (_src, data) = filled("lo de antes");
        let out = tempfile::tempdir().unwrap();
        let file = out.path().join("tisty.zip");
        write(&data, &file).unwrap();

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

    /// A photograph, and what came after it is lost on purpose. The machine
    /// takes a new name so its directory starts empty and can never shrink
    /// what the others already hold.
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
        write(paths.data(), &file).unwrap();

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
        write(&one, &file).unwrap();

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
        write(&data, &file).unwrap();

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

    /// The file picker filters by `*.zip`, and every machine has a zip that is
    /// not a backup. Choosing one must cost nothing at all.
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
        assert!(data.join("attachments/ab/cd.png").exists());
    }

    #[test]
    fn a_corrupt_backup_costs_nothing() {
        let (_src, data) = filled("lo mio");
        let paths = Paths::new(&data, data.parent().unwrap().join("config"));
        let out = tempfile::tempdir().unwrap();
        let file = out.path().join("tisty.zip");
        write(&data, &file).unwrap();

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

    /// An unmarked backup used to walk straight past the guard.
    #[test]
    fn a_backup_with_no_marker_is_refused_onto_a_store_that_has_one() {
        let (_src, data) = filled("lo mio");
        let paths = Paths::new(&data, data.parent().unwrap().join("config"));
        let (_b, other) = filled("lo de otro");
        let out = tempfile::tempdir().unwrap();
        let file = out.path().join("tisty.zip");
        write(&other, &file).unwrap();

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

    /// The old id would keep writing into a directory that just went back in
    /// time, shrinking what the other machines already hold.
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
        write(paths.data(), &file).unwrap();
        read(&paths, &file).unwrap();

        let now = Config::load(&paths.config_file()).unwrap().unwrap();
        assert_ne!(now.device_id.0, was);
        assert!(
            now.synced_at.is_none(),
            "it claimed a sync that predates it"
        );
    }

    /// A hundred megabytes from a hundred kilobytes, and the store gone first.
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

    /// Windows refuses to rename a directory holding an open file; here the
    /// same refusal is forced on purpose, because half a restore is the one
    /// outcome worth less than either whole.
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

        // `old/attachments` already taken by a directory that is not empty, so
        // its rename is refused and the swap walks `store` back on its own.
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
    fn what_a_dead_restore_left_behind_can_be_found() {
        let (_src, data) = filled("lo mio");
        std::fs::create_dir_all(data.join(".replaced-4242/store")).unwrap();
        std::fs::create_dir_all(data.join(".restoring-4242")).unwrap();

        let found = leftovers(&data);
        assert_eq!(found.len(), 2, "{found:?}");
    }

    #[test]
    fn a_zip_cannot_name_its_way_out_of_the_data_directory() {
        assert_eq!(
            safe("store/dev_a/active.tisty"),
            Some(PathBuf::from("store/dev_a/active.tisty"))
        );
        assert_eq!(
            safe("attachments/ab/cd.png"),
            Some(PathBuf::from("attachments/ab/cd.png"))
        );

        for climbing in [
            "../secrets",
            "store/../../etc/passwd",
            "/etc/passwd",
            "config/config.toml",
            "",
        ] {
            assert_eq!(safe(climbing), None, "«{climbing}» got out");
        }
    }
}
