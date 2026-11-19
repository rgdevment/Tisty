use std::path::{Path, PathBuf};

use super::{Kind, Ledger, Refused, doc_of};
use crate::docs::BODY_AT_MOST;

const DEEPEST: usize = 8;
const FOUND_AT_MOST: usize = 2_000;
const SETTINGS: [&str; 9] = [
    "settings.json",
    "settings.local.json",
    "config.toml",
    "config.json",
    "hooks.json",
    "mcp.json",
    "mcp-config.json",
    ".claude.json",
    "keybindings.json",
];
const CREDENTIALS: [&str; 5] = [
    ".credentials.json",
    "credentials.json",
    "auth.json",
    ".env",
    ".netrc",
];
const CREDENTIAL_ENDINGS: [&str; 5] = [".pem", ".key", ".p12", ".pfx", ".env"];
const SKILL_FILE: &str = "SKILL.md";
const MANIFEST: &str = "manifest.json";
const MARKDOWN: &str = ".md";
// Copilot reads only `*.instructions.md` in its instructions folder; every other rules folder is Markdown.
const READ_AS: [(&str, &str, &str); 1] = [("copilot", "instructions", ".instructions.md")];
// Skill folders an assistant rewrites itself: Claude Code's claude.ai copy (`.trash` and Codex's `.system` fall to the dot rule).
const KEPT_BY_ASSISTANT: [&str; 1] = ["synced"];

fn kept_by_assistant(leaf: &str) -> bool {
    KEPT_BY_ASSISTANT
        .iter()
        .any(|one| one.eq_ignore_ascii_case(leaf))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub assistant: String,
    pub kind: Kind,
    pub at: PathBuf,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    Settings,
    Credentials,
    Tisty,
    Linked,
    TooBig,
    Crowded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub assistant: String,
    pub kind: Kind,
    pub at: PathBuf,
    pub name: String,
    pub skip: Option<Skip>,
}

// A skill may carry its own config.json, which the assistant never reads as its settings.
fn skipped(kind: Kind, leaf: &str) -> Option<Skip> {
    let leaf = leaf.to_ascii_lowercase();
    if leaf.ends_with(".before-tisty") || leaf == "tisty.json" || leaf.ends_with(".tmp") {
        return Some(Skip::Tisty);
    }
    if kind != Kind::Skill && SETTINGS.contains(&leaf.as_str()) {
        return Some(Skip::Settings);
    }
    let credential = CREDENTIALS.contains(&leaf.as_str())
        || leaf.starts_with(".env.")
        || leaf.starts_with("id_rsa")
        || leaf.starts_with("id_ed25519")
        || CREDENTIAL_ENDINGS.iter().any(|end| leaf.ends_with(end));
    credential.then_some(Skip::Credentials)
}

fn looked_at(place: &Place, at: PathBuf, name: String) -> Found {
    let leaf = at
        .file_name()
        .map(|one| one.to_string_lossy().into_owned())
        .unwrap_or_default();
    let skip = skipped(place.kind, &leaf).or_else(|| match super::link(&at) {
        true => Some(Skip::Linked),
        false => std::fs::metadata(&at)
            .is_ok_and(|meta| meta.len() > BODY_AT_MOST)
            .then_some(Skip::TooBig),
    });
    Found {
        assistant: place.assistant.clone(),
        kind: place.kind,
        at,
        name,
        skip,
    }
}

/// How a folder is walked: the skills folder only names skills, a skill keeps every file, a rules folder only what its assistant reads.
#[derive(Clone, Copy)]
enum Walk {
    Skills,
    Skill,
    Read(&'static str),
}

fn walk_of(place: &Place) -> Walk {
    match place.kind {
        Kind::Skill => Walk::Skills,
        _ => Walk::Read(
            READ_AS
                .iter()
                .find(|(who, folder, _)| *who == place.assistant && *folder == place.name)
                .map_or(MARKDOWN, |(_, _, ending)| ending),
        ),
    }
}

// A skill is a folder right under the skills folder with its own SKILL.md, as the skills format defines it.
fn is_skill(at: &Path, leaf: &str) -> bool {
    !kept_by_assistant(leaf) && at.join(SKILL_FILE).is_file()
}

fn kept_away(at: &Path, leaf: &str, walk: Walk) -> bool {
    match walk {
        Walk::Skills => !is_skill(at, leaf),
        Walk::Skill => leaf == "node_modules",
        Walk::Read(_) => leaf == "node_modules" || at.join(MANIFEST).is_file(),
    }
}

fn read_here(leaf: &str, walk: Walk) -> bool {
    match walk {
        Walk::Skills => false,
        Walk::Skill => true,
        Walk::Read(ending) => leaf.to_ascii_lowercase().ends_with(ending),
    }
}

fn walked(place: &Place, dir: &Path, prefix: &str, depth: usize, walk: Walk, out: &mut Vec<Found>) {
    if depth > DEEPEST {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|one| one.file_name());
    for entry in entries {
        let leaf = entry.file_name().to_string_lossy().into_owned();
        // Nothing hidden, at any depth: dot files are the assistant's or the system's, never a rule.
        if leaf.starts_with('.') {
            continue;
        }
        if out.len() >= FOUND_AT_MOST {
            if out.iter().any(|one| one.skip == Some(Skip::Crowded)) {
                return;
            }
            out.push(Found {
                assistant: place.assistant.clone(),
                kind: place.kind,
                at: dir.to_path_buf(),
                name: prefix.to_string(),
                skip: Some(Skip::Crowded),
            });
            return;
        }
        let name = match prefix.is_empty() {
            true => leaf.clone(),
            false => format!("{prefix}/{leaf}"),
        };
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        let at = entry.path();
        match kind.is_dir() {
            true if kept_away(&at, &leaf, walk) => {}
            true => {
                let inner = match walk {
                    Walk::Skills => Walk::Skill,
                    other => other,
                };
                walked(place, &at, &name, depth + 1, inner, out);
            }
            false if read_here(&leaf, walk) => out.push(looked_at(place, at, name)),
            false => {}
        }
    }
}

pub fn found(places: &[Place]) -> Vec<Found> {
    let mut out = Vec::new();
    for place in places {
        match std::fs::symlink_metadata(&place.at) {
            Ok(meta) if meta.is_dir() => {
                walked(place, &place.at, &place.name, 0, walk_of(place), &mut out)
            }
            Ok(_) => out.push(looked_at(place, place.at.clone(), place.name.clone())),
            Err(_) => {}
        }
    }
    out
}

pub fn brought_in(ledger: &Ledger, found: &Found) -> Result<String, Refused> {
    if !ledger.consented() {
        return Err(Refused::NoConsent);
    }
    let leaf = found.at.file_name().map(|one| one.to_string_lossy());
    if let Some(why) = found
        .skip
        .or_else(|| skipped(found.kind, &leaf.unwrap_or_default()))
    {
        return Err(Refused::Skipped(why));
    }
    if super::link(&found.at) {
        return Err(Refused::Linked);
    }
    if std::fs::metadata(&found.at)?.len() > BODY_AT_MOST {
        return Err(Refused::TooBig);
    }
    let bytes = std::fs::read(&found.at)?;
    if let Some(told) = crate::agent::secret_in(&bytes) {
        return Err(Refused::Secret(told.said()));
    }
    doc_of(&found.assistant, found.kind, &found.name, &bytes)
}

#[cfg(test)]
#[path = "found_test.rs"]
mod tests;
