use std::io::Write;
use std::process::ExitCode;
use std::time::Duration;

use tisty_core::Paths;
use tisty_core::agents::{self, HookYes, Ledger, hook_run};

use crate::hook::Settings;
use crate::hook::said::{Client, Heard};

const BEFORE_TOOL: &str = "before-tool";
const CURSOR: &str = "cursor";
const CLAUDE_CODE: &str = "claude-code";
const REASON_AT_MOST: usize = 2_000;

// Like Tisty's own hooks, this never fails the agent's turn: only a script's own 2 before a tool stops it.
pub fn run(args: &[String]) -> ExitCode {
    let Some((name, agent)) = asked(args) else {
        return ExitCode::SUCCESS;
    };
    let heard = crate::hook::heard();
    if let Some(said) = answered(&name, &agent, heard) {
        let mut out = std::io::stdout().lock();
        let _ = out.write_all(said.as_bytes());
        let _ = out.flush();
    }
    ExitCode::SUCCESS
}

fn asked(args: &[String]) -> Option<(String, String)> {
    match args {
        [name_flag, name, agent_flag, agent]
            if name_flag == "--name" && agent_flag == "--agent" =>
        {
            Some((name.clone(), agent.clone()))
        }
        _ => None,
    }
}

fn answered(name: &str, agent: &str, heard: Vec<u8>) -> Option<String> {
    let paths = Paths::resolve().ok()?;
    if !Settings::of(&paths).door_open() {
        return None;
    }
    let here = tisty_core::machine::here()?;
    let ledger = Ledger::mine(paths.data(), &here)?;
    let Some((_, yes)) = ledger.hook_named(name) else {
        told(
            name,
            "is not registered on this computer, so it did not run",
        );
        return None;
    };
    let listened = Heard::read(&heard);
    let client = speaker(agent, &listened, yes)?;
    let Some(script) = agents::script_kept(paths.data(), yes) else {
        told(
            name,
            "changed since you registered it, so it did not run: register it again in Tisty",
        );
        return None;
    };
    let ran = hook_run::run(hook_run::Asked {
        name,
        script: &script,
        scratch: &paths.cache().join("hooks"),
        heard,
        event: &yes.event,
        agent,
        timeout: Duration::from_secs(u64::from(yes.timeout)),
    });
    match ran {
        Ok(ran) if ran.blocks() && yes.event == BEFORE_TOOL => {
            Some(client.denied(&reason(name, &ran.said)))
        }
        Ok(_) => None,
        Err(hook_run::NotRun::NoInterpreter(ext)) => {
            told(
                name,
                &format!("found nothing on this computer to run a .{ext} script"),
            );
            None
        }
        Err(hook_run::NotRun::Io(why)) => {
            told(name, &format!("could not start: {why}"));
            None
        }
    }
}

// Cursor runs Claude Code's hooks too: there the hook answers as Cursor, and only once.
fn speaker(agent: &str, heard: &Heard, yes: &HookYes) -> Option<Client> {
    if !yes.agents.iter().any(|one| one == agent) {
        return None;
    }
    if agent == CLAUDE_CODE && heard.in_cursor() {
        let also_cursor = yes.agents.iter().any(|one| one == CURSOR);
        return (!also_cursor).then_some(Client::Cursor);
    }
    Client::named(agent)
}

fn reason(name: &str, said: &str) -> String {
    let said: String = said.chars().take(REASON_AT_MOST).collect();
    match said.trim().is_empty() {
        true => format!("The hook {name} stopped this call."),
        false => said,
    }
}

fn told(name: &str, what: &str) {
    eprintln!("Tisty: the hook {name} {what}.");
}

#[cfg(test)]
#[path = "run_hook_test.rs"]
mod tests;
