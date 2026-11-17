use serde_json::{Value, json};

const SESSION_KEYS: [&str; 3] = ["session_id", "sessionId", "conversation_id"];
const TOOL_KEYS: [&str; 2] = ["tool_name", "toolName"];
const EDITING: [&str; 10] = [
    "Edit",
    "Write",
    "MultiEdit",
    "NotebookEdit",
    "write_file",
    "replace",
    "create",
    "edit",
    "apply_patch",
    "str_replace_editor",
];
const SERVER: &str = "tisty";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Client {
    ClaudeCode,
    Cursor,
    GeminiCli,
    Copilot,
}

impl Client {
    pub fn named(id: &str) -> Option<Self> {
        match id {
            "claude-code" => Some(Client::ClaudeCode),
            "cursor" => Some(Client::Cursor),
            "gemini-cli" => Some(Client::GeminiCli),
            "copilot" => Some(Client::Copilot),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Client::ClaudeCode => "claude-code",
            Client::Cursor => "cursor",
            Client::GeminiCli => "gemini-cli",
            Client::Copilot => "copilot",
        }
    }

    // Cursor runs the hooks in `~/.claude/settings.json` too, and its own agent is the one writing.
    pub fn hand(self, heard: &Value) -> &'static str {
        match heard.get("cursor_version") {
            Some(_) => Client::Cursor.id(),
            None => self.id(),
        }
    }

    pub fn started(self, context: &str) -> String {
        match self {
            Client::ClaudeCode | Client::GeminiCli => json!({
                "hookSpecificOutput": {
                    "hookEventName": "SessionStart",
                    "additionalContext": context,
                }
            }),
            Client::Cursor => json!({ "additional_context": context }),
            Client::Copilot => json!({ "additionalContext": context }),
        }
        .to_string()
    }

    pub fn handed_back(self, reason: &str) -> String {
        match self {
            Client::Cursor => json!({ "followup_message": reason }),
            Client::ClaudeCode | Client::GeminiCli | Client::Copilot => {
                json!({ "decision": "block", "reason": reason })
            }
        }
        .to_string()
    }
}

fn first<'a>(heard: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| heard.get(*key)?.as_str())
}

pub fn session(heard: &Value) -> String {
    first(heard, &SESSION_KEYS).unwrap_or_default().to_string()
}

pub fn source(heard: &Value) -> &str {
    heard
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("startup")
}

pub fn tisty_tool(heard: &Value) -> Option<&str> {
    let named = first(heard, &TOOL_KEYS)?;
    if let Some(server) = heard.get("mcp_server_name").and_then(Value::as_str) {
        return ours(server).then_some(named);
    }
    let tool = if let Some(rest) = named.strip_prefix("mcp__") {
        let (server, tool) = rest.rsplit_once("__")?;
        ours(server).then_some(tool)?
    } else if let Some(rest) = named.strip_prefix("MCP:") {
        crate::mcp::is_tool(rest).then_some(rest)?
    } else {
        named
            .strip_prefix("mcp_tisty_")
            .or_else(|| named.strip_prefix("tisty-"))?
    };
    (!tool.is_empty()).then_some(tool)
}

fn ours(server: &str) -> bool {
    server.to_lowercase().contains(SERVER)
}

pub fn edits(heard: &Value) -> bool {
    first(heard, &TOOL_KEYS).is_some_and(|named| EDITING.contains(&named))
}

pub fn again(heard: &Value) -> bool {
    heard.get("stop_hook_active").and_then(Value::as_bool) == Some(true)
        || heard
            .get("loop_count")
            .and_then(Value::as_u64)
            .is_some_and(|count| count > 0)
}

#[cfg(test)]
#[path = "said_test.rs"]
mod tests;
