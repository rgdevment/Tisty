use std::path::PathBuf;

use serde_json::{Value, json};

use super::files::{self, Read};
use super::json;
use super::leads::{Leads, leads};
use super::places::{Os, Place};
use super::running::{self, New, Program, Run, Spoken};

const HOOKS: &str = "hooks";
/// Antigravity keys its hooks by name, so Tisty's live under a name of their own.
const ANTIGRAVITY_ROOT: &str = "tisty";
const START_SECONDS: u64 = 15;
const TURN_SECONDS: u64 = 5;
const BEFORE_SECONDS: u64 = 3;
const BEFORE: &str = "before";
const SHELL: &str = "shell";
// Cursor runs a case-sensitive JS RegExp, so character classes match TODO/NOTES/.MD without ES2025 modifiers.
const PROSE_WRITTEN: &str = concat!(
    r"(\.([mM][dD]|[mM][aA][rR][kK][dD][oO][wW][nN]|[tT][xX][tT]|[rR][sS][tT]|[aA][dD][oO][cC]|[oO][rR][gG])",
    r"|\b([hH][aA][nN][dD][oO][fF][fF]|[nN][oO][tT][eE][sS]|[tT][oO][dD][oO]|[pP][lL][aA][nN]|[pP][rR][oO][mM][pP][tT]|[bB][rR][iI][eE][fF])\b)"
);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Claude,
    Cursor,
    Gemini,
    Copilot,
    Codex,
    Antigravity,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hooks {
    pub on: bool,
    pub at: String,
    /// The Tisty plugin brings its own hooks, which live in the plugin and never in this file.
    pub plugged: bool,
    pub plugin_unknown: bool,
    pub unread: bool,
    /// Run in a way of the person's own, so Tisty neither changes it nor adds hers beside it.
    pub apart: bool,
    /// Added before the hook that asks first existed, or with it taken out by hand.
    pub lacks_before: bool,
    pub stray: Vec<Stray>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Stray {
    pub said: String,
    pub gone: bool,
}

pub fn kind(client: &str) -> Option<Kind> {
    match client {
        "claude-code" => Some(Kind::Claude),
        "cursor" => Some(Kind::Cursor),
        "gemini-cli" => Some(Kind::Gemini),
        "copilot" => Some(Kind::Copilot),
        "codex" => Some(Kind::Codex),
        "antigravity" => Some(Kind::Antigravity),
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
            Kind::Codex => "~/.codex/hooks.json",
            // Antigravity CLI reads hooks here, never from its own settings.json.
            Kind::Antigravity => "~/.gemini/config/hooks.json",
        })
    }

    fn root(self) -> &'static str {
        match self {
            Kind::Antigravity => ANTIGRAVITY_ROOT,
            _ => HOOKS,
        }
    }

    pub(super) fn client(self) -> &'static str {
        match self {
            Kind::Claude => "claude-code",
            Kind::Cursor => "cursor",
            Kind::Gemini => "gemini-cli",
            Kind::Copilot => "copilot",
            Kind::Codex => "codex",
            Kind::Antigravity => "antigravity",
        }
    }

    fn mark(self) -> String {
        format!("hook {}", self.client())
    }

    fn events(self) -> &'static [&'static str] {
        match self {
            Kind::Claude => &["SessionStart", "PreToolUse", "PostToolUse", "Stop"],
            Kind::Cursor => &[
                "sessionStart",
                "preToolUse",
                "beforeShellExecution",
                "afterShellExecution",
                "afterFileEdit",
                "afterMCPExecution",
                "stop",
            ],
            Kind::Gemini => &["SessionStart", "BeforeTool", "AfterTool", "AfterAgent"],
            Kind::Copilot => &["sessionStart", "preToolUse", "postToolUse", "agentStop"],
            Kind::Codex | Kind::Antigravity => {
                &["SessionStart", "PreToolUse", "PostToolUse", "Stop"]
            }
        }
    }

    fn entries(self, place: &Place) -> Vec<(&'static str, Value)> {
        let mut all = self.turn_entries(place);
        all.extend(self.before_entries(place));
        all
    }

    fn before_entries(self, place: &Place) -> Vec<(&'static str, Value)> {
        let run = self.run(place, BEFORE, BEFORE_SECONDS);
        let matched = |matcher: &str| {
            let mut run = run.clone();
            run["matcher"] = json!(matcher);
            run
        };
        match self {
            Kind::Claude => vec![(
                "PreToolUse",
                json!({
                    "matcher": "^(Write|Bash|PowerShell|mcp__ccd_session__start_session)$",
                    "hooks": [run]
                }),
            )],
            Kind::Cursor => vec![
                ("preToolUse", matched("Write")),
                ("beforeShellExecution", matched(PROSE_WRITTEN)),
            ],
            Kind::Gemini => vec![(
                "BeforeTool",
                json!({ "matcher": "^(write_file|run_shell_command)$", "hooks": [run] }),
            )],
            Kind::Copilot => vec![("preToolUse", matched("create|bash|powershell|shell"))],
            Kind::Codex => vec![(
                "PreToolUse",
                json!({ "matcher": "^(Bash|apply_patch)$", "hooks": [run] }),
            )],
            Kind::Antigravity => vec![(
                "PreToolUse",
                json!({ "matcher": "^(write_to_file|run_command)$", "hooks": [run] }),
            )],
        }
    }

    fn turn_entries(self, place: &Place) -> Vec<(&'static str, Value)> {
        let run = |moment: &str, seconds: u64| self.run(place, moment, seconds);
        let start = run("start", START_SECONDS);
        let tool = run("tool", TURN_SECONDS);
        let stop = run("stop", TURN_SECONDS);
        let shell = run(SHELL, TURN_SECONDS);
        let after_shell = |matcher: &str| json!({ "matcher": matcher, "hooks": [shell.clone()] });
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
                ("PostToolUse", after_shell("^(Bash|PowerShell)$")),
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
                    ("afterShellExecution", shell),
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
                ("AfterTool", after_shell("^run_shell_command$")),
                ("AfterAgent", json!({ "hooks": [stop] })),
            ],
            Kind::Copilot => {
                let mut edits = tool.clone();
                edits["matcher"] = json!("create|edit|apply_patch|str_replace_editor");
                let mut ours = tool;
                ours["matcher"] = json!("tisty-.*");
                let mut shells = shell;
                shells["matcher"] = json!("bash|powershell|shell");
                vec![
                    ("sessionStart", start),
                    ("postToolUse", edits),
                    ("postToolUse", ours),
                    ("postToolUse", shells),
                    ("agentStop", stop),
                ]
            }
            Kind::Codex => vec![
                ("SessionStart", json!({ "hooks": [start] })),
                (
                    "PostToolUse",
                    json!({ "matcher": "^apply_patch$", "hooks": [tool] }),
                ),
                (
                    "PostToolUse",
                    json!({ "matcher": "^mcp__tisty__", "hooks": [tool] }),
                ),
                ("PostToolUse", after_shell("^Bash$")),
                ("Stop", json!({ "hooks": [stop] })),
            ],
            Kind::Antigravity => vec![
                ("SessionStart", start),
                (
                    "PostToolUse",
                    json!({
                        "matcher": "^(write_to_file|replace_file_content|multi_replace_file_content)$",
                        "hooks": [tool]
                    }),
                ),
                (
                    "PostToolUse",
                    json!({ "matcher": "^mcp_tisty_", "hooks": [tool] }),
                ),
                ("PostToolUse", after_shell("^run_command$")),
                ("Stop", stop),
            ],
        }
    }

    fn run(self, place: &Place, moment: &str, seconds: u64) -> Value {
        let args = ["hook", self.client(), moment];
        let named = format!("tisty-{moment}");
        self.run_with(place, &args, seconds, moment == BEFORE, &named)
    }

    pub(super) fn run_with(
        self,
        place: &Place,
        args: &[&str],
        seconds: u64,
        never_fails: bool,
        named: &str,
    ) -> Value {
        match self {
            Kind::Claude => json!({
                "type": "command", "command": place.command, "args": args, "timeout": seconds
            }),
            Kind::Cursor => json!({ "command": self.spoken(place, args), "timeout": seconds }),
            Kind::Gemini => json!({
                "name": named, "type": "command",
                "command": self.spoken(place, args), "timeout": seconds * 1_000
            }),
            Kind::Codex | Kind::Antigravity => json!({
                "type": "command", "command": self.spoken(place, args), "timeout": seconds
            }),
            // Copilot refuses the call when this hook fails, so it goes through a shell that never does.
            Kind::Copilot if never_fails => {
                let said = self.spoken(place, args);
                let (shell, never_fails) = match windows(place) {
                    true => ("powershell", format!("{said}; exit 0")),
                    false => ("bash", format!("{said} || true")),
                };
                let mut run = json!({ "type": "command", "timeoutSec": seconds });
                run[shell] = json!(never_fails);
                run
            }
            Kind::Copilot => json!({
                "type": "command", "exec": place.command, "args": args, "timeoutSec": seconds
            }),
        }
    }

    pub(super) fn spoken(self, place: &Place, args: &[&str]) -> String {
        format!("{} {}", self.quoted(place), args.join(" "))
    }

    // On Windows Codex and Antigravity run hooks through cmd.exe, others through PowerShell; macOS and WSL2 use a POSIX shell.
    pub(super) fn quoted(self, place: &Place) -> String {
        match (windows(place), self) {
            (true, Kind::Codex | Kind::Antigravity) => format!("\"{}\"", place.command),
            (true, _) => format!("& '{}'", place.command.replace('\'', "''")),
            (false, _) => format!("'{}'", place.command.replace('\'', "'\\''")),
        }
    }
}

