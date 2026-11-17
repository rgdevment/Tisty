use serde::Deserialize;
use serde_json::{Value, json};

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

/// Only the fields a hook reads: what an edit wrote is skipped, never held.
#[derive(Debug, Default, Deserialize)]
pub struct Heard {
    #[serde(default)]
    session_id: Option<Value>,
    #[serde(default, rename = "sessionId")]
    session_id_camel: Option<Value>,
    #[serde(default)]
    conversation_id: Option<Value>,
    #[serde(default)]
    tool_name: Option<Value>,
    #[serde(default, rename = "toolName")]
    tool_name_camel: Option<Value>,
    #[serde(default)]
    mcp_server_name: Option<Value>,
    #[serde(default)]
    source: Option<Value>,
    #[serde(default)]
    stop_hook_active: Option<Value>,
    #[serde(default)]
    cursor_version: Option<serde::de::IgnoredAny>,
}

impl Heard {
    pub fn read(raw: &[u8]) -> Self {
        serde_json::from_slice(raw).unwrap_or_default()
    }

    pub fn session(&self) -> &str {
        [
            &self.session_id,
            &self.session_id_camel,
            &self.conversation_id,
        ]
        .into_iter()
        .find_map(|one| one.as_ref()?.as_str())
        .unwrap_or_default()
    }

    fn tool(&self) -> Option<&str> {
        [&self.tool_name, &self.tool_name_camel]
            .into_iter()
            .find_map(|one| one.as_ref()?.as_str())
    }

    pub fn source(&self) -> &str {
        self.source
            .as_ref()
            .and_then(Value::as_str)
            .unwrap_or("startup")
    }

    pub fn tisty_tool(&self) -> Option<&str> {
        let named = self.tool()?;
        if let Some(server) = self.mcp_server_name.as_ref().and_then(Value::as_str) {
            return ours(server).then_some(named);
        }
        let tool = if let Some(rest) = named.strip_prefix("mcp__") {
            let (server, tool) = rest.rsplit_once("__")?;
            ours(server).then_some(tool)?
        } else if let Some(rest) = named.strip_prefix("MCP:") {
            // Cursor names no server here, so only a name no other server would use counts.
            let telling = rest.contains('_') || ["tick", "untick", "propose"].contains(&rest);
            (telling && crate::mcp::is_tool(rest)).then_some(rest)?
        } else {
            named
                .strip_prefix("mcp_tisty_")
                .or_else(|| named.strip_prefix("tisty-"))?
        };
        (!tool.is_empty()).then_some(tool)
    }

    pub fn edits(&self) -> bool {
        self.tool().is_some_and(|named| EDITING.contains(&named))
    }

    pub fn again(&self) -> bool {
        self.stop_hook_active.as_ref().and_then(Value::as_bool) == Some(true)
    }

    fn in_cursor(&self) -> bool {
        self.cursor_version.is_some()
    }
}

fn ours(server: &str) -> bool {
    server.to_lowercase().contains(SERVER)
}

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
    pub fn hand(self, heard: &Heard) -> &'static str {
        match heard.in_cursor() {
            true => Client::Cursor.id(),
            false => self.id(),
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

#[cfg(test)]
#[path = "said_test.rs"]
mod tests;
