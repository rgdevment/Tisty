use serde_json::{Value, json};
use tisty_core::witness::{self, Fact};

use super::{VERSIONS, instructions};

static SPEAKING_THROUGH: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub(super) fn introduced(params: &Value) {
    let said = params
        .get("clientInfo")
        .or_else(|| {
            params
                .get("_meta")
                .and_then(|meta| meta.get("io.modelcontextprotocol/clientInfo"))
        })
        .and_then(|info| info.get("name"))
        .and_then(Value::as_str)
        .and_then(tisty_core::agent::client_said);
    let protocol = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or("?");
    let mut facts = vec![("protocol", Fact::Id(short(protocol)))];
    if let Some(said) = &said {
        facts.push(("as", Fact::Why(said.clone())));
    }
    witness::note(
        witness::channel::AGENT,
        "a client introduced itself",
        &facts,
    );
    if let Some(said) = said {
        let _ = SPEAKING_THROUGH.set(said);
    }
}

/// The name kept from the greeting, or nothing: an unnamed hand is still let in.
pub(super) fn speaking_through() -> Option<String> {
    SPEAKING_THROUGH.get().cloned()
}

/// What the client chose to send is cut, so it cannot fill the log.
pub(super) fn short(said: &str) -> String {
    said.chars().take(64).collect()
}

pub(super) fn named_tool(params: &Value) -> String {
    short(params.get("name").and_then(Value::as_str).unwrap_or("?"))
}

#[derive(Default)]
pub(super) struct Tally {
    pub(super) calls: usize,
    pub(super) refused: usize,
}

impl Tally {
    pub(super) fn closed(&self) {
        witness::note(
            witness::channel::DOOR,
            "the assistant's door closed",
            &[
                ("calls", Fact::Count(self.calls)),
                ("refused", Fact::Count(self.refused)),
            ],
        );
    }
}

pub(super) fn discovered() -> Value {
    json!({
        "resultType": "complete",
        "supportedVersions": VERSIONS,
        "capabilities": { "tools": {} },
        "instructions": instructions(jiff::Zoned::now().date()),
        "ttlMs": until_the_day_turns(),
        "cacheScope": "public",
        "_meta": { "io.modelcontextprotocol/serverInfo": who() },
    })
}

/// The instructions name today, so a copy kept past midnight would teach the wrong date.
pub(super) fn until_the_day_turns() -> i64 {
    let now = jiff::Zoned::now();
    now.tomorrow()
        .and_then(|then| then.start_of_day())
        .map(|turn| turn.timestamp().as_millisecond() - now.timestamp().as_millisecond())
        .unwrap_or(0)
        .max(0)
}

pub(super) fn legacy_greeting(params: &Value) -> Value {
    let asked = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(VERSIONS[0]);
    let speaking = if VERSIONS.contains(&asked) {
        asked
    } else {
        VERSIONS[0]
    };
    json!({
        "protocolVersion": speaking,
        "capabilities": { "tools": {} },
        "serverInfo": who(),
        "instructions": instructions(jiff::Zoned::now().date()),
    })
}

pub(super) fn who() -> Value {
    json!({ "name": "tisty", "version": env!("CARGO_PKG_VERSION") })
}

pub(super) fn reply(id: Value, result: Value) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string()
}

pub(super) fn fault(id: Value, code: i32, message: &str) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }).to_string()
}

pub(super) fn wrong(why: &str) -> Value {
    json!({
        "resultType": "complete",
        "content": [{ "type": "text", "text": why }],
        "isError": true,
    })
}

/// Some clients hand the model the structured answer alone, so a short text travels inside it too.
pub(super) fn told(text: String, structured: Value) -> Value {
    let structured = match structured {
        Value::Object(mut fields) if text.chars().count() <= SAID_WITHIN => {
            fields.entry("said").or_insert_with(|| json!(text));
            Value::Object(fields)
        }
        other => other,
    };
    json!({
        "resultType": "complete",
        "content": [{ "type": "text", "text": text }],
        "structuredContent": structured,
    })
}

const SAID_WITHIN: usize = 600;
