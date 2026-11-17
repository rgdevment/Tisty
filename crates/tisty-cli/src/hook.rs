mod marks;
mod said;

use std::io::{Read, Write};
use std::process::ExitCode;

use serde_json::Value;
use tisty_core::Paths;

use marks::{Mark, Session, Verdict, verdict};
use said::Client;

const HEARD_AT_MOST: u64 = 32 * 1024 * 1024;
const SWITCH: &str = "turn_hooks";

// A hook never fails the assistant's turn: whatever goes wrong, it exits 0 and says nothing.
pub fn run(args: &[String]) -> ExitCode {
    let [client, moment] = args else {
        return ExitCode::SUCCESS;
    };
    let heard = heard();
    let Some(client) = Client::named(client) else {
        return ExitCode::SUCCESS;
    };
    let Ok(paths) = Paths::resolve() else {
        return ExitCode::SUCCESS;
    };
    if let Some(said) = answered(&paths, client, moment, &heard, now()) {
        let mut out = std::io::stdout().lock();
        let _ = out.write_all(said.as_bytes());
        let _ = out.flush();
    }
    ExitCode::SUCCESS
}

fn heard() -> Value {
    let mut raw = Vec::new();
    let _ = std::io::stdin().take(HEARD_AT_MOST).read_to_end(&mut raw);
    serde_json::from_slice(&raw).unwrap_or(Value::Null)
}

fn now() -> i128 {
    jiff::Timestamp::now().as_nanosecond()
}

fn answered(
    paths: &Paths,
    client: Client,
    moment: &str,
    heard: &Value,
    at: i128,
) -> Option<String> {
    let settings = settings(paths);
    if !on(settings.as_ref()) {
        return None;
    }
    let session = Session::of(paths.cache(), &said::session(heard));
    match moment {
        "start" => {
            marks::sweep(paths.cache(), std::time::SystemTime::now());
            let key = format!("start-{}-{}", said::source(heard), at / 30_000_000_000);
            if !session.claim(&key) {
                return None;
            }
            let opening = crate::mcp::opening(paths, client.hand(heard))?;
            Some(client.started(&opening))
        }
        "edit" => {
            session.mark(Mark::Edited, at);
            None
        }
        "tool" => {
            if let Some(tool) = said::tisty_tool(heard) {
                session.mark(Mark::Called, at);
                if crate::mcp::writes(tool) {
                    session.mark(Mark::Wrote, at);
                }
            } else if said::edits(heard) {
                session.mark(Mark::Edited, at);
            }
            None
        }
        "stop" => {
            let marks = session.marks();
            let door_open = settings
                .as_ref()
                .and_then(|it| it.get("agent_id"))
                .and_then(toml::Value::as_str)
                .is_some_and(|agent| !agent.trim().is_empty());
            match door_open && verdict(&marks, said::again(heard)) == Verdict::HandBack {
                true => {
                    let key = format!("stop-{}", marks.edited.unwrap_or_default());
                    if !session.claim(&key) {
                        return None;
                    }
                    session.mark(Mark::Handed, at);
                    Some(client.handed_back(crate::mcp::TURN_LEFT_UNWRITTEN))
                }
                false => {
                    session.mark(Mark::Ended, at);
                    None
                }
            }
        }
        _ => None,
    }
}

fn settings(paths: &Paths) -> Option<toml::Table> {
    let text = std::fs::read_to_string(paths.config_file()).ok()?;
    toml::from_str(&text).ok()
}

fn on(settings: Option<&toml::Table>) -> bool {
    settings
        .and_then(|it| it.get(SWITCH))
        .and_then(toml::Value::as_bool)
        != Some(false)
}

#[cfg(test)]
#[path = "hook_test.rs"]
mod tests;
