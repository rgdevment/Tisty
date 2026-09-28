use serde_json::Value;
use tisty_core::model::{DateSpec, Priority};

use super::{Refused, catalogue::tools};

const AT_MOST: &[(&str, usize)] = &[
    ("title", 500),
    ("description", 64_000),
    ("body", 64_000),
    ("old", 64_000),
    ("new", 64_000),
    ("source", 512),
    ("label", 200),
    ("step", EACH_AT_MOST),
];

const MANY_AT_MOST: &[(&str, usize)] = &[("tags", 32), ("steps", 200), ("remind", 32), ("at", 32)];

const EACH_AT_MOST: usize = 2_000;

pub(super) fn only_what_it_takes(name: &str, args: &Value) -> Result<(), Refused> {
    let Some(said) = args.as_object() else {
        return Ok(());
    };
    let tools = tools();
    let Some(taken) = tools
        .as_array()
        .and_then(|all| all.iter().find(|one| one["name"] == name))
        .and_then(|one| one["inputSchema"]["properties"].as_object())
    else {
        return Ok(());
    };
    if let Some(stray) = said.keys().find(|key| !taken.contains_key(*key)) {
        let mut known: Vec<&str> = taken.keys().map(String::as_str).collect();
        known.sort_unstable();
        return Err(Refused::Tool(format!(
            "`{stray}` is not something `{name}` takes. It takes: {}.",
            known.join(", ")
        )));
    }
    for (key, sent) in said {
        if sent.is_null() {
            continue;
        }
        let kinds = kinds_of(&taken[key]);
        if kinds.is_empty() || kinds.iter().any(|one| holds(one, sent)) {
            continue;
        }
        let says = taken[key]
            .get("description")
            .and_then(Value::as_str)
            .map(|one| match one.trim_end().ends_with('.') {
                true => format!(" It takes: {one}"),
                false => format!(" It takes: {one}."),
            })
            .unwrap_or_default();
        return Err(Refused::Tool(format!(
            "`{key}` takes {}, and what came was {}. Nothing was read from it, because reading \
             it another way would be a guess.{says}",
            kinds
                .iter()
                .map(|one| shaped_as(one))
                .collect::<Vec<_>>()
                .join(" or "),
            came_as(sent)
        )));
    }
    Ok(())
}

pub(super) fn kinds_of(shape: &Value) -> Vec<String> {
    let named = |one: &Value| match one {
        Value::String(said) => vec![said.clone()],
        Value::Array(all) => all
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    };
    let mut out = shape.get("type").map(named).unwrap_or_default();
    if let Some(all) = shape.get("oneOf").and_then(Value::as_array) {
        for one in all {
            out.extend(one.get("type").map(&named).unwrap_or_default());
        }
    }
    out.retain(|one| one != "null");
    out.sort_unstable();
    out.dedup();
    out
}

pub(super) fn holds(wants: &str, sent: &Value) -> bool {
    match wants {
        "string" => sent.is_string(),
        "integer" => {
            sent.is_i64() || sent.is_u64() || sent.as_f64().is_some_and(|one| one.fract() == 0.0)
        }
        "number" => sent.is_number(),
        "boolean" => sent.is_boolean(),
        "array" => sent.is_array(),
        "object" => sent.is_object(),
        _ => true,
    }
}

pub(super) fn shaped_as(wants: &str) -> &'static str {
    match wants {
        "integer" => "a whole number",
        "number" => "a number",
        "boolean" => "true or false",
        "array" => "a list",
        "object" => "a set of fields",
        _ => "text",
    }
}

pub(super) fn came_as(sent: &Value) -> &'static str {
    match sent {
        Value::String(_) => "text",
        Value::Number(_) => "a number",
        Value::Bool(_) => "true or false",
        Value::Array(_) => "a list",
        Value::Object(_) => "a set of fields",
        Value::Null => "nothing",
    }
}

pub(super) fn body_at_most() -> usize {
    AT_MOST
        .iter()
        .find(|(key, _)| *key == "body")
        .map(|(_, most)| *most)
        .unwrap_or(64_000)
}

