use crate::{
    Config, Result,
    event::{DeviceId, DeviceKind, Op},
    paths::Paths,
    store::Store,
};

/// Minting is the person's act: nothing reachable over a wire calls this.
pub fn register(paths: &Paths) -> Result<DeviceId> {
    let mut config = Config::load_or_init(paths)?;
    if let Some(held) = config.agent_id.clone() {
        return Ok(held);
    }

    let who = DeviceId(crate::config::new_device_id());
    // Joining without a key would write what every other machine turns away from the 17 on.
    let Some(key) = crate::signing::mine(paths, &who) else {
        return Err(crate::Error::Io(std::io::Error::other(
            "a key for the agent could not be kept on this machine",
        )));
    };
    let shown = crate::signing::shown(&key);
    config.agent_id = Some(who.clone());
    config.save(paths)?;

    crate::vouched::confirm(paths.data(), &who, &shown);
    let mut store = Store::open(paths.store(), who.clone())?.signing_with(Some(key));
    store.append_batch(vec![
        Op::DeviceJoin {
            d: who.clone(),
            k: Some(DeviceKind::Agent),
            p: Some(shown),
        },
        Op::DeviceHost {
            d: who.clone(),
            of: config.device_id.clone(),
            p: None,
        },
    ])?;
    if let Some(key) = crate::signing::shown_kept(paths, &who) {
        speak_for(paths, &mut config, key)?;
    }
    Ok(who)
}

/// Opening Tisty is the person's act, so the door opens with it until they choose otherwise.
pub fn open_by_default(paths: &Paths) -> Result<Option<DeviceId>> {
    let mut config = Config::load_or_init(paths)?;
    if config.agent_id.is_some() || config.door_chosen.is_some() {
        return Ok(None);
    }
    // An agent that lived here and left was shut out by the person before this choice was kept.
    let shut_before = crate::store::read_all(paths.store())?
        .iter()
        .any(|one| matches!(&one.op, Op::DeviceHost { of, .. } if *of == config.device_id));
    if shut_before {
        config.door_chosen = Some(false);
        config.save(paths)?;
        return Ok(None);
    }
    register(paths).map(Some)
}

/// An agent that joined before `device.host` existed says where it lives, even with no key to vouch.
pub fn unhosted(config: &Config, state: &crate::State) -> Option<Op> {
    let who = config.agent_id.clone()?;
    if state.hosts.contains_key(&who) || !state.assistants.contains(&who) {
        return None;
    }
    Some(Op::DeviceHost {
        d: who,
        of: config.device_id.clone(),
        p: None,
    })
}

/// For an agent minted before hosts spoke for theirs: once, and never with a key made up on the spot.
pub fn vouch(config: &Config, paths: &Paths, log: &[crate::event::Event]) -> Option<String> {
    let who = config.agent_id.clone()?;
    let me = &config.device_id;
    let joined = log
        .iter()
        .any(|one| one.device == who && matches!(&one.op, Op::DeviceJoin { d, .. } if d == &who));
    let said = log.iter().any(|one| {
        &one.device == me
            && matches!(&one.op, Op::DeviceHost { d, of, p: Some(_) } if d == &who && of == me)
    });
    if !joined || said {
        return None;
    }
    crate::signing::shown_kept(paths, &who)
}

/// Written as the host, in the host's own history, which is what a machine that confirmed it trusts.
pub fn speak_for(paths: &Paths, config: &mut Config, key: String) -> Result<()> {
    let Some(who) = config.agent_id.clone() else {
        return Ok(());
    };
    let me = config.device_id.clone();
    let mut mine =
        Store::open(paths.store(), me.clone())?.signing_with(crate::signing::mine(paths, &me));
    mine.append(Op::DeviceHost {
        d: who.clone(),
        of: me,
        p: Some(key),
    })?;
    config.agent_vouched = Some(who);
    config.save(paths)
}

