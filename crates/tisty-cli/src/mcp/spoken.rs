use serde_json::Value;

use super::Refused;

/// Markdown from anywhere is taken: it is turned into the Tisty language first, and what that
/// changed rides back on the answer so the person can hear it.
pub(super) fn spoken(
    args: &Value,
    field: &str,
    write: impl FnOnce(&Value) -> Result<Value, Refused>,
) -> Result<Value, Refused> {
    let mut sent = args.clone();
    let mut changed = Vec::new();
    turned(&mut sent, field, &mut changed);
    Ok(heard(write(&sent)?, &changed))
}

/// What turning changed rides back on the answer, so the next write comes in the Tisty language.
pub(super) fn heard(mut reply: Value, changed: &[&str]) -> Value {
    if changed.is_empty() {
        return reply;
    }
    let note = format!(
        " It came in another Markdown and was written in the Tisty language: {}. Writing it that way to begin with keeps every word as you meant it; `language` gives each part of it by topic, and {} shows it drawn ({}es/ in Spanish).",
        changed.join(", "),
        super::looking::GUIDE,
        super::looking::GUIDE
    );
    if let Some(text) = reply["content"][0]["text"].as_str() {
        reply["content"][0]["text"] = Value::String(format!("{text}{note}"));
    }
    reply["structuredContent"]["turned"] = serde_json::json!(changed);
    reply["structuredContent"]["guide"] = serde_json::json!(super::looking::GUIDE);
    reply
}

/// One piece of Markdown in the Tisty language, noting what had to change.
pub(super) fn said_in_tisty(said: &str, changed: &mut Vec<&'static str>) -> String {
    let made = tisty_core::arriving::tidied(said);
    for what in made.changed {
        if what != "Windows line endings" && !changed.contains(&what) {
            changed.push(what);
        }
    }
    made.body
}

// A task proposed with its parts, or several at once, carries the same field one level down.
const NESTED: [&str; 2] = ["tasks", "parts"];

fn turned(value: &mut Value, field: &str, changed: &mut Vec<&'static str>) {
    if let Some(said) = value.get(field).and_then(Value::as_str) {
        value[field] = Value::String(said_in_tisty(said, changed));
    }
    for nested in NESTED {
        if let Some(Value::Array(all)) = value.get_mut(nested) {
            all.iter_mut().for_each(|one| turned(one, field, changed));
        }
    }
}
