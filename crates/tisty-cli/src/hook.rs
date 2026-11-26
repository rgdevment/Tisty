mod asking;
mod call;
mod command;
mod marks;
mod outside;
pub(crate) mod said;

use std::io::{Read, Write};
use std::process::ExitCode;

use tisty_core::Paths;

use call::Call;
use marks::{Mark, Session, Verdict, verdict};
use outside::Here;
use said::{Client, Heard};

const HEARD_AT_MOST: u64 = 32 * 1024 * 1024;

// A hook never fails the assistant's turn: whatever goes wrong, it exits 0 and says nothing.
pub fn run(args: &[String]) -> ExitCode {
    let [client, moment] = args else {
        return ExitCode::SUCCESS;
    };
    let raw = heard();
    let Some(client) = Client::named(client) else {
        return ExitCode::SUCCESS;
    };
    let Ok(paths) = Paths::resolve() else {
        return ExitCode::SUCCESS;
    };
    if let Some(said) = answered(&paths, client, moment, &raw, now()) {
        let mut out = std::io::stdout().lock();
        let _ = out.write_all(said.as_bytes());
        let _ = out.flush();
    }
    ExitCode::SUCCESS
}

pub(crate) fn heard() -> Vec<u8> {
    let mut raw = Vec::new();
    let _ = std::io::stdin().take(HEARD_AT_MOST).read_to_end(&mut raw);
    raw
}

fn now() -> i128 {
    jiff::Timestamp::now().as_nanosecond()
}

fn answered(paths: &Paths, client: Client, moment: &str, raw: &[u8], at: i128) -> Option<String> {
    let settings = Settings::of(paths);
    let heard = Heard::read(raw);
    // Without its session a mark would mix with other sessions', so only the start speaks.
    if heard.session().is_empty() && moment != "start" {
        return None;
    }
    let session = Session::of(paths.cache(), heard.session());
    if moment == "start" {
        marks::sweep(paths.cache(), std::time::SystemTime::now());
    }
    if settings.turn_hooks == Some(false) {
        return None;
    }
    match moment {
        "start" => started(paths, client, &heard, &session, at),
        "edit" => {
            session.mark(Mark::Edited, at);
            None
        }
        "tool" => {
            used(&heard, &session, at);
            None
        }
        "stop" => ended(&settings, client, &heard, &session, at),
        // Cursor runs Claude Code's hooks too, and shows their reason to the person, not to its agent.
        "before" if settings.door_open() && !borrowed(client, &heard) => {
            let call = Call::read(raw);
            asking::asked(&session, &Here::now(), &call, heard.tool(), at)
                .map(|question| client.denied(question))
        }
        _ => None,
    }
}

fn borrowed(client: Client, heard: &Heard) -> bool {
    client == Client::ClaudeCode && heard.in_cursor()
}

fn started(
    paths: &Paths,
    client: Client,
    heard: &Heard,
    session: &Session,
    at: i128,
) -> Option<String> {
    let key = format!("start-{}-{}", heard.source(), at / 30_000_000_000);
    if !heard.session().is_empty() && !session.claim(&key) {
        return None;
    }
    let opening = crate::mcp::opening(paths, client.hand(heard))?;
    Some(client.started(&opening))
}

fn used(heard: &Heard, session: &Session, at: i128) {
    if let Some(tool) = heard.tisty_tool() {
        session.mark(Mark::Called, at);
        if crate::mcp::writes(tool) {
            session.mark(Mark::Wrote, at);
        }
    } else if heard.edits() {
        session.mark(Mark::Edited, at);
    }
}

fn ended(
    settings: &Settings,
    client: Client,
    heard: &Heard,
    session: &Session,
    at: i128,
) -> Option<String> {
    let marks = session.marks();
    if !settings.door_open() || verdict(&marks, heard.again()) == Verdict::LetGo {
        session.mark(Mark::Ended, at);
        return None;
    }
    if !session.claim(&format!("stop-{}", marks.edited.unwrap_or_default())) {
        return None;
    }
    session.mark(Mark::Handed, at);
    Some(client.handed_back(crate::mcp::TURN_LEFT_UNWRITTEN))
}

#[derive(Debug, Default, serde::Deserialize)]
pub(crate) struct Settings {
    #[serde(default)]
    turn_hooks: Option<bool>,
    #[serde(default)]
    agent_id: Option<String>,
}

impl Settings {
    pub(crate) fn of(paths: &Paths) -> Self {
        std::fs::read_to_string(paths.config_file())
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub(crate) fn door_open(&self) -> bool {
        let agent = self.agent_id.as_deref().unwrap_or_default();
        !agent.trim().is_empty()
    }
}

#[cfg(test)]
#[path = "hook_test.rs"]
mod tests;
