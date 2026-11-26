use std::sync::Mutex;
use std::time::Duration;

use serde_json::json;
use tisty_core::agents::{self, Card, Event, HookYes, Ledger, Seen, Tools, Trigger, hook_run};

use crate::wiring::user_hooks::{self, AgentHook, Hook};
use crate::{Answer, Refusal, Session, held};

const TRY_AS: &str = "tisty-try";
const NEW_SCRIPT: &str = "";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookLook {
    pub file: String,
    pub name: String,
    pub script: String,
    pub event: String,
    pub tools: String,
    pub timeout: u32,
    pub agents: Vec<String>,
    pub print: String,
    pub state: &'static str,
    pub door: bool,
    pub kept: bool,
    pub here: Vec<AgentHook>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tried {
    pub code: Option<i32>,
    pub said: String,
    pub timed_out: bool,
    pub took_ms: u64,
    pub missing: Option<String>,
}

struct Opened {
    card: Card,
    trigger: Trigger,
    body: String,
    print: String,
    seen: Seen,
    ledger: Ledger,
}

fn opened(session: &Session, file: &str) -> Answer<Opened> {
    let body = super::agents_sync::bodies(session)
        .into_iter()
        .find(|(one, _)| one == file)
        .map(|(_, body)| body)
        .ok_or_else(|| Refusal::of("noSuchDoc"))?;
    let card = Card::of(&body).ok_or_else(|| Refusal::of("agentHookUnread"))?;
    let trigger = card
        .trigger
        .clone()
        .ok_or_else(|| Refusal::of("agentHookUnread"))?;
    let machine = super::agents_sync::here();
    let ledger = Ledger::open(session.paths.data(), &machine);
    let seen = super::agents_sync::gated(session, &ledger)
        .seen(file)
        .unwrap_or(Seen::Away);
    Ok(Opened {
        print: tisty_core::docs::print_of_text(&body),
        card,
        trigger,
        body,
        seen,
        ledger,
    })
}

fn hook_of<'a>(card: &'a Card, trigger: &Trigger) -> Hook<'a> {
    Hook {
        name: &card.name,
        event: trigger.event,
        tools: trigger.tools,
        timeout: trigger.timeout,
    }
}

fn state_of(seen: Seen, yes: Option<&HookYes>) -> &'static str {
    match seen {
        Seen::HookRegistered => "registered",
        Seen::HookWaiting if yes.is_some() => "changed",
        Seen::HookWaiting => "waiting",
        Seen::NotAgreed => "notAgreed",
        Seen::PlacedByAgent => "placedByAgent",
        Seen::OtherComputer | Seen::Apart => "elsewhere",
        _ => "away",
    }
}

fn looked(session: &Session, file: &str) -> Answer<HookLook> {
    let open = opened(session, file)?;
    let script = agents::rendered(&open.body)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    let yes = open.ledger.hook_yes(file);
    let hook = hook_of(&open.card, &open.trigger);
    Ok(HookLook {
        file: file.to_string(),
        name: open.card.name.clone(),
        script,
        event: open.trigger.event.word().to_string(),
        tools: open.trigger.tools.word().to_string(),
        timeout: open.trigger.timeout,
        agents: open.trigger.agents.clone(),
        print: open.print.clone(),
        state: state_of(open.seen, yes),
        door: session.config.agent_id.is_some(),
        kept: yes.is_some_and(|yes| agents::script_kept(session.paths.data(), yes).is_some()),
        here: user_hooks::verify(&hook, &open.trigger.agents),
    })
}

fn seed(
    name: &str,
    trigger: &Trigger,
    here_only: Option<&tisty_core::event::DeviceId>,
) -> Answer<String> {
    if !agents::hook_name(name) {
        return Err(Refusal::of("agentHookNameBad"));
    }
    agents::hook_doc(name, here_only, trigger, NEW_SCRIPT).map_err(super::agents_sync::refused)
}

