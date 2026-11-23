use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::json;
use super::places::{Os, Place};

const HOOKS: &str = "hooks";
const COMMANDS: [&str; 4] = ["command", "exec", "bash", "powershell"];
const START_SECONDS: u64 = 15;
const TURN_SECONDS: u64 = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Claude,
    Cursor,
    Gemini,
    Copilot,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hooks {
    pub on: bool,
    pub at: String,
    /// The Tisty plugin brings its own hooks, which live in the plugin and never in this file.
    pub plugged: bool,
    pub astray: bool,
    pub gone: bool,
    pub points: Option<String>,
}

pub fn kind(client: &str) -> Option<Kind> {
    match client {
        "claude-code" => Some(Kind::Claude),
        "cursor" => Some(Kind::Cursor),
        "gemini-cli" => Some(Kind::Gemini),
        "copilot" => Some(Kind::Copilot),
        _ => None,
    }
}

impl Kind {
    pub fn file(self, place: &Place) -> Option<PathBuf> {
        place.resolve(match self {
            Kind::Claude => "~/.claude/settings.json",
            Kind::Cursor => "~/.cursor/hooks.json",
            Kind::Gemini => "~/.gemini/settings.json",
            Kind::Copilot => "~/.copilot/hooks/tisty.json",
        })
    }

    fn client(self) -> &'static str {
        match self {
            Kind::Claude => "claude-code",
            Kind::Cursor => "cursor",
            Kind::Gemini => "gemini-cli",
            Kind::Copilot => "copilot",
        }
    }

    fn events(self) -> &'static [&'static str] {
        match self {
            Kind::Claude => &["SessionStart", "PostToolUse", "Stop"],
            Kind::Cursor => &["sessionStart", "afterFileEdit", "afterMCPExecution", "stop"],
            Kind::Gemini => &["SessionStart", "AfterTool", "AfterAgent"],
            Kind::Copilot => &["sessionStart", "postToolUse", "agentStop"],
        }
    }

    fn entries(self, place: &Place) -> Vec<(&'static str, Value)> {
        let run = |moment: &str, seconds: u64| self.run(place, moment, seconds);
        let start = run("start", START_SECONDS);
        let tool = run("tool", TURN_SECONDS);
        let stop = run("stop", TURN_SECONDS);
        match self {
            Kind::Claude => vec![
                ("SessionStart", json!({ "hooks": [start] })),
                (
                    "PostToolUse",
                    json!({ "matcher": "Edit|Write|MultiEdit|NotebookEdit", "hooks": [tool] }),
                ),
                (
                    "PostToolUse",
                    json!({ "matcher": "^(mcp__(tisty|plugin_tisty_tisty)__|MCP:)", "hooks": [tool] }),
                ),
                ("Stop", json!({ "hooks": [stop] })),
            ],
            Kind::Cursor => {
                let mut stop = stop;
                // Cursor counts follow-ups over a whole conversation; Tisty's marks keep it to one a turn.
                stop["loop_limit"] = Value::Null;
                vec![
                    ("sessionStart", start),
                    ("afterFileEdit", run("edit", TURN_SECONDS)),
                    ("afterMCPExecution", tool),
                    ("stop", stop),
                ]
            }
            Kind::Gemini => vec![
                ("SessionStart", json!({ "matcher": "*", "hooks": [start] })),
                (
                    "AfterTool",
                    json!({ "matcher": "^(write_file|replace)$", "hooks": [tool] }),
                ),
                (
                    "AfterTool",
                    json!({ "matcher": "^mcp_tisty_", "hooks": [tool] }),
                ),
                ("AfterAgent", json!({ "hooks": [stop] })),
            ],
            Kind::Copilot => {
                let mut edits = tool.clone();
                edits["matcher"] = json!("create|edit|apply_patch|str_replace_editor");
                let mut ours = tool;
                ours["matcher"] = json!("tisty-.*");
                vec![
                    ("sessionStart", start),
                    ("postToolUse", edits),
                    ("postToolUse", ours),
                    ("agentStop", stop),
                ]
            }
        }
    }

    fn run(self, place: &Place, moment: &str, seconds: u64) -> Value {
        let args = ["hook", self.client(), moment];
        match self {
            Kind::Claude => json!({
                "type": "command", "command": place.command, "args": args, "timeout": seconds
            }),
            Kind::Cursor => json!({ "command": spoken(place, &args), "timeout": seconds }),
            Kind::Gemini => json!({
                "name": format!("tisty-{moment}"), "type": "command",
                "command": spoken(place, &args), "timeout": seconds * 1_000
            }),
            Kind::Copilot => json!({
                "type": "command", "exec": place.command, "args": args, "timeoutSec": seconds
            }),
        }
    }
}

// Cursor and Gemini run a hook through PowerShell on Windows and a POSIX shell elsewhere.
fn spoken(place: &Place, args: &[&str]) -> String {
    let rest = args.join(" ");
    match place.os == Os::Windows && place.distro.is_none() {
        true => format!("& '{}' {rest}", place.command.replace('\'', "''")),
        false => format!("'{}' {rest}", place.command.replace('\'', "'\\''")),
    }
}

pub fn seen(kind: Kind, place: &Place) -> Option<Hooks> {
    let at = kind.file(place)?;
    let text = std::fs::read_to_string(&at).unwrap_or_default();
    let stray: Vec<String> = programs(kind, &text)
        .into_iter()
        .filter(|one| !super::this_tisty(place, one))
        .collect();
    let gone = stray.iter().find(|one| super::lost(place, one));
    Some(Hooks {
        on: hooked(kind, &text),
        at: at.display().to_string(),
        plugged: false,
        astray: !stray.is_empty(),
        gone: gone.is_some(),
        points: gone.or(stray.first()).cloned(),
    })
}

