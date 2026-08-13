use std::io::Read;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{
    Error, Result,
    witness::{self, Fact, channel},
};

pub const COPIED_UP_TO: u64 = 5 * 1024 * 1024;

const SHORTENS_TO: usize = 56;

pub const COPIED_LEAST: u64 = 64 * 1024;
pub const COPIED_MOST: u64 = 200 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    pub at: String,
    pub sha256: String,
}

impl Kept {
    pub fn written(&self, label: &str) -> String {
        let name = spoken(label);
        let target = self.at.clone();
        if pictorial(&target) {
            format!("![{name}](<{target}>)")
        } else {
            format!("[{name}](<{target}>)")
        }
    }
}

fn spoken(label: &str) -> String {
    let flat: String = label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .map(|c| if c == '[' || c == ']' { '_' } else { c })
        .collect();
    if flat.is_empty() { "file".into() } else { flat }
}

pub fn keep(source: &Path, root: &Path, limit: u64) -> Result<Kept> {
    let mut file = std::fs::File::open(source)?;
    let opened = file.metadata()?;
    if !opened.is_file() {
        return Err(Error::OutsideTheStore(source.display().to_string()));
    }
    if opened.len() > limit {
        return Err(Error::AttachmentTooBig {
            bytes: opened.len(),
            limit,
        });
    }

    let mut bytes = Vec::new();
    let read = file.by_ref().take(limit + 1).read_to_end(&mut bytes)? as u64;
    if read > limit {
        return Err(Error::AttachmentTooBig { bytes: read, limit });
    }

    let sha256 = fingerprint(&bytes);
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .filter(|e| plain(e))
        .map(|e| format!(".{e}"))
        .unwrap_or_default();

    let (shelf, rest) = sha256.split_at(2);
    let stamp = &rest[..8];
    let folder = root.join("attachments").join(shelf);
    std::fs::create_dir_all(&folder)?;
    let _ = crate::paths::ours_alone(root);
    let _ = crate::paths::ours_alone(&root.join("attachments"));
    let _ = crate::paths::ours_alone(&folder);

    let name = match already(&folder, stamp) {
        Some(kept) => kept,
        None => {
            let name = named(source, stamp, &ext);
            let target = folder.join(&name);
            std::fs::write(&target, &bytes)?;
            let _ = crate::paths::ours_alone(&target);
            name
        }
    };
    Ok(Kept {
        at: format!("attachments/{shelf}/{name}"),
        sha256,
    })
}

fn named(source: &Path, stamp: &str, ext: &str) -> String {
    let slug: String = source
        .file_stem()
        .and_then(|one| one.to_str())
        .map(|one| crate::text::composed(one).to_lowercase())
        .unwrap_or_default()
        .chars()
        .map(plainly)
        .collect();

    let slug: String = slug
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(SHORTENS_TO)
        .collect();
    let slug = slug.trim_matches('-');

    if slug.is_empty() {
        format!("{stamp}{ext}")
    } else {
        format!("{slug}-{stamp}{ext}")
    }
}

fn plainly(c: char) -> char {
    match c {
        'a'..='z' | '0'..='9' => c,
        'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        _ => '-',
    }
}

fn already(folder: &Path, stamp: &str) -> Option<String> {
    std::fs::read_dir(folder)
        .ok()?
        .filter_map(|one| one.ok())
        .find_map(|one| {
            let name = one.file_name().to_str()?.to_string();
            let stem = name.split('.').next().unwrap_or(&name);
            (stem == stamp || stem.ends_with(&format!("-{stamp}"))).then_some(name)
        })
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Loose {
    pub files: usize,
    pub bytes: u64,
}

pub fn loose(root: &Path, referenced: &[String]) -> Loose {
    let held: std::collections::BTreeSet<&str> = referenced
        .iter()
        .map(|one| one.trim_start_matches("attachments/"))
        .collect();

    let mut found = Loose::default();
    let at = root.join("attachments");
    let shelves = match std::fs::read_dir(&at) {
        Ok(shelves) => shelves,
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                witness::warn(
                    channel::ATTACH,
                    "attachments unreadable",
                    &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
                );
            }
            return found;
        }
    };
    for shelf in shelves.filter_map(|e| e.ok()) {
        let Some(name) = shelf.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|e| e.ok()) {
            let Some(leaf) = file.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            if held.contains(format!("{name}/{leaf}").as_str()) {
                continue;
            }
            found.files += 1;
            found.bytes += file.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    found
}