/// What the MCP client called itself, made fit to keep: one line, composed, forty characters.
pub fn client_said(raw: &str) -> Option<String> {
    let one: String = crate::text::composed(raw.trim())
        .chars()
        .filter(|c| {
            !c.is_control()
                && !matches!(
                    *c,
                    '\u{200b}'..='\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2060}'..='\u{206f}' | '\u{feff}'
                )
        })
        .take(40)
        .collect();
    let one = one.trim().to_string();
    (!one.is_empty()).then_some(one)
}

const CLIENTS: &[(&str, &str, &[&str])] = &[
    ("claude-code", "Claude Code", &["claude code"]),
    (
        "claude-desktop",
        "Claude Desktop",
        &["claude desktop", "claude-ai"],
    ),
    (
        "codex",
        "Codex",
        &["codex-cli", "codex cli", "codex-mcp-client"],
    ),
    ("antigravity", "Antigravity", &[]),
    (
        "gemini-cli",
        "Gemini CLI",
        &["gemini", "gemini-cli-mcp-client"],
    ),
    ("opencode", "OpenCode", &[]),
    (
        "vscode",
        "Visual Studio Code",
        &["visual studio code", "visual studio code - insiders"],
    ),
    ("cursor", "Cursor", &["cursor-agent"]),
    ("windsurf", "Windsurf", &[]),
    ("zed", "Zed", &[]),
];

/// The id a known client is wired under, whatever it called itself: `codex-mcp-client` is
/// the same hand as the `codex` in the person's settings.
pub fn client_id(raw: &str) -> Option<&'static str> {
    let key = raw.trim().to_lowercase();
    let said = |name: &str| key == name || key.starts_with(&format!("{name}/"));
    CLIENTS
        .iter()
        .find(|(id, _, aliases)| said(id) || aliases.iter().any(|alias| said(alias)))
        .map(|(id, _, _)| *id)
}

