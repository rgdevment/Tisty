use std::sync::OnceLock;

use serde_json::Value;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

fn whole() -> &'static Value {
    static WHOLE: OnceLock<Value> = OnceLock::new();
    WHOLE.get_or_init(|| serde_json::from_str(CONTRACT).expect("the contract is generated as JSON"))
}

fn days_in(year: u32, month: u32) -> u32 {
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn is_time(said: &str) -> bool {
    let b = said.as_bytes();
    if b.len() != 20 {
        return false;
    }
    let digits = |from: usize, to: usize| b[from..to].iter().all(u8::is_ascii_digit);
    let number = |from: usize, to: usize| said[from..to].parse::<u32>().unwrap_or(99);
    digits(0, 4)
        && b[4] == b'-'
        && digits(5, 7)
        && b[7] == b'-'
        && digits(8, 10)
        && b[10] == b'T'
        && digits(11, 13)
        && b[13] == b':'
        && digits(14, 16)
        && b[16] == b':'
        && digits(17, 19)
        && b[19] == b'Z'
        && (1..=12).contains(&number(5, 7))
        && (1..=days_in(number(0, 4), number(5, 7))).contains(&number(8, 10))
        && number(11, 13) <= 23
        && number(14, 16) <= 59
        && number(17, 19) <= 59
}

fn base64url_of(said: &str, length: usize) -> bool {
    said.len() == length
        && said
            .bytes()
            .all(|one| one.is_ascii_alphanumeric() || one == b'-' || one == b'_')
}

fn is_language(said: &str) -> bool {
    let lower = |one: &str| one.bytes().all(|letter| letter.is_ascii_lowercase());
    let mut parts = said.split('-');
    let language = parts.next().unwrap_or_default();
    if !(2..=3).contains(&language.len()) || !lower(language) {
        return false;
    }
    let mut script = false;
    let mut region = false;
    for part in parts {
        let bytes = part.as_bytes();
        let is_script = bytes.len() == 4 && bytes[0].is_ascii_uppercase() && lower(&part[1..]);
        let is_region = (bytes.len() == 2 && bytes.iter().all(u8::is_ascii_uppercase))
            || (bytes.len() == 3 && bytes.iter().all(u8::is_ascii_digit));
        if is_script && !script && !region {
            script = true;
        } else if is_region && !region {
            region = true;
        } else {
            return false;
        }
    }
    true
}

fn is_tag_word(said: &str) -> bool {
    said == said.to_lowercase()
        && said.nfd().filter(|one| !is_combining_mark(*one)).eq(said.chars())
        && said
            .split('-')
            .all(|part| !part.is_empty() && part.chars().all(char::is_alphanumeric))
}

fn check_type(all: &Value, kind: &str, value: &Value, at: &str, out: &mut Vec<String>) {
    let wrong = |said: &str| format!("{at} {said}");
    match kind {
        "text" => {
            if !value.is_string() {
                out.push(wrong("is not text"));
            }
        }
        "bool" => {
            if !value.is_boolean() {
                out.push(wrong("is not true or false"));
            }
        }
        "whole" => {
            if !value.is_u64() {
                out.push(wrong("is not a whole number"));
            }
        }
        "number" => {
            if !value.is_number() {
                out.push(wrong("is not a number"));
            }
        }
        "time" => {
            if !value.as_str().is_some_and(is_time) {
                out.push(wrong("is not a date in whole seconds"));
            }
        }
        "language" => {
            if !value.as_str().is_some_and(is_language) {
                out.push(wrong("is not a language tag"));
            }
        }
        "tag_word" => {
            if !value.as_str().is_some_and(is_tag_word) {
                out.push(wrong("is not a tag"));
            }
        }
        "key" => {
            if !value.as_str().is_some_and(|one| base64url_of(one, 43)) {
                out.push(wrong("is not a key"));
            }
        }
        "hash" => {
            if !value.as_str().is_some_and(|one| base64url_of(one, 43)) {
                out.push(wrong("is not a hash"));
            }
        }
        "signed" => {
            if !value.as_str().is_some_and(|one| base64url_of(one, 86)) {
                out.push(wrong("is not a signature"));
            }
        }
        _ => match all["enums"].get(kind) {
            Some(values) => {
                let listed = values.as_array().is_some_and(|many| many.contains(value));
                if !listed {
                    out.push(wrong("is none of what its list allows"));
                }
            }
            None => check_fields(all, kind, value, at, out),
        },
    }
}

fn check_fields(all: &Value, name: &str, value: &Value, at: &str, out: &mut Vec<String>) {
    let Some(given) = value.as_object() else {
        out.push(format!("{at} is not a {name}"));
        return;
    };
    let gone = all["contracts"][name]["except"].as_array().into_iter().flatten();
    for field in gone.filter_map(Value::as_str).filter(|field| given.contains_key(*field)) {
        let here = if at.is_empty() { field.to_owned() } else { format!("{at}.{field}") };
        out.push(format!("{here} is not something a {name} has"));
    }
    let Some(fields) = all["contracts"][name]["fields"].as_object() else {
        return;
    };
    for (field, said) in fields {
        let here = if at.is_empty() { field.clone() } else { format!("{at}.{field}") };
        let one = given.get(field).filter(|one| !one.is_null());
        let Some(one) = one else {
            let when = said["required_when"].as_str().and_then(|field| given.get(field));
            if said["required"].as_bool() == Some(true) || when == Some(&Value::Bool(true)) {
                out.push(format!("{here} is missing"));
            }
            continue;
        };
        let least = said["at_least"].as_u64();
        let most = said["at_most"].as_u64();
        if said["type"] == "list" {
            let Some(many) = one.as_array() else {
                out.push(format!("{here} is not a list"));
                continue;
            };
            if least.is_some_and(|floor| (many.len() as u64) < floor) {
                out.push(format!("{here} has too few"));
            }
            if most.is_some_and(|top| many.len() as u64 > top) {
                out.push(format!("{here} has too many"));
            }
            let of = said["of"].as_str().unwrap_or_default();
            for (n, item) in many.iter().enumerate() {
                check_type(all, of, item, &format!("{here}[{n}]"), out);
            }
            continue;
        }
        let kind = said["type"].as_str().unwrap_or_default();
        check_type(all, kind, one, &here, out);
        let size = match one {
            Value::String(text) => Some(text.chars().count() as u64),
            Value::Number(number) => number.as_u64(),
            _ => None,
        };
        if let Some(size) = size {
            if least.is_some_and(|floor| size < floor) {
                out.push(format!("{here} is too short or too low"));
            }
            if most.is_some_and(|top| size > top) {
                out.push(format!("{here} is too long or too high"));
            }
        }
    }
}

/// What is wrong with a value under this contract: nothing when it holds.
pub fn check(contract: &str, value: &Value) -> Vec<String> {
    let all = whole();
    if all["contracts"].get(contract).is_none() {
        return vec![format!("there is no contract called {contract}")];
    }
    let mut out = Vec::new();
    check_fields(all, contract, value, "", &mut out);
    out
}

fn is_space(one: char) -> bool {
    matches!(one, '\t' | '\n' | '\x0C' | '\r' | ' ')
}

struct Opened {
    tag: String,
    attributes: Vec<(String, String)>,
    unclosed: bool,
}

// Each tag the HTML opens, with its attributes. It walks as checkHtml does in TypeScript, so one page gets one verdict.
fn opened(html: &str) -> Vec<Opened> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        if !rest.starts_with(|one: char| one.is_ascii_alphabetic()) {
            continue;
        }
        let end = rest
            .find(|one: char| is_space(one) || one == '/' || one == '>')
            .unwrap_or(rest.len());
        let mut one = Opened { tag: rest[..end].to_ascii_lowercase(), attributes: Vec::new(), unclosed: false };
        rest = &rest[end..];
        loop {
            rest = rest.trim_start_matches(|one: char| is_space(one) || one == '/');
            if rest.is_empty() {
                break;
            }
            if let Some(after) = rest.strip_prefix('>') {
                rest = after;
                break;
            }
            let end = rest
                .find(|one: char| is_space(one) || one == '=' || one == '/' || one == '>')
                .unwrap_or(rest.len());
            let name = rest[..end].to_ascii_lowercase();
            rest = rest[end..].trim_start_matches(is_space);
            let Some(after) = rest.strip_prefix('=') else {
                one.attributes.push((name, String::new()));
                continue;
            };
            rest = after.trim_start_matches(is_space);
            match rest.chars().next() {
                Some(quote @ ('"' | '\'')) => {
                    let inner = &rest[1..];
                    let Some(close) = inner.find(quote) else {
                        one.unclosed = true;
                        rest = "";
                        break;
                    };
                    one.attributes.push((name, inner[..close].to_owned()));
                    rest = &inner[close + 1..];
                }
                _ => {
                    let end = rest.find(|one: char| is_space(one) || one == '>').unwrap_or(rest.len());
                    one.attributes.push((name, rest[..end].to_owned()));
                    rest = &rest[end..];
                }
            }
        }
        out.push(one);
    }
    out
}

