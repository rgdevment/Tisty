use std::collections::HashSet;
use std::fs::Permissions;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use super::Stuck;

const BEFORE: &str = "before-tisty";
const LAST: &str = "before-tisty-last";
const WRITING: Duration = Duration::from_millis(60);

#[derive(Debug, PartialEq, Eq)]
pub enum Read {
    Missing,
    Unread,
    Text(String),
}

pub fn read(at: &Path) -> Read {
    match std::fs::read(at) {
        Ok(bytes) => String::from_utf8(bytes).map_or(Read::Unread, Read::Text),
        Err(why) if why.kind() == ErrorKind::NotFound && !there(at) => Read::Missing,
        Err(_) => Read::Unread,
    }
}

fn there(at: &Path) -> bool {
    std::fs::symlink_metadata(at).is_ok()
}

// An agent empties its file before it writes it again, so one seen empty is looked at once more.
pub fn text(at: &Path) -> Result<String, Stuck> {
    match settled(|| read(at)) {
        Read::Text(text) => Ok(text),
        Read::Missing => Ok(String::new()),
        Read::Unread => Err(Stuck::Unread(at.display().to_string())),
    }
}

fn settled(mut look: impl FnMut() -> Read) -> Read {
    match look() {
        Read::Text(text) if text.is_empty() => {
            std::thread::sleep(WRITING);
            look()
        }
        seen => seen,
    }
}

#[derive(Default)]
pub struct Writes {
    kept: HashSet<PathBuf>,
}

impl Writes {
    pub fn change(&mut self, at: &Path, was: &str, now: &str, aside: bool) -> Result<(), Stuck> {
        if now == was {
            return Ok(());
        }
        unmoved(at, was)?;
        self.keep(at, was)?;
        laid(at, was, now, aside)
    }

    pub fn remove(&mut self, at: &Path, was: &str) -> Result<(), Stuck> {
        unmoved(at, was)?;
        self.keep(at, was)?;
        unmoved(at, was)?;
        match std::fs::remove_file(at) {
            Err(why) if why.kind() != ErrorKind::NotFound => Err(sour(why)),
            _ => Ok(()),
        }
    }

    fn keep(&mut self, at: &Path, was: &str) -> Result<(), Stuck> {
        if was.is_empty() || !self.kept.insert(at.to_path_buf()) {
            return Ok(());
        }
        let Some(named) = at.file_name().map(|it| it.to_string_lossy().into_owned()) else {
            return Ok(());
        };
        let dress = permissions(at);
        let first = at.with_file_name(format!("{named}.{BEFORE}"));
        if !first.exists() {
            copied(&first, was, dress.clone())?;
        }
        copied(&at.with_file_name(format!("{named}.{LAST}")), was, dress)
    }
}

fn permissions(at: &Path) -> Option<Permissions> {
    std::fs::metadata(at).ok().map(|meta| meta.permissions())
}

fn copied(to: &Path, text: &str, dress: Option<Permissions>) -> Result<(), Stuck> {
    tisty_core::store::write_atomic(to, text.as_bytes()).map_err(cannot)?;
    dressed(to, dress);
    Ok(())
}

fn dressed(at: &Path, dress: Option<Permissions>) {
    if let Some(dress) = dress.filter(|_| cfg!(unix)) {
        let _ = std::fs::set_permissions(at, dress);
    }
}

pub fn laid(at: &Path, was: &str, text: &str, aside: bool) -> Result<(), Stuck> {
    if let Some(folder) = at.parent() {
        std::fs::create_dir_all(folder).map_err(sour)?;
    }
    let target = led_to(at)?;
    if aside && target.is_file() {
        unmoved(at, was)?;
        return tisty_core::store::write_in_place(&target, text.as_bytes()).map_err(cannot);
    }
    let beside = beside(&target);
    let done = copied(&beside, text, permissions(&target))
        .and_then(|()| renamed(&beside, &target, || unmoved(at, was)));
    if done.is_err() {
        let _ = std::fs::remove_file(&beside);
    }
    done
}

fn unmoved(at: &Path, was: &str) -> Result<(), Stuck> {
    match read(at) {
        Read::Text(text) if text == was => Ok(()),
        Read::Missing if was.is_empty() => Ok(()),
        Read::Unread => Err(Stuck::Unread(at.display().to_string())),
        _ => Err(Stuck::Moved(at.display().to_string())),
    }
}

fn beside(target: &Path) -> PathBuf {
    let named = target.file_name().unwrap_or_default().to_string_lossy();
    target.with_file_name(format!("{named}.{}.tmp", ulid::Ulid::generate()))
}

// Windows refuses for a moment a file somebody has open, and every try may find it changed.
fn renamed(from: &Path, to: &Path, unmoved: impl Fn() -> Result<(), Stuck>) -> Result<(), Stuck> {
    let mut wait = Duration::from_millis(10);
    loop {
        unmoved()?;
        match std::fs::rename(from, to) {
            Err(why) if held(&why) && wait < Duration::from_millis(700) => {
                std::thread::sleep(wait);
                wait *= 2;
            }
            done => return done.map_err(sour),
        }
    }
}

fn held(why: &std::io::Error) -> bool {
    cfg!(windows) && matches!(why.raw_os_error(), Some(5 | 32))
}

// Renaming over a link would leave the file it leads to, in a repository of the person's, without the change.
fn led_to(at: &Path) -> Result<PathBuf, Stuck> {
    match std::fs::symlink_metadata(at).is_ok_and(|meta| meta.is_symlink()) {
        true => at.canonicalize().map_err(sour),
        false => Ok(at.to_path_buf()),
    }
}

fn cannot(why: tisty_core::Error) -> Stuck {
    Stuck::Cannot(why.to_string())
}

fn sour(why: std::io::Error) -> Stuck {
    Stuck::Cannot(why.to_string())
}

#[cfg(test)]
#[path = "files_test.rs"]
mod tests;
