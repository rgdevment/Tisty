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
#[path = "wiring_test.rs"]
mod tests;
