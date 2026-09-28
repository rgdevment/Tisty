use std::sync::Mutex;

use tisty_core::witness::channel;
use tisty_core::{Config, Event, Op};

use crate::{Answer, Session, blamed, held, wiring};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    on: bool,
    called: Option<String>,
}

/// One assistant as the person meets it: a client Tisty can wire, and what a hand of that name
/// has written in the whole log — on this machine and on the others.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Assistant {
    /// The id the client is wired under when Tisty knows it, else what it called itself,
    /// lower-case; `None` for what was written before clients were named.
    pub via: Option<String>,
    pub named: String,
    pub wired: Option<bool>,
    pub filed: usize,
    pub wrote: usize,
    pub last: Option<String>,
}

#[tauri::command]
pub fn assistants(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<Assistant>> {
    let mut session = held(&session);
    session.reload()?;
    let assistants = session.state.assistants.clone();
    let events = session.log()?;
    Ok(hands(events, &assistants, &wiring::seen()))
}

/// One row per hand, keyed by the id a known client is wired under so what `codex-mcp-client`
/// wrote sits on the Codex row; only what reached the list counts, never a join or a host.
pub fn hands(
    events: &[Event],
    assistants: &std::collections::BTreeSet<tisty_core::DeviceId>,
    seen: &[wiring::Seen],
) -> Vec<Assistant> {
    let mut tally: std::collections::BTreeMap<
        Option<String>,
        (usize, usize, Option<jiff::Timestamp>),
    > = Default::default();
    for event in events
        .iter()
        .filter(|one| assistants.contains(&one.device) && one.entity_id().is_some())
    {
        let key = event.via.as_deref().map(|via| {
            tisty_core::agent::client_id(via).map_or_else(|| via.to_lowercase(), str::to_string)
        });
        let told = tally.entry(key).or_default();
        if matches!(event.op, Op::TaskAdd { .. } | Op::DocAdd { .. }) {
            told.0 += 1;
        }
        told.1 += 1;
        told.2 = Some(
            told.2
                .map_or(event.timestamp, |had| had.max(event.timestamp)),
        );
    }
    let wired: std::collections::BTreeMap<String, bool> = seen
        .iter()
        .map(|one| (one.id.to_string(), one.wired))
        .collect();
    let mut all: Vec<Assistant> = wired
        .keys()
        .map(|id| Some(id.clone()))
        .chain(tally.keys().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .map(|via| {
            let (filed, wrote, last) = tally.get(&via).cloned().unwrap_or_default();
            Assistant {
                named: via
                    .as_deref()
                    .map(tisty_core::agent::client_named)
                    .unwrap_or_default(),
                wired: via.as_deref().and_then(|id| wired.get(id).copied()),
                filed,
                wrote,
                last: last.map(|at| at.to_string()),
                via,
            }
        })
        .collect();
    all.sort_by(|a, b| {
        b.wired
            .is_some()
            .cmp(&a.wired.is_some())
            .then(b.wrote.cmp(&a.wrote))
            .then(a.named.cmp(&b.named))
    });
    all
}

#[tauri::command]
pub fn agent(session: tauri::State<'_, Mutex<Session>>) -> Answer<Agent> {
    let mut session = held(&session);
    session.reload()?;
    let who = session.config.agent_id.clone();
    Ok(Agent {
        on: who.is_some(),
        called: who.map(|one| tisty_core::config::nicknamed(&one.0)),
    })
}

/// Registering is the person's act. Nothing an assistant can say over the wire reaches here,
/// which is what stops one granting itself a voice by connecting.
#[tauri::command]
pub fn agent_turn(session: tauri::State<'_, Mutex<Session>>, on: bool) -> Answer<Agent> {
    {
        let mut session = held(&session);
        session.reload()?;
        let paths = session.paths.clone();
        if on {
            tisty_core::agent::register(&paths)
                .map_err(|e| blamed(channel::STORE, "the agent could not be registered", e))?;
        } else {
            tisty_core::agent::retire(&paths)
                .map_err(|e| blamed(channel::STORE, "the agent could not be retired", e))?;
        }
        session.config = Config::load(&session.paths.config_file())
            .ok()
            .flatten()
            .unwrap_or_else(|| session.config.clone());
        session.reproject()?;
    }
    agent(session)
}