fn named_free(session: &Session, name: &str) -> bool {
    !super::agents_sync::bodies(session).iter().any(|(_, body)| {
        Card::of(body).is_some_and(|card| card.kind == agents::Kind::Hook && card.name == name)
    })
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewHook {
    pub name: String,
    pub event: String,
    pub tools: String,
    pub timeout: u32,
    pub agents: Vec<String>,
    pub everyone: bool,
}

fn made(session: &mut Session, asked: &NewHook) -> Answer<tisty_core::docs::Doc> {
    let trigger = Trigger {
        event: Event::read(&asked.event).ok_or_else(|| Refusal::of("agentHookUnread"))?,
        tools: Tools::read(&asked.tools).ok_or_else(|| Refusal::of("agentHookUnread"))?,
        timeout: asked.timeout,
        agents: asked.agents.clone(),
    };
    if !named_free(session, &asked.name) {
        return Err(Refusal::of("agentHookNameTaken"));
    }
    let me = session.config.device_id.clone();
    let body = seed(&asked.name, &trigger, (!asked.everyone).then_some(&me))?;
    let at = match asked.everyone {
        true => super::agent_docs::global_made(session)?,
        false => mine_made(session, &me)?,
    };
    super::papers::made_with(session, Some(at), None, &body)
}

// A hook for this computer alone lives in this computer's own folder, the one its `on-` line names.
fn mine_made(
    session: &mut Session,
    me: &tisty_core::event::DeviceId,
) -> Answer<tisty_core::model::FolderId> {
    let root =
        super::agents_sync::root_of(&session.state).ok_or_else(|| Refusal::of("noSuchFolder"))?;
    let (at, made) = agents::computer_folder(&session.state, root, me);
    if let Some(op) = made {
        session
            .commit_all(vec![op])
            .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    }
    crate::folder_open(&session.state, at, true)?;
    super::placing::may_make(session, Some(at), None, tisty_core::places::Making::Doc)?;
    Ok(at)
}

fn registered(session: &Session, file: &str, print: &str) -> Answer<HookLook> {
    let open = opened(session, file)?;
    let fits = matches!(open.seen, Seen::HookWaiting | Seen::HookRegistered);
    if !fits || open.print != print {
        return Err(Refusal::of("agentHookChangedSince"));
    }
    let mut ledger = open.ledger;
    if !ledger.named() {
        return Err(Refusal::of("agentHookNoMachine"));
    }
    let (yes, bytes) = agents::yes_of(&open.body).map_err(super::agents_sync::refused)?;
    let data = session.paths.data();
    agents::keep_script(data, &yes, &bytes).map_err(super::agents_sync::refused)?;
    let hook = hook_of(&open.card, &open.trigger);
    user_hooks::register(&hook, &open.trigger.agents).map_err(super::wired::stuck)?;
    ledger.register_hook(file, yes);
    ledger.keep(data).map_err(super::agents_sync::refused)?;
    looked(session, file)
}

fn unregistered(session: &Session, file: &str) -> Answer<HookLook> {
    let machine = super::agents_sync::here();
    let data = session.paths.data();
    let mut ledger = Ledger::open(data, &machine);
    if let Some(yes) = ledger.hook_yes(file).cloned() {
        user_hooks::unregister(&yes.name, &yes.agents).map_err(super::wired::stuck)?;
        agents::drop_script(data, &yes.name);
        ledger.drop_hook(file);
        ledger.keep(data).map_err(super::agents_sync::refused)?;
    }
    looked(session, file)
}

struct Trial {
    name: String,
    script: Vec<u8>,
    scratch: std::path::PathBuf,
    heard: Vec<u8>,
    event: &'static str,
    agent: String,
    timeout: Duration,
}

// A dry run executes for real, so it takes this computer's yes, or the print of the script just shown.
fn trial(session: &Session, file: &str, print: &str) -> Answer<Trial> {
    let open = opened(session, file)?;
    let data = session.paths.data();
    let kept = open
        .ledger
        .hook_yes(file)
        .filter(|yes| yes.print == open.print)
        .and_then(|yes| agents::script_kept(data, yes));
    let script = match kept {
        Some(bytes) => bytes,
        None if print == open.print => {
            agents::rendered(&open.body).map_err(super::agents_sync::refused)?
        }
        None => return Err(Refusal::of("agentHookChangedSince")),
    };
    Ok(Trial {
        name: open.card.name.clone(),
        script,
        scratch: session.paths.cache().join("hooks"),
        heard: sample(open.trigger.event, open.trigger.tools),
        event: open.trigger.event.word(),
        agent: open.trigger.agents.first().cloned().unwrap_or_default(),
        timeout: Duration::from_secs(u64::from(open.trigger.timeout)),
    })
}

fn tried(trial: Trial) -> Answer<Tried> {
    let ran = hook_run::run(hook_run::Asked {
        name: &trial.name,
        script: &trial.script,
        scratch: &trial.scratch,
        heard: trial.heard,
        event: trial.event,
        agent: &trial.agent,
        timeout: trial.timeout,
    });
    Ok(match ran {
        Ok(ran) => Tried {
            code: ran.code,
            said: ran.said,
            timed_out: ran.timed_out,
            took_ms: u64::try_from(ran.took.as_millis()).unwrap_or(u64::MAX),
            missing: None,
        },
        Err(hook_run::NotRun::NoInterpreter(ext)) => Tried {
            code: None,
            said: String::new(),
            timed_out: false,
            took_ms: 0,
            missing: Some(ext.to_string()),
        },
        Err(hook_run::NotRun::Io(why)) => return Err(Refusal::about("cannotWrite", why)),
    })
}

fn sample(event: Event, tools: Tools) -> Vec<u8> {
    let tool = match tools {
        Tools::Any | Tools::Shell => "Bash",
        Tools::Write => "Write",
        Tools::Mcp => "mcp__tisty__read",
    };
    let said = match event {
        Event::Start => {
            json!({ "session_id": TRY_AS, "hook_event_name": "SessionStart", "source": "startup" })
        }
        Event::Prompt => {
            json!({ "session_id": TRY_AS, "hook_event_name": "UserPromptSubmit", "prompt": "hello" })
        }
        Event::BeforeTool => json!({
            "session_id": TRY_AS, "hook_event_name": "PreToolUse", "tool_name": tool,
            "tool_input": { "command": "echo hello", "file_path": "notes.md" }
        }),
        Event::AfterTool => json!({
            "session_id": TRY_AS, "hook_event_name": "PostToolUse", "tool_name": tool,
            "tool_input": { "command": "echo hello", "file_path": "notes.md" },
            "tool_response": { "output": "hello" }
        }),
        Event::Stop => {
            json!({ "session_id": TRY_AS, "hook_event_name": "Stop", "stop_hook_active": false })
        }
    };
    said.to_string().into_bytes()
}

/// A hook goes from the agents only on proof its document is gone; changed words just stop its script.
pub(crate) fn retired(session: &Session, gate: &agents::Gate, ledger: &mut Ledger) -> bool {
    let data = session.paths.data();
    let mut changed = false;
    let all: Vec<(String, HookYes)> = ledger
        .hooks()
        .map(|(file, yes)| (file.to_string(), yes.clone()))
        .collect();
    for (file, yes) in all {
        let kept = session.state.docs.values().find(|one| one.file == file);
        let gone = kept.is_none_or(|one| one.archived || !agents::holds(&session.state, one));
        let seen = gate.seen(&file);
        let elsewhere = matches!(seen, Some(Seen::OtherComputer | Seen::Apart));
        if gone || elsewhere {
            if user_hooks::unregister(&yes.name, &yes.agents).is_ok() {
                agents::drop_script(data, &yes.name);
                ledger.drop_hook(&file);
                changed = true;
            }
        } else if seen == Some(Seen::HookWaiting) {
            agents::drop_script(data, &yes.name);
        }
    }
    changed
}

#[tauri::command(async)]
pub fn agent_hook_new(
    session: tauri::State<'_, Mutex<Session>>,
    asked: NewHook,
) -> Answer<tisty_core::docs::Doc> {
    made(&mut held(&session), &asked)
}

#[tauri::command(async)]
pub fn agent_hook_look(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
) -> Answer<HookLook> {
    looked(&held(&session), &file)
}

#[tauri::command(async)]
pub fn agent_hook_register(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    print: String,
) -> Answer<HookLook> {
    registered(&held(&session), &file, &print)
}

#[tauri::command(async)]
pub fn agent_hook_unregister(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
) -> Answer<HookLook> {
    unregistered(&held(&session), &file)
}

#[tauri::command(async)]
pub fn agent_hook_try(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    print: String,
) -> Answer<Tried> {
    let trial = trial(&held(&session), &file, &print)?;
    tried(trial)
}

#[cfg(test)]
#[path = "user_hooks_test.rs"]
mod tests;