pub(super) fn windows(place: &Place) -> bool {
    place.os == Os::Windows && place.distro.is_none()
}

pub fn seen(kind: Kind, place: &Place) -> Option<Hooks> {
    let at = kind.file(place)?;
    let shown = at.display().to_string();
    let text = match files::read(&at) {
        Read::Text(text) => text,
        Read::Missing => String::new(),
        Read::Unread => {
            return Some(Hooks {
                at: shown,
                unread: true,
                ..Hooks::default()
            });
        }
    };
    let stray = stray(kind, &text, place);
    Some(Hooks {
        on: hooked(kind, &text),
        at: shown,
        apart: apart(kind, &text),
        lacks_before: lacks_before(kind, &text),
        stray,
        ..Hooks::default()
    })
}

fn lacks_before(kind: Kind, text: &str) -> bool {
    let runs = runs(kind, text);
    [BEFORE, SHELL]
        .iter()
        .any(|moment| !runs.iter().any(|run| run.all_read && run.moment == *moment))
}

pub fn hooked(kind: Kind, text: &str) -> bool {
    runs(kind, text).iter().any(|run| run.all_read)
}

pub fn apart(kind: Kind, text: &str) -> bool {
    own_ways(kind, text) > 0 || (kind.root() != HOOKS && named_apart(kind, text))
}

