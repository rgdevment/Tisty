use serde_json::{Value, json};
use tisty_core::agents::{Event, Tools};

use super::catalog::{self, Client};
use super::files::{self, Read, Writes};
use super::hooks::{self, Kind, Marked};
use super::leads::{Leads, leads};
use super::places::Place;
use super::user_hook_table::{events, placed};
use super::{Stuck, here, json, running};

const SLACK_SECONDS: u64 = 2;
const ANYTHING: &str = "*";

pub struct Hook<'a> {
    pub name: &'a str,
    pub event: Event,
    pub tools: Tools,
    pub timeout: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentHook {
    pub agent: String,
    pub name: String,
    pub at: String,
    pub takes: bool,
    pub installed: bool,
    pub present: bool,
    pub here: bool,
    pub apart: bool,
    pub unread: bool,
    /// The agent runs it only once the person approves it there too, as Codex does with `/hooks`.
    pub trusted_by_hand: bool,
    pub stuck: Option<String>,
}

// The name sits between flags, so no hook's name can ever read as Tisty's own `hook <client>`.
pub fn mark(name: &str) -> String {
    format!("run-hook --name {name} --agent")
}

fn root(kind: Kind, name: &str) -> String {
    match kind {
        Kind::Antigravity => format!("tisty-{name}"),
        _ => "hooks".to_string(),
    }
}

fn marked<'a>(kind: Kind, mark: &'a str, root: &'a str) -> Marked<'a> {
    Marked {
        mark,
        root,
        events: events(kind),
    }
}

pub fn takes(kind: Kind, event: Event, tools: Tools) -> bool {
    placed(kind, event, tools).is_some()
}

pub fn with(kind: Kind, text: &str, place: &Place, hook: &Hook) -> Option<String> {
    let at = placed(kind, hook.event, hook.tools)?;
    let (mark, root) = (mark(hook.name), root(kind, hook.name));
    let marked = marked(kind, &mark, &root);
    let fits = events(kind)
        .iter()
        .all(|event| running::list_fits(text, &root, event));
    if !fits || hooks::ways_of(kind, text, &marked) > 0 || turned_off(text, &root) {
        return None;
    }
    let entry = entry(kind, place, hook, at.matcher);
    if already(kind, text, &marked, at.event, &entry) {
        return Some(text.to_string());
    }
    let mut now = hooks::taken_out_of(kind, text, &marked, |run| run.all_read)?;
    if now.trim().is_empty() {
        now = hooks::blank(kind);
    }
    let now = hooks::pushed(now, &root, vec![(at.event, entry)])?;
    let present = hooks::runs_of(kind, &now, &marked)
        .iter()
        .any(|run| run.all_read);
    (hooks::sound_of(kind, text, &now, &marked) && present).then_some(now)
}

// Codex trusts an entry by its place in the list too, so one already written as it would be stays put.
fn already(kind: Kind, text: &str, marked: &Marked, event: &str, entry: &Value) -> bool {
    let ours = hooks::runs_of(kind, text, marked);
    let listed = json::value(text)
        .and_then(|doc| doc.get(marked.root)?.get(event)?.as_array().cloned())
        .unwrap_or_default();
    ours.len() == 1 && listed.contains(entry)
}

pub fn without(kind: Kind, text: &str, name: &str) -> Option<String> {
    let (mark, root) = (mark(name), root(kind, name));
    hooks::taken_out_of(kind, text, &marked(kind, &mark, &root), |run| run.all_read)
}

fn entry(kind: Kind, place: &Place, hook: &Hook, matcher: Option<&str>) -> Value {
    let args = ["run-hook", "--name", hook.name, "--agent", kind.client()];
    let before = hook.event == Event::BeforeTool;
    let seconds = u64::from(hook.timeout) + SLACK_SECONDS;
    let named = format!("tisty-{}", hook.name);
    let mut run = kind.run_with(
        place,
        &args,
        seconds,
        kind == Kind::Copilot && before,
        &named,
    );
    let grouped = |run: Value, matcher: Option<&str>| match matcher {
        Some(matcher) => json!({ "matcher": matcher, "hooks": [run] }),
        None => json!({ "hooks": [run] }),
    };
    match kind {
        Kind::Claude | Kind::Codex | Kind::Gemini => grouped(run, matcher),
        Kind::Antigravity if hook.event.on_tools() => {
            grouped(run, Some(matcher.unwrap_or(ANYTHING)))
        }
        Kind::Antigravity => run,
        Kind::Cursor | Kind::Copilot => {
            if let Some(matcher) = matcher {
                run["matcher"] = json!(matcher);
            }
            run
        }
    }
}

