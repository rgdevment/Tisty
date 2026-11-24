use std::path::{Path, PathBuf};

use super::places::{Os, Place};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Leads {
    Here,
    Another,
    Gone,
    /// Nothing here can tell, so it is neither said to be broken nor written over.
    Unsaid,
}

impl Leads {
    pub fn astray(self) -> bool {
        matches!(self, Leads::Another | Leads::Gone)
    }
}

const MOUNTS: [&str; 5] = ["/Volumes/", "/media/", "/run/media/", "/mnt/", "/net/"];

pub fn leads(place: &Place, said: &str) -> Leads {
    let said = said.trim();
    if same(said, &place.command) {
        return Leads::Here;
    }
    let bare = !said.contains(['/', '\\']);
    let Some(at) = (match bare {
        true => place.on_path(said),
        false => sure(place, said),
    }) else {
        return Leads::Unsaid;
    };
    if here(place).is_some_and(|mine| one_file(&at, &mine)) {
        return Leads::Here;
    }
    match at.try_exists() {
        _ if bare => Leads::Unsaid,
        Ok(true) => Leads::Another,
        Ok(false) if volume_there(place, said, &at) => Leads::Gone,
        _ => Leads::Unsaid,
    }
}

fn here(place: &Place) -> Option<PathBuf> {
    match place.command.contains(['/', '\\']) {
        true => sure(place, &place.command),
        false => place.on_path(&place.command),
    }
}

// Paths on Windows and macOS ignore case, so the same file written two ways is still this Tisty.
pub fn same(one: &str, other: &str) -> bool {
    let tidy = |it: &str| it.trim().replace('\\', "/").to_lowercase();
    tidy(one) == tidy(other)
}

fn one_file(one: &Path, other: &Path) -> bool {
    same(&one.display().to_string(), &other.display().to_string())
        || matches!((one.canonicalize(), other.canonicalize()), (Ok(one), Ok(other)) if one == other)
}

fn windows(place: &Place) -> bool {
    place.os == Os::Windows && place.distro.is_none()
}

fn sure(place: &Place, said: &str) -> Option<PathBuf> {
    let rooted = match windows(place) {
        true => drive(said).is_some(),
        false => said.starts_with('/') && !said.starts_with("//"),
    };
    if !rooted || said.contains(['$', '%', '"', '\n']) || !named_tisty(place, said) {
        return None;
    }
    match place.distro {
        Some(_) => place.reached(said),
        None => Some(PathBuf::from(said)),
    }
}

fn named_tisty(place: &Place, said: &str) -> bool {
    let name = match windows(place) {
        true => said.rsplit(['/', '\\']).next(),
        false => said.rsplit('/').next(),
    };
    name.is_some_and(|name| {
        ["tisty", "tisty.exe"]
            .iter()
            .any(|one| name.eq_ignore_ascii_case(one))
    })
}

fn drive(said: &str) -> Option<String> {
    let raw = said.as_bytes();
    let rooted = raw.len() > 2
        && raw[0].is_ascii_alphabetic()
        && raw[1] == b':'
        && matches!(raw[2], b'\\' | b'/');
    rooted.then(|| format!("{}:\\", raw[0] as char))
}

fn volume_there(place: &Place, said: &str, at: &Path) -> bool {
    let on_a_drive = drive(&at.display().to_string());
    if let Some(root) = on_a_drive.filter(|_| windows(place) || place.distro.is_some()) {
        return Path::new(&root).try_exists().unwrap_or(false);
    }
    place.distro.is_some() || !MOUNTS.iter().any(|under| said.starts_with(under))
}

#[cfg(test)]
#[path = "leads_test.rs"]
mod tests;
