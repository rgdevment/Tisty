use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const MOVED_BY: [&str; 4] = [
    "CLAUDE_CONFIG_DIR",
    "CODEX_HOME",
    "COPILOT_HOME",
    "GEMINI_CLI_HOME",
];

pub const KNOWN: [&str; 5] = ["claude", "codex", "copilot", "gemini", "cursor"];

pub type Told = BTreeMap<&'static str, PathBuf>;

pub fn assistant_root(home: &Path, told: &Told, id: &str) -> Option<PathBuf> {
    let moved =
        |named: &str, usual: &str| told.get(named).cloned().unwrap_or_else(|| home.join(usual));
    Some(match id {
        "claude" => moved("CLAUDE_CONFIG_DIR", ".claude"),
        "codex" => moved("CODEX_HOME", ".codex"),
        "copilot" => moved("COPILOT_HOME", ".copilot"),
        // Gemini's variable stands for the home folder, and the CLI still keeps `.gemini` inside it.
        "gemini" => told
            .get("GEMINI_CLI_HOME")
            .map_or(home, PathBuf::as_path)
            .join(".gemini"),
        "cursor" => home.join(".cursor"),
        _ => return None,
    })
}

pub fn assistant_folders(home: &Path, told: &Told) -> Vec<PathBuf> {
    KNOWN
        .iter()
        .filter_map(|id| assistant_root(home, told, id))
        .chain([home.join(".agents")])
        .collect()
}

pub fn inside_any(at: &Path, folders: &[PathBuf]) -> bool {
    folders
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .any(|one| at.starts_with(one))
}

fn var(named: &str) -> Option<PathBuf> {
    std::env::var_os(named).map(PathBuf::from)
}

pub fn home_here() -> Option<PathBuf> {
    var(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
}

// A relative folder would land wherever Tisty happens to run, so it counts as never said.
pub fn told_here() -> Told {
    MOVED_BY
        .into_iter()
        .filter_map(|named| var(named).map(|at| (named, at)))
        .filter(|(_, at)| at.is_absolute())
        .collect()
}

pub fn inside_an_assistant(at: &Path) -> bool {
    let told = told_here();
    let folders = match home_here().filter(|home| home.is_absolute()) {
        Some(home) => assistant_folders(&home, &told),
        None => told_alone(&told),
    };
    inside_any(at, &folders)
}

fn told_alone(told: &Told) -> Vec<PathBuf> {
    told.iter()
        .map(|(named, at)| match *named {
            "GEMINI_CLI_HOME" => at.join(".gemini"),
            _ => at.clone(),
        })
        .collect()
}

#[cfg(test)]
#[path = "homes_test.rs"]
mod tests;