// Antigravity names each hook: Tisty run from a hook of another name, or Tisty's turned off by hand, is the person's way.
fn named_apart(kind: Kind, text: &str) -> bool {
    let Some(Value::Object(doc)) = json::value(text) else {
        return false;
    };
    let mark = kind.mark();
    doc.iter().any(|(name, hook)| match name == kind.root() {
        true => hook.get("enabled") == Some(&Value::Bool(false)),
        false => {
            let said = hook.to_string();
            said.to_lowercase().contains("tisty") && said.contains(&mark)
        }
    })
}

/// Which entries are whose: Tisty's own by `hook <client>`, each of the person's hooks by its own mark.
pub(super) struct Marked<'a> {
    pub mark: &'a str,
    pub root: &'a str,
    pub events: &'a [&'static str],
}

fn own<R>(kind: Kind, then: impl FnOnce(&Marked) -> R) -> R {
    let mark = kind.mark();
    then(&Marked {
        mark: &mark,
        root: kind.root(),
        events: kind.events(),
    })
}

fn own_ways(kind: Kind, text: &str) -> usize {
    own(kind, |marked| ways_of(kind, text, marked))
}

pub(super) fn ways_of(kind: Kind, text: &str, marked: &Marked) -> usize {
    let runs = runs_of(kind, text, marked);
    runs.iter().filter(|run| !run.all_read).count()
}

fn runs(kind: Kind, text: &str) -> Vec<Run> {
    own(kind, |marked| runs_of(kind, text, marked))
}

pub(super) fn runs_of(kind: Kind, text: &str, marked: &Marked) -> Vec<Run> {
    let spoken = Spoken {
        mark: marked.mark,
        root: marked.root,
        posix: kind == Kind::Claude,
    };
    running::found(text, marked.events, &spoken)
}

fn stray(kind: Kind, text: &str, place: &Place) -> Vec<Stray> {
    let mut all: Vec<Stray> = Vec::new();
    let own = runs(kind, text).into_iter().flat_map(|run| run.programs);
    for program in own.chain(super::user_hooks::programs(kind, text)) {
        let leads = leads(place, &program.said);
        if leads.astray() && !all.iter().any(|one| one.said == program.said) {
            all.push(Stray {
                said: program.said,
                gone: leads == Leads::Gone,
            });
        }
    }
    all
}

pub(super) fn named(program: &Program, place: &Place, named: &[String]) -> bool {
    named.contains(&program.said) && leads(place, &program.said).astray()
}

pub fn without(kind: Kind, text: &str) -> Option<String> {
    taken_out(kind, text, |run| run.all_read)
}

pub fn without_those(kind: Kind, text: &str, place: &Place, said: &[String]) -> Option<String> {
    taken_out(kind, text, |run| {
        run.all_read && run.programs.iter().all(|one| named(one, place, said))
    })
}

fn taken_out(kind: Kind, text: &str, which: impl Fn(&Run) -> bool) -> Option<String> {
    own(kind, |marked| taken_out_of(kind, text, marked, which))
}

