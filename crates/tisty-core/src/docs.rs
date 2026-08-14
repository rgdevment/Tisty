use std::path::{Path, PathBuf};

use std::io::{BufRead, Read};

use crate::{Error, Result, event::DeviceId, store::write_atomic};

const EXTENSION: &str = "md";
const DIGITS: usize = 4;
const MOST_DIGITS: u64 = 999_999_999_999;
const TITLE_AT_MOST: u64 = 4 * 1024;
pub const BODY_AT_MOST: u64 = 500 * 1024;
pub const BODY_ROOMY: u64 = 300 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Doc {
    pub id: String,
    pub title: String,
}

pub fn titled(body: &str) -> String {
    let first = body.lines().next().unwrap_or_default();
    crate::text::composed(first.trim_start_matches('#').trim())
}

pub fn create(root: &Path, device: &DeviceId, body: &str) -> Result<Doc> {
    std::fs::create_dir_all(root)?;
    let _ = crate::paths::ours_alone(root);
    let mut number = next(root, device);
    loop {
        if number > MOST_DIGITS {
            return Err(Error::OutsideTheStore(format!("{}-{number}", stem(device))));
        }
        let id = format!("{}-{number:0width$}", stem(device), width = DIGITS);
        let at = resolve(root, &id)?;
        match std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&at)
        {
            Ok(_) => {
                write(root, &id, body)?;
                return Ok(Doc {
                    title: titled(body),
                    id,
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => number += 1,
            Err(e) => return Err(Error::Io(e)),
        }
    }
}

pub fn write(root: &Path, id: &str, body: &str) -> Result<()> {
    let at = resolve(root, id)?;
    std::fs::create_dir_all(root)?;
    let _ = crate::paths::ours_alone(root);
    write_atomic(&at, body.as_bytes())
}

pub fn read_outside(at: &Path) -> Result<String> {
    let file = std::fs::File::open(at)?;
    if !file.metadata()?.is_file() {
        return Err(Error::OutsideTheStore(at.display().to_string()));
    }
    let mut body = String::new();
    let read = file.take(BODY_AT_MOST + 1).read_to_string(&mut body)? as u64;
    if read > BODY_AT_MOST {
        return Err(Error::DocumentTooBig {
            bytes: read,
            limit: BODY_AT_MOST,
        });
    }
    Ok(body)
}

pub fn read(root: &Path, id: &str) -> Result<String> {
    let at = resolve(root, id)?;
    let file = std::fs::File::open(&at)?;
    if !file.metadata()?.is_file() {
        return Err(Error::OutsideTheStore(id.to_string()));
    }
    let mut body = String::new();
    let read = file.take(BODY_AT_MOST + 1).read_to_string(&mut body)? as u64;
    if read > BODY_AT_MOST {
        return Err(Error::DocumentTooBig {
            bytes: read,
            limit: BODY_AT_MOST,
        });
    }
    Ok(body)
}

pub fn referenced(root: &Path) -> Vec<String> {
    all(root)
        .iter()
        .filter_map(|doc| match read(root, &doc.id) {
            Ok(body) => Some(body),
            Err(e) => {
                crate::witness::warn(
                    crate::witness::channel::ATTACH,
                    "a document could not be read while counting what is still named",
                    &[
                        ("id", crate::witness::Fact::Id(doc.id.clone())),
                        ("why", crate::witness::Fact::Why(e.to_string())),
                    ],
                );
                None
            }
        })
        .flat_map(|body| crate::refs::extract(&body))
        .map(|one| one.target)
        .collect()
}

pub fn remove(root: &Path, id: &str) -> Result<()> {
    let at = resolve(root, id)?;
    match std::fs::remove_file(at) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::Io(e)),
    }
}

pub fn all(root: &Path) -> Vec<Doc> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut found: Vec<Doc> = entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_type().map(|kind| kind.is_file()).unwrap_or(false))
        .filter_map(|one| {
            let at = one.path();
            let id = named(&at)?;
            Some(Doc {
                title: opening(&at),
                id,
            })
        })
        .collect();
    found.sort_by(|a, b| a.id.cmp(&b.id));
    found
}

fn resolve(root: &Path, id: &str) -> Result<PathBuf> {
    if !well_formed(id) {
        return Err(Error::OutsideTheStore(id.to_string()));
    }
    Ok(root.join(format!("{id}.{EXTENSION}")))
}