pub fn client_named(raw: &str) -> String {
    if let Some(id) = client_id(raw)
        && let Some((_, named, _)) = CLIENTS.iter().find(|(one, _, _)| one == &id)
    {
        return (*named).to_string();
    }
    raw.trim()
        .to_lowercase()
        .split(['-', '_', ' '])
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// What it wrote stays: retiring takes the voice, never the words.
pub fn retire(paths: &Paths) -> Result<Option<DeviceId>> {
    let mut config = Config::load_or_init(paths)?;
    let Some(who) = config.agent_id.clone() else {
        return Ok(None);
    };

    config.agent_id = None;
    config.save(paths)?;

    let mut store = Store::open(paths.store(), config.device_id.clone())?
        .signing_with(crate::signing::mine(paths, &config.device_id));
    store.append(Op::DeviceRemove { d: who.clone() })?;
    Ok(Some(who))
}

const KNOCKED: &str = "knocked.json";
const KNOCKED_AGAIN: jiff::SignedDuration = jiff::SignedDuration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Knock {
    pub at: jiff::Timestamp,
    pub by: Option<String>,
}

/// A hand turned away at a closed door, kept so the window can tell the person it came.
pub fn knock(paths: &Paths, by: Option<&str>) -> Result<()> {
    knocked_at(paths, by, jiff::Timestamp::now())
}

fn knocked_at(paths: &Paths, by: Option<&str>, now: jiff::Timestamp) -> Result<()> {
    let again = knocked(paths).is_some_and(|last| {
        last.by.as_deref() == by && now.duration_since(last.at) < KNOCKED_AGAIN
    });
    if again {
        return Ok(());
    }
    let private = paths.private();
    std::fs::create_dir_all(&private)?;
    let _ = crate::paths::ours_alone(&private);
    let told = serde_json::to_vec(&Knock {
        at: now,
        by: by.map(str::to_string),
    })?;
    crate::store::write_atomic(&private.join(KNOCKED), &told)
}

pub fn knocked(paths: &Paths) -> Option<Knock> {
    let told = std::fs::read(paths.private().join(KNOCKED)).ok()?;
    serde_json::from_slice(&told).ok()
}

pub fn registered(paths: &Paths) -> Result<Option<DeviceId>> {
    Ok(Config::load_or_init(paths)?.agent_id)
}

/// Where an agent may take a file from: what it attaches reaches the shared folder.
pub fn may_attach(source: &std::path::Path, paths: &Paths) -> Result<std::path::PathBuf> {
    let refused = || crate::Error::OutsideTheStore(source.display().to_string());
    let at = source.canonicalize().map_err(|_| refused())?;

    let mine = [paths.data(), paths.config(), paths.cache()];
    if mine
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .any(|one| at.starts_with(one))
    {
        return Err(refused());
    }

    if !reachable().iter().any(|root| at.starts_with(root)) {
        return Err(refused());
    }
    fit_to_keep(&at)?;
    Ok(at)
}

fn named_type(at: &std::path::Path) -> Option<String> {
    Some(at.extension()?.to_str()?.to_ascii_lowercase())
}

fn signed_as(kind: &str, head: &[u8]) -> bool {
    let starts = |mark: &[u8]| head.starts_with(mark);
    match kind {
        "png" => starts(b"\x89PNG\r\n\x1a\n"),
        "jpg" | "jpeg" => starts(&[0xFF, 0xD8, 0xFF]),
        "gif" => starts(b"GIF87a") || starts(b"GIF89a"),
        "webp" => starts(b"RIFF") && head.len() > 12 && &head[8..12] == b"WEBP",
        "avif" | "heic" | "mp4" | "m4v" | "mov" => head.len() > 12 && &head[4..8] == b"ftyp",
        "webm" | "ogv" | "ogg" => starts(&[0x1A, 0x45, 0xDF, 0xA3]) || starts(b"OggS"),
        "mp3" => starts(b"ID3") || (head.len() > 1 && head[0] == 0xFF && head[1] & 0xE0 == 0xE0),
        "m4a" => head.len() > 12 && &head[4..8] == b"ftyp",
        "wav" => starts(b"RIFF") && head.len() > 12 && &head[8..12] == b"WAVE",
        "pdf" => starts(b"%PDF-"),
        "docx" | "xlsx" | "pptx" | "odt" | "ods" | "zip" => starts(&[0x50, 0x4B]),
        "7z" => starts(&[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]),
        "gz" | "tgz" => starts(&[0x1F, 0x8B]),
        // A tar says what it is 257 bytes in, where the first entry's header carries the marker.
        "tar" => head.len() > 262 && &head[257..262] == b"ustar",
        _ => true,
    }
}

pub struct Telling {
    pub line: usize,
    pub named: String,
    pub why: &'static str,
}

impl Telling {
    pub fn said(&self) -> String {
        match self.named.is_empty() {
            true => format!("line {}: {}", self.line, self.why),
            false => format!("line {}, {}: {}", self.line, self.named, self.why),
        }
    }
}

const SOUNDS_LIKE: &[&str] = &[
    "SECRET",
    "PASSWORD",
    "PASSWD",
    "PWD",
    "TOKEN",
    "APIKEY",
    "API_KEY",
    "CREDENTIAL",
    "PRIVATE",
    "AUTH",
    "SIGNATURE",
    "KEY",
];

const KNOWN_STARTS: &[&str] = &[
    "AKIA",
    "ASIA",
    "ghp_",
    "gho_",
    "ghu_",
    "ghs_",
    "ghr_",
    "github_pat_",
    "glpat-",
    "sk-",
    "sk_live_",
    "sk_test_",
    "rk_live_",
    "pk_live_",
    "xoxb-",
    "xoxp-",
    "xoxa-",
    "xoxe-",
    "xapp-",
    "AIza",
    "ya29.",
    "eyJ",
    "SG.",
    "npm_",
    "dop_v1_",
    "doo_v1_",
    "hf_",
    "shpat_",
    "shpss_",
    "sq0atp-",
    "sq0csp-",
    "lin_api_",
    "https://hooks.slack.com/services/",
    "https://discord.com/api/webhooks/",
    "figd_",
];

const SHORTEST: usize = 8;
const SHORTEST_ALONE: usize = 24;
const SHORTEST_KNOWN: usize = 20;

fn bared(said: &str) -> &str {
    let said = said.trim().trim_end_matches([',', ';']).trim();
    for mark in ['"', '\'', '`'] {
        if let Some(inner) = said
            .strip_prefix(mark)
            .and_then(|one| one.strip_suffix(mark))
        {
            return inner.trim();
        }
    }
    said
}

fn a_name(key: &str) -> Option<&str> {
    let name = key.trim().rsplit([' ', '\t']).next()?.trim();
    let name = name.trim_start_matches(['"', '\'', '`']);
    let plain = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    plain.then_some(name)
}

fn sounds_like_a_credential(name: &str) -> bool {
    let loud = name.to_ascii_uppercase();
    SOUNDS_LIKE.iter().any(|one| loud.contains(one))
}

fn stands_for_something_else(value: &str) -> bool {
    value.len() < SHORTEST
        || value.chars().any(char::is_whitespace)
        || value.starts_with(['$', '%', '<', '{', '~', '/'])
        || value.starts_with("./")
        || value.starts_with("..")
        || value.contains("${")
        || value.contains("$(")
}

fn mixed(value: &str) -> bool {
    let upper = value.chars().any(|c| c.is_ascii_uppercase());
    let lower = value.chars().any(|c| c.is_ascii_lowercase());
    let digit = value.chars().any(|c| c.is_ascii_digit());
    usize::from(upper) + usize::from(lower) + usize::from(digit) >= 2
}

fn a_link_with_a_password(value: &str) -> bool {
    let Some((_, rest)) = value.split_once("://") else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    host.split_once('@')
        .and_then(|(who, _)| who.split_once(':'))
        .is_some_and(|(_, word)| !stands_for_something_else(word) && mixed(word))
}

fn only_a_link(value: &str) -> bool {
    value.contains("://") && !a_link_with_a_password(value)
}

fn carries_a_prefix(value: &str) -> bool {
    value.split_whitespace().any(|word| {
        word.len() >= SHORTEST_KNOWN && KNOWN_STARTS.iter().any(|one| word.starts_with(one))
    })
}

fn told_alone(line: &str) -> Option<(bool, &str, &'static str)> {
    let flat = bared(line.trim());
    if flat.split_whitespace().count() != 1 {
        return None;
    }
    if carries_a_prefix(flat) {
        let named = KNOWN_STARTS
            .iter()
            .find(|one| flat.starts_with(*one))
            .copied()
            .unwrap_or_default();
        return Some((true, named, "a value carrying a provider's key prefix"));
    }
    a_link_with_a_password(flat).then(|| {
        (
            true,
            flat.split("://").next().unwrap_or_default(),
            "a link with a password written into it",
        )
    })
}

fn told_by(line: &str) -> Option<(bool, &str, &'static str)> {
    written_as(line, '=').or_else(|| written_as(line, ':'))
}

// A shell line goes on past ` \`, and the quote that opened before its name closes after the value.
fn in_a_command(value: &str) -> &str {
    let value = bared(value.trim().trim_end_matches('\\'));
    match value.strip_suffix(['"', '\'', '`']) {
        Some(inner) if !value.starts_with(['"', '\'', '`']) => inner,
        _ => value,
    }
}

// An authorization header names its scheme before the credential itself.
fn past_the_scheme<'a>(name: &str, value: &'a str) -> &'a str {
    if !name.to_ascii_uppercase().contains("AUTH") {
        return value;
    }
    let word = value.split_whitespace().next().unwrap_or_default();
    match ["Bearer", "Basic", "Token"]
        .iter()
        .any(|one| word.eq_ignore_ascii_case(one))
    {
        true => value[word.len()..].trim(),
        false => value,
    }
}