pub fn hooked(kind: Kind, text: &str) -> bool {
    !runs(kind, text).is_empty()
}

fn runs(kind: Kind, text: &str) -> Vec<Value> {
    let spans = |event: &&str| {
        list(text, event)
            .and_then(|at| json::elements(text.as_bytes(), at))
            .unwrap_or_default()
    };
    kind.events()
        .iter()
        .flat_map(spans)
        .flat_map(|span| ours(kind, &text[span.0..span.1]))
        .collect()
}

// What cannot be read for certain is left out, so a wrapper or a bare name is never said to be broken.
fn programs(kind: Kind, text: &str) -> Vec<String> {
    runs(kind, text)
        .iter()
        .filter_map(|run| program(kind, run))
        .filter(|one| rooted(one) && named_tisty(one))
        .collect()
}

fn program(kind: Kind, run: &Value) -> Option<String> {
    let said = COMMANDS
        .iter()
        .find_map(|field| run.get(*field)?.as_str())?;
    if run.get("args").is_some_and(Value::is_array) {
        return Some(said.to_string());
    }
    let cut = said.find(&format!(" {}", marked(kind)))?;
    Some(unquoted(said[..cut].trim()))
}

fn unquoted(said: &str) -> String {
    let said = said.strip_prefix("& ").unwrap_or(said).trim();
    for (quote, escapes) in [("'", ["'\\''", "''"]), ("\"", ["\\\"", "\"\""])] {
        let inside = said
            .strip_prefix(quote)
            .and_then(|it| it.strip_suffix(quote));
        if let Some(inside) = inside {
            return escapes
                .iter()
                .fold(inside.to_string(), |it, escape| it.replace(escape, quote));
        }
    }
    said.to_string()
}

fn rooted(said: &str) -> bool {
    let drive = said.as_bytes();
    let on_a_drive = drive.len() > 2
        && drive[0].is_ascii_alphabetic()
        && drive[1] == b':'
        && matches!(drive[2], b'\\' | b'/');
    said.starts_with(['/', '\\']) || on_a_drive
}

fn named_tisty(said: &str) -> bool {
    let name = said.rsplit(['/', '\\']).next().unwrap_or(said);
    ["tisty", "tisty.exe"]
        .iter()
        .any(|one| name.eq_ignore_ascii_case(one))
}

fn list(text: &str, event: &str) -> Option<usize> {
    let raw = text.as_bytes();
    let under = json::member(raw, json::opens(raw)?, HOOKS)?;
    let at = json::member(raw, under.value.0, event)?.value.0;
    (raw.get(at) == Some(&b'[')).then_some(at)
}

fn marked(kind: Kind) -> String {
    format!("hook {}", kind.client())
}

fn ours(kind: Kind, element: &str) -> Vec<Value> {
    let Ok(read) = serde_json::from_str::<Value>(element) else {
        return Vec::new();
    };
    let inner = read.get(HOOKS).and_then(Value::as_array).cloned();
    std::iter::once(read)
        .chain(inner.unwrap_or_default())
        .filter(|run| is_ours(kind, run))
        .collect()
}

fn is_ours(kind: Kind, run: &Value) -> bool {
    let mut said: Vec<&str> = COMMANDS
        .iter()
        .filter_map(|field| run.get(*field)?.as_str())
        .collect();
    if let Some(args) = run.get("args").and_then(Value::as_array) {
        said.extend(args.iter().filter_map(Value::as_str));
    }
    let said = said.join(" ");
    said.to_lowercase().contains("tisty") && said.contains(&marked(kind))
}

pub fn without(kind: Kind, text: &str) -> Option<String> {
    let mut now = text.to_string();
    for event in kind.events() {
        while let Some(at) = list(&now, event) {
            let all = json::elements(now.as_bytes(), at)?;
            let Some(&span) = all
                .iter()
                .find(|span| !ours(kind, &now[span.0..span.1]).is_empty())
            else {
                break;
            };
            now = json::cut(&now, span);
            if all.len() == 1 {
                now = json::unset(&now, HOOKS, event)?;
            }
        }
    }
    Some(now)
}

pub fn with(kind: Kind, text: &str, place: &Place) -> Option<String> {
    let mut now = without(kind, text)?;
    if now.trim().is_empty() {
        now = match kind {
            Kind::Cursor | Kind::Copilot => "{\n  \"version\": 1\n}\n".to_string(),
            Kind::Claude | Kind::Gemini => "{\n}\n".to_string(),
        };
    }
    for (event, entry) in kind.entries(place) {
        let piece = serde_json::to_string_pretty(&entry).ok()?;
        now = match list(&now, event) {
            Some(at) => json::push(&now, at, &piece)?,
            None => {
                let laid: Vec<String> = piece.lines().map(|line| format!("  {line}")).collect();
                json::set(&now, HOOKS, event, &format!("[\n{}\n]", laid.join("\n")))?
            }
        };
    }
    Some(now)
}

pub fn alone(kind: Kind) -> bool {
    kind == Kind::Copilot
}

pub fn emptied(at: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(at) {
        Err(why) if why.kind() != std::io::ErrorKind::NotFound => Err(why),
        _ => Ok(()),
    }
}

#[cfg(test)]
#[path = "hooks_test.rs"]
mod tests;
