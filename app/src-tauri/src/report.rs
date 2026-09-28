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
    told: &[tisty_core::event::Event],
    mine: &str,
    gone: &std::collections::BTreeSet<tisty_core::DeviceId>,
    assistants: &std::collections::BTreeSet<tisty_core::DeviceId>,
) -> Vec<Machine> {
    let mut last: std::collections::BTreeMap<&tisty_core::DeviceId, i64> = Default::default();
    for one in told {
        let when = one.timestamp.as_second();
        last.entry(&one.device)
            .and_modify(|held| *held = (*held).max(when))
            .or_insert(when);
    }

    let mut all: Vec<Machine> = last
        .into_iter()
        .filter(|(who, _)| !gone.contains(*who) && !assistants.contains(*who))
        .map(|(who, when)| Machine {
            id: who.0.clone(),
            called: tisty_core::config::nicknamed(&who.0),
            when,
            mine: who.0 == mine,
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

#[cfg(not(any(windows, target_os = "macos")))]
compile_error!("Tisty builds for macOS and Windows only");

#[cfg(test)]
#[path = "report_test.rs"]
mod tests;
