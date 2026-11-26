use tisty_core::agents::{Event, Tools};

use super::hooks::Kind;

pub struct Placed {
    pub event: &'static str,
    pub matcher: Option<&'static str>,
}

const fn at(event: &'static str, matcher: Option<&'static str>) -> Placed {
    Placed { event, matcher }
}

pub fn events(kind: Kind) -> &'static [&'static str] {
    match kind {
        Kind::Claude | Kind::Codex => &[
            "SessionStart",
            "UserPromptSubmit",
            "PreToolUse",
            "PostToolUse",
            "Stop",
        ],
        Kind::Cursor => &[
            "sessionStart",
            "beforeSubmitPrompt",
            "preToolUse",
            "beforeShellExecution",
            "beforeMCPExecution",
            "postToolUse",
            "afterShellExecution",
            "afterFileEdit",
            "afterMCPExecution",
            "stop",
        ],
        Kind::Gemini => &[
            "SessionStart",
            "BeforeAgent",
            "BeforeTool",
            "AfterTool",
            "AfterAgent",
        ],
        Kind::Copilot => &[
            "sessionStart",
            "userPromptSubmitted",
            "preToolUse",
            "postToolUse",
            "agentStop",
        ],
        Kind::Antigravity => &["SessionStart", "PreToolUse", "PostToolUse", "Stop"],
    }
}

// Where an agent has no such moment, or names no such tools, the hook is not offered there.
pub fn placed(kind: Kind, event: Event, tools: Tools) -> Option<Placed> {
    match kind {
        Kind::Cursor => cursor(event, tools),
        _ => {
            let (start, prompt, before, after, stop) = moments(kind)?;
            let named = match event {
                Event::Start => start,
                Event::Prompt => prompt?,
                Event::BeforeTool => before,
                Event::AfterTool => after,
                Event::Stop => stop,
            };
            let matcher = match tools {
                Tools::Any => None,
                one => Some(tools_of(kind, one)?),
            };
            Some(at(named, matcher))
        }
    }
}

type Moments = (
    &'static str,
    Option<&'static str>,
    &'static str,
    &'static str,
    &'static str,
);

fn moments(kind: Kind) -> Option<Moments> {
    Some(match kind {
        Kind::Claude | Kind::Codex => (
            "SessionStart",
            Some("UserPromptSubmit"),
            "PreToolUse",
            "PostToolUse",
            "Stop",
        ),
        Kind::Gemini => (
            "SessionStart",
            Some("BeforeAgent"),
            "BeforeTool",
            "AfterTool",
            "AfterAgent",
        ),
        Kind::Copilot => (
            "sessionStart",
            Some("userPromptSubmitted"),
            "preToolUse",
            "postToolUse",
            "agentStop",
        ),
        Kind::Antigravity => ("SessionStart", None, "PreToolUse", "PostToolUse", "Stop"),
        Kind::Cursor => return None,
    })
}

fn tools_of(kind: Kind, tools: Tools) -> Option<&'static str> {
    match (kind, tools) {
        (Kind::Claude, Tools::Shell) => Some("^(Bash|PowerShell)$"),
        (Kind::Claude, Tools::Write) => Some("^(Write|Edit|MultiEdit|NotebookEdit)$"),
        (Kind::Claude | Kind::Codex, Tools::Mcp) => Some("^mcp__"),
        (Kind::Codex, Tools::Shell) => Some("^Bash$"),
        (Kind::Codex, Tools::Write) => Some("^apply_patch$"),
        (Kind::Gemini, Tools::Shell) => Some("^run_shell_command$"),
        (Kind::Gemini, Tools::Write) => Some("^(write_file|replace)$"),
        (Kind::Gemini | Kind::Antigravity, Tools::Mcp) => Some("^mcp_"),
        (Kind::Copilot, Tools::Shell) => Some("^(bash|powershell)$"),
        (Kind::Copilot, Tools::Write) => Some("^(create|edit|apply_patch|str_replace_editor)$"),
        (Kind::Antigravity, Tools::Shell) => Some("^run_command$"),
        (Kind::Antigravity, Tools::Write) => {
            Some("^(write_to_file|replace_file_content|multi_replace_file_content)$")
        }
        _ => None,
    }
}

fn cursor(event: Event, tools: Tools) -> Option<Placed> {
    Some(match (event, tools) {
        (Event::Start, _) => at("sessionStart", None),
        (Event::Prompt, _) => at("beforeSubmitPrompt", None),
        (Event::Stop, _) => at("stop", None),
        (Event::BeforeTool, Tools::Any) => at("preToolUse", None),
        (Event::BeforeTool, Tools::Shell) => at("beforeShellExecution", None),
        (Event::BeforeTool, Tools::Write) => at("preToolUse", Some("Write")),
        (Event::BeforeTool, Tools::Mcp) => at("beforeMCPExecution", None),
        (Event::AfterTool, Tools::Any) => at("postToolUse", None),
        (Event::AfterTool, Tools::Shell) => at("afterShellExecution", None),
        (Event::AfterTool, Tools::Write) => at("afterFileEdit", None),
        (Event::AfterTool, Tools::Mcp) => at("afterMCPExecution", None),
    })
}
