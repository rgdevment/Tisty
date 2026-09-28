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
        Some(one) => format!("{}\n{one}\n", dest.display()),
        None => format!("{}\n", dest.display()),
    }
}

fn is_the_one(kept: &str, dest: &Path) -> bool {
    let mut lines = kept.lines().map(str::trim).filter(|one| !one.is_empty());
    let Some(path) = lines.next() else {
        return false;
    };
    if let (Some(kept), Some(now)) = (lines.next(), tisty_core::paths::told_apart(dest))
        && kept == now
    {
        return true;
    }
    path == dest.display().to_string() || as_written(Path::new(path)) == as_written(dest)
}
