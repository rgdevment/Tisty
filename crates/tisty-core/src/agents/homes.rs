use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const MOVED_BY: [&str; 4] = [
    "CLAUDE_CONFIG_DIR",
    "CODEX_HOME",
    "COPILOT_HOME",
    "GEMINI_CLI_HOME",
];

const KNOWN: [&str; 5] = ["claude", "codex", "copilot", "gemini", "cursor"];

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

pub fn inside_an_assistant(at: &Path) -> bool {
    let var = |named: &str| std::env::var_os(named).map(PathBuf::from);
    let Some(home) = var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }) else {
        return false;
    };
    let told: Told = MOVED_BY
        .into_iter()
        .filter_map(|named| var(named).map(|at| (named, at)))
        .filter(|(_, at)| at.is_absolute())
        .collect();
    inside_any(at, &assistant_folders(&home, &told))
}

#[cfg(test)]
#[path = "homes_test.rs"]
mod tests;
