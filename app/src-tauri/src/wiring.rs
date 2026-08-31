mod json;
mod toml;

use std::path::{Path, PathBuf};

use crate::command;

const NAME: &str = "tisty";
const BEFORE: &str = "before-tisty";

#[derive(Clone, Copy)]
enum Kind {
    Json(&'static str),
    Toml(&'static str),
}

#[derive(Clone, Copy)]
enum Root {
    Home,
    Roaming,
    Store(&'static str),
    Support,
}

struct Spot(Root, &'static str);

struct Client {
    id: &'static str,
    name: &'static str,
    kind: Kind,
    files: &'static [Spot],
    signs: &'static [Spot],
}

const CLIENTS: &[Client] = &[
    Client {
        id: "claude-code",
        name: "Claude Code",
        kind: Kind::Json("mcpServers"),
        files: &[Spot(Root::Home, ".claude.json")],
        signs: &[Spot(Root::Home, ".claude")],
    },
    Client {
        id: "claude-desktop",
        name: "Claude Desktop",
        kind: Kind::Json("mcpServers"),
        files: &[
            Spot(Root::Store("Claude_"), "Claude/claude_desktop_config.json"),
            Spot(Root::Roaming, "Claude/claude_desktop_config.json"),
            Spot(Root::Support, "Claude/claude_desktop_config.json"),
        ],
        signs: &[
            Spot(Root::Store("Claude_"), "Claude"),
            Spot(Root::Roaming, "Claude"),
            Spot(Root::Support, "Claude"),
        ],
    },
    Client {
        id: "codex",
        name: "Codex",
        kind: Kind::Toml("mcp_servers"),
        files: &[Spot(Root::Home, ".codex/config.toml")],
        signs: &[Spot(Root::Home, ".codex")],
    },
    Client {
        id: "antigravity",
        name: "Antigravity",
        kind: Kind::Json("mcpServers"),
        files: &[Spot(Root::Home, ".gemini/config/mcp_config.json")],
        signs: &[
            Spot(Root::Home, ".gemini/config"),
            Spot(Root::Roaming, "Antigravity"),
            Spot(Root::Support, "Antigravity"),
        ],
    },
    Client {
        id: "vscode",
        name: "Visual Studio Code",
        kind: Kind::Json("servers"),
        files: &[
            Spot(Root::Roaming, "Code/User/mcp.json"),
            Spot(Root::Support, "Code/User/mcp.json"),
        ],
        signs: &[
            Spot(Root::Roaming, "Code/User"),
            Spot(Root::Support, "Code/User"),
        ],
    },
    Client {
        id: "cursor",
        name: "Cursor",
        kind: Kind::Json("mcpServers"),
        files: &[Spot(Root::Home, ".cursor/mcp.json")],
        signs: &[Spot(Root::Home, ".cursor")],
    },
    Client {
        id: "windsurf",
        name: "Windsurf",
        kind: Kind::Json("mcpServers"),
        files: &[Spot(Root::Home, ".codeium/windsurf/mcp_config.json")],
        signs: &[Spot(Root::Home, ".codeium/windsurf")],
    },
];

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Seen {
    pub id: &'static str,
    pub name: &'static str,
    pub at: String,
    pub wired: bool,
    pub astray: bool,
    pub points: Option<String>,
}

#[derive(Debug)]
pub enum Stuck {
    NoSuch,
    Puzzling(String),
    Cannot(String),
}

pub fn seen() -> Vec<Seen> {
    CLIENTS
        .iter()
        .filter(|it| about(it))
        .filter_map(|it| Some(told(it, &file(it)?)))
        .collect()
}

pub fn wire(id: &str) -> Result<Vec<Seen>, Stuck> {
    let (client, at) = asked(id)?;
    let was = std::fs::read_to_string(&at).unwrap_or_default();
    let entry = entry(client.kind);
    let now = match client.kind {
        Kind::Json(key) => json::set(&was, key, NAME, &entry),
        Kind::Toml(key) => toml::set(&was, key, NAME, &entry),
    }
    .ok_or_else(|| Stuck::Puzzling(at.display().to_string()))?;

    kept(&at, &was)?;
    laid(&at, &now)?;
    Ok(seen())
}

pub fn unwire(id: &str) -> Result<Vec<Seen>, Stuck> {
    let (client, at) = asked(id)?;
    let was = std::fs::read_to_string(&at).unwrap_or_default();
    let now = match client.kind {
        Kind::Json(key) => json::unset(&was, key, NAME),
        Kind::Toml(key) => toml::unset(&was, key, NAME),
    };
    if let Some(now) = now {
        kept(&at, &was)?;
        laid(&at, &now)?;
    }
    Ok(seen())
}

fn asked(id: &str) -> Result<(&'static Client, PathBuf), Stuck> {
    let client = CLIENTS.iter().find(|it| it.id == id).ok_or(Stuck::NoSuch)?;
    let at = file(client).ok_or(Stuck::NoSuch)?;
    Ok((client, at))
}

fn told(client: &'static Client, at: &Path) -> Seen {
    let text = std::fs::read_to_string(at).unwrap_or_default();
    let points = match client.kind {
        Kind::Json(key) => json::reads(&text, key, NAME),
        Kind::Toml(key) => toml::reads(&text, key, NAME),
    };
    Seen {
        id: client.id,
        name: client.name,
        at: at.display().to_string(),
        wired: points.is_some(),
        astray: points.as_deref().is_some_and(adrift),
        points,
    }
}

/// A bare name is answered by the PATH — which is the Store's own answer — and only a path we can
/// look at says whether an older copy left it pointing at nothing.
fn adrift(command: &str) -> bool {
    let at = Path::new(command);
    at.components().count() > 1 && !at.is_file()
}

fn entry(kind: Kind) -> String {
    let calling = command::calling();
    match kind {
        Kind::Json(_) => format!(
            "{{ \"command\": {}, \"args\": [\"mcp\"] }}",
            serde_json::Value::from(calling)
        ),
        Kind::Toml(_) => format!(
            "command = {}\nargs = [\"mcp\"]",
            ::toml::Value::from(calling)
        ),
    }
}

fn kept(at: &Path, was: &str) -> Result<(), Stuck> {
    if was.is_empty() {
        return Ok(());
    }
    let named = at.file_name().map(|it| it.to_string_lossy().into_owned());
    let Some(named) = named else {
        return Ok(());
    };
    std::fs::write(at.with_file_name(format!("{named}.{BEFORE}")), was).map_err(sour)
}

fn laid(at: &Path, text: &str) -> Result<(), Stuck> {
    if let Some(folder) = at.parent() {
        std::fs::create_dir_all(folder).map_err(sour)?;
    }
    tisty_core::store::write_atomic(at, text.as_bytes()).map_err(|e| Stuck::Cannot(e.to_string()))
}

fn sour(why: std::io::Error) -> Stuck {
    Stuck::Cannot(why.to_string())
}

fn about(client: &Client) -> bool {
    client
        .files
        .iter()
        .chain(client.signs)
        .filter_map(spot)
        .any(|at| at.exists())
}

fn file(client: &Client) -> Option<PathBuf> {
    let all: Vec<PathBuf> = client.files.iter().filter_map(spot).collect();
    all.iter()
        .find(|at| at.is_file())
        .cloned()
        .or_else(|| all.into_iter().next())
}

fn spot(it: &Spot) -> Option<PathBuf> {
    let Spot(root, tail) = it;
    let mut at = based(*root)?;
    for step in tail.split('/') {
        at.push(step);
    }
    Some(at)
}

fn based(root: Root) -> Option<PathBuf> {
    match root {
        Root::Home => home(),
        Root::Roaming => windows_only("APPDATA"),
        Root::Store(named) => windows_only("LOCALAPPDATA").and_then(|at| store(at, named)),
        Root::Support => cfg!(target_os = "macos")
            .then(home)
            .flatten()
            .map(|at| at.join("Library").join("Application Support")),
    }
}

fn home() -> Option<PathBuf> {
    let named = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var_os(named).map(PathBuf::from)
}

fn windows_only(named: &str) -> Option<PathBuf> {
    cfg!(windows)
        .then(|| std::env::var_os(named))
        .flatten()
        .map(PathBuf::from)
}

/// Installed from the Store an app never sees `%APPDATA%`: Windows hands it a private copy under
/// its package, and the settings the person edits are the ones in there.
fn store(local: PathBuf, named: &str) -> Option<PathBuf> {
    let packages = local.join("Packages");
    let mut found: Vec<PathBuf> = std::fs::read_dir(&packages)
        .ok()?
        .flatten()
        .map(|it| it.path())
        .filter(|at| {
            at.file_name()
                .is_some_and(|it| it.to_string_lossy().starts_with(named))
        })
        .collect();
    found.sort();
    let at = found.into_iter().next()?;
    Some(at.join("LocalCache").join("Roaming"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CODE: Client = Client {
        id: "claude-code",
        name: "Claude Code",
        kind: Kind::Json("mcpServers"),
        files: &[],
        signs: &[],
    };

    const CODEX: Client = Client {
        id: "codex",
        name: "Codex",
        kind: Kind::Toml("mcp_servers"),
        files: &[],
        signs: &[],
    };

    fn wire_at(client: &'static Client, at: &Path) -> Result<(), Stuck> {
        let was = std::fs::read_to_string(at).unwrap_or_default();
        let entry = entry(client.kind);
        let now = match client.kind {
            Kind::Json(key) => json::set(&was, key, NAME, &entry),
            Kind::Toml(key) => toml::set(&was, key, NAME, &entry),
        }
        .ok_or_else(|| Stuck::Puzzling(at.display().to_string()))?;
        kept(at, &was)?;
        laid(at, &now)
    }

    fn unwire_at(client: &'static Client, at: &Path) -> Option<()> {
        let was = std::fs::read_to_string(at).unwrap_or_default();
        let now = match client.kind {
            Kind::Json(key) => json::unset(&was, key, NAME),
            Kind::Toml(key) => toml::unset(&was, key, NAME),
        }?;
        laid(at, &now).ok()
    }

    #[test]
    fn a_settings_file_that_is_not_there_yet_is_written() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("deeper/mcp.json");

        wire_at(&CODE, &at).unwrap();

        assert!(told(&CODE, &at).wired);
        assert!(!at.with_file_name(format!("mcp.json.{BEFORE}")).exists());
    }

    #[test]
    fn what_was_written_before_is_kept_beside_it() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join(".claude.json");
        let was = "{\n  \"numStartups\": 53,\n  \"mcpServers\": {\n    \"sereno\": { \"command\": \"s\" }\n  }\n}\n";
        std::fs::write(&at, was).unwrap();

        wire_at(&CODE, &at).unwrap();

        let beside = at.with_file_name(format!(".claude.json.{BEFORE}"));
        assert_eq!(std::fs::read_to_string(beside).unwrap(), was);
        let now = std::fs::read_to_string(&at).unwrap();
        assert!(now.contains("\"numStartups\": 53"));
        assert!(now.contains("\"sereno\""));
    }

    #[test]
    fn wiring_twice_leaves_one_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join(".claude.json");

        wire_at(&CODE, &at).unwrap();
        wire_at(&CODE, &at).unwrap();

        let now = std::fs::read_to_string(&at).unwrap();
        assert_eq!(now.match_indices("\"tisty\"").count(), 1, "{now}");
    }

    #[test]
    fn a_file_written_in_a_way_we_cannot_follow_is_left_untouched() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join(".claude.json");
        let was = "{ \"mcpServers\": \"none of your business\" }";
        std::fs::write(&at, was).unwrap();

        let stuck = wire_at(&CODE, &at).unwrap_err();

        assert!(matches!(stuck, Stuck::Puzzling(_)));
        assert_eq!(std::fs::read_to_string(&at).unwrap(), was);
    }

    #[test]
    fn a_table_is_written_and_taken_back_out_of_a_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("config.toml");
        std::fs::write(&at, "model = \"gpt\"\n").unwrap();

        wire_at(&CODEX, &at).unwrap();
        assert!(told(&CODEX, &at).wired);

        unwire_at(&CODEX, &at).unwrap();
        assert!(!told(&CODEX, &at).wired);
        assert!(std::fs::read_to_string(&at).unwrap().contains("model"));
    }

    #[test]
    fn what_it_points_at_is_reported_back() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("mcp.json");

        wire_at(&CODE, &at).unwrap();

        assert_eq!(told(&CODE, &at).points, Some(command::calling()));
    }

    #[test]
    fn a_command_that_is_no_longer_there_is_astray() {
        let gone = match cfg!(windows) {
            true => "C:\\Programs\\Gone\\tisty.exe",
            false => "/Applications/Gone.app/Contents/MacOS/tisty",
        };

        assert!(adrift(gone));
    }

    #[test]
    fn a_bare_name_is_left_to_the_path_rather_than_called_astray() {
        assert!(!adrift("tisty"));
    }

    #[test]
    fn the_command_beside_the_window_is_not_astray() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join(command::CLI);
        std::fs::write(&at, b"a command").unwrap();

        assert!(!adrift(&at.display().to_string()));
    }

    #[test]
    fn every_client_is_named_once() {
        let mut ids: Vec<&str> = CLIENTS.iter().map(|it| it.id).collect();
        ids.sort_unstable();
        let mut alone = ids.clone();
        alone.dedup();
        assert_eq!(ids, alone);
    }

    #[cfg(any(windows, target_os = "macos"))]
    #[test]
    fn every_client_has_somewhere_to_write_on_this_machine() {
        for client in CLIENTS {
            assert!(
                client.files.iter().filter_map(spot).next().is_some(),
                "{} has nowhere to write",
                client.id
            );
        }
    }
}
