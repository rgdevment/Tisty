use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use tisty_core::agents::{self, Card, GLOBAL, Gate, Kind, Ledger, Place as Spot, Target};

pub const ICON: &str = tisty_core::agents::ICON;
pub const HUE: &str = "purple";
pub const FOLDER: &str = tisty_core::agents::FOLDER;
pub const GLOBAL_FOLDER: &str = "Global";

const SHARED_SKILLS_OWNERS: [&str; 4] = ["codex", "copilot", "gemini", "dsh"];
const GEMINI_RULES: &str = "tisty-rules.md";
const GEMINI_MEMORY: &str = "GEMINI.md";
const CODEX_OVERRIDE: &str = "AGENTS.override.md";
const TISTY_PLUGIN: &str = "tisty@";
const JSON_AT_MOST: u64 = 4 * 1024 * 1024;

fn record(at: &Path) -> Result<serde_json::Value, PathBuf> {
    let Ok(meta) = std::fs::symlink_metadata(at) else {
        return Ok(serde_json::Value::Null);
    };
    let text = std::fs::metadata(at)
        .ok()
        .filter(|it| it.len() <= JSON_AT_MOST && !meta.is_dir())
        .and_then(|_| std::fs::read_to_string(at).ok())
        .ok_or(at)?;
    let text = text.trim_start_matches('\u{feff}').trim();
    match text.is_empty() {
        true => Ok(serde_json::Value::Null),
        false => serde_json::from_str(text).map_err(|_| at.to_path_buf()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Assistant {
    pub id: &'static str,
    pub name: &'static str,
    main: Option<&'static str>,
    rule_dir: Option<&'static str>,
}

pub const ASSISTANTS: [Assistant; 6] = [
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
    Assistant {
        id: "dsh",
        name: "DeepSeek Harness",
        main: Some("AGENTS.md"),
        rule_dir: None,
    },
];

pub fn assistant(id: &str) -> Option<&'static Assistant> {
    ASSISTANTS.iter().find(|one| one.id == id)
}

/// The one rules file an assistant reads, when it reads no folder of rules beside it.
pub fn single_rule(id: &str) -> Option<&'static str> {
    assistant(id).filter(|one| one.rule_dir.is_none())?.main
}

/// The first name a new rule of `level` can take that no document holds yet: the main file while it is free, then one in its rules folder.
pub fn fresh_rule(level: &str, taken: impl Fn(&str) -> bool) -> Option<String> {
    let numbered = |stem: &str, ext: &str| {
        (1..)
            .map(|n| match n {
                1 => format!("{stem}{ext}"),
                _ => format!("{stem}-{n}{ext}"),
            })
            .find(|name| !taken(name))
    };
    let one = assistant(level)?;
    if let Some(main) = one.main.filter(|main| !taken(main)) {
        return Some(main.to_string());
    }
    let ext = match one.id {
        "copilot" => ".instructions.md",
        _ => ".md",
    };
    numbered(&format!("{}/rule", one.rule_dir?), ext)
}

#[derive(Debug, Clone, Default)]
pub struct Homes {
    home: PathBuf,
    told: tisty_core::agents::Told,
}

impl Homes {
    pub fn here() -> Option<Self> {
        Self::told(
            tisty_core::agents::home_here()?,
            tisty_core::agents::told_here(),
        )
    }

    fn told(
        home: PathBuf,
        told: impl IntoIterator<Item = (&'static str, PathBuf)>,
    ) -> Option<Self> {
        home.is_absolute().then(|| Self {
            home,
            told: told
                .into_iter()
                .filter(|(_, at)| at.is_absolute())
                .collect(),
        })
    }

    #[cfg(test)]
    pub fn with(home: &Path, told: &[(&'static str, &Path)]) -> Self {
        Self::told(
            home.to_path_buf(),
            told.iter().map(|(named, at)| (*named, at.to_path_buf())),
        )
        .unwrap_or_default()
    }

    pub fn root(&self, id: &str) -> Option<PathBuf> {
        tisty_core::agents::assistant_root(&self.home, &self.told, id)
    }

    fn shared(&self) -> PathBuf {
        self.told
            .get("DSH_AGENTS_HOME")
            .cloned()
            .unwrap_or_else(|| self.home.join(".agents"))
    }

    pub fn detected(&self) -> Vec<&'static Assistant> {
        ASSISTANTS
            .iter()
            .filter(|one| self.root(one.id).is_some_and(|at| at.is_dir()))
            .collect()
    }

    // Codex, Copilot, Gemini and the DeepSeek Harness all read `~/.agents/skills`; the first
    // one here owns it, so it is written once.
    fn shared_owner(&self, detected: &[&Assistant]) -> Option<&'static str> {
        SHARED_SKILLS_OWNERS
            .into_iter()
            .find(|id| detected.iter().any(|one| one.id == *id))
    }

    /// The agents whose Tisty plugin already brings Tisty's skills.
    pub fn plugged(&self) -> BTreeSet<&'static str> {
        match self.claude_plugged() {
            true => BTreeSet::from(["claude"]),
            false => BTreeSet::new(),
        }
    }

    /// Whether the plugin brings Claude Code's hooks; the file that could not be read when nothing can tell.
    pub fn plugin_brings_hooks(&self) -> Result<bool, PathBuf> {
        let Some(root) = self.root("claude") else {
            return Ok(false);
        };
        let added = plugins_added(&root)?;
        if added.is_empty() {
            return Ok(false);
        }
        let settings = record(&root.join("settings.json"))?;
        Ok(added.iter().any(|one| turned_on(&settings, one)))
    }

    pub fn of(home: &Path) -> Option<Self> {
        Self::told(home.to_path_buf(), [])
    }

    // Claude Code lists a plugin's skills beside the person's, so the plugin's copy is the one it keeps.
    fn claude_plugged(&self) -> bool {
        let Some(root) = self.root("claude") else {
            return false;
        };
        let added = plugins_added(&root).unwrap_or_default();
        let settings = record(&root.join("settings.json")).unwrap_or_default();
        added.iter().any(|one| turned_on(&settings, one))
    }

    fn skills_of(&self, id: &str) -> Option<(PathBuf, PathBuf)> {
        let root = match id {
            "codex" | "dsh" => self.shared(),
            _ => self.root(id)?,
        };
        Some((root.clone(), root.join("skills")))
    }
}

fn plugins_added(root: &Path) -> Result<Vec<String>, PathBuf> {
    let installed = record(&root.join("plugins").join("installed_plugins.json"))?;
    let for_the_person = |installs: &serde_json::Value| {
        installs
            .as_array()
            .into_iter()
            .flatten()
            .any(|one| one.get("scope").and_then(serde_json::Value::as_str) == Some("user"))
    };
    let all = installed
        .get("plugins")
        .and_then(serde_json::Value::as_object);
    Ok(all
        .into_iter()
        .flatten()
        .filter(|(key, installs)| key.starts_with(TISTY_PLUGIN) && for_the_person(installs))
        .map(|(key, _)| key.clone())
        .collect())
}

fn turned_on(settings: &serde_json::Value, plugin: &str) -> bool {
    let said = settings.get("enabledPlugins").and_then(|it| it.get(plugin));
    said != Some(&false.into())
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
        Kind::Hook => None,
    }
}