fn listed(many: &Value, one: &str) -> bool {
    many.as_array().is_some_and(|many| many.iter().any(|known| known == one))
}

fn words(said: &str) -> impl Iterator<Item = &str> {
    said.split(is_space).filter(|one| !one.is_empty())
}

fn broken_by(rule: &Value, value: &str) -> bool {
    words(value).any(|one| listed(&rule["never"], one))
        || rule["starts_with"].as_str().is_some_and(|start| !value.starts_with(start))
}

// Browsers drop spaces and control characters and decode entities, so a scheme hidden behind either counts as wrong.
fn wrong_url(all: &Value, value: &str) -> bool {
    let bare: String = value.chars().filter(|one| *one > ' ').collect();
    let end = bare.find(['/', '?', '#']).unwrap_or(bare.len());
    let head = &bare[..end];
    if head.contains('&') {
        return true;
    }
    head.split_once(':')
        .is_some_and(|(scheme, _)| !listed(&all["urls"]["schemes"], &scheme.to_ascii_lowercase()))
}

fn is_size_in(said: &str, unit: &str) -> bool {
    let all_digits = |one: &str| !one.is_empty() && one.bytes().all(|digit| digit.is_ascii_digit());
    said.strip_suffix(unit).is_some_and(|number| match number.split_once('.') {
        Some((whole, part)) => all_digits(whole) && all_digits(part),
        None => all_digits(number),
    })
}

