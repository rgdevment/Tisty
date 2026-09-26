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
    if let Some(said) = said {
        witness::note(
            witness::channel::AGENT,
            "a client introduced itself",
            &[("as", Fact::Why(said.clone()))],
        );
        let _ = SPEAKING_THROUGH.set(said);
    }
}

/// The name kept from the greeting, or nothing: an unnamed hand is still let in.
pub(super) fn speaking_through() -> Option<String> {
    SPEAKING_THROUGH.get().cloned()
}

pub(super) fn named_tool(params: &Value) -> String {
    params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string()
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

pub(super) fn told(text: String, structured: Value) -> Value {
    json!({
        "resultType": "complete",
        "content": [{ "type": "text", "text": text }],
        "structuredContent": structured,
    })
}