/// Whether some assistant on this computer would read the document at all.
pub fn lands_here(homes: &Homes, card: &Card) -> bool {
    !landings(homes, card).is_empty()
}

/// One file a document is written into on this computer.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Landing {
    assistant: &'static str,
    root: PathBuf,
    at: PathBuf,
}

// A Global rule has no file of its own in any agent, so only a skill lands everywhere.
fn landings(homes: &Homes, card: &Card) -> Vec<Landing> {
    let detected = homes.detected();
    let Some(one) = assistant(&card.level) else {
        return match card.level == GLOBAL && card.kind == Kind::Skill {
            true => everywhere(homes, &detected, card),
            false => Vec::new(),
        };
    };
    let here = detected.iter().any(|known| known.id == one.id);
    target_of(homes, card)
        .filter(|_| here)
        .map(|(root, at)| Landing {
            assistant: one.id,
            root,
            at,
        })
        .into_iter()
        .collect()
}

fn everywhere(homes: &Homes, detected: &[&'static Assistant], card: &Card) -> Vec<Landing> {
    let owner = homes.shared_owner(detected);
    detected
        .iter()
        .filter_map(|one| {
            let shared = SHARED_SKILLS_OWNERS.contains(&one.id);
            if shared && owner != Some(one.id) {
                return None;
            }
            let (root, skills) = homes.skills_of(match shared {
                true => "codex",
                false => one.id,
            })?;
            Some(Landing {
                assistant: one.id,
                root,
                at: under(&skills, &card.name),
            })
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub target: Target,
    pub files: Vec<String>,
    pub lead_in: Option<Target>,
}

pub fn wordless(piece: &agents::Admitted) -> bool {
    agents::rendered(piece.body())
        .is_ok_and(|bytes| String::from_utf8_lossy(&bytes).trim().is_empty())
}

// Tisty's own skills come in only with this computer's yes, so the plugin's records are never read before it.
pub fn targets(homes: &Homes, gate: &Gate, ledger: &Ledger) -> Vec<Placed> {
    let plugged = match gate
        .admitted()
        .iter()
        .any(|one| agents::builtin(one.file()))
    {
        true => homes.plugged(),
        false => BTreeSet::new(),
    };
    placed(homes, gate, ledger, &plugged)
}

/// What one choice covers: a rule or memory is its file, a skill is its whole folder.
fn unit_of(card: &Card, at: &Path) -> String {
    let steps = match card.kind {
        Kind::Skill => card.name.split('/').count().saturating_sub(1),
        _ => 0,
    };
    agents::path_key(at.ancestors().nth(steps).unwrap_or(at))
}

/// Whose file it is: the files of one skill stand together, any other document stands alone.
fn source_of(file: &str, card: &Card) -> String {
    match card.kind {
        Kind::Skill => format!(
            "{}/{}/{}",
            card.level,
            card.computer.as_ref().map_or("", |one| one.0.as_str()),
            agents::skill_of(&card.name).to_lowercase()
        ),
        _ => file.to_string(),
    }
}

type Sources = BTreeMap<String, BTreeSet<String>>;

fn claims<'a>(
    homes: &Homes,
    docs: impl Iterator<Item = (&'a str, &'a Card)>,
) -> BTreeMap<String, Sources> {
    let mut units: BTreeMap<String, Sources> = BTreeMap::new();
    for (file, card) in docs.filter(|(file, _)| !agents::builtin(file)) {
        for one in landings(homes, card) {
            units
                .entry(unit_of(card, &one.at))
                .or_default()
                .entry(source_of(file, card))
                .or_default()
                .insert(file.to_string());
        }
    }
    units
}

/// The places this document would write, the files that go with it, and those that would write the same places.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Sides {
    pub places: Vec<(String, String)>,
    pub mine: BTreeSet<String>,
    pub rivals: BTreeMap<String, BTreeSet<String>>,
}

impl Sides {
    pub fn rival_files(&self) -> BTreeSet<String> {
        self.rivals.values().flatten().cloned().collect()
    }
}

// One left out on this computer is nobody's rival.
pub fn sides(homes: &Homes, gate: &Gate, file: &str) -> Sides {
    let standing = gate
        .for_here()
        .filter(|(one, _)| *one == file || gate.seen(one) != Some(agents::Seen::DeclinedHere));
    let mut out = Sides {
        mine: BTreeSet::from([file.to_string()]),
        ..Sides::default()
    };
    for (place, sources) in claims(homes, standing) {
        let Some(source) = sources
            .iter()
            .find(|(_, files)| files.contains(file))
            .map(|(source, _)| source.clone())
        else {
            continue;
        };
        for (other, files) in sources {
            match other == source {
                true => out.mine.extend(files),
                false => out.rivals.entry(other).or_default().extend(files),
            }
        }
        out.places.push((place, source));
    }
    out
}

/// Files where a document that still waits for the person would be written.
pub fn awaited(homes: &Homes, gate: &Gate) -> BTreeSet<String> {
    gate.for_here()
        .filter(|(file, _)| gate.seen(file) == Some(agents::Seen::WaitingHere))
        .flat_map(|(_, card)| landings(homes, card))
        .map(|one| agents::path_key(&one.at))
        .collect()
}

/// Who writes a place: one side, or nobody until the person chooses.
#[derive(Debug, PartialEq, Eq)]
enum Stand {
    Goes(String),
    Asks,
}

type Written<'a> = Vec<(String, &'a BTreeSet<String>)>;

