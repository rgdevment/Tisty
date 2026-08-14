use std::io::Read;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{
    Error, Result,
    witness::{self, Fact, channel},
};

pub const COPIED_UP_TO: u64 = 50 * 1024 * 1024;
pub const COPIED_IN_DOC: u64 = 500 * 1024 * 1024;

const SHORTENS_TO: usize = 56;

pub const COPIED_LEAST: u64 = 64 * 1024;
pub const COPIED_MOST: u64 = COPIED_IN_DOC;

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

    if let Some(at) = listed(root, &sha256) {
        match resolve(&at, root) {
            Ok(held) if holds(&held, &bytes) => return Ok(Kept { at, sha256 }),
            Ok(held) => witness::warn(
                channel::ATTACH,
                "what the ledger points at is not what it says it is",
                &[
                    ("at", Fact::Path(held)),
                    ("sha256", Fact::Id(sha256.clone())),
                ],
            ),
            Err(_) => witness::warn(
                channel::ATTACH,
                "the ledger names a path outside the store",
                &[("at", Fact::Id(at)), ("sha256", Fact::Id(sha256.clone()))],
            ),
        }
    }

    let name = match already(&folder, stamp, &bytes) {
        Some(kept) => kept,
        None => {
            let mut name = named(source, stamp, &ext);
            if folder.join(&name).exists() {
                name = named(source, &rest[..16], &ext);
            }
            let target = folder.join(&name);
            std::fs::write(&target, &bytes)?;
            let _ = crate::paths::ours_alone(&target);
            name
        }
    };
    let kept = Kept {
        at: format!("attachments/{shelf}/{name}"),
        sha256,
    };
    note(root, &kept, bytes.len() as u64);
    Ok(kept)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Noted {
    at: String,
    sha256: String,
    bytes: u64,
}

fn ledger(root: &Path) -> PathBuf {
    root.join("attachments.jsonl")
}

fn listed(root: &Path, sha256: &str) -> Option<String> {
    let text = std::fs::read_to_string(ledger(root)).ok()?;
    text.lines()
        .filter_map(|line| serde_json::from_str::<Noted>(line).ok())
        .find(|one| one.sha256 == sha256)
        .map(|one| one.at)
}

fn holds(at: &Path, bytes: &[u8]) -> bool {
    std::fs::metadata(at).is_ok_and(|held| held.is_file() && held.len() == bytes.len() as u64)
        && std::fs::read(at).is_ok_and(|held| held == bytes)
}

fn tailed(at: &Path) -> bool {
    let Ok(mut file) = std::fs::File::open(at) else {
        return true;
    };
    let Ok(size) = file.metadata().map(|one| one.len()) else {
        return true;
    };
    if size == 0 {
        return true;
    }
    use std::io::{Read, Seek};
    if file.seek(std::io::SeekFrom::End(-1)).is_err() {
        return true;
    }
    let mut last = [0u8; 1];
    file.read_exact(&mut last).is_ok_and(|()| last[0] == b'\n')
}

fn note(root: &Path, kept: &Kept, bytes: u64) {
    if listed(root, &kept.sha256).is_some() {
        return;
    }
    let line = match serde_json::to_string(&Noted {
        at: kept.at.clone(),
        sha256: kept.sha256.clone(),
        bytes,
    }) {
        Ok(line) => line,
        Err(_) => return,
    };
    let at = ledger(root);
    let whole = if tailed(&at) {
        format!("{line}\n")
    } else {
        witness::warn(
            channel::ATTACH,
            "the ledger had no newline to append after",
            &[("at", Fact::Path(at.clone()))],
        );
        format!("\n{line}\n")
    };
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&at)
        .and_then(|mut file| std::io::Write::write_all(&mut file, whole.as_bytes()));
    let _ = crate::paths::ours_alone(&at);
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