fn wrong_style(all: &Value, tag: &str, one: &str) -> bool {
    let (property, said) = one.split_once(':').unwrap_or(("", one));
    let property = property.trim().to_ascii_lowercase();
    let said = said.trim().to_ascii_lowercase();
    !all["styles"].as_array().into_iter().flatten().any(|rule| {
        rule["on"] == tag
            && rule["property"] == property.as_str()
            && match rule["unit"].as_str() {
                Some(unit) => is_size_in(&said, unit),
                None => listed(&rule["values"], &said),
            }
    })
}

/// What in the HTML of a document is outside the contracts. It is no cleaner: it says whether a document speaks this language.
pub fn check_html(html: &str) -> Vec<String> {
    let all = whole();
    let through = &all["passthrough"]["attributes"];
    let mut out: Vec<String> = Vec::new();
    for Opened { tag, attributes, unclosed } in opened(html) {
        let drawn = &all["allowed"][tag.as_str()];
        if drawn.is_null() && !listed(&all["passthrough"]["tags"], &tag) {
            out.push(format!("<{tag}> is in no contract"));
            continue;
        }
        if unclosed {
            out.push(format!("<{tag}> leaves a quote open"));
        }
        for (name, value) in attributes {
            if !listed(&drawn["attributes"], &name)
                && !listed(&through["*"], &name)
                && !listed(&through[tag.as_str()], &name)
            {
                out.push(format!("{name} on <{tag}> is in no contract"));
            }
            if name == "class" {
                for one in words(&value) {
                    let starts = drawn["class_starts"].as_array().is_some_and(|many| {
                        many.iter().any(|start| start.as_str().is_some_and(|start| one.starts_with(start)))
                    });
                    if !starts && !listed(&drawn["classes"], one) {
                        out.push(format!("the class {one} on <{tag}> is in no contract"));
                    }
                }
            }
            if listed(&all["urls"]["attributes"], &name) && wrong_url(all, &value) {
                out.push(format!("{name} on <{tag}> goes where no contract lets it"));
            }
            if name == "style" {
                for one in value.split(';').map(str::trim).filter(|one| !one.is_empty()) {
                    if wrong_style(all, &tag, one) {
                        out.push(format!("the style {one} on <{tag}> is in no contract"));
                    }
                }
            }
            for rule in all["html_rules"].as_array().into_iter().flatten() {
                if rule["tag"] == tag.as_str() && rule["attribute"] == name.as_str() && broken_by(rule, &value) {
                    out.push(rule["says"].as_str().unwrap_or_default().to_owned());
                }
            }
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    out.retain(|one| seen.insert(one.clone()));
    out
}
