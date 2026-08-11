//! One file you can keep anywhere. A plain zip, so it opens without Tisty.

use std::io::{Read, Seek, Write};
use std::path::{Component, Path, PathBuf};

use crate::{Config, Error, Paths, Result, store};

/// Never the configuration: a shared `device_id` puts two machines in one file.
const CARRIED: [&str; 2] = ["store", "attachments"];

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

pub fn write(data: &Path, into: &Path) -> Result<Made> {
    let store_id = store::identity(data.join("store"))?;
    let file = std::fs::File::create(into)?;
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
            let named = rest.to_string_lossy().replace('\\', "/");
            let body = std::fs::read(&at)?;

            zip.start_file(named, zip::write::SimpleFileOptions::default())
                .map_err(zipped)?;
            zip.write_all(&body)?;
            made.files += 1;
            made.bytes += body.len() as u64;
        }
    }
    zip.finish().map_err(zipped)?;
    Ok(made)
}

/// A photograph: back to that moment, and what came after is lost on purpose.
/// The machine takes a new identity so it can never shrink what others hold.
pub fn read(paths: &Paths, from: &Path) -> Result<Restored> {
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
    if !theirs.is_empty() && Some(&theirs) != ours.as_ref() && !free {
        return Err(Error::OtherStore { theirs });
    }

    for folder in CARRIED {
        let at = data.join(folder);
        if at.exists() {
            std::fs::remove_dir_all(&at)?;
        }
    }

    let mut restored = Restored {
        files: 0,
        devices: 0,
    };
    for i in 0..zip.len() {
        let mut held = zip.by_index(i).map_err(zipped)?;
        if held.is_dir() {
            continue;
        }
        let Some(rest) = safe(held.name()) else {
            continue;
        };
        let at = data.join(&rest);
        if let Some(parent) = at.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut body = Vec::new();
        held.read_to_end(&mut body)?;
        store::write_atomic(&at, &body)?;
        restored.files += 1;
    }

    store::read_all(&root)?;

    let _ = std::fs::remove_dir_all(paths.cache());

    let mut config = Config::load(&paths.config_file())?.unwrap_or(Config::load_or_init(paths)?);
    config.device_id = crate::DeviceId(crate::config::new_device_id());
    config.save(paths)?;

    restored.devices = std::fs::read_dir(data.join("store"))
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0);
    Ok(restored)
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
    let Ok(entries) = std::fs::read_dir(root) else {
        return found;
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