// What this computer last wrote and is still there: another document never takes it over without the person's choice.
fn written(ledger: &Ledger) -> Written<'_> {
    ledger
        .written_files()
        .filter(|(at, _)| !ledger.stays(at) && std::fs::symlink_metadata(at).is_ok())
        .map(|(at, files)| (agents::path_key(at), files))
        .collect()
}

fn within(at: &str, place: &str) -> bool {
    at.strip_prefix(place)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(['/', '\\']))
}

// Two sides of one place ask, whichever came first, until the person chose between those very two.
fn stand(ledger: &Ledger, written: &Written, place: &str, sources: &Sources) -> Stand {
    let weighed = |chosen: &&str| {
        sources
            .keys()
            .all(|one| one == chosen || ledger.chosen_over(place, one))
    };
    if let Some(chosen) = ledger
        .chosen(place)
        .filter(|it| sources.contains_key(*it))
        .filter(weighed)
    {
        return Stand::Goes(chosen.to_string());
    }
    let mut all = sources.iter();
    let (Some((source, files)), None) = (all.next(), all.next()) else {
        return Stand::Asks;
    };
    let credited: BTreeSet<&String> = written
        .iter()
        .filter(|(at, _)| within(at, place))
        .flat_map(|(_, files)| files.iter())
        .filter(|file| !agents::builtin(file))
        .collect();
    match credited.is_empty() || files.iter().any(|file| credited.contains(file)) {
        true => Stand::Goes(source.clone()),
        false => Stand::Asks,
    }
}

