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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Machine {
    pub id: String,
    pub called: String,
    pub when: i64,
    pub mine: bool,
}

pub fn machines(
    store: &Path,
    mine: &str,
    gone: &std::collections::BTreeSet<tisty_core::DeviceId>,
) -> Vec<Machine> {
    let Ok(entries) = std::fs::read_dir(store) else {
        return Vec::new();
    };
    let mut all: Vec<Machine> = entries
        .filter_map(|one| one.ok())
        .filter(|one| one.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .filter_map(|one| {
            let id = one.file_name().to_str()?.to_string();
            let when = tisty_core::store::segments_in(&one.path())
                .ok()?
                .iter()
                .filter_map(|at| std::fs::metadata(at).ok()?.modified().ok())
                .filter_map(|when| when.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|gone| gone.as_secs() as i64)
                .max()
                .unwrap_or(0);
            if gone.contains(&tisty_core::DeviceId(id.clone())) {
                return None;
            }
            let mine = id == mine;
            let called = tisty_core::config::nicknamed(&id);
            Some(Machine {
                id,
                called,
                when,
                mine,
            })
        })
        .collect();
    all.sort_by(|a, b| b.when.cmp(&a.when).then_with(|| a.id.cmp(&b.id)));
    all
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
    fn every_machine_that_ever_wrote_is_named() {
        let tmp = tempfile::tempdir().unwrap();
        for who in ["mac0", "win1"] {
            let at = tmp.path().join(who);
            std::fs::create_dir_all(&at).unwrap();
            std::fs::write(at.join("active.tisty"), b"an event").unwrap();
        }

        let all = machines(tmp.path(), "mac0", &Default::default());

        assert_eq!(all.len(), 2);
        assert_eq!(all.iter().filter(|one| one.mine).count(), 1);
        assert!(
            all.iter().all(|one| one.when > 0),
            "without a date there is no way to see who is behind"
        );
    }

    #[test]
    fn a_machine_that_was_removed_stops_being_listed_even_though_its_history_stays() {
        let tmp = tempfile::tempdir().unwrap();
        for who in ["mac0", "win1"] {
            let at = tmp.path().join(who);
            std::fs::create_dir_all(&at).unwrap();
            std::fs::write(at.join("active.tisty"), b"an event").unwrap();
        }

        let all = machines(
            tmp.path(),
            "mac0",
            &[tisty_core::DeviceId("win1".into())].into(),
        );

        assert_eq!(all.len(), 1, "removing did not remove it from the list");
        assert_eq!(all[0].id, "mac0");
        assert!(
            tmp.path().join("win1").join("active.tisty").exists(),
            "removing a machine threw away its history"
        );
    }

    #[test]
    fn a_machine_that_never_wrote_still_shows_up_with_nothing_to_show() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("win1")).unwrap();

        let all = machines(tmp.path(), "mac0", &Default::default());

        assert_eq!(all.len(), 1);
        assert_eq!(all[0].when, 0);
        assert!(!all[0].mine);
    }

    #[test]
    fn the_one_that_wrote_last_is_shown_first() {
        let tmp = tempfile::tempdir().unwrap();
        for who in ["old0", "new1"] {
            let at = tmp.path().join(who);
            std::fs::create_dir_all(&at).unwrap();
            std::fs::write(at.join("active.tisty"), b"an event").unwrap();
        }
        let older =
            std::time::SystemTime::now() - std::time::Duration::from_secs(60 * 60 * 24 * 12);
        std::fs::File::options()
            .write(true)
            .open(tmp.path().join("old0").join("active.tisty"))
            .unwrap()
            .set_modified(older)
            .unwrap();

        let all = machines(tmp.path(), "new1", &Default::default());

        assert_eq!(all[0].id, "new1");
        assert!(
            all[0].when - all[1].when > 60 * 60 * 24 * 11,
            "a machine twelve days behind has to look twelve days behind"
        );
    }

    #[test]
    fn a_machine_is_dated_by_its_last_write_and_not_its_first() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("mac0");
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join("000001.tisty"), b"an old event").unwrap();
        std::fs::write(at.join("active.tisty"), b"what was written just now").unwrap();
        let older =
            std::time::SystemTime::now() - std::time::Duration::from_secs(60 * 60 * 24 * 30);
        std::fs::File::options()
            .write(true)
            .open(at.join("000001.tisty"))
            .unwrap()
            .set_modified(older)
            .unwrap();

        let when = machines(tmp.path(), "mac0", &Default::default())[0].when;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        assert!(
            now - when < 60 * 60,
            "an old segment must not make a busy machine look abandoned"
        );
    }

    #[test]
    fn what_is_not_a_segment_never_passes_for_one() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("mac0");
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join("notes.txt"), b"not a segment").unwrap();

        assert_eq!(machines(tmp.path(), "mac0", &Default::default())[0].when, 0);
    }

    #[test]
    fn the_operating_system_says_something_it_could_be_asked_about() {
        let said = os();
        assert!(!said.is_empty());
        assert!(said.chars().any(|c| c.is_alphabetic()), "{said}");
    }
}
