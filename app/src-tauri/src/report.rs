use std::path::Path;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Facts {
    pub version: String,
    pub dev: bool,
    pub sandbox: Option<String>,
    pub locale: String,
    pub zone: String,
    pub os: String,
    pub arch: &'static str,
    pub webview: Option<String>,
    pub store: String,
    pub devices: usize,
    pub events: usize,
    pub open: usize,
    pub archived: usize,
    pub lists: usize,
    pub tags: usize,
    pub list_names: Vec<String>,
    pub tag_names: Vec<String>,
    pub cache: &'static str,
    pub attachments: usize,
    pub attachment_bytes: u64,
    pub loose: usize,
    pub loose_bytes: u64,
    pub weight: u64,
    pub syncs: bool,
    pub shared: bool,
    pub backed_up_at: Option<String>,
    pub quiet: Vec<String>,
    pub attach_up_to: u64,
    pub in_path: bool,
    pub shortcut: Option<String>,
}

pub use tisty_core::witness::hidden;

pub fn weighed(root: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(root) else {
        return 0;
    };
    entries
        .filter_map(|one| one.ok())
        .map(|one| match one.file_type() {
            Ok(kind) if kind.is_dir() => weighed(&one.path()),
            Ok(_) => one.metadata().map(|m| m.len()).unwrap_or(0),
            Err(_) => 0,
        })
        .sum()
}

pub struct Held {
    pub files: usize,
    pub bytes: u64,
}

pub fn attachments(root: &Path) -> Held {
    let mut held = Held { files: 0, bytes: 0 };
    let Ok(shelves) = std::fs::read_dir(root.join("attachments")) else {
        return held;
    };
    for shelf in shelves.filter_map(|one| one.ok()) {
        let Ok(files) = std::fs::read_dir(shelf.path()) else {
            continue;
        };
        for file in files.filter_map(|one| one.ok()) {
            held.files += 1;
            held.bytes += file.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    held
}

pub fn devices(store: &Path) -> usize {
    std::fs::read_dir(store)
        .map(|entries| {
            entries
                .filter_map(|one| one.ok())
                .filter(|one| one.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
                .count()
        })
        .unwrap_or(0)
}

#[cfg(windows)]
pub fn os() -> String {
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
    else {
        return "Windows".into();
    };

    let name: String = key
        .get_value("ProductName")
        .unwrap_or_else(|_| "Windows".into());
    let build: String = key.get_value("CurrentBuild").unwrap_or_default();
    let display: String = key.get_value("DisplayVersion").unwrap_or_default();
    let revision: u32 = key.get_value("UBR").unwrap_or(0);

    let eleven = build.parse::<u32>().map(|n| n >= 22000).unwrap_or(false);
    let name = if eleven {
        name.replace("Windows 10", "Windows 11")
    } else {
        name
    };

    let mut said = name;
    if !display.is_empty() {
        said.push_str(&format!(" {display}"));
    }
    if !build.is_empty() {
        said.push_str(&format!(" (10.0.{build}.{revision})"));
    }
    said
}

#[cfg(target_os = "macos")]
pub fn os() -> String {
    let plist = std::fs::read_to_string("/System/Library/CoreServices/SystemVersion.plist")
        .unwrap_or_default();
    match after(&plist, "<key>ProductVersion</key>") {
        Some(version) => format!("macOS {version}"),
        None => "macOS".into(),
    }
}

#[cfg(target_os = "macos")]
fn after(plist: &str, key: &str) -> Option<String> {
    let rest = plist.split_once(key)?.1;
    let open = rest.find("<string>")? + "<string>".len();
    let shut = rest[open..].find("</string>")? + open;
    Some(rest[open..shut].trim().to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn os() -> String {
    let release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    release
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|name| name.trim_matches('"').to_string())
        .unwrap_or_else(|| "Linux".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_store_weighs_nothing_instead_of_failing() {
        let tmp = tempfile::tempdir().unwrap();

        assert_eq!(weighed(&tmp.path().join("absent")), 0);
        assert_eq!(devices(&tmp.path().join("absent")), 0);
        assert_eq!(attachments(tmp.path()).files, 0);
    }

    #[test]
    fn what_is_under_the_root_is_weighed_all_the_way_down() {
        let tmp = tempfile::tempdir().unwrap();
        let deep = tmp.path().join("store").join("dev_a");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join("0001.jsonl"), b"0123456789").unwrap();
        std::fs::write(tmp.path().join("loose.txt"), b"12345").unwrap();

        assert_eq!(weighed(tmp.path()), 15);
        assert_eq!(devices(&tmp.path().join("store")), 1);
    }

    #[test]
    fn attachments_are_counted_across_shelves() {
        let tmp = tempfile::tempdir().unwrap();
        for shelf in ["2026-08", "2026-07"] {
            let at = tmp.path().join("attachments").join(shelf);
            std::fs::create_dir_all(&at).unwrap();
            std::fs::write(at.join("one.pdf"), b"1234").unwrap();
        }

        let held = attachments(tmp.path());
        assert_eq!(held.files, 2);
        assert_eq!(held.bytes, 8);
    }

    #[test]
    fn the_operating_system_says_something_it_could_be_asked_about() {
        let said = os();
        assert!(!said.is_empty());
        assert!(said.chars().any(|c| c.is_alphabetic()), "{said}");
    }
}
