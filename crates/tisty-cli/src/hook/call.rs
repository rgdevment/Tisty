use serde::Deserialize;
use serde::de::IgnoredAny;
use serde_json::Value;

const CREATING: [&str; 3] = ["Write", "write_file", "create"];
const SHELLS: [&str; 3] = ["Bash", "PowerShell", "Shell"];
const CARD: &str = "mcp__ccd_session__start_session";
const OWN_INITIATIVE: &str = "own_initiative";

#[derive(Debug, PartialEq, Eq)]
pub enum Leaving {
    File(String),
    Command(String),
    Card,
    Nothing,
}

/// The call an agent is about to make: what a file would hold is skipped, never kept.
#[derive(Debug, Default)]
pub struct Call {
    input: Input,
    command: Option<Value>,
    cwd: Option<Value>,
}

#[derive(Debug, Default, Deserialize)]
struct Sent {
    #[serde(default)]
    tool_input: Option<Args>,
    #[serde(default, rename = "toolArgs")]
    tool_args: Option<Args>,
    #[serde(default)]
    command: Option<Value>,
    #[serde(default)]
    cwd: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Args {
    Input(Input),
    // Copilot hands its arguments over as JSON inside a string.
    Said(String),
    Other(IgnoredAny),
}

#[derive(Debug, Default, Deserialize)]
struct Input {
    #[serde(default)]
    file_path: Option<Value>,
    #[serde(default, rename = "filePath")]
    file_path_camel: Option<Value>,
    #[serde(default)]
    path: Option<Value>,
    #[serde(default)]
    target_file: Option<Value>,
    #[serde(default)]
    command: Option<Value>,
    #[serde(default)]
    working_directory: Option<Value>,
    #[serde(default)]
    initiation: Option<Value>,
}

impl Call {
    pub fn read(raw: &[u8]) -> Self {
        let sent: Sent = serde_json::from_slice(raw).unwrap_or_default();
        let input = match sent.tool_input.or(sent.tool_args) {
            Some(Args::Input(input)) => input,
            Some(Args::Said(said)) => serde_json::from_str(&said).unwrap_or_default(),
            Some(Args::Other(_)) | None => Input::default(),
        };
        Self {
            input,
            command: sent.command,
            cwd: sent.cwd,
        }
    }

    pub fn leaving(&self, tool: Option<&str>) -> Leaving {
        match tool {
            Some(tool) if CREATING.contains(&tool) => self.file(),
            Some(tool) if SHELLS.contains(&tool) => command(&self.input.command),
            Some(CARD) if text(&self.input.initiation) == Some(OWN_INITIATIVE) => Leaving::Card,
            Some(_) => Leaving::Nothing,
            // Cursor hands a terminal command over on its own, with no tool named.
            None => command(&self.command),
        }
    }

    pub fn cwd(&self) -> &str {
        text(&self.input.working_directory)
            .or(text(&self.cwd))
            .unwrap_or_default()
    }

    fn file(&self) -> Leaving {
        let input = &self.input;
        [
            &input.file_path,
            &input.file_path_camel,
            &input.path,
            &input.target_file,
        ]
        .into_iter()
        .find_map(text)
        .map_or(Leaving::Nothing, |it| Leaving::File(it.to_string()))
    }
}

fn text(said: &Option<Value>) -> Option<&str> {
    said.as_ref()?.as_str()
}

fn command(said: &Option<Value>) -> Leaving {
    text(said).map_or(Leaving::Nothing, |it| Leaving::Command(it.to_string()))
}

#[cfg(test)]
#[path = "call_test.rs"]
mod tests;
