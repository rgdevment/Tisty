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
use marks::{Mark, Marks, Session, Verdict, verdict};
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
            used(&heard, raw, &session, at);
            None
        }
        "shell" => shelled(&settings, client, &heard, raw, &session, at),
        "stop" => ended(paths, &settings, client, &heard, &session, at),
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

fn used(heard: &Heard, raw: &[u8], session: &Session, at: i128) {
    if let Some(tool) = heard.tisty_tool() {
        session.mark(Mark::Called, at);
        if crate::mcp::writes(tool) {
            session.mark(Mark::Wrote, at);
            if let Some(task) = Call::read(raw).task() {
                session.touched(task);
            }
        }
    } else if heard.edits() {
        session.mark(Mark::Edited, at);
    }
}

fn shelled(
    settings: &Settings,
    client: Client,
    heard: &Heard,
    raw: &[u8],
    session: &Session,
    at: i128,
) -> Option<String> {
    let call = Call::read(raw);
    if !settings.door_open() || !call.command().is_some_and(command::publishes) {
        return None;
    }
    session.mark(Mark::Published, at);
    if borrowed(client, heard) {
        return None;
    }
    let said = client.after_shell(crate::mcp::PUBLISHED_ASK)?;
    session.mark(Mark::AskedPublished, at);
    Some(said)
}

fn ended(
    paths: &Paths,
    settings: &Settings,
    client: Client,
    heard: &Heard,
    session: &Session,
    at: i128,
) -> Option<String> {
    let marks = session.marks();
    if !settings.door_open() {
        session.mark(Mark::Ended, at);
        return None;
    }
    if verdict(&marks, heard.again()) == Verdict::HandBack {
        if !session.claim(&format!("stop-{}", marks.edited.unwrap_or_default())) {
            return None;
        }
        session.mark(Mark::Handed, at);
        return Some(client.handed_back(crate::mcp::TURN_LEFT_UNWRITTEN));
    }
    if !heard.again()
        && let Some(said) = asked_at_the_end(paths, session, &marks, at)
    {
        return Some(client.handed_back(&said));
    }
    session.mark(Mark::Ended, at);
    None
}

fn asked_at_the_end(paths: &Paths, session: &Session, marks: &Marks, at: i128) -> Option<String> {
    if marks.published_unasked()
        && session.claim(&format!(
            "published-{}",
            marks.published.unwrap_or_default()
        ))
    {
        session.mark(Mark::AskedPublished, at);
        return Some(crate::mcp::PUBLISHED_ASK.to_string());
    }
    let wrote_this_turn = marks
        .wrote
        .is_some_and(|wrote| marks.ended.is_none_or(|end| wrote > end));
    if !wrote_this_turn {
        return None;
    }
    let task = session.last_touched()?;
    let said = crate::mcp::ready_to_say_done(paths, &task)?;
    session.claim(&format!("ready-{task}")).then_some(said)
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
