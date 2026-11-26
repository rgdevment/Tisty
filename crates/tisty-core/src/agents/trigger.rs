use tisty_markdown::front::Field;

pub const EVENT: &str = "hook-event";
pub const TOOLS: &str = "hook-tools";
pub const TIMEOUT: &str = "hook-timeout";
pub const AGENTS: &str = "hook-agents";
const FIELDS: [&str; 4] = [EVENT, TOOLS, TIMEOUT, AGENTS];
const PREFIX: &str = "hook-";
pub const EXTENSIONS: [&str; 4] = ["ps1", "sh", "py", "js"];
pub const TIMEOUT_AT_MOST: u32 = 600;
const NAME_AT_MOST: usize = 64;
const AGENT_AT_MOST: usize = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
    Start,
    Prompt,
    BeforeTool,
    AfterTool,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tools {
    Any,
    Shell,
    Write,
    Mcp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trigger {
    pub event: Event,
    pub tools: Tools,
    pub timeout: u32,
    pub agents: Vec<String>,
}

impl Event {
    pub const ALL: [Self; 5] = [
        Self::Start,
        Self::Prompt,
        Self::BeforeTool,
        Self::AfterTool,
        Self::Stop,
    ];

    pub fn word(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Prompt => "prompt",
            Self::BeforeTool => "before-tool",
            Self::AfterTool => "after-tool",
            Self::Stop => "stop",
        }
    }

    pub fn read(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|one| one.word() == word)
    }

    pub fn on_tools(self) -> bool {
        matches!(self, Self::BeforeTool | Self::AfterTool)
    }
}

impl Tools {
    pub const ALL: [Self; 4] = [Self::Any, Self::Shell, Self::Write, Self::Mcp];

    pub fn word(self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Shell => "shell",
            Self::Write => "write",
            Self::Mcp => "mcp",
        }
    }

    pub fn read(word: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|one| one.word() == word)
    }
}

pub fn hook_name(name: &str) -> bool {
    let fits = !name.is_empty()
        && name.len() <= NAME_AT_MOST
        && !name.starts_with(['.', '-'])
        && name.bytes().all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
        });
    fits && extension(name).is_some()
}

pub fn extension(name: &str) -> Option<&'static str> {
    let (stem, ext) = name.rsplit_once('.')?;
    (!stem.is_empty())
        .then(|| EXTENSIONS.into_iter().find(|one| *one == ext))
        .flatten()
}

fn agent_word(word: &str) -> bool {
    !word.is_empty()
        && word.len() <= AGENT_AT_MOST
        && word
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

// A hook half read never runs: every field once and readable, or no trigger at all.
pub fn read(fields: &[Field]) -> Option<Trigger> {
    let ours: Vec<&Field> = fields
        .iter()
        .filter(|one| one.key.to_ascii_lowercase().starts_with(PREFIX))
        .collect();
    let known = ours
        .iter()
        .all(|one| FIELDS.iter().any(|key| one.key.eq_ignore_ascii_case(key)));
    if !known || ours.len() != FIELDS.len() {
        return None;
    }
    let named = |key: &str| ours.iter().find(|one| one.key.eq_ignore_ascii_case(key));
    let single = |key: &str| match named(key)?.values.as_slice() {
        [one] => Some(one.trim().to_string()),
        _ => None,
    };
    let event = Event::read(&single(EVENT)?)?;
    let tools = Tools::read(&single(TOOLS)?)?;
    if !event.on_tools() && tools != Tools::Any {
        return None;
    }
    let timeout: u32 = single(TIMEOUT)?.parse().ok()?;
    if timeout == 0 || timeout > TIMEOUT_AT_MOST {
        return None;
    }
    let agents: Vec<String> = named(AGENTS)?
        .values
        .iter()
        .map(|one| one.trim().to_string())
        .collect();
    let unique = agents
        .iter()
        .enumerate()
        .all(|(at, one)| !agents[..at].contains(one));
    let readable = !agents.is_empty() && unique && agents.iter().all(|one| agent_word(one));
    readable.then_some(Trigger {
        event,
        tools,
        timeout,
        agents,
    })
}

impl Trigger {
    pub fn lines(&self) -> String {
        format!(
            "{EVENT}: {}\n{TOOLS}: {}\n{TIMEOUT}: {}\n{AGENTS}: [{}]\n",
            self.event.word(),
            self.tools.word(),
            self.timeout,
            self.agents.join(", ")
        )
    }
}

#[cfg(test)]
#[path = "trigger_test.rs"]
mod tests;