pub fn resolve(reference: &str, root: &Path) -> Result<PathBuf> {
    let cleaned = reference.split(['?', '#']).next().unwrap_or("");
    let refused = || Err(Error::OutsideTheStore(reference.to_string()));
    if cleaned.is_empty() {
        return refused();
    }

    if cleaned.contains('\\') {
        return refused();
    }

    let mut walked = root.to_path_buf();
    let mut steps = 0;
    for part in Path::new(cleaned).components() {
        let Component::Normal(name) = part else {
            return refused();
        };
        let Some(name) = name.to_str() else {
            return refused();
        };
        if name.contains(':') || reserved(name) {
            return refused();
        }
        walked.push(name);
        steps += 1;
    }
    if steps == 0 {
        return refused();
    }
    Ok(walked)
}

fn reserved(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit())
}

fn plain(ext: &str) -> bool {
    !ext.is_empty() && ext.len() <= 16 && ext.chars().all(|c| c.is_ascii_alphanumeric())
}

fn fingerprint(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn pictorial(name: &str) -> bool {
    let lower = name.to_lowercase();
    [".png", ".jpg", ".jpeg", ".gif", ".webp", ".avif", ".svg"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dropped(name: &str, bytes: &[u8]) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join(name);
        std::fs::write(&file, bytes).unwrap();
        (dir, file)
    }

    #[test]
    fn a_small_file_is_copied_in_and_named_by_its_contents() {
        let (_src, file) = dropped("shot.PNG", b"pretend this is a screenshot");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, sha256 } = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert!(at.starts_with("attachments/"), "{at}");
        assert!(at.ends_with(".png"), "the extension is lowercased: {at}");
        assert!(
            at.contains(&sha256[..2]),
            "the shelf is the first two: {at}"
        );
        assert!(root.path().join(&at).exists());
    }

    #[test]
    fn the_same_file_twice_is_one_file() {
        let (_src, file) = dropped("shot.png", b"the very same bytes");
        let (_other, again) = dropped("renamed.png", b"the very same bytes");
        let root = tempfile::tempdir().unwrap();

        let first = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        let second = keep(&again, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(
            first, second,
            "the name comes from the contents, not the name"
        );
        let shelves = std::fs::read_dir(root.path().join("attachments")).unwrap();
        assert_eq!(shelves.count(), 1);
    }

    #[test]
    fn a_heavy_file_is_refused_and_never_copied() {
        let (_src, file) = dropped("recording.mkv", b"pretend this is fifteen gigabytes");
        let root = tempfile::tempdir().unwrap();

        let refused = keep(&file, root.path(), 4).unwrap_err();

        assert!(
            matches!(
                refused,
                crate::Error::AttachmentTooBig { bytes, limit } if bytes > 4 && limit == 4
            ),
            "{refused:?}"
        );
        assert!(
            !root.path().join("attachments").exists(),
            "nothing was copied in"
        );
    }

    #[test]
    fn a_picture_is_shown_and_everything_else_is_linked() {
        let held = |at: &str| Kept {
            at: at.into(),
            sha256: "ab".into(),
        };

        assert_eq!(
            held("attachments/ab/cd.png").written("the screen"),
            "![the screen](<attachments/ab/cd.png>)"
        );
        assert_eq!(
            held("attachments/ab/cd.pdf").written("the invoice"),
            "[the invoice](<attachments/ab/cd.pdf>)"
        );
    }

    #[test]
    fn a_path_with_spaces_is_still_a_link() {
        let kept = Kept {
            at: "attachments/ab/clip (1).mkv".into(),
            sha256: "ab".into(),
        };
        let written = kept.written("clip (1).mkv");
        assert!(written.starts_with("[clip (1).mkv](<"), "{written}");
        assert!(written.ends_with(">)"), "{written}");
    }

    #[test]
    fn a_name_that_would_break_the_link_is_flattened() {
        let one = Kept {
            at: "attachments/ab/cd.png".into(),
            sha256: "ab".into(),
        };
        assert_eq!(
            one.written("shot](javascript:alert(1))["),
            "![shot_(javascript:alert(1))_](<attachments/ab/cd.png>)"
        );
        assert_eq!(
            one.written("two\nlines"),
            "![two lines](<attachments/ab/cd.png>)"
        );
        assert_eq!(one.written("   "), "![file](<attachments/ab/cd.png>)");
    }

    #[test]
    fn an_extension_that_hides_a_stream_is_dropped() {
        let (_src, file) = dropped("carrier.txt-evil", b"hidden");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, .. } = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        assert!(!at.contains(':'), "a stream reached the store: {at}");
        assert!(
            at.ends_with("-evil") || !at.contains('.'),
            "odd extension kept: {at}"
        );
    }

    #[test]
    fn a_reserved_name_is_a_device_and_not_a_file() {
        let root = Path::new("/data");
        for device in ["NUL", "CON", "COM1", "LPT9", "nul.png", "attachments/CON"] {
            assert!(resolve(device, root).is_err(), "«{device}» got through");
        }
    }

    #[test]
    fn a_drive_letter_without_a_root_is_still_a_way_out() {
        let root = Path::new("/data");
        for climbing in ["C:foo", "attachments/ab/cd.png:hidden", "//server/share"] {
            assert!(resolve(climbing, root).is_err(), "«{climbing}» got through");
        }
    }

    #[test]
    fn a_backslash_is_refused_wherever_it_is_read() {
        let root = Path::new("/data");

        for climbing in [
            r"..\..\.ssh\id_rsa",
            r"attachments\..\..\secrets",
            r"attachments\ab\cd.png",
        ] {
            assert!(
                resolve(climbing, root).is_err(),
                "«{climbing}» is read as a climb on Windows and as a name elsewhere"
            );
        }
    }

    #[test]
    fn two_dots_inside_a_name_are_not_a_climb() {
        let root = Path::new("/data");
        assert!(resolve("attachments/ab/my..file.png", root).is_ok());
    }

    #[test]
    fn nothing_reaches_outside_the_data_root() {
        let root = Path::new("/data");

        for climbing in [
            "../../.ssh/id_rsa",
            "attachments/../../secrets",
            "/etc/passwd",
            "C:/Windows/System32/config",
            "",
        ] {
            assert!(
                resolve(climbing, root).is_err(),
                "«{climbing}» got out of the store"
            );
        }
        assert!(resolve("attachments/ab/cd.png", root).is_ok());
        assert!(resolve("docs/notes.md", root).is_ok());
    }

    #[test]
    fn a_file_the_exact_size_of_the_limit_is_copied() {
        let (_src, file) = dropped("shot.png", b"1234");
        let root = tempfile::tempdir().unwrap();

        assert!(keep(&file, root.path(), 4).is_ok());
    }

    #[test]
    fn a_file_without_an_extension_keeps_its_name_and_its_stamp() {
        let (_src, file) = dropped("README", b"no extension here");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, sha256 } = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert!(at.ends_with(&format!("readme-{}", &sha256[2..10])), "{at}");
    }

    fn stored(name: &str, bytes: &[u8]) -> String {
        let (_src, file) = dropped(name, bytes);
        let root = tempfile::tempdir().unwrap();
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        kept.at.rsplit('/').next().unwrap().to_string()
    }

    #[test]
    fn the_name_on_disk_is_the_one_a_person_would_recognise() {
        assert!(stored("Informe final.pdf", b"a").starts_with("informe-final-"));
        assert!(stored("captura de pantalla.png", b"b").starts_with("captura-de-pantalla-"));
    }

    #[test]
    fn nothing_a_system_argues_about_survives_in_the_name() {
        let kept = stored("Diseño Técnico: v2 <final>.PDF", b"c");

        assert!(kept.starts_with("diseno-tecnico-v2-final-"), "{kept}");
        assert!(kept.ends_with(".pdf"), "{kept}");
        assert!(
            kept.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.'),
            "{kept}"
        );
    }

    #[test]
    fn a_name_windows_reserves_stops_being_reserved() {
        let kept = stored("CON.txt", b"d");

        assert!(kept.starts_with("con-"), "{kept}");
        assert_ne!(kept, "con.txt");
    }

    #[test]
    fn a_name_nobody_could_shorten_falls_back_to_the_stamp() {
        let kept = stored("привет мир.txt", b"e");

        assert!(kept.ends_with(".txt"), "{kept}");
        assert_eq!(kept.len(), "12345678.txt".len(), "{kept}");
    }

    #[test]
    fn a_very_long_name_is_cut_without_losing_the_stamp() {
        let (_src, file) = dropped(&format!("{}.pdf", "nombre-larguisimo-".repeat(10)), b"f");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, sha256 } = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        let kept = at.rsplit('/').next().unwrap();

        assert!(kept.len() < 80, "{} chars: {kept}", kept.len());
        assert!(
            kept.ends_with(&format!("-{}.pdf", &sha256[2..10])),
            "the stamp was cut off with the name: {kept}"
        );
        assert!(kept.starts_with("nombre-larguisimo-"), "{kept}");
    }

    #[test]
    fn what_a_real_name_looks_like_on_disk() {
        for (given, wanted) in [
            ("Informe Técnico Final v2.pdf", "informe-tecnico-final-v2"),
            (
                "Captura de pantalla 2026-08-13.png",
                "captura-de-pantalla-2026-08-13",
            ),
            ("presupuesto (copia).xlsx", "presupuesto-copia"),
            ("Diseño & Maquetación.sketch", "diseno-maquetacion"),
        ] {
            let kept = stored(given, given.as_bytes());
            assert!(kept.starts_with(wanted), "«{given}» quedó como «{kept}»");
        }
    }

    #[test]
    fn the_same_bytes_under_two_names_are_still_one_file() {
        let root = tempfile::tempdir().unwrap();
        let (_a, first) = dropped("informe.pdf", b"same bytes");
        let (_b, second) = dropped("copia del informe.pdf", b"same bytes");

        let one = keep(&first, root.path(), COPIED_UP_TO).unwrap();
        let two = keep(&second, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(one.at, two.at, "kept twice");
        assert_eq!(one.sha256, two.sha256);
    }

    #[test]
    fn a_directory_is_not_a_file_and_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();

        assert!(keep(dir.path(), root.path(), COPIED_UP_TO).is_err());
    }

    #[test]
    fn what_no_prose_names_any_more_is_counted() {
        let (_src, one) = dropped("kept.png", b"still referenced");
        let (_other, two) = dropped("gone.png", b"nobody points here");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, .. } = keep(&one, root.path(), COPIED_UP_TO).unwrap();
        keep(&two, root.path(), COPIED_UP_TO).unwrap();

        let counted = loose(root.path(), &[at]);
        assert_eq!(counted.files, 1);
        assert_eq!(counted.bytes, b"nobody points here".len() as u64);

        assert_eq!(loose(root.path(), &[]).files, 2);
    }

    #[test]
    fn a_store_without_attachments_has_nothing_loose() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(loose(root.path(), &[]), Loose::default());
    }

    #[test]
    fn what_follows_a_question_mark_is_not_the_file() {
        let root = Path::new("/data");
        assert_eq!(
            resolve("attachments/ab/cd.png?v=2", root).unwrap(),
            root.join("attachments/ab/cd.png")
        );
    }
}
