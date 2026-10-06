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

pub fn also_weighed(data: &Path, also: Option<&Path>) -> u64 {
    let Some(also) = also else {
        return 0;
    };
    fn missing(from: &Path, at: &Path, data: &Path) -> u64 {
        let Ok(entries) = std::fs::read_dir(at) else {
            return 0;
        };
        entries
            .filter_map(|one| one.ok())
            .map(|one| match one.file_type() {
                Ok(kind) if kind.is_dir() => missing(from, &one.path(), data),
                Ok(kind) if kind.is_file() => one
                    .path()
                    .strip_prefix(from)
                    .ok()
                    .filter(|rest| !data.join(rest).exists())
                    .filter(|rest| tisty_core::backup::carried_alone(from, &from.join(rest)))
                    .and_then(|_| one.metadata().ok())
                    .map(|m| m.len())
                    .unwrap_or(0),
                _ => 0,
            })
            .sum()
    }
    missing(also, &also.join("attachments"), data)
}

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
    /// What the computer calls itself, said by that machine; the tree nickname stands in otherwise.
    pub name: Option<String>,
    pub os: Option<String>,
    pub when: i64,
    pub since: i64,
    pub mine: bool,
    pub signs: Option<String>,
    pub code: Option<String>,
    pub confirmed: Option<String>,
    pub confirmed_when: u64,
    pub carried: bool,
    /// What the last round did with its history. Only the round knows this; the two keys above
    /// can agree while the folder is being refused.
    pub turned_away: Option<String>,
}

pub struct Known<'a> {
    pub gone: &'a std::collections::BTreeSet<tisty_core::DeviceId>,
    pub assistants: &'a std::collections::BTreeSet<tisty_core::DeviceId>,
    pub keys: &'a std::collections::BTreeMap<tisty_core::DeviceId, String>,
    pub named: &'a std::collections::BTreeMap<tisty_core::DeviceId, tisty_core::Named>,
}

pub fn machines(
    told: &[tisty_core::event::Event],
    mine: &str,
    known: &Known,
    paths: &tisty_core::Paths,
    dest: Option<&std::path::Path>,
) -> Vec<Machine> {
    let data = paths.data();
    let stood = tisty_core::vouched::all_confirmed(data);
    let away = tisty_sync::turned::of(data);
    // The log keeps the first key a machine published and never another, so for this machine the
    // claim can be years stale while the key on disk is what it actually signs with.
    let ours = tisty_core::signing::shown_kept(paths, &tisty_core::DeviceId(mine.to_string()));
    // What a waiting machine says of itself sits in the very history that waits, never in the state.
    let waiting = |who: &tisty_core::DeviceId| {
        away.get(&who.0) == Some(&tisty_sync::turned::Away::Unconfirmed)
    };
    let introduced = |who: &tisty_core::DeviceId| match (waiting(who), dest) {
        (true, Some(at)) => tisty_core::store::introduced::introduced_in(
            &at.join(tisty_sync::STORE).join(&who.0),
            who,
        ),
        _ => Default::default(),
    };
    let mut span: std::collections::BTreeMap<&tisty_core::DeviceId, (i64, i64)> =
        Default::default();
    for one in told {
        let when = one.timestamp.as_second();
        span.entry(&one.device)
            .and_modify(|(first, last)| {
                *first = (*first).min(when);
                *last = (*last).max(when);
            })
            .or_insert((when, when));
    }
    let seen = |who: &tisty_core::DeviceId, when: i64, since: i64| {
        let said = introduced(who);
        let signs = match who.0 == mine {
            true => ours.clone().or_else(|| known.keys.get(who).cloned()),
            false => known.keys.get(who).cloned().or(said.key),
        };
        let named = known.named.get(who).cloned().or(said.named);
        Machine {
            id: who.0.clone(),
            called: tisty_core::config::nicknamed(&who.0),
            name: named.as_ref().map(|one| one.name.clone()),
            os: named.and_then(|one| one.os),
            when,
            since: said.since.map_or(since, |at| at.as_second()),
            mine: who.0 == mine,
            code: signs.as_deref().and_then(tisty_core::signing::spoken),
            signs,
            confirmed: stood.get(who).map(|one| one.key.clone()),
            confirmed_when: stood.get(who).map_or(0, |one| one.when),
            carried: stood.get(who).is_some_and(|one| one.carried),
            turned_away: away.get(&who.0).map(|one| match one {
                tisty_sync::turned::Away::Disowned => "disowned".to_string(),
                tisty_sync::turned::Away::Unreadable => "unreadable".to_string(),
                tisty_sync::turned::Away::Unconfirmed => "unconfirmed".to_string(),
            }),
        }
    };

    let mut all: Vec<Machine> = span
        .iter()
        .filter(|(who, _)| {
            !known.gone.contains(**who) && (!known.assistants.contains(**who) || waiting(who))
        })
        .map(|(who, (first, last))| seen(who, *last, *first))
        .collect();
    for (whose, away) in &away {
        let who = tisty_core::DeviceId(whose.clone());
        if *away != tisty_sync::turned::Away::Unconfirmed
            || all.iter().any(|one| &one.id == whose)
            || !tisty_core::store::is_device_name(whose)
            || known.gone.contains(&who)
        {
            continue;
        }
        all.push(seen(&who, 0, 0));
    }
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