pub(super) fn taken_out_of(
    kind: Kind,
    text: &str,
    marked: &Marked,
    which: impl Fn(&Run) -> bool,
) -> Option<String> {
    let mut now = text.to_string();
    while let Some(run) = runs_of(kind, &now, marked)
        .into_iter()
        .find(|run| which(run))
    {
        now = running::taken_out(&now, &run)?;
    }
    if marked.root != HOOKS && emptied_root(&now, marked.root) {
        now = json::unset_top(&now, marked.root)?;
    }
    sound_of(kind, text, &now, marked).then_some(now)
}

// A hook named Tisty's with nothing left in it would still stand in the person's file.
fn emptied_root(text: &str, root: &str) -> bool {
    let bare = |it: &Value| it.as_object().is_some_and(serde_json::Map::is_empty);
    json::value(text).is_some_and(|doc| doc.get(root).is_some_and(bare))
}

pub fn repointed(kind: Kind, text: &str, place: &Place, said: &[String]) -> Option<String> {
    let now = running::repointed(
        text,
        &runs(kind, text),
        |one| named(one, place, said),
        &New {
            command: &place.command,
            spoken: &kind.quoted(place),
            windows: windows(place),
        },
    )?;
    sound(kind, text, &now).then_some(now)
}

pub fn with(kind: Kind, text: &str, place: &Place) -> Option<String> {
    let fits = kind
        .events()
        .iter()
        .all(|event| running::list_fits(text, kind.root(), event));
    if !fits || apart(kind, text) {
        return None;
    }
    let mut now = without(kind, text)?;
    if now.trim().is_empty() {
        now = blank(kind);
    }
    let now = pushed(now, kind.root(), kind.entries(place))?;
    (sound(kind, text, &now) && hooked(kind, &now)).then_some(now)
}

pub(super) fn blank(kind: Kind) -> String {
    match kind {
        Kind::Cursor | Kind::Copilot => "{\n  \"version\": 1\n}\n".to_string(),
        _ => "{\n}\n".to_string(),
    }
}

pub(super) fn pushed(
    text: String,
    root: &str,
    entries: Vec<(&'static str, Value)>,
) -> Option<String> {
    let mut now = text;
    for (event, entry) in entries {
        let piece = serde_json::to_string_pretty(&entry).ok()?;
        now = match running::list(&now, root, event) {
            Some(at) => json::push(&now, at, &piece)?,
            None => {
                let laid: Vec<String> = piece.lines().map(|line| format!("  {line}")).collect();
                json::set(&now, root, event, &format!("[\n{}\n]", laid.join("\n")))?
            }
        };
    }
    Some(now)
}

fn sound(kind: Kind, was: &str, now: &str) -> bool {
    own(kind, |marked| sound_of(kind, was, now, marked))
}

pub(super) fn sound_of(kind: Kind, was: &str, now: &str, marked: &Marked) -> bool {
    let kept = json::same_but(was, now, |doc| {
        doc.remove("version");
        let root = marked.root;
        if let Some(Value::Object(hooks)) = doc.get_mut(root) {
            let named = marked.events.iter().copied();
            named.for_each(|event| others_alone(marked.mark, hooks, event));
        }
        let bare = |it: &Value| it.as_object().is_some_and(serde_json::Map::is_empty);
        if doc.get(root).is_some_and(bare) {
            doc.remove(root);
        }
    });
    kept && ways_of(kind, was, marked) == ways_of(kind, now, marked)
}

// Read back apart from what cut it, so whatever is not Tisty's has to be there still.
fn others_alone(mark: &str, hooks: &mut serde_json::Map<String, Value>, event: &str) {
    let Some(Value::Array(groups)) = hooks.get_mut(event) else {
        return;
    };
    groups.retain_mut(|group| {
        let Some(Value::Array(inner)) = group.get_mut(HOOKS) else {
            return !ours(group, mark);
        };
        inner.retain(|one| !ours(one, mark));
        !inner.is_empty()
    });
    if groups.is_empty() {
        hooks.remove(event);
    }
}

pub(super) fn ours(run: &Value, mark: &str) -> bool {
    let fields =
        ["command", "commandWindows", "exec", "bash", "powershell"].map(|named| run.get(named));
    let args = run.get("args").and_then(Value::as_array);
    let said: Vec<&str> = fields
        .into_iter()
        .flatten()
        .chain(args.into_iter().flatten())
        .filter_map(Value::as_str)
        .collect();
    let said = said.join(" ");
    said.to_lowercase().contains("tisty") && said.contains(mark)
}

pub fn emptied(text: &str) -> bool {
    serde_json::from_str::<serde_json::Map<String, Value>>(text).is_ok_and(|doc| {
        doc.iter().all(|(key, value)| match key.as_str() {
            "version" => true,
            HOOKS => value.as_object().is_some_and(serde_json::Map::is_empty),
            _ => false,
        })
    })
}

pub fn alone(kind: Kind) -> bool {
    kind == Kind::Copilot
}

#[cfg(test)]
#[path = "hooks_test.rs"]
mod tests;