fn written_as(line: &str, mark: char) -> Option<(bool, &str, &'static str)> {
    let (key, value) = line.split_once(mark)?;
    let name = a_name(bared(key))?;
    let value = past_the_scheme(name, in_a_command(bared(value)));
    if carries_a_prefix(value) {
        return Some((true, name, "a value carrying a provider's key prefix"));
    }
    if a_link_with_a_password(value) {
        return Some((true, name, "a link with a password written into it"));
    }
    if stands_for_something_else(value) || only_a_link(value) || !mixed(value) {
        return None;
    }
    if !sounds_like_a_credential(name) {
        return None;
    }
    Some((
        value.len() >= SHORTEST_ALONE,
        name,
        "a name that says credential and a value that is not a placeholder",
    ))
}

pub fn a_key_itself(head: &[u8]) -> Option<Telling> {
    let told = |line: usize, why| Telling {
        line,
        named: String::new(),
        why,
    };
    if head.starts_with(&[0x30, 0x82]) {
        return Some(told(1, "a DER-encoded key"));
    }
    let text = String::from_utf8_lossy(head);
    if text.contains("-----BEGIN") && (text.contains("PRIVATE KEY") || text.contains("CERTIFICATE"))
    {
        let at = text
            .lines()
            .position(|one| one.contains("-----BEGIN"))
            .unwrap_or(0);
        return Some(told(at + 1, "a PEM private key or certificate"));
    }
    None
}

