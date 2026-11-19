use serde_json::{Map, Value, json};
use tisty_core::docs::{Heading, Highlight};

use super::super::jsonrpc::mirrored;
use super::super::{Refused, WHOLE_UP_TO};

const NOT_WITH: [&str; 4] = ["from", "to", "chars", "cursor"];

pub(super) fn asked(args: &Value) -> Result<bool, Refused> {
    if !args.get("marks").and_then(Value::as_bool).unwrap_or(false) {
        return Ok(false);
    }
    match NOT_WITH.into_iter().find(|key| args.get(*key).is_some()) {
        Some(key) => Err(Refused::Tool(format!(
            "`marks` reads the whole document, or one `section`, so it takes no `{key}`. \
             Nothing was read."
        ))),
        None => Ok(true),
    }
}

pub(super) fn answered(
    body: &str,
    args: &Value,
    named: &str,
    warned: &str,
    mut kept: Map<String, Value>,
) -> Result<Value, Refused> {
    let outline = tisty_core::docs::outlined(body);
    let (from, to) = match args.get("section").and_then(Value::as_u64) {
        Some(at) => tisty_core::docs::section_lines(body, at as usize).ok_or_else(|| {
            Refused::Tool(format!(
                "this document has no section {at}. Its card from `brain` numbers them from 0."
            ))
        })?,
        None => (1, usize::MAX),
    };
    let found: Vec<Highlight> = tisty_core::docs::highlighted(body)
        .into_iter()
        .filter(|one| (from..=to).contains(&one.line))
        .collect();
    let mut room = WHOLE_UP_TO;
    let mut groups: Vec<Value> = Vec::new();
    let mut said = String::new();
    let mut shown = 0;
    for one in &found {
        let (passage, clipped) = match one.said.char_indices().nth(WHOLE_UP_TO) {
            Some((at, _)) => (format!("{}…", &one.said[..at]), true),
            None => (one.said.clone(), false),
        };
        let cost = passage.chars().count();
        if cost > room && shown > 0 {
            break;
        }
        room = room.saturating_sub(cost);
        shown += 1;
        let under = section_of(&outline, one.line);
        let opens = groups
            .last()
            .is_none_or(|last| last["at"] != json!(under.map(|h| h.at)));
        if opens {
            let mut group = json!({ "marks": [] });
            if let Some(head) = under {
                group["at"] = json!(head.at);
                group["section"] = json!(head.title);
                said.push_str(&format!("\n\n{} (at {})", head.title, head.at));
            } else {
                group["at"] = Value::Null;
            }
            groups.push(group);
        }
        if let Some(Value::Array(marks)) = groups.last_mut().map(|last| &mut last["marks"]) {
            let mut mark = json!({ "line": one.line, "said": passage });
            if clipped {
                mark["cut"] = json!(true);
            }
            marks.push(mark);
        }
        said.push_str(&format!("\n{}: {passage}", one.line));
    }
    for group in &mut groups {
        if group["at"].is_null()
            && let Some(fields) = group.as_object_mut()
        {
            fields.remove("at");
        }
    }
    let left = found.len() - shown;
    kept.insert("marks".into(), json!(groups));
    let head = match found.is_empty() {
        true => format!("{warned}Nothing in {named} is highlighted or bold."),
        false => format!(
            "{warned}What {named} highlights or sets in bold, {shown} passage(s) by section, each \
             with its line:{said}"
        ),
    };
    let tail = match left {
        0 => String::new(),
        n => {
            kept.insert("marks_left".into(), json!(n));
            format!("\n\n{n} more past these: ask for them a `section` at a time.")
        }
    };
    Ok(mirrored(format!("{head}{tail}"), Value::Object(kept)))
}

fn section_of(outline: &[Heading], line: usize) -> Option<&Heading> {
    outline
        .iter()
        .rev()
        .find(|head| head.line <= line && line <= head.to.max(head.line))
}

#[cfg(test)]
#[path = "highlights_test.rs"]
mod tests;
