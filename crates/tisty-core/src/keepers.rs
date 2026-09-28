use std::path::{Path, PathBuf};

pub const DRIVE: &str = "drive";
pub const ONEDRIVE: &str = "onedrive";
pub const ICLOUD: &str = "icloud";
pub const DROPBOX: &str = "dropbox";

pub const OURS: &str = "Tisty";

const MINE: [&str; 8] = [
    "My Drive",
    "Mi unidad",
    "Mon Drive",
    "Meine Ablage",
    "Il mio Drive",
    "Meu Drive",
    "Mijn Drive",
    "マイドライブ",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offer {
    pub key: &'static str,
    pub named: &'static str,
    pub at: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keeper {
    Cloud(&'static str),
    Away,
    Plain,
}

pub fn offers() -> Vec<Offer> {
    let home = directories::UserDirs::new().map(|dirs| dirs.home_dir().to_path_buf());
    ordered()
        .into_iter()
        .map(|(key, named)| Offer {
            key,
            named,
            at: home.as_deref().and_then(|home| found(key, home)),
        })
        .collect()
}

pub fn keeper(at: &Path) -> Keeper {
    whose(at, &offers())
}

pub fn whose(at: &Path, offers: &[Offer]) -> Keeper {
    let held = offers
        .iter()
        .filter_map(|one| one.at.as_deref().map(|root| (one.key, root)))
        .filter(|(_, root)| under(at, root))
        .max_by_key(|(_, root)| root.as_os_str().len());

    match held {
        Some((key, _)) => Keeper::Cloud(key),
        None if away(at) => Keeper::Away,
        None => Keeper::Plain,
    }
}

pub fn suggested(at: &Path) -> PathBuf {
    at.join(OURS)
}

#[cfg(windows)]
pub fn away(at: &Path) -> bool {
    let said = at.as_os_str().to_string_lossy();
    said.starts_with("\\\\") || said.starts_with("//")
}

#[cfg(target_os = "macos")]
pub fn away(at: &Path) -> bool {
    at.starts_with("/Volumes")
}

#[cfg(not(any(windows, target_os = "macos")))]
pub fn away(at: &Path) -> bool {
    at.starts_with("/mnt") || at.starts_with("/media") || at.starts_with("/net")
}

fn under(at: &Path, root: &Path) -> bool {
    if cfg!(windows) {
        return folded(at).starts_with(folded(root));
    }
    at.starts_with(root)
}

fn folded(at: &Path) -> PathBuf {
    PathBuf::from(at.as_os_str().to_string_lossy().to_lowercase())
}

#[cfg(windows)]
fn ordered() -> Vec<(&'static str, &'static str)> {
    vec![
        (DRIVE, "Google Drive"),
        (ONEDRIVE, "OneDrive"),
        (DROPBOX, "Dropbox"),
        (ICLOUD, "iCloud Drive"),
    ]
}

#[cfg(not(windows))]
fn ordered() -> Vec<(&'static str, &'static str)> {
    vec![
        (ICLOUD, "iCloud Drive"),
        (DRIVE, "Google Drive"),
        (ONEDRIVE, "OneDrive"),
        (DROPBOX, "Dropbox"),
    ]
}

#[cfg(windows)]
fn found(key: &str, home: &Path) -> Option<PathBuf> {
    match key {
        DRIVE => my_drive(),
        ONEDRIVE => ["OneDrive", "OneDriveConsumer", "OneDriveCommercial"]
            .iter()
            .filter_map(std::env::var_os)
            .map(PathBuf::from)
            .find(|at| at.is_dir()),
        DROPBOX => ["LOCALAPPDATA", "APPDATA"]
            .iter()
            .filter_map(std::env::var_os)
            .map(|at| PathBuf::from(at).join("Dropbox").join("info.json"))
            .find_map(|at| told_by(&at)),
        ICLOUD => here(home.join("iCloudDrive")),
        _ => None,
    }
}

#[cfg(not(windows))]
fn found(key: &str, home: &Path) -> Option<PathBuf> {
    match key {
        ICLOUD => here(home.join("Library/Mobile Documents/com~apple~CloudDocs")),
        DRIVE => beside(home, "GoogleDrive-").as_deref().and_then(mine),
        ONEDRIVE => beside(home, "OneDrive"),
        DROPBOX => beside(home, "Dropbox")
            .or_else(|| told_by(&home.join(".dropbox/info.json")))
            .or_else(|| here(home.join("Dropbox"))),
        _ => None,
    }
}

#[cfg(not(windows))]
fn beside(home: &Path, starting: &str) -> Option<PathBuf> {
    std::fs::read_dir(home.join("Library/CloudStorage"))
        .ok()?
        .filter_map(|one| one.ok())
        .map(|one| one.path())
        .filter(|at| at.is_dir())
        .find(|at| {
            at.file_name()
                .and_then(|named| named.to_str())
                .is_some_and(|named| named.starts_with(starting))
        })
}

#[cfg(windows)]
fn my_drive() -> Option<PathBuf> {
    let local = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
    if !local.join("Google").join("DriveFS").is_dir() {
        return None;
    }
    (b'C'..=b'Z').find_map(|letter| mine(&PathBuf::from(format!("{}:\\", letter as char))))
}

fn told_by(at: &Path) -> Option<PathBuf> {
    let said: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(at).ok()?).ok()?;
    ["personal", "business"]
        .iter()
        .filter_map(|which| said.get(which)?.get("path")?.as_str())
        .map(PathBuf::from)
        .find(|at| at.is_dir())
}

fn here(at: PathBuf) -> Option<PathBuf> {
    at.is_dir().then_some(at)
}

fn mine(root: &Path) -> Option<PathBuf> {
    MINE.iter()
        .map(|named| root.join(named))
        .find(|at| at.is_dir())
}

#[cfg(test)]
#[path = "keepers_test.rs"]
mod tests;
