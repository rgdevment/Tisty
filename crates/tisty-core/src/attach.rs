//! Files brought in from outside. What reaches the prose is an ordinary
//! Markdown link to a path under the data root, so the file stays readable
//! without Tisty and a Windows path never travels to a Linux machine.

use std::io::Read;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{
    Error, Result,
    witness::{self, Fact, channel},
};

/// Above this the file is left where it is and only its path is kept: a 40 MB
/// video inside the store is a store nobody can move again.
pub const COPIED_UP_TO: u64 = 5 * 1024 * 1024;

/// The band the setting may move in. Below the floor nothing would ever be
/// copied; above the ceiling the store stops being a thing you can move.
pub const COPIED_LEAST: u64 = 64 * 1024;
pub const COPIED_MOST: u64 = 200 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Always copied in. Pointing at a file where it already lives made an
/// attachment that existed on one machine and nowhere else, and that survived
/// exactly until somebody moved it.
pub struct Kept {
    /// Relative to the data root.
    pub at: String,
    pub sha256: String,
}

impl Kept {
    /// The Markdown that goes into the description or the journal entry. The
    /// target is wrapped, or a Windows path with a space stops being a link.
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

/// Brackets and line breaks in a file name would cut the link in half.
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

/// Copies under the threshold, points above it.
pub fn keep(source: &Path, root: &Path, limit: u64) -> Result<Kept> {
    let mut file = std::fs::File::open(source)?;
    // Not `metadata(path)`: a pipe reports nothing and then hands over anything.
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
    let name = format!("{rest}{ext}");
    let folder = root.join("attachments").join(shelf);
    std::fs::create_dir_all(&folder)?;
    let _ = crate::paths::ours_alone(root);
    let _ = crate::paths::ours_alone(&root.join("attachments"));
    let _ = crate::paths::ours_alone(&folder);

    let target = folder.join(&name);
    if !target.exists() {
        std::fs::write(&target, &bytes)?;
        let _ = crate::paths::ours_alone(&target);
    }
    Ok(Kept {
        at: format!("attachments/{shelf}/{name}"),
        sha256,
    })
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Loose {
    pub files: usize,
    pub bytes: u64,
}

/// Files under `attachments/` that no prose names any more. Collecting them is
/// a separate decision: another machine may still reference what this one dropped.
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

/// Rejects anything that climbs out of the data root, whoever wrote it.
pub fn resolve(reference: &str, root: &Path) -> Result<PathBuf> {
    let cleaned = reference.split(['?', '#']).next().unwrap_or("");
    let refused = || Err(Error::OutsideTheStore(reference.to_string()));
    if cleaned.is_empty() {
        return refused();
    }

    // A reference is written with `/` wherever it was made. Windows eats a
    // backslash as a separator and everywhere else it is an ordinary
    // letter, so one left in would name two different things.
    if cleaned.contains('\\') {
        return refused();
    }

    let mut walked = root.to_path_buf();
    let mut steps = 0;
    // By component, not by substring: `C:foo` carries a prefix and no root, so
    // `is_absolute` says no and `join` would still replace the whole path.
    for part in Path::new(cleaned).components() {
        let Component::Normal(name) = part else {
            return refused();
        };
        let Some(name) = name.to_str() else {
            return refused();
        };
        // A colon opens an NTFS stream; the rest name devices, not files.
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

/// An extension is letters and digits. Anything else is someone being clever.
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

    /// Pointing at a file where it lives made an attachment that existed on one
    /// machine and nowhere else, and only until somebody moved it. It is
    /// refused now, and the refusal says both numbers.
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

    /// The usual Windows path has spaces and brackets, and both cut a link.
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

    /// Windows keeps a per-drive current directory, so `C:foo` has a prefix
    /// and no root: `is_absolute` says no and `join` replaces everything.
    #[test]
    fn a_drive_letter_without_a_root_is_still_a_way_out() {
        let root = Path::new("/data");
        for climbing in ["C:foo", "attachments/ab/cd.png:hidden", "//server/share"] {
            assert!(resolve(climbing, root).is_err(), "«{climbing}» got through");
        }
    }

    /// One store is opened from more than one machine, so a reference has to
    /// mean the same thing on all of them.
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

    /// The old guard matched the substring, so an ordinary name was refused.
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

    /// The threshold is «up to», so the exact size is still copied in.
    #[test]
    fn a_file_the_exact_size_of_the_limit_is_copied() {
        let (_src, file) = dropped("shot.png", b"1234");
        let root = tempfile::tempdir().unwrap();

        assert!(keep(&file, root.path(), 4).is_ok());
    }

    #[test]
    fn a_file_without_an_extension_keeps_its_hash_alone() {
        let (_src, file) = dropped("README", b"no extension here");
        let root = tempfile::tempdir().unwrap();

        let Kept { at, sha256 } = keep(&file, root.path(), COPIED_UP_TO).unwrap();
        assert!(at.ends_with(&sha256[2..]), "{at}");
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

    /// A query string is not part of the path and would hide a climb.
    #[test]
    fn what_follows_a_question_mark_is_not_the_file() {
        let root = Path::new("/data");
        assert_eq!(
            resolve("attachments/ab/cd.png?v=2", root).unwrap(),
            root.join("attachments/ab/cd.png")
        );
    }
}