fn already(folder: &Path, stamp: &str, bytes: &[u8]) -> Option<String> {
    std::fs::read_dir(folder)
        .ok()?
        .filter_map(|one| one.ok())
        .find_map(|one| {
            let name = one.file_name().to_str()?.to_string();
            let stem = name.split('.').next().unwrap_or(&name);
            if stem != stamp && !stem.ends_with(&format!("-{stamp}")) {
                return None;
            }
            let held = one.metadata().ok()?;
            if held.len() != bytes.len() as u64 {
                return None;
            }
            (std::fs::read(one.path()).ok()? == bytes).then_some(name)
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
    fn a_stamp_worn_by_unlike_bytes_does_not_hand_back_the_wrong_file() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("mine.bin", b"the bytes that are mine");
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        let stem = kept.at.rsplit('/').next().unwrap().to_string();
        let stamp = stem
            .split('.')
            .next()
            .unwrap()
            .rsplit('-')
            .next()
            .unwrap()
            .to_string();
        let shelf = kept.at.split('/').nth(1).unwrap().to_string();
        let impostor = root
            .path()
            .join("attachments")
            .join(&shelf)
            .join(format!("impostor-{stamp}.bin"));
        std::fs::write(&impostor, b"entirely other bytes, same stamp").unwrap();
        std::fs::remove_file(root.path().join("attachments.jsonl")).unwrap();

        let again = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(again.at, kept.at, "the bytes decide, never the name");
        assert_eq!(
            std::fs::read(root.path().join(&again.at)).unwrap(),
            b"the bytes that are mine"
        );
    }

    #[test]
    fn the_digest_is_written_down_whole_so_it_can_be_asked_for() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("mine.bin", b"some bytes worth keeping");

        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        let noted = std::fs::read_to_string(root.path().join("attachments.jsonl")).unwrap();
        assert!(
            noted.contains(&kept.sha256),
            "the whole digest, not ten characters"
        );
        assert_eq!(kept.sha256.len(), 64);
        assert!(noted.contains(&kept.at));
        assert!(noted.contains("24"), "the weight travels with it");
    }

    #[test]
    fn what_is_written_down_saves_the_search_but_never_the_checking() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("mine.bin", b"kept once");
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        let (_other, again) = dropped("elsewhere.bin", b"kept once");
        let second = keep(&again, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(second.at, kept.at);
        assert_eq!(
            std::fs::read(root.path().join(&second.at)).unwrap(),
            b"kept once"
        );
    }

    #[test]
    fn a_kept_file_changed_underneath_is_written_again_rather_than_handed_back() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("mine.bin", b"the only copy of my report");
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        std::fs::write(root.path().join(&kept.at), b"junk").unwrap();
        let again = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert_eq!(
            std::fs::read(root.path().join(&again.at)).unwrap(),
            b"the only copy of my report",
            "the ledger said it was kept, but the bytes said otherwise"
        );
    }

    #[test]
    fn a_ledger_that_lost_its_last_newline_does_not_swallow_the_entry_after_it() {
        let root = tempfile::tempdir().unwrap();
        let (_src, first) = dropped("uno.bin", b"the first one");
        let one = keep(&first, root.path(), COPIED_UP_TO).unwrap();

        let ledger = root.path().join("attachments.jsonl");
        let held = std::fs::read_to_string(&ledger).unwrap();
        std::fs::write(&ledger, held.trim_end()).unwrap();

        let (_other, second) = dropped("dos.bin", b"the second one");
        let two = keep(&second, root.path(), COPIED_UP_TO).unwrap();

        let written = std::fs::read_to_string(&ledger).unwrap();
        let lines: Vec<&str> = written.lines().filter(|one| !one.is_empty()).collect();
        assert_eq!(lines.len(), 2, "one line ate the other: {lines:?}");
        for line in lines {
            serde_json::from_str::<Noted>(line).expect("still readable");
        }
        assert_ne!(one.at, two.at);
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

    #[test]
    fn a_line_that_arrived_half_written_is_skipped_not_fatal() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("attachments.jsonl"),
            "{\"at\":\"attachments/ab/cut-off\",\"sha256\":\"dead\n{\"at\":\"attachments/ab/real.bin\",\"sha256\":\"beef\",\"bytes\":3}\n",
        )
        .unwrap();

        assert_eq!(
            listed(root.path(), "beef"),
            Some("attachments/ab/real.bin".to_string())
        );
        assert_eq!(listed(root.path(), "dead"), None);
    }

    #[test]
    fn a_line_that_is_not_json_at_all_does_not_hide_the_line_beside_it() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("attachments.jsonl"),
            "not json at all\n{\"at\":\"attachments/ab/real.bin\",\"sha256\":\"beef\",\"bytes\":3}\n",
        )
        .unwrap();

        assert_eq!(
            listed(root.path(), "beef"),
            Some("attachments/ab/real.bin".to_string())
        );
    }

    #[test]
    fn an_empty_ledger_file_is_read_as_if_nothing_had_been_noted() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("attachments.jsonl"), "").unwrap();

        assert_eq!(listed(root.path(), "anything"), None);
    }

    #[cfg(unix)]
    #[test]
    fn a_ledger_the_process_cannot_read_is_treated_as_unwritten() {
        use std::os::unix::fs::PermissionsExt;
        let root = tempfile::tempdir().unwrap();
        let ledger_at = root.path().join("attachments.jsonl");
        std::fs::write(
            &ledger_at,
            "{\"at\":\"attachments/ab/real.bin\",\"sha256\":\"beef\",\"bytes\":3}\n",
        )
        .unwrap();
        std::fs::set_permissions(&ledger_at, std::fs::Permissions::from_mode(0o000)).unwrap();

        let found = listed(root.path(), "beef");

        std::fs::set_permissions(&ledger_at, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            found, None,
            "unreadable, so nothing was found rather than trusted"
        );
    }

    #[test]
    fn a_ledger_entry_whose_file_is_gone_is_not_trusted_blindly() {
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("informe.pdf", b"the only copy of the report");

        let first = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        std::fs::remove_file(root.path().join(&first.at)).unwrap();

        let second = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert!(
            root.path().join(&second.at).is_file(),
            "keep() handed back a path with nothing behind it: {}",
            second.at
        );
        assert_eq!(
            std::fs::read(root.path().join(&second.at)).unwrap(),
            b"the only copy of the report"
        );
    }

    #[test]
    fn when_the_ledger_repeats_a_hash_the_first_line_written_is_the_one_trusted() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("attachments.jsonl"),
            "{\"at\":\"attachments/ab/first.bin\",\"sha256\":\"dupe\",\"bytes\":1}\n{\"at\":\"attachments/ab/second.bin\",\"sha256\":\"dupe\",\"bytes\":1}\n",
        )
        .unwrap();

        assert_eq!(
            listed(root.path(), "dupe"),
            Some("attachments/ab/first.bin".to_string())
        );
    }

    #[test]
    fn a_name_already_taken_by_other_content_gets_a_longer_stamp_instead_of_being_overwritten() {
        let root = tempfile::tempdir().unwrap();
        let bytes = b"the real bytes of the real file";
        let sha256 = fingerprint(bytes);
        let (shelf, rest) = sha256.split_at(2);
        let stamp = &rest[..8];
        let folder = root.path().join("attachments").join(shelf);
        std::fs::create_dir_all(&folder).unwrap();
        let squatted = folder.join(format!("clash-{stamp}.bin"));
        std::fs::write(&squatted, b"unrelated content squatting the name").unwrap();

        let (_src, file) = dropped("clash.bin", bytes);
        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert_ne!(
            kept.at.rsplit('/').next().unwrap(),
            format!("clash-{stamp}.bin"),
            "the real file was written under the squatter's name"
        );
        assert_eq!(
            std::fs::read(&squatted).unwrap(),
            b"unrelated content squatting the name",
            "the squatter was clobbered"
        );
        assert_eq!(std::fs::read(root.path().join(&kept.at)).unwrap(), bytes);
    }

    #[test]
    fn a_file_one_byte_over_the_limit_is_refused() {
        let (_src, file) = dropped("shot.png", b"12345");
        let root = tempfile::tempdir().unwrap();

        let refused = keep(&file, root.path(), 4).unwrap_err();

        assert!(
            matches!(refused, Error::AttachmentTooBig { bytes: 5, limit: 4 }),
            "{refused:?}"
        );
    }

    #[test]
    fn a_name_made_only_of_dots_still_lands_as_a_normal_file() {
        let kept = stored("...png", b"g");

        assert!(!kept.starts_with('.'), "{kept}");
        assert!(kept.ends_with(".png"), "{kept}");
    }

    #[test]
    fn a_name_of_pure_emoji_falls_back_to_the_stamp_instead_of_writing_pictographs_to_disk() {
        let kept = stored("😀😀.png", b"h");

        assert!(kept.is_ascii(), "{kept}");
        assert!(kept.ends_with(".png"), "{kept}");
    }

    #[test]
    fn a_name_three_hundred_characters_long_is_still_cut_to_the_limit() {
        let source = Path::new("").join(format!("{}.txt", "x".repeat(300)));

        let named = named(&source, "12345678", ".txt");

        assert_eq!(named, format!("{}-12345678.txt", "x".repeat(SHORTENS_TO)));
    }

    #[test]
    fn a_ledger_entry_is_not_trusted_when_it_climbs_out_of_the_store() {
        let outside = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();
        let (_src, file) = dropped("informe.pdf", b"the only copy of my report");

        let kept = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        std::fs::remove_file(root.path().join(&kept.at)).unwrap();

        let planted = outside.path().join("not-an-attachment.pdf");
        std::fs::write(&planted, b"something else entirely").unwrap();
        let climbing = format!(
            "../{}/not-an-attachment.pdf",
            outside.path().file_name().unwrap().to_str().unwrap()
        );
        std::fs::write(
            root.path().join("attachments.jsonl"),
            format!(
                "{{\"at\":\"{climbing}\",\"sha256\":\"{}\",\"bytes\":7}}\n",
                kept.sha256
            ),
        )
        .unwrap();

        let again = keep(&file, root.path(), COPIED_UP_TO).unwrap();

        assert!(
            !again.at.contains(".."),
            "a corrupted ledger line handed back a path that climbs out of the store: {}",
            again.at
        );
    }
}