/// An append-only log rereads a ten-megabyte title forever, and control characters in one
/// would rewrite the terminal it prints on.
pub(super) fn short_and_plain(args: &Value) -> Result<(), Refused> {
    let Some(said) = args.as_object() else {
        return Ok(());
    };
    for (key, most) in AT_MOST {
        let Some(one) = said.get(*key).and_then(Value::as_str) else {
            continue;
        };
        if one.chars().count() > *most {
            return Err(Refused::Tool(format!(
                "`{key}` is longer than the {most} characters Tisty keeps. Shorten it."
            )));
        }
        // A passage copied out of a document written on Windows carries its carriage returns.
        let carriage = matches!(*key, "old" | "new");
        if one
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t' && !(carriage && c == '\r'))
        {
            return Err(Refused::Tool(format!(
                "`{key}` carries control characters. Send plain text."
            )));
        }
    }
    for (key, most) in MANY_AT_MOST {
        let Some(all) = said.get(*key).and_then(Value::as_array) else {
            continue;
        };
        if all.len() > *most {
            return Err(Refused::Tool(format!("`{key}` takes at most {most}.")));
        }
        // Counting them is not enough: one ten-megabyte step is read back forever, and an escape
        // sequence inside one rewrites the terminal that prints it.
        for one in all.iter().filter_map(Value::as_str) {
            if one.chars().count() > EACH_AT_MOST {
                return Err(Refused::Tool(format!(
                    "each of `{key}` is at most {EACH_AT_MOST} characters. Shorten them."
                )));
            }
            if one
                .chars()
                .any(|c| c.is_control() && c != '\n' && c != '\t')
            {
                return Err(Refused::Tool(format!(
                    "`{key}` carries control characters. Send plain text."
                )));
            }
        }
    }
    Ok(())
}

/// Like `listed`, but a list that is not one of strings is refused rather than thinned.
pub(super) fn strings(args: &Value, key: &str) -> Result<Vec<String>, Refused> {
    let Some(given) = args.get(key).filter(|one| !one.is_null()) else {
        return Ok(Vec::new());
    };
    let Some(all) = given.as_array() else {
        return Err(Refused::Tool(format!(
            "`{key}` has to be a list of strings, one per entry."
        )));
    };
    if all.iter().any(|one| !one.is_string()) {
        return Err(Refused::Tool(format!(
            "`{key}` has to be a list of strings, one per entry; one entry is not text."
        )));
    }
    Ok(listed(args, key))
}

pub(super) fn text(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|said| !said.is_empty())
        .map(ToString::to_string)
}

pub(super) fn in_order(on: Option<&DateSpec>, owed: Option<&DateSpec>) -> Result<(), Refused> {
    let (Some(on), Some(owed)) = (on, owed) else {
        return Ok(());
    };
    let today = jiff::Zoned::now().date();
    if owed.date() < on.date() && owed.date() >= today {
        return Err(Refused::Tool(format!(
            "a deadline of {} falls before {}, the day it would be worked on, and neither day \
             has gone by. Send both in one call with the days the right way round, or leave one \
             out and only the other moves. A deadline already past is a different thing and is \
             taken as it is.",
            owed.date(),
            on.date()
        )));
    }
    Ok(())
}

pub(super) fn day(args: &Value, key: &str) -> Result<Option<DateSpec>, Refused> {
    let Some(said) = text(args, key) else {
        return Ok(None);
    };
    let zone = jiff::tz::TimeZone::system();
    let named = zone.iana_name().unwrap_or("UTC").to_string();
    said.parse::<jiff::civil::Date>()
        .map(|on| Some(DateSpec::all_day(on, named)))
        .map_err(|_| {
            Refused::Tool(format!(
                "`{key}` has to be a plain date like 2026-08-31, not {said:?}. Work out the day \
                 yourself before calling."
            ))
        })
}

pub(super) fn moments(args: &Value, key: &str) -> Result<Vec<DateSpec>, Refused> {
    if args.get(key).is_some_and(|one| {
        one.as_array()
            .is_none_or(|all| !all.iter().all(Value::is_string))
    }) {
        return Err(Refused::Tool(format!(
            "`{key}` takes a list of moments, each a day and an hour like \
             [\"2026-08-31T09:00\"]."
        )));
    }
    let zone = jiff::tz::TimeZone::system();
    let named = zone.iana_name().unwrap_or("UTC").to_string();
    let mut out: Vec<DateSpec> = Vec::new();
    for said in strings(args, key)? {
        let at = said
            .contains('T')
            .then(|| said.parse::<jiff::civil::DateTime>().ok())
            .flatten()
            .ok_or_else(|| {
                Refused::Tool(format!(
                    "`{key}` takes a day and an hour like 2026-08-31T09:00, not {said:?}. Work \
                     out the moment yourself before calling."
                ))
            })?;
        if at < jiff::Zoned::now().datetime() {
            return Err(Refused::Tool(format!(
                "`{key}` cannot ring in the past: {said:?} has already gone."
            )));
        }
        let one = DateSpec::floating(at, named.clone());
        if !out.iter().any(|kept| kept.at == one.at) {
            out.push(one);
        }
    }
    Ok(out)
}

pub(super) fn ranked(args: &Value) -> Result<Option<Priority>, Refused> {
    let Some(said) = text(args, "priority") else {
        return Ok(None);
    };
    said.parse::<Priority>().map(Some).map_err(|_| {
        Refused::Tool(format!(
            "`priority` is do, decide, delegate or minor — not {said:?}. Leave it out if nobody \
             said which."
        ))
    })
}

pub(super) fn listed(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|all| {
            all.iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|said| !said.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}
