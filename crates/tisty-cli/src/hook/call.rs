use serde::Deserialize;
use serde::de::IgnoredAny;
use serde_json::Value;

const CREATING: [&str; 4] = ["Write", "write_file", "create", "write_to_file"];
const SHELLS: [&str; 8] = [
    "Bash",
    "PowerShell",
    "Shell",
    "run_command",
    "run_shell_command",
    "bash",
    "powershell",
    "shell",
];
const PATCH: &str = "apply_patch";
const ADDS_FILE: &str = "*** Add File: ";
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
    cwd: String,
}

#[derive(Debug, Default, Deserialize)]
struct Sent {
    #[serde(default)]
    tool_input: Option<Args>,
    #[serde(default, rename = "toolArgs")]
    tool_args: Option<Args>,
    #[serde(default, rename = "toolCall")]
    tool_call: Option<ToolCall>,
    #[serde(default)]
    command: Option<Value>,
    #[serde(default)]
    cwd: Option<Value>,
}

#[derive(Debug, Default, Deserialize)]
struct ToolCall {
    #[serde(default)]
    args: Option<Args>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Args {
    Input(Box<Input>),
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
    #[serde(default, alias = "TargetFile")]
    target_file: Option<Value>,
    #[serde(default, alias = "CommandLine")]
    command: Option<Value>,
    #[serde(default, alias = "Cwd")]
    working_directory: Option<Value>,
    #[serde(default)]
    dir_path: Option<Value>,
    #[serde(default)]
    initiation: Option<Value>,
    #[serde(default)]
    input: Option<Value>,
    #[serde(default)]
    patch: Option<Value>,
    #[serde(default)]
    task: Option<Value>,
}

impl Call {
    pub fn read(raw: &[u8]) -> Self {
        let sent: Sent = serde_json::from_slice(raw).unwrap_or_default();
        let called = sent.tool_call.and_then(|call| call.args);
        let input = match sent.tool_input.or(sent.tool_args).or(called) {
            Some(Args::Input(input)) => *input,
            Some(Args::Said(said)) => serde_json::from_str(&said).unwrap_or_default(),
            Some(Args::Other(_)) | None => Input::default(),
        };
        let cwd = resolve_cwd(
            text(&input.working_directory),
            text(&input.dir_path),
            text(&sent.cwd),
        );
        Self {
            input,
            command: sent.command,
            cwd,
        }
    }

    pub fn leaving(&self, tool: Option<&str>) -> Leaving {
        match tool {
            Some(tool) if CREATING.contains(&tool) => self.file(),
            Some(tool) if SHELLS.contains(&tool) => command(&self.input.command),
            Some(PATCH) => self.added(),
            Some(CARD) if text(&self.input.initiation) == Some(OWN_INITIATIVE) => Leaving::Card,
            Some(_) => Leaving::Nothing,
            // Cursor hands a terminal command over on its own, with no tool named.
            None => command(&self.command),
        }
    }

    pub fn task(&self) -> Option<&str> {
        text(&self.input.task)
    }

    pub fn command(&self) -> Option<&str> {
        text(&self.input.command).or(text(&self.command))
    }

    pub fn cwd(&self) -> &str {
        &self.cwd
    }

    // Codex writes a new file through a patch, which names it on a line of its own.
    fn added(&self) -> Leaving {
        let input = &self.input;
        let named = [&input.command, &input.input, &input.patch]
            .into_iter()
            .filter_map(text)
            .flat_map(str::lines)
            .find_map(|line| line.strip_prefix(ADDS_FILE))
            .map(str::trim)
            .filter(|path| !path.is_empty());
        named.map_or(Leaving::Nothing, |it| Leaving::File(it.to_string()))
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

fn resolve_cwd(
    working_directory: Option<&str>,
    dir_path: Option<&str>,
    base: Option<&str>,
) -> String {
    if let Some(dir) = working_directory {
        return dir.to_string();
    }
    let Some(dir) = dir_path.map(str::trim) else {
        return base.unwrap_or_default().to_string();
    };
    if dir.is_empty() || dir == "." {
        return base.unwrap_or_default().to_string();
    }
    if is_rooted(dir) {
        return dir.to_string();
    }
    let Some(base) = base.filter(|one| !one.is_empty()) else {
        return String::new();
    };
    let sep = if base.contains('\\') && !base.contains('/') {
        '\\'
    } else {
        '/'
    };
    format!("{}{sep}{dir}", base.trim_end_matches(['/', '\\']))
}

fn is_rooted(path: &str) -> bool {
    let bytes = path.as_bytes();
    path.starts_with(['/', '\\'])
        || path == "~"
        || path.starts_with("~/")
        || path.starts_with("~\\")
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'/' | b'\\'))
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
