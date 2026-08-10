//! Files brought in from outside. What reaches the prose is an ordinary
//! Markdown link to a path under the data root, so the file stays readable
//! without Tisty and a Windows path never travels to a Linux machine.

use std::io::Read;
use std::path::{Component, Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::{Error, Result};

/// Above this the file is left where it is and only its path is kept: a 40 MB
/// video inside the store is a store nobody can move again.
pub const COPIED_UP_TO: u64 = 5 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kept {
    /// Copied in; the reference is relative to the data root.
    Held { at: String, sha256: String },
    /// Left outside, so it only exists on the machine that has it.
    Named { at: PathBuf },
}

impl Kept {
    /// The Markdown that goes into the description or the journal entry. The
    /// target is wrapped, or a Windows path with a space stops being a link.
    pub fn written(&self, label: &str) -> String {
        let name = spoken(label);
        match self {
            Kept::Held { at, .. } if pictorial(at) => format!("![{name}](<{at}>)"),
            Kept::Held { at, .. } => format!("[{name}](<{at}>)"),
            Kept::Named { at } => format!("[{name}](<{}>)", at.display()),
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

/// Copies under the threshold, points above it. Never rewrites an existing
/// file: same contents means same name, so a second copy is a no-op.
pub fn keep(source: &Path, root: &Path, limit: u64) -> Result<Kept> {
    let mut file = std::fs::File::open(source)?;
    // Not `metadata(path)`: a pipe or a device reports nothing and then hands
    // over as much as it likes, and the file can grow between the two calls.
    let opened = file.metadata()?;
    if !opened.is_file() {
        return Err(Error::OutsideTheStore(source.display().to_string()));
    }
    if opened.len() > limit {
        return Ok(Kept::Named {
            at: source.to_path_buf(),
        });
    }

    let mut bytes = Vec::new();
    let read = file.by_ref().take(limit + 1).read_to_end(&mut bytes)? as u64;
    if read > limit {
        return Ok(Kept::Named {
            at: source.to_path_buf(),
        });
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

    let target = folder.join(&name);
    if !target.exists() {
        std::fs::write(&target, &bytes)?;
    }
    Ok(Kept::Held {
        at: format!("attachments/{shelf}/{name}"),
        sha256,
    })
}

/// Rejects anything that climbs out of the data root, whoever wrote it.
pub fn resolve(reference: &str, root: &Path) -> Result<PathBuf> {
    let cleaned = reference.split(['?', '#']).next().unwrap_or("");
    let refused = || Err(Error::OutsideTheStore(reference.to_string()));
    if cleaned.is_empty() {
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

        let Kept::Held { at, sha256 } = keep(&file, root.path(), COPIED_UP_TO).unwrap() else {
            panic!("a small file must be copied in");
        };

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
    fn a_heavy_file_is_pointed_at_and_never_copied() {
        let (_src, file) = dropped("recording.mkv", b"pretend this is fifteen gigabytes");
        let root = tempfile::tempdir().unwrap();

        let kept = keep(&file, root.path(), 4).unwrap();

        assert_eq!(kept, Kept::Named { at: file.clone() });
        assert!(
            !root.path().join("attachments").exists(),
            "nothing was copied in"
        );
    }

    #[test]
    fn a_picture_is_shown_and_everything_else_is_linked() {
        let held = |at: &str| Kept::Held {
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
        let kept = Kept::Named {
            at: PathBuf::from(r"C:\Users\Mario\My Docs\clip (1).mkv"),
        };
        let written = kept.written("clip (1).mkv");
        assert!(written.starts_with("[clip (1).mkv](<"), "{written}");
        assert!(written.ends_with(">)"), "{written}");
    }

    #[test]
    fn a_name_that_would_break_the_link_is_flattened() {
        let one = Kept::Held {
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

        let Kept::Held { at, .. } = keep(&file, root.path(), COPIED_UP_TO).unwrap() else {
            panic!("it should be copied in");
        };
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

        assert!(matches!(
            keep(&file, root.path(), 4).unwrap(),
            Kept::Held { .. }
        ));
    }

    #[test]
    fn a_file_without_an_extension_keeps_its_hash_alone() {
        let (_src, file) = dropped("README", b"no extension here");
        let root = tempfile::tempdir().unwrap();

        let Kept::Held { at, sha256 } = keep(&file, root.path(), COPIED_UP_TO).unwrap() else {
            panic!("it should be copied in");
        };
        assert!(at.ends_with(&sha256[2..]), "{at}");
    }

    #[test]
    fn a_directory_is_not_a_file_and_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = tempfile::tempdir().unwrap();

        assert!(keep(dir.path(), root.path(), COPIED_UP_TO).is_err());
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
