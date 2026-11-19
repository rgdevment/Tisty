use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use tisty_core::agents::{Admitted, Card, GLOBAL, Gate, Kind, Ledger, Place as Spot, Target};

use super::places::var;

pub const ICON: &str = tisty_core::agents::ICON;
pub const HUE: &str = "purple";
pub const FOLDER: &str = tisty_core::agents::FOLDER;
pub const GLOBAL_FOLDER: &str = "Global";

const SHARED_SKILLS_OWNERS: [&str; 3] = ["codex", "copilot", "gemini"];
const GEMINI_RULES: &str = "tisty-rules.md";
const GEMINI_MEMORY: &str = "GEMINI.md";
const CODEX_OVERRIDE: &str = "AGENTS.override.md";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assistant {
    pub id: &'static str,
    pub name: &'static str,
    main: Option<&'static str>,
    rule_dir: Option<&'static str>,
}

pub const ASSISTANTS: [Assistant; 5] = [
    Assistant {
        id: "claude",
        name: "Claude",
        main: Some("CLAUDE.md"),
        rule_dir: Some("rules"),
    },
    Assistant {
        id: "codex",
        name: "Codex",
        main: Some("AGENTS.md"),
        rule_dir: None,
    },
    Assistant {
        id: "copilot",
        name: "Copilot",
        main: Some("copilot-instructions.md"),
        rule_dir: Some("instructions"),
    },
    Assistant {
        id: "gemini",
        name: "Gemini",
        main: Some(GEMINI_RULES),
        rule_dir: None,
    },
    Assistant {
        id: "cursor",
        name: "Cursor",
        main: None,
        rule_dir: None,
    },
];

pub fn assistant(id: &str) -> Option<&'static Assistant> {
    ASSISTANTS.iter().find(|one| one.id == id)
}

#[derive(Debug, Clone, Default)]
pub struct Homes {
    home: PathBuf,
    told: BTreeMap<&'static str, PathBuf>,
}

const MOVED_BY: [&str; 4] = [
    "CLAUDE_CONFIG_DIR",
    "CODEX_HOME",
    "COPILOT_HOME",
    "GEMINI_CLI_HOME",
];

impl Homes {
    pub fn here() -> Option<Self> {
        let home = var(if cfg!(windows) { "USERPROFILE" } else { "HOME" })?;
        let told = MOVED_BY
            .into_iter()
            .filter_map(|named| var(named).map(|at| (named, at)))
            .collect();
        Some(Self { home, told })
    }

    #[cfg(test)]
    pub fn with(home: &Path, told: &[(&'static str, &Path)]) -> Self {
        Self {
            home: home.to_path_buf(),
            told: told
                .iter()
                .map(|(named, at)| (*named, at.to_path_buf()))
                .collect(),
        }
    }

    fn moved(&self, named: &str, usual: &str) -> PathBuf {
        self.told
            .get(named)
            .cloned()
            .unwrap_or_else(|| self.home.join(usual))
    }

    pub fn root(&self, id: &str) -> Option<PathBuf> {
        Some(match id {
            "claude" => self.moved("CLAUDE_CONFIG_DIR", ".claude"),
            "codex" => self.moved("CODEX_HOME", ".codex"),
            "copilot" => self.moved("COPILOT_HOME", ".copilot"),
            // Gemini's variable stands for the home folder, and the CLI still keeps `.gemini` inside it.
            "gemini" => self
                .told
                .get("GEMINI_CLI_HOME")
                .unwrap_or(&self.home)
                .join(".gemini"),
            "cursor" => self.home.join(".cursor"),
            _ => return None,
        })
    }

    fn shared(&self) -> PathBuf {
        self.home.join(".agents")
    }

    pub fn detected(&self) -> Vec<&'static Assistant> {
        ASSISTANTS
            .iter()
            .filter(|one| self.root(one.id).is_some_and(|at| at.is_dir()))
            .collect()
    }

    // Codex, Copilot and Gemini all read `~/.agents/skills`; the first one here owns it, so it is written once.
    fn shared_owner(&self, detected: &[&Assistant]) -> Option<&'static str> {
        SHARED_SKILLS_OWNERS
            .into_iter()
            .find(|id| detected.iter().any(|one| one.id == *id))
    }

    fn skills_of(&self, id: &str) -> Option<(PathBuf, PathBuf)> {
        let root = match id {
            "codex" => self.shared(),
            _ => self.root(id)?,
        };
        Some((root.clone(), root.join("skills")))
    }

    // Codex reads the override instead of AGENTS.md whenever it is there, so Global rules must go where it reads.
    fn main_of(&self, one: &Assistant) -> Option<&'static str> {
        let overridden = one.id == "codex"
            && self
                .root(one.id)
                .is_some_and(|root| root.join(CODEX_OVERRIDE).is_file());
        match overridden {
            true => Some(CODEX_OVERRIDE),
            false => one.main,
        }
    }
}

fn under(at: &Path, name: &str) -> PathBuf {
    name.split('/')
        .fold(at.to_path_buf(), |at, step| at.join(step))
}

pub fn spots(homes: &Homes, one: &Assistant) -> Vec<Spot> {
    let Some(root) = homes.root(one.id) else {
        return Vec::new();
    };
    let spot = |kind: Kind, at: PathBuf, name: &str| Spot {
        assistant: one.id.to_string(),
        kind,
        at,
        name: name.to_string(),
    };
    let mut all = Vec::new();
    if let Some(main) = one.main.filter(|main| *main != GEMINI_RULES) {
        all.push(spot(Kind::Rule, root.join(main), main));
    }
    if one.id == "codex" {
        all.push(spot(Kind::Rule, root.join(CODEX_OVERRIDE), CODEX_OVERRIDE));
    }
    if let Some(dir) = one.rule_dir {
        all.push(spot(Kind::Rule, root.join(dir), dir));
    }
    if one.id == "gemini" {
        all.push(spot(Kind::Memory, root.join(GEMINI_MEMORY), GEMINI_MEMORY));
    }
    if one.id == "claude" {
        all.extend(
            memories(&root)
                .into_iter()
                .map(|(at, name)| spot(Kind::Memory, at, &name)),
        );
    }
    if let Some((_, skills)) = homes.skills_of(one.id) {
        all.push(spot(Kind::Skill, skills, ""));
    }
    all
}