#[derive(Debug, Default)]
pub struct Stands(BTreeMap<String, Stand>);

pub fn stands(homes: &Homes, gate: &Gate, ledger: &Ledger) -> Stands {
    let admitted = gate
        .admitted()
        .iter()
        .filter(|one| one.card().kind != Kind::Memory)
        .map(|one| (one.file(), one.card()));
    let written = written(ledger);
    let all = claims(homes, admitted)
        .into_iter()
        .map(|(place, sources)| {
            let stand = stand(ledger, &written, &place, &sources);
            (place, stand)
        })
        .collect();
    Stands(all)
}

/// How a document let in here stands in the places it would write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance {
    Goes,
    Asks,
    Aside,
}

pub fn stance(homes: &Homes, gate: &Gate, ledger: &Ledger, file: &str) -> Stance {
    let Some(card) = gate.card_here(file) else {
        return Stance::Goes;
    };
    let source = source_of(file, card);
    let stands = stands(homes, gate, ledger);
    let here: Vec<&Stand> = landings(homes, card)
        .iter()
        .filter_map(|one| stands.0.get(&unit_of(card, &one.at)))
        .collect();
    let aside = |one: &&Stand| matches!(one, Stand::Goes(it) if *it != source);
    if here.contains(&&Stand::Asks) {
        return Stance::Asks;
    }
    match !here.is_empty() && here.iter().all(aside) {
        true => Stance::Aside,
        false => Stance::Goes,
    }
}

impl Stands {
    /// Whether another document was chosen over this one for what it once wrote at this path.
    pub fn aside_at(&self, homes: &Homes, card: &Card, file: &str, at: &Path) -> bool {
        let source = source_of(file, card);
        landings(homes, card)
            .iter()
            .filter(|one| agents::same_file(&one.at, at))
            .any(|one| {
                let stand = self.0.get(&unit_of(card, &one.at));
                matches!(stand, Some(Stand::Goes(it)) if *it != source)
            })
    }
}

