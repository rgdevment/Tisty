use std::path::Path;

use crate::written;
use tisty_core::paths::as_written;

pub(crate) const CARRIED_TO: &str = "carried-to";

pub(crate) fn carried_here(aside: Option<&Path>, dest: &Path) -> bool {
    let Some(aside) = aside else {
        return false;
    };
    std::fs::read_to_string(aside.join(CARRIED_TO)).is_ok_and(|last| is_the_one(&last, dest))
}

pub(crate) fn note_carried(aside: Option<&Path>, dest: &Path) {
    let Some(aside) = aside else {
        return;
    };
    if std::fs::create_dir_all(aside).is_ok() {
        let _ = written(&aside.join(CARRIED_TO), said_of(dest).as_bytes());
    }
}

fn said_of(dest: &Path) -> String {
    match tisty_core::paths::told_apart(dest) {
        Some(one) => format!("{one}\n{}", dest.display()),
        None => dest.display().to_string(),
    }
}

fn a_mark(one: &str) -> bool {
    matches!(one.split_once(':'), Some((volume, at))
        if !volume.is_empty()
            && !at.is_empty()
            && [volume, at]
                .iter()
                .all(|one| one.bytes().all(|b| b.is_ascii_hexdigit())))
}

fn is_the_one(kept: &str, dest: &Path) -> bool {
    let (mark, path) = match kept.split_once('\n') {
        Some((one, rest)) if a_mark(one.trim()) => (Some(one.trim()), rest),
        _ => (None, kept),
    };
    let path = path.trim();
    if let (Some(mark), Some(now)) = (mark, tisty_core::paths::told_apart(dest)) {
        return mark == now;
    }
    !path.is_empty() && as_written(Path::new(path)) == as_written(dest)
}