// Only each project's memory folder: the rest of `projects` is session transcripts, never the person's rules.
fn memories(root: &Path) -> Vec<(PathBuf, String)> {
    let Ok(projects) = std::fs::read_dir(root.join("projects")) else {
        return Vec::new();
    };
    let mut found: Vec<(PathBuf, String)> = projects
        .flatten()
        .filter_map(|one| {
            let project = one.file_name().to_string_lossy().into_owned();
            let at = one.path().join("memory");
            at.is_dir()
                .then(|| (at, format!("projects/{project}/memory")))
        })
        .collect();
    found.sort();
    found
}

pub fn target_of(homes: &Homes, card: &Card) -> Option<(PathBuf, PathBuf)> {
    let one = assistant(&card.level)?;
    let root = homes.root(one.id)?;
    match card.kind {
        Kind::Skill => {
            let (root, skills) = homes.skills_of(one.id)?;
            Some((root, under(&skills, &card.name)))
        }
        Kind::Rule => {
            let main = one.main == Some(card.name.as_str());
            let in_dir = one.rule_dir.is_some_and(|dir| {
                card.name
                    .strip_prefix(dir)
                    .is_some_and(|rest| rest.starts_with('/'))
            });
            let override_md = one.id == "codex" && card.name == CODEX_OVERRIDE;
            (main || in_dir || override_md).then(|| (root.clone(), under(&root, &card.name)))
        }
        Kind::Memory => {
            let fits = match one.id {
                "claude" => {
                    let steps: Vec<&str> = card.name.split('/').collect();
                    steps.len() >= 4 && steps[0] == "projects" && steps[2] == "memory"
                }
                "gemini" => card.name == GEMINI_MEMORY,
                _ => false,
            };
            fits.then(|| (root.clone(), under(&root, &card.name)))
        }
    }
}

/// Whether some assistant on this computer would read the document at all.
pub fn lands_here(homes: &Homes, card: &Card) -> bool {
    match card.level == GLOBAL {
        true => card.kind != Kind::Memory && !homes.detected().is_empty(),
        false => {
            homes.detected().iter().any(|one| one.id == card.level)
                && target_of(homes, card).is_some()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub target: Target,
    pub files: Vec<String>,
    pub lead_in: Option<Target>,
}

pub fn targets(homes: &Homes, gate: &Gate, ledger: &Ledger) -> Vec<Placed> {
    let detected = homes.detected();
    let mut pieces: Vec<&Admitted> = gate.admitted().iter().collect();
    pieces.sort_by(|a, b| {
        let key = |one: &&Admitted| {
            (
                one.card().level != GLOBAL,
                one.card().name.clone(),
                one.file().to_string(),
            )
        };
        key(a).cmp(&key(b))
    });
    // The same file brought on two computers before they synced is one file, never written twice.
    let mut seen = BTreeSet::new();
    pieces.retain(|one| seen.insert(one.card().address()));
    let mut out: BTreeMap<PathBuf, Placed> = BTreeMap::new();
    let mut put = |assistant: &str, (root, at): (PathBuf, PathBuf), piece: &Admitted| {
        let placed = out.entry(at.clone()).or_insert_with(|| Placed {
            target: gate.target(ledger, root, at, assistant),
            files: Vec::new(),
            lead_in: None,
        });
        placed.target.put(piece);
        placed.files.push(piece.file().to_string());
    };
    for piece in pieces {
        let card = piece.card();
        if card.level != GLOBAL {
            if detected.iter().any(|one| one.id == card.level)
                && let Some(spot) = target_of(homes, card)
            {
                put(&card.level, spot, piece);
            }
            continue;
        }
        for one in &detected {
            let spot = match card.kind {
                Kind::Rule => homes.main_of(one).and_then(|main| {
                    let card = Card {
                        level: one.id.to_string(),
                        name: main.to_string(),
                        ..card.clone()
                    };
                    target_of(homes, &card)
                }),
                Kind::Skill => {
                    let shared = SHARED_SKILLS_OWNERS.contains(&one.id);
                    if shared && homes.shared_owner(&detected) != Some(one.id) {
                        continue;
                    }
                    let (root, skills) = match shared {
                        true => homes.skills_of("codex"),
                        false => homes.skills_of(one.id),
                    }
                    .unzip();
                    root.zip(skills.map(|at| under(&at, &card.name)))
                }
                Kind::Memory => None,
            };
            if let Some(spot) = spot {
                put(one.id, spot, piece);
            }
        }
    }
    let mut all: Vec<Placed> = out.into_values().collect();
    for placed in &mut all {
        if placed.target.assistant == "gemini" && placed.target.at.ends_with(GEMINI_RULES) {
            let at = placed.target.root.join(GEMINI_MEMORY);
            placed.lead_in = Some(gate.target(
                ledger,
                placed.target.root.clone(),
                at,
                &placed.target.assistant,
            ));
        }
    }
    all
}

pub fn lead_in_line() -> String {
    format!("@{GEMINI_RULES}")
}

#[cfg(test)]
#[path = "agents_gated.rs"]
pub(crate) mod gated;

#[cfg(test)]
#[path = "agents_test.rs"]
mod tests;