pub fn placed(
    homes: &Homes,
    gate: &Gate,
    ledger: &Ledger,
    plugged: &BTreeSet<&'static str>,
) -> Vec<Placed> {
    let applied = ledger.applied();
    let mut pieces: Vec<&agents::Admitted> = gate.admitted().iter().collect();
    pieces.sort_by_key(|one| (!applied.contains(one.file()), one.file()));
    let stands = stands(homes, gate, ledger);
    let mut out: BTreeMap<String, (Placed, bool)> = BTreeMap::new();
    // The same skill brought twice is one skill, never written twice.
    let mut seen = BTreeSet::new();
    for piece in pieces {
        let card = piece.card();
        let shipped = agents::builtin(piece.file());
        for one in landings(homes, card) {
            if shipped && plugged.contains(one.assistant) {
                continue;
            }
            let source = source_of(piece.file(), card);
            let stand = stands
                .0
                .get(&unit_of(card, &one.at))
                .filter(|_| !shipped && card.kind != Kind::Memory);
            if matches!(stand, Some(Stand::Goes(it)) if *it != source) {
                continue;
            }
            let asks = stand == Some(&Stand::Asks);
            let key = agents::path_key(&one.at);
            if !seen.insert((key.clone(), source)) {
                continue;
            }
            let (placed, held) = out.entry(key).or_insert_with(|| {
                let target = gate.target(ledger, one.root, one.at, one.assistant);
                let placed = Placed {
                    target,
                    files: Vec::new(),
                    lead_in: None,
                };
                (placed, false)
            });
            placed.target.put(piece);
            placed.files.push(piece.file().to_string());
            *held |= asks;
        }
    }
    out.into_values()
        .map(|(mut placed, held)| {
            if held {
                placed.target.hold();
            }
            if placed.target.assistant == "gemini" && placed.target.at.ends_with(GEMINI_RULES) {
                let at = placed.target.root.join(GEMINI_MEMORY);
                let root = placed.target.root.clone();
                placed.lead_in = Some(gate.target(ledger, root, at, "gemini"));
            }
            placed
        })
        .collect()
}

/// The card of the file an assistant reads at this path, when it is one of its rules or skills.
pub fn card_at(homes: &Homes, at: &Path) -> Option<(&'static str, Kind, String)> {
    let named = |at: &Path, root: &Path| {
        at.strip_prefix(root).ok().map(|rest| {
            rest.components()
                .map(|step| step.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
    };
    homes.detected().into_iter().find_map(|one| {
        let root = homes.root(one.id)?;
        let (_, skills) = homes.skills_of(one.id)?;
        let (kind, name) = match named(at, &skills) {
            Some(name) => (Kind::Skill, name),
            None => (Kind::Rule, named(at, &root)?),
        };
        let card = Card {
            level: one.id.to_string(),
            kind,
            name: name.clone(),
            endings: agents::Endings::Lf,
            bom: false,
            open_end: false,
            computer: None,
            trigger: None,
        };
        let (_, there) = target_of(homes, &card)?;
        agents::same_file(&there, at).then_some((one.id, kind, name))
    })
}

pub fn owner_of(homes: &Homes, at: &Path) -> Option<(&'static str, PathBuf)> {
    let detected = homes.detected();
    if at.starts_with(homes.shared()) {
        return homes.shared_owner(&detected).map(|id| (id, homes.shared()));
    }
    detected.into_iter().find_map(|one| {
        homes
            .root(one.id)
            .filter(|root| at.starts_with(root))
            .map(|root| (one.id, root))
    })
}

pub fn readers_of(homes: &Homes, at: &Path, owner: &str) -> Vec<&'static Assistant> {
    if !at.starts_with(homes.shared()) {
        return assistant(owner).into_iter().collect();
    }
    homes
        .detected()
        .into_iter()
        .filter(|one| SHARED_SKILLS_OWNERS.contains(&one.id))
        .collect()
}

// A copy Tisty wrote stays its to take away even after its agent left this computer or moved its folder.
pub fn owner_any(homes: &Homes, at: &Path) -> Option<(&'static str, PathBuf)> {
    if let Some(found) = owner_of(homes, at) {
        return Some(found);
    }
    if at.starts_with(homes.shared()) {
        return Some((SHARED_SKILLS_OWNERS[0], homes.shared()));
    }
    ASSISTANTS.iter().find_map(|one| {
        homes
            .root(one.id)
            .filter(|root| at.starts_with(root))
            .map(|root| (one.id, root))
    })
}

// The agent writes its memory itself, so Tisty never offers to take it away.
pub fn agents_own(homes: &Homes, at: &Path) -> bool {
    let gemini = homes
        .root("gemini")
        .is_some_and(|root| at == root.join(GEMINI_MEMORY));
    let claude = homes
        .root("claude")
        .is_some_and(|root| at.starts_with(root.join("projects")));
    gemini || claude
}

// Gemini reads Tisty's rules only through the line in its own file.
pub fn lead_of(rules: &Target) -> Option<Target> {
    (rules.assistant == "gemini" && rules.at == rules.root.join(GEMINI_RULES)).then(|| {
        let mut lead = rules.clone();
        lead.at = rules.root.join(GEMINI_MEMORY);
        lead
    })
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