fn well_formed(id: &str) -> bool {
    let Some((device, number)) = id.rsplit_once('-') else {
        return false;
    };
    !device.is_empty()
        && device.len() <= 48
        && device
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !number.is_empty()
        && number.len() <= 12
        && number.chars().all(|c| c.is_ascii_digit())
}

fn named(at: &Path) -> Option<String> {
    if at.extension()? != EXTENSION {
        return None;
    }
    let id = at.file_stem()?.to_str()?.to_string();
    well_formed(&id).then_some(id)
}

fn next(root: &Path, device: &DeviceId) -> u64 {
    let mine = format!("{}-", stem(device));
    let highest = all(root)
        .iter()
        .filter_map(|doc| doc.id.strip_prefix(&mine))
        .filter_map(|number| number.parse::<u64>().ok())
        .max();
    highest.map_or(1, |last| last + 1)
}

fn stem(device: &DeviceId) -> String {
    let plain = device.0.strip_prefix("dev_").unwrap_or(&device.0);
    let kept: String = plain
        .chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(48)
        .collect();
    if kept.is_empty() {
        "device".to_string()
    } else {
        kept
    }
}

fn opening(at: &Path) -> String {
    let Ok(file) = std::fs::File::open(at) else {
        return String::new();
    };
    let mut first = String::new();
    let _ = std::io::BufReader::new(file.take(TITLE_AT_MOST)).read_line(&mut first);
    titled(&first)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(named: &str) -> DeviceId {
        DeviceId(named.to_string())
    }

    fn root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn what_only_a_document_names_is_not_adrift() {
        let root = root();
        create(
            root.path(),
            &device("dev_a"),
            "# Notas\n\nver [el informe](<attachments/ab/informe-91f2.pdf>)",
        )
        .unwrap();

        let named = referenced(root.path());

        assert!(
            named.contains(&"attachments/ab/informe-91f2.pdf".to_string()),
            "counting only tasks called this one loose: {named:?}"
        );
    }

    #[test]
    fn a_document_that_names_nothing_adds_nothing() {
        let root = root();
        create(root.path(), &device("dev_a"), "# Solo palabras").unwrap();

        assert!(referenced(root.path()).is_empty());
    }

    #[test]
    fn two_documents_made_at_once_never_land_in_one_file() {
        let root = root();
        let made: Vec<Doc> = std::thread::scope(|scope| {
            let hands: Vec<_> = (0..8)
                .map(|which| {
                    let at = root.path().to_path_buf();
                    scope.spawn(move || {
                        create(&at, &device("dev_a"), &format!("el numero {which}")).unwrap()
                    })
                })
                .collect();
            hands.into_iter().map(|one| one.join().unwrap()).collect()
        });

        let mut ids: Vec<String> = made.iter().map(|one| one.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 8, "two of them share a file: {made:?}");
        assert_eq!(all(root.path()).len(), 8);
    }

    #[test]
    fn a_device_name_with_nothing_usable_still_makes_documents() {
        let root = root();

        let made = create(root.path(), &device("dev_ÁÉÍ"), "").expect("a document");

        assert!(named(&root.path().join(format!("{}.md", made.id))).is_some());
    }

    #[test]
    fn a_title_is_read_without_pulling_in_the_whole_body() {
        let root = root();
        let body = "x".repeat(2 * 1024 * 1024);
        create(root.path(), &device("dev_a"), &body).unwrap();

        let title = all(root.path())[0].title.clone();

        assert!(
            title.len() <= TITLE_AT_MOST as usize,
            "read {} bytes of a body with no newline",
            title.len()
        );
    }

    #[test]
    fn the_ceiling_is_what_the_editor_can_hold_not_what_the_disk_can() {
        let room = tempfile::tempdir().unwrap();
        let write = |name: &str, size: usize| {
            let at = room.path().join(name);
            std::fs::write(&at, "x".repeat(size)).unwrap();
            read_outside(&at)
        };

        assert!(write("half.md", 250 * 1024).is_ok(), "a long note still opens");
        assert!(
            matches!(
                write("huge.md", 900 * 1024),
                Err(Error::DocumentTooBig { .. })
            ),
            "typing in it would cost more than a frame"
        );
    }

    #[test]
    fn an_imported_file_right_at_the_limit_still_comes_in_whole() {
        let room = tempfile::tempdir().unwrap();
        let big = room.path().join("edge.md");
        std::fs::write(&big, "z".repeat(BODY_AT_MOST as usize)).unwrap();

        assert_eq!(read_outside(&big).unwrap().len(), BODY_AT_MOST as usize);
    }

    #[test]
    fn an_imported_file_too_big_to_hold_is_refused_whole() {
        let room = tempfile::tempdir().unwrap();
        let big = room.path().join("big.md");
        std::fs::write(&big, "z".repeat(BODY_AT_MOST as usize + 8192)).unwrap();

        let refused = read_outside(&big);

        assert!(
            matches!(refused, Err(Error::DocumentTooBig { .. })),
            "half a document imported in silence is worse than none"
        );
    }

    #[test]
    fn a_body_too_big_to_hold_is_refused_rather_than_opened_with_its_tail_cut() {
        let root = root();
        let made = create(root.path(), &device("dev_a"), "").unwrap();
        std::fs::write(
            root.path().join(format!("{}.md", made.id)),
            "y".repeat(BODY_AT_MOST as usize + 4096),
        )
        .unwrap();

        let refused = read(root.path(), &made.id);

        assert!(
            matches!(refused, Err(Error::DocumentTooBig { .. })),
            "opening it cut would have saved it cut seven hundred milliseconds later"
        );
    }

    #[test]
    fn a_body_right_at_the_ceiling_still_opens_whole() {
        let root = root();
        let made = create(root.path(), &device("dev_a"), "").unwrap();
        std::fs::write(
            root.path().join(format!("{}.md", made.id)),
            "y".repeat(BODY_AT_MOST as usize),
        )
        .unwrap();

        assert_eq!(
            read(root.path(), &made.id).unwrap().len(),
            BODY_AT_MOST as usize
        );
    }

    #[test]
    fn a_name_that_is_not_a_regular_file_is_never_listed() {
        let root = root();
        create(root.path(), &device("dev_a"), "# Compras").unwrap();
        std::fs::create_dir(root.path().join("dev_b-0001.md")).unwrap();

        let found = all(root.path());

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Compras");
    }

    #[test]
    fn a_file_at_the_last_number_refuses_instead_of_naming_one_too_long() {
        let root = root();
        std::fs::create_dir_all(root.path()).unwrap();
        std::fs::write(root.path().join("a-999999999999.md"), "").unwrap();

        assert!(create(root.path(), &device("dev_a"), "").is_err());
    }

    #[test]
    fn two_device_names_that_differ_never_collapse_to_one_prefix() {
        let mine = root();
        let yours = root();

        let a = create(mine.path(), &device("dev_a3f1"), "").unwrap();
        let b = create(yours.path(), &device("dev_a-3f1"), "").unwrap();

        assert_ne!(
            a.id.rsplit_once('-').unwrap().0,
            b.id.rsplit_once('-').unwrap().0
        );
    }

    #[test]
    fn the_first_line_is_the_title_with_or_without_a_hash() {
        assert_eq!(titled("## Compras\n\ncuerpo"), "Compras");
        assert_eq!(titled("Compras\n\ncuerpo"), "Compras");
        assert_eq!(titled("###   Compras   "), "Compras");
    }

    #[test]
    fn a_document_with_nothing_written_yet_has_no_title_rather_than_failing() {
        assert_eq!(titled(""), "");
        assert_eq!(titled("\n\ncuerpo"), "");
    }

    #[test]
    fn a_title_is_read_in_one_spelling() {
        assert_eq!(titled("# Disen\u{0303}o"), "Diseño");
    }

    #[test]
    fn a_document_survives_the_round_trip_byte_for_byte() {
        let root = root();
        let body = "# Compras\n\n- uno\n\ttabulado  \ny espacios al final   \n";

        let doc = create(root.path(), &device("dev_a3f1"), body).unwrap();

        assert_eq!(read(root.path(), &doc.id).unwrap(), body);
    }

    #[test]
    fn the_name_carries_the_device_so_two_machines_cannot_collide() {
        let root = root();

        let mine = create(root.path(), &device("dev_a3f1"), "# mío").unwrap();
        let theirs = create(root.path(), &device("dev_b7c2"), "# suyo").unwrap();

        assert_eq!(mine.id, "a3f1-0001");
        assert_eq!(theirs.id, "b7c2-0001");
        assert_eq!(all(root.path()).len(), 2);
    }

    #[test]
    fn each_device_counts_on_its_own() {
        let root = root();
        let mine = device("dev_a3f1");

        create(root.path(), &mine, "# uno").unwrap();
        create(root.path(), &device("dev_b7c2"), "# suyo").unwrap();
        let third = create(root.path(), &mine, "# dos").unwrap();

        assert_eq!(third.id, "a3f1-0002");
    }

    fn made(root: &Path, id: &str, body: &str) {
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join(format!("{id}.md")), body).unwrap();
    }

    #[test]
    fn a_number_already_on_disk_is_never_minted_again() {
        let root = root();
        made(root.path(), "a3f1-0007", "# llegó del otro lado");

        let next = create(root.path(), &device("dev_a3f1"), "# nuevo").unwrap();

        assert_eq!(next.id, "a3f1-0008");
        assert_eq!(
            read(root.path(), "a3f1-0007").unwrap(),
            "# llegó del otro lado"
        );
    }

    #[test]
    fn an_id_that_climbs_out_of_the_store_is_refused() {
        let root = root();
        for id in [
            "../../.ssh/id_rsa",
            "..",
            "a3f1-0001/../../x",
            "a3f1",
            "-0001",
            "a3f1-",
            "A3F1-0001",
            "a3f1-00x1",
            "",
        ] {
            assert!(read(root.path(), id).is_err(), "{id} was allowed");
            assert!(write(root.path(), id, "x").is_err(), "{id} was allowed");
        }
    }

    #[test]
    fn what_is_not_a_document_is_not_listed() {
        let root = root();
        made(root.path(), "a3f1-0001", "# real");
        std::fs::write(root.path().join(".meta.toml"), "x").unwrap();
        std::fs::write(root.path().join("notas.txt"), "x").unwrap();
        std::fs::write(root.path().join("suelto.md"), "# sin id").unwrap();

        let found = all(root.path());

        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].id, "a3f1-0001");
    }

    #[test]
    fn a_directory_that_is_not_there_yet_lists_nothing_instead_of_failing() {
        let root = root();

        assert!(all(&root.path().join("absent")).is_empty());
    }

    #[test]
    fn rewriting_keeps_the_name_and_the_reference_with_it() {
        let root = root();
        let doc = create(root.path(), &device("dev_a3f1"), "# Compras").unwrap();

        write(root.path(), &doc.id, "# Compras del mes\n\notra cosa").unwrap();

        let found = all(root.path());
        assert_eq!(found[0].id, doc.id);
        assert_eq!(found[0].title, "Compras del mes");
    }

    #[test]
    fn removing_what_is_not_there_is_not_an_error() {
        let root = root();

        assert!(remove(root.path(), "a3f1-0001").is_ok());
    }

    #[test]
    fn an_imported_file_one_byte_past_the_limit_is_refused_not_truncated() {
        let room = tempfile::tempdir().unwrap();
        let big = room.path().join("edge.md");
        std::fs::write(&big, "z".repeat(BODY_AT_MOST as usize + 1)).unwrap();

        let refused = read_outside(&big);

        assert!(
            matches!(
                refused,
                Err(Error::DocumentTooBig { bytes, limit })
                    if bytes == BODY_AT_MOST + 1 && limit == BODY_AT_MOST
            ),
            "{refused:?}"
        );
    }

    #[test]
    fn a_file_that_is_not_valid_utf8_is_refused_rather_than_read_as_garbage() {
        let room = tempfile::tempdir().unwrap();
        let bad = room.path().join("bad.md");
        std::fs::write(&bad, [0x66, 0x6f, 0xff, 0xfe, 0x62, 0x61, 0x72]).unwrap();

        assert!(matches!(read_outside(&bad), Err(Error::Io(_))));
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_to_a_real_file_is_read_like_any_other_file() {
        let room = tempfile::tempdir().unwrap();
        let real = room.path().join("real.md");
        std::fs::write(&real, "# contenido real").unwrap();
        let link = room.path().join("link.md");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        assert_eq!(read_outside(&link).unwrap(), "# contenido real");
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_to_a_directory_is_refused_like_any_other_directory() {
        let room = tempfile::tempdir().unwrap();
        let target_dir = room.path().join("adir");
        std::fs::create_dir(&target_dir).unwrap();
        let link = room.path().join("link_to_dir.md");
        std::os::unix::fs::symlink(&target_dir, &link).unwrap();

        assert!(matches!(
            read_outside(&link),
            Err(Error::OutsideTheStore(_))
        ));
    }

    #[cfg(unix)]
    #[test]
    fn a_directory_given_as_a_document_is_refused_not_read() {
        let room = tempfile::tempdir().unwrap();
        let dir = room.path().join("adir");
        std::fs::create_dir(&dir).unwrap();

        assert!(matches!(read_outside(&dir), Err(Error::OutsideTheStore(_))));
    }
}