fn turned_off(text: &str, root: &str) -> bool {
    json::value(text).is_some_and(|doc| {
        doc.get(root).and_then(|hook| hook.get("enabled")) == Some(&Value::Bool(false))
    })
}

fn standing_in(kind: Kind, text: &str, place: &Place, name: &str) -> AgentHook {
    let (mark, root) = (mark(name), root(kind, name));
    let marked = marked(kind, &mark, &root);
    let runs = hooks::runs_of(kind, text, &marked);
    let read: Vec<_> = runs.iter().filter(|run| run.all_read).collect();
    AgentHook {
        present: !read.is_empty(),
        here: !read.is_empty()
            && read
                .iter()
                .flat_map(|run| &run.programs)
                .all(|one| leads(place, &one.said) == Leads::Here),
        apart: read.len() != runs.len() || turned_off(text, &root),
        ..AgentHook::default()
    }
}

fn agents_here(asked: &[String]) -> Option<(Place, Vec<(Client, Kind)>)> {
    let place = here()?;
    let all = catalog::all()
        .into_iter()
        .filter(|client| asked.contains(&client.id))
        .filter_map(|client| Some((hooks::kind(&client.id)?, client)))
        .map(|(kind, client)| (client, kind))
        .collect();
    Some((place, all))
}

fn looked(client: &Client, kind: Kind, place: &Place, hook: &Hook) -> (AgentHook, Option<String>) {
    let at = kind.file(place);
    let shown = at
        .as_ref()
        .map(|it| it.display().to_string())
        .unwrap_or_default();
    let read = at.as_deref().map(files::read);
    let text = match &read {
        Some(Read::Text(text)) => Some(text.clone()),
        Some(Read::Missing) => Some(String::new()),
        Some(Read::Unread) | None => None,
    };
    let mut seen = match &text {
        Some(text) => standing_in(kind, text, place, hook.name),
        None => AgentHook::default(),
    };
    seen.agent = client.id.clone();
    seen.name = client.name.clone();
    seen.at = shown;
    seen.takes = takes(kind, hook.event, hook.tools);
    seen.installed = client.installed(place);
    seen.unread = text.is_none();
    seen.trusted_by_hand = kind == Kind::Codex;
    (seen, text)
}

pub fn verify(hook: &Hook, asked: &[String]) -> Vec<AgentHook> {
    let Some((place, all)) = agents_here(asked) else {
        return Vec::new();
    };
    all.iter()
        .map(|(client, kind)| looked(client, *kind, &place, hook).0)
        .collect()
}

pub fn register(hook: &Hook, asked: &[String]) -> Result<Vec<AgentHook>, Stuck> {
    let (place, all) = agents_here(asked).ok_or(Stuck::NoSuch)?;
    if !place.open {
        return Err(Stuck::Shut);
    }
    let mut writes = Writes::default();
    let mut seen = Vec::new();
    for (client, kind) in &all {
        let (mut one, text) = looked(client, *kind, &place, hook);
        one.stuck = match (text, kind.file(&place)) {
            _ if !one.takes => Some("unsupported".to_string()),
            _ if !one.installed => Some("notInstalled".to_string()),
            (Some(was), Some(at)) => match with(*kind, &was, &place, hook) {
                Some(now) => writes
                    .change(&at, &was, &now, place.moved_aside(&at))
                    .err()
                    .map(|_| "unwritten".to_string()),
                None if one.apart => Some("apart".to_string()),
                None => Some("puzzling".to_string()),
            },
            _ => Some("unread".to_string()),
        };
        seen.push(one);
    }
    Ok(verify(hook, asked)
        .into_iter()
        .zip(seen)
        .map(|(now, was)| AgentHook {
            stuck: was.stuck,
            ..now
        })
        .collect())
}

pub fn unregister(name: &str, asked: &[String]) -> Result<(), Stuck> {
    let (place, all) = agents_here(asked).ok_or(Stuck::NoSuch)?;
    let mut writes = Writes::default();
    for (_, kind) in &all {
        let Some(at) = kind.file(&place) else {
            continue;
        };
        let was = files::text(&at)?;
        if was.is_empty() {
            continue;
        }
        let now = without(*kind, &was, name)
            .ok_or_else(|| Stuck::HooksPuzzling(at.display().to_string()))?;
        match now != was && hooks::alone(*kind) && hooks::emptied(&now) {
            true => writes.remove(&at, &was)?,
            false => writes.change(&at, &was, &now, place.moved_aside(&at))?,
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "user_hooks_test.rs"]
mod tests;