pub fn secrets_in(head: &[u8]) -> Vec<Telling> {
    if let Some(told) = a_key_itself(head) {
        return vec![told];
    }
    let text = String::from_utf8_lossy(head);
    let mut seen = Vec::new();
    let mut sure = false;
    for (at, line) in text.lines().enumerate() {
        let Some((certain, name, why)) = told_by(line).or_else(|| told_alone(line)) else {
            continue;
        };
        sure |= certain;
        seen.push(Telling {
            line: at + 1,
            named: name.to_string(),
            why,
        });
    }
    match sure || seen.len() >= 2 {
        true => seen,
        false => Vec::new(),
    }
}

pub fn secret_in(head: &[u8]) -> Option<Telling> {
    secrets_in(head).into_iter().next()
}

/// Whether the file itself, not its name, is something an assistant is allowed to copy.
pub fn fit_to_keep(at: &std::path::Path) -> Result<()> {
    let refused = || crate::Error::NotForAnAgent(at.display().to_string());
    let kind = named_type(at).unwrap_or_default();
    let head = read_head(at).map_err(|_| refused())?;
    if !signed_as(&kind, &head) {
        return Err(refused());
    }
    Ok(())
}

fn read_head(at: &std::path::Path) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let mut head = vec![0u8; 4096];
    let read = std::fs::File::open(at)?.read(&mut head)?;
    head.truncate(read);
    Ok(head)
}

pub fn may_reach(at: &std::path::Path, paths: &Paths) -> Result<std::path::PathBuf> {
    let refused = || crate::Error::OutsideTheStore(at.display().to_string());
    let found = at.canonicalize().map_err(|_| refused())?;

    let mine = [paths.data(), paths.config(), paths.cache()];
    if mine
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .any(|one| found.starts_with(one))
    {
        return Err(refused());
    }
    if !reachable().iter().any(|root| found.starts_with(root)) {
        return Err(refused());
    }
    Ok(found)
}

pub fn reachable() -> Vec<std::path::PathBuf> {
    let mut roots = vec![std::env::temp_dir()];
    if let Some(dirs) = directories::UserDirs::new() {
        for one in [
            dirs.download_dir(),
            dirs.document_dir(),
            dirs.picture_dir(),
            dirs.desktop_dir(),
        ] {
            roots.extend(one.map(std::path::Path::to_path_buf));
        }
    }
    roots
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .collect()
}

#[cfg(test)]
#[path = "agent_test.rs"]
mod tests;

#[cfg(test)]
#[path = "agent_naming.rs"]
mod naming;
