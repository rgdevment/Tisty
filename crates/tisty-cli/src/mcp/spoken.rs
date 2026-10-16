use serde_json::Value;

use super::Refused;

/// Markdown from anywhere is taken: it is turned into the Tisty language first, and what that
/// changed rides back on the answer so the person can hear it.
pub(super) fn spoken(
    args: &Value,
    field: &str,
    write: impl FnOnce(&Value) -> Result<Value, Refused>,
) -> Result<Value, Refused> {
    let Some(said) = args.get(field).and_then(Value::as_str) else {
        return write(args);
    };
    let made = tisty_core::arriving::tidied(said);
    let changed: Vec<&str> = made
        .changed
        .into_iter()
        .filter(|one| *one != "Windows line endings")
        .collect();
    let mut sent = args.clone();
    sent[field] = Value::String(made.body);
    let mut reply = write(&sent)?;
    if changed.is_empty() {
        return Ok(reply);
    }
    let note = format!(
        " It came in another Markdown and was written in the Tisty language: {}. Writing it that way to begin with keeps every word as you meant it; `language` has the whole of it.",
        changed.join(", ")
    );
    if let Some(text) = reply["content"][0]["text"].as_str() {
        reply["content"][0]["text"] = Value::String(format!("{text}{note}"));
    }
    reply["structuredContent"]["turned"] = serde_json::json!(changed);
    Ok(reply)
}
