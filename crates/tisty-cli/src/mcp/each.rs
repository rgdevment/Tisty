use serde_json::{Map, Value};

// What tells one row from another, or is the row's own text, never moves out of it.
const OWN: [&str; 7] = ["id", "doc", "title", "at", "line", "body", "said"];

pub(super) fn said_once(whole: &mut Value, lists: &[&str]) {
    for list in lists {
        let Some(Value::Array(rows)) = whole.get_mut(*list) else {
            continue;
        };
        let Some(shared) = shared(rows) else {
            continue;
        };
        for row in rows.iter_mut().filter_map(Value::as_object_mut) {
            for key in shared.keys() {
                row.remove(key);
            }
        }
        if !whole["each"].is_object() {
            whole["each"] = Value::Object(Map::new());
        }
        whole["each"][*list] = Value::Object(shared);
    }
}

fn shared(rows: &[Value]) -> Option<Map<String, Value>> {
    let (first, rest) = rows.split_first()?;
    if rest.is_empty() {
        return None;
    }
    let shared: Map<String, Value> = first
        .as_object()?
        .iter()
        .filter(|(key, value)| {
            !OWN.contains(&key.as_str())
                && rest.iter().all(|row| row.get(key.as_str()) == Some(value))
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    (!shared.is_empty()).then_some(shared)
}

#[cfg(test)]
#[path = "each_test.rs"]
mod tests;
