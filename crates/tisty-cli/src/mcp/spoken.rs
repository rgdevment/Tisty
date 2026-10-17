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
    let mut reply = write(&sent)?;
    if changed.is_empty() {
        return Ok(reply);
    }
    let note = format!(
        " It came in another Markdown and was written in the Tisty language: {}. Writing it that way to begin with keeps every word as you meant it; `language` has the whole of it, and {} shows it drawn ({}es/ in Spanish).",
        changed.join(", "),
        super::looking::GUIDE,
        super::looking::GUIDE
    );
    if let Some(text) = reply["content"][0]["text"].as_str() {
        reply["content"][0]["text"] = Value::String(format!("{text}{note}"));
    }
    reply["structuredContent"]["turned"] = serde_json::json!(changed);
    reply["structuredContent"]["guide"] = serde_json::json!(super::looking::GUIDE);
    Ok(reply)
}

// A task proposed with its parts, or several at once, carries the same field at every level.
fn turned(value: &mut Value, field: &str, changed: &mut Vec<&'static str>) {
    match value {
        Value::Object(fields) => {
            for (key, one) in fields.iter_mut() {
                match (key == field, one.as_str()) {
                    (true, Some(said)) => {
                        let made = tisty_core::arriving::tidied(said);
                        for what in made.changed {
                            if what != "Windows line endings" && !changed.contains(&what) {
                                changed.push(what);
                            }
                        }
                        *one = Value::String(made.body);
                    }
                    _ => turned(one, field, changed),
                }
            }
        }
        Value::Array(all) => all.iter_mut().for_each(|one| turned(one, field, changed)),
        _ => {}
    }
}
