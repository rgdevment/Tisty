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

// A moment nothing signs may hold a fraction of a second, as the log keeps them.
fn is_moment(said: &str) -> bool {
    match said.split_once('.') {
        Some((whole, part)) => {
            let digits = part.strip_suffix('Z').unwrap_or_default();
            (1..=9).contains(&digits.len())
                && digits.bytes().all(|digit| digit.is_ascii_digit())
                && is_time(&format!("{whole}Z"))
        }
        None => is_time(said),
    }
}

fn unseen(one: char) -> bool {
    one.is_control()
        || matches!(
            one,
            '\u{00AD}'
                | '\u{034F}'
                | '\u{061C}'
                | '\u{115F}'
                | '\u{1160}'
                | '\u{17B4}'..='\u{17B5}'
                | '\u{180B}'..='\u{180F}'
                | '\u{200B}' | '\u{200E}' | '\u{200F}'
                | '\u{2028}'..='\u{202E}'
                | '\u{2060}'..='\u{206F}'
                | '\u{2800}'
                | '\u{3164}'
                | '\u{FEFF}'
                | '\u{FFA0}'
                | '\u{FFF9}'..='\u{FFFB}'
                | '\u{1D173}'..='\u{1D17A}'
                | '\u{E0000}'..='\u{E007F}'
        )
}

// Many scripts write with a joiner, and it only hides a letter where no script needs one: between Latin, Greek or Cyrillic.
fn needs_no_joiner(one: Option<&char>) -> bool {
    one.is_none_or(|one| *one < '\u{0590}' || ('\u{1E00}'..='\u{1FFF}').contains(one))
}

fn is_line(said: &str) -> bool {
    let points: Vec<char> = said.chars().collect();
    let joined = |at: usize| !needs_no_joiner(at.checked_sub(1).and_then(|before| points.get(before))) && !needs_no_joiner(points.get(at + 1));
    !points.iter().any(|one| unseen(*one)) && points.iter().enumerate().all(|(at, one)| !matches!(one, '\u{200C}' | '\u{200D}') || joined(at))
}

fn is_file_name(said: &str) -> bool {
    is_line(said) && !said.contains(['/', '\\'])
}

fn is_name(said: &str) -> bool {
    let letters = said.chars().filter(|one| one.is_alphabetic()).count();
    let digits = said.chars().filter(char::is_ascii_digit).count();
    said.trim() == said && said.chars().count() <= 80 && letters > 0 && digits <= letters && is_line(said)
}

const BASE64URL: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
const WHOLE_AT_MOST: u64 = (1 << 53) - 1;

const SMALL_ORDER: [&str; 14] = [
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIA",
    "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
    "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAIA",
    "JuiVj8KyJ7BFw_SJ8u-Y8NXfrAXTxjM5sTgCiG1T_AU",
    "JuiVj8KyJ7BFw_SJ8u-Y8NXfrAXTxjM5sTgCiG1T_IU",
    "xxdqcD1N2E-6PAt2DRBnDyogU_osOczGTsf9d5KsA3o",
    "xxdqcD1N2E-6PAt2DRBnDyogU_osOczGTsf9d5KsA_o",
    "7P_______________________________________38",
    "7P________________________________________8",
    "7f_______________________________________38",
    "7f________________________________________8",
    "7v_______________________________________38",
    "7v________________________________________8",
];

// The last character holds spare bits, and they are zero: one key has one spelling, so comparing two is comparing them.
fn base64url_of(said: &str, length: usize, spare: usize) -> bool {
    said.len() == length
        && said.bytes().all(|one| BASE64URL.as_bytes().contains(&one))
        && said.bytes().last().and_then(|one| BASE64URL.bytes().position(|known| known == one)).is_some_and(|at| at % spare == 0)
}

fn is_id(said: &str) -> bool {
    !said.is_empty() && said.bytes().all(|one| one.is_ascii_alphanumeric() || one == b'-' || one == b'_')
}

// What JavaScript can tell of a number: an integer it holds exactly, however it was written.
fn is_whole(value: &Value) -> bool {
    match value.as_u64() {
        Some(number) => number <= WHOLE_AT_MOST,
        None => value.as_f64().is_some_and(|number| number.fract() == 0.0 && (0.0..=WHOLE_AT_MOST as f64).contains(&number)),
    }
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
    let apart: Vec<char> = said.nfd().collect();
    // Accents come off the alphabets that have them, and elsewhere a mark is how the word is written.
    let accented = apart.windows(2).any(|one| (one[0] <= '\u{052F}' || ('\u{1E00}'..='\u{1FFF}').contains(&one[0])) && is_combining_mark(one[1]));
    said == said.to_lowercase()
        && !accented
        && said
            .split('-')
            .all(|part| !part.is_empty() && part.chars().all(|one| one.is_alphanumeric() || is_combining_mark(one)))
}

fn check_type(all: &Value, kind: &str, value: &Value, at: &str, out: &mut Vec<String>) {
    let wrong = |said: &str| format!("{at} {said}");
    match kind {
        "line" => {
            if !value.as_str().is_some_and(is_line) {
                out.push(wrong("is not one line a person can read as written"));
            }
        }
        "file_name" => {
            if !value.as_str().is_some_and(is_file_name) {
                out.push(wrong("is not the name of a file"));
            }
        }
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
            if !is_whole(value) {
                out.push(wrong("is not a whole number"));
            }
        }
        "id" => {
            if !value.as_str().is_some_and(is_id) {
                out.push(wrong("is not an id"));
            }
        }
        "time" => {
            if !value.as_str().is_some_and(is_time) {
                out.push(wrong("is not a date in whole seconds"));
            }
        }
        "moment" => {
            if !value.as_str().is_some_and(is_moment) {
                out.push(wrong("is not a date"));
            }
        }
        "name" => {
            if !value.as_str().is_some_and(is_name) {
                out.push(wrong("is not a name a person can read as written"));
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
            if !value.as_str().is_some_and(|one| base64url_of(one, 43, 4) && !SMALL_ORDER.contains(&one)) {
                out.push(wrong("is not a key"));
            }
        }
        "hash" => {
            if !value.as_str().is_some_and(|one| base64url_of(one, 43, 4)) {
                out.push(wrong("is not a hash"));
            }
        }
        "signed" => {
            if !value.as_str().is_some_and(|one| base64url_of(one, 86, 16)) {
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
        let only_when = said["only_when"].as_str();
        if let Some(field) = only_when.filter(|field| given.get(*field) != Some(&Value::Bool(true))) {
            out.push(format!("{here} is only for what is {field}"));
        }
        // A word this version does not list wears its default, so a later one can add it.
        if said.get("otherwise").is_some() && one.is_string() {
            continue;
        }
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
        let bytes = said["bytes_at_most"].as_u64();
        if let (Value::String(text), Some(top)) = (one, bytes) {
            if text.len() as u64 > top {
                out.push(format!("{here} is too long"));
            }
        }
        let size = match one {
            Value::String(text) => Some(text.chars().count() as u64),
            Value::Number(number) => number.as_f64().map(|number| number as u64),
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

fn closes_well(said: &str) -> bool {
    let end = said.find(|one: char| is_space(one) || one == '/' || one == '>').unwrap_or(said.len());
    said.starts_with(|one: char| one.is_ascii_alphabetic()) && said[end..].trim_start_matches(is_space).starts_with('>')
}

// Each tag the HTML opens, with its attributes. It walks as checkHtml does in TypeScript, so one page gets one verdict.
fn opened(html: &str) -> Vec<Opened> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        if rest.starts_with(['!', '?']) {
            out.push(Opened { tag: "!".to_owned(), attributes: Vec::new(), unclosed: false });
        }
        // A closing tag says nothing but its name: a quote in one would hide from here what a browser goes on to run.
        if let Some(closing) = rest.strip_prefix('/') {
            let tag = if closes_well(closing) {
                let end = closing.find(|one: char| is_space(one) || one == '/' || one == '>').unwrap_or(closing.len());
                format!("</{}", closing[..end].to_ascii_lowercase())
            } else {
                "/".to_owned()
            };
            out.push(Opened { tag, attributes: Vec::new(), unclosed: false });
        }
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
        || rule["only"].as_array().is_some_and(|only| !only.iter().any(|one| one == value))
}

// Browsers drop spaces and control characters and decode entities, so a scheme hidden behind either counts as wrong.
// A link says where it goes: nothing without a scheme means anything once it travels, except a heading of its own page.
fn wrong_url(all: &Value, value: &str) -> bool {
    let bare: String = value.chars().filter(|one| *one > ' ').collect();
    if bare.starts_with('#') {
        return false;
    }
    let Some((scheme, place)) = bare.split_once(':') else {
        return true;
    };
    let scheme = scheme.to_ascii_lowercase();
    if !listed(&all["urls"]["schemes"], &scheme) {
        return true;
    }
    scheme == "tisty" && reference_in(all, place).is_none()
}

// One spelling for a place too, and it fits a whole number in every language.
fn reference_in(all: &Value, place: &str) -> Option<(String, u64)> {
    let (kind, at) = place.split_once('/')?;
    let spelled = at == "0" || (!at.starts_with('0') && !at.is_empty() && at.len() <= 15 && at.bytes().all(|digit| digit.is_ascii_digit()));
    if !spelled || !listed(&all["urls"]["references"], kind) {
        return None;
    }
    Some((kind.to_owned(), at.parse().ok()?))
}

/// Each picture the HTML draws, with the width and the height it holds for it.
pub fn pictures_in(html: &str) -> Vec<(u64, String, String)> {
    let all = whole();
    let mut out = Vec::new();
    for Opened { tag, attributes, .. } in opened(html) {
        if tag != "img" {
            continue;
        }
        let said = |name: &str| attributes.iter().find(|(one, _)| one == name).map(|(_, value)| value.clone());
        let Some(src) = said("src") else {
            continue;
        };
        let place = src.strip_prefix("tisty:").filter(|one| one.starts_with("picture/"));
        if let Some((_, at)) = place.and_then(|one| reference_in(all, one)) {
            out.push((at, said("width").unwrap_or_default(), said("height").unwrap_or_default()));
        }
    }
    out
}

/// Each place inside what travels that the HTML leads to, read as a browser reads its attributes.
pub fn references_in(html: &str) -> Vec<(String, u64)> {
    let all = whole();
    let mut out = Vec::new();
    for Opened { attributes, .. } in opened(html) {
        for (name, value) in attributes {
            if !listed(&all["urls"]["attributes"], &name) {
                continue;
            }
            let bare: String = value.chars().filter(|one| *one > ' ').collect();
            let Some((scheme, place)) = bare.split_once(':') else {
                continue;
            };
            if scheme.eq_ignore_ascii_case("tisty") {
                out.extend(reference_in(all, place));
            }
        }
    }
    out
}

// A paint or a marker names something in its own drawing, and nothing else.
fn wrong_paint(value: &str) -> bool {
    let said = value.trim_matches(is_space).to_ascii_lowercase();
    if said.contains('&') || said.contains('\\') {
        return true;
    }
    if !said.contains("url(") {
        return false;
    }
    !said
        .strip_prefix("url(#")
        .and_then(|rest| rest.strip_suffix(')'))
        .is_some_and(|name| !name.is_empty() && !name.contains(|one: char| is_space(one) || one == '(' || one == ')'))
}

fn is_size_in(said: &str, unit: &str) -> bool {
    let all_digits = |one: &str| !one.is_empty() && one.bytes().all(|digit| digit.is_ascii_digit());
    said.strip_suffix(unit).is_some_and(|number| match number.split_once('.') {
        Some((whole, part)) => all_digits(whole) && all_digits(part),
        None => all_digits(number),
    })
}

fn is_sized(said: &str, units: &Value) -> bool {
    let signed = said.strip_prefix('-').unwrap_or(said);
    let number = |one: &str| {
        let bare = one.replacen('.', "", 1);
        !bare.is_empty() && !one.ends_with('.') && bare.bytes().all(|digit| digit.is_ascii_digit())
    };
    if number(signed) && signed.bytes().all(|one| one == b'0' || one == b'.') {
        return true;
    }
    units.as_array().into_iter().flatten().filter_map(Value::as_str).any(|unit| signed.strip_suffix(unit).is_some_and(number))
}

fn is_color(said: &str) -> bool {
    said.strip_prefix('#').is_some_and(|hex| matches!(hex.len(), 3 | 6 | 8) && hex.bytes().all(|one| one.is_ascii_digit() || (b'a'..=b'f').contains(&one)))
}

fn wrong_style(all: &Value, tag: &str, one: &str) -> bool {
    let (property, said) = one.split_once(':').unwrap_or(("", one));
    let property = property.trim_matches(is_space).to_ascii_lowercase();
    let said = said.trim_matches(is_space).to_ascii_lowercase();
    !all["styles"].as_array().into_iter().flatten().any(|rule| {
        rule["on"] == tag
            && rule["property"] == property.as_str()
            && match rule["unit"].as_str() {
                Some(unit) => is_size_in(&said, unit),
                None if !rule["sizes"].is_null() => is_sized(&said, &rule["sizes"]),
                None if !rule["color"].is_null() => is_color(&said),
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
        if tag == "!" {
            out.push("a comment is in no contract".to_owned());
            continue;
        }
        if tag == "/" {
            out.push("a closing tag that says more than its name is in no contract".to_owned());
            continue;
        }
        if let Some(name) = tag.strip_prefix("</") {
            if all["allowed"].get(name).is_none() && !listed(&all["passthrough"]["tags"], name) {
                out.push(format!("{tag}> is in no contract"));
            }
            continue;
        }
        for rule in all["html_rules"].as_array().into_iter().flatten().filter(|rule| rule["tag"] == tag.as_str()) {
            let missing = rule["requires"].as_array().into_iter().flatten().filter_map(Value::as_str).any(|name| !attributes.iter().any(|(said, _)| said == name));
            if missing {
                out.push(rule["says"].as_str().unwrap_or_default().to_owned());
            }
        }
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
                    let ends = drawn["class_ends"].as_array().is_some_and(|many| {
                        many.iter().any(|end| end.as_str().is_some_and(|end| one.ends_with(end)))
                    });
                    if one.contains('&') || (!starts && !ends && !listed(&drawn["classes"], one)) {
                        out.push(format!("the class {one} on <{tag}> is in no contract"));
                    }
                }
            }
            if listed(&all["urls"]["attributes"], &name) && wrong_url(all, &value) {
                out.push(format!("{name} on <{tag}> goes where no contract lets it"));
            }
            if listed(&all["urls"]["paints"], &name) && wrong_paint(&value) {
                out.push(format!("{name} on <{tag}> asks for what is not in its drawing"));
            }
            if name == "style" {
                for one in value.split(';').map(|one| one.trim_matches(is_space)).filter(|one| !one.is_empty()) {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Playable {
    pub format: String,
    pub player: String,
    pub media_type: String,
}

fn says(bytes: &[u8], at: usize, said: &[u8]) -> bool {
    bytes.get(at..at + said.len()) == Some(said)
}

const OGG_SOUNDS: [&[u8]; 3] = [b"OpusHead", b"\x01vorbis", b"\x7fFLAC"];
const EBML_HEADER: usize = 64;

fn is_ogg_sound(bytes: &[u8]) -> bool {
    let packet = 27 + bytes.get(26).copied().map_or(0, usize::from);
    OGG_SOUNDS.iter().any(|said| says(bytes, packet, said))
}

fn is_webm(bytes: &[u8]) -> bool {
    (4..bytes.len().min(EBML_HEADER)).any(|at| says(bytes, at, b"\x42\x82\x84webm"))
}

fn branded_format(bytes: &[u8]) -> Option<String> {
    let size = u32::from_be_bytes(bytes.get(..4)?.try_into().ok()?);
    if size < 16 {
        return None;
    }
    let brand = bytes.get(8..12)?;
    whole()["plays"].as_object()?.iter().find_map(|(format, said)| {
        let branded = said["brands"]
            .as_array()?
            .iter()
            .any(|one| one.as_str().map(str::as_bytes) == Some(brand));
        branded.then(|| format.clone())
    })
}

// Two bytes of sync say little on their own, so the values no frame can hold rule it out.
fn frame_format(bytes: &[u8]) -> Option<&'static str> {
    let [first, second, third, _, ..] = *bytes else {
        return None;
    };
    if first != 0xff || second & 0xe0 != 0xe0 {
        return None;
    }
    let layer = second & 0x06;
    if layer == 0 {
        return (second & 0xf0 == 0xf0 && (third >> 2) & 0x0f <= 12).then_some("aac");
    }
    let fair = layer == 0x02 && (second >> 3) & 0x03 != 1 && third >> 4 != 0x0f && (third >> 2) & 0x03 != 3;
    fair.then_some("mp3")
}

fn format_of(bytes: &[u8]) -> Option<String> {
    if says(bytes, 0, b"RIFF") && says(bytes, 8, b"WAVE") {
        return Some("wav".to_string());
    }
    if says(bytes, 0, b"fLaC") {
        return Some("flac".to_string());
    }
    if says(bytes, 0, b"OggS") {
        return is_ogg_sound(bytes).then(|| "ogg".to_string());
    }
    if says(bytes, 0, b"\x1a\x45\xdf\xa3") {
        return is_webm(bytes).then(|| "webm".to_string());
    }
    if says(bytes, 4, b"ftyp") {
        return branded_format(bytes);
    }
    let tagged = says(bytes, 0, b"ID3")
        && bytes.get(3).is_some_and(|one| *one < 0xff)
        && bytes.get(4).is_some_and(|one| *one < 0xff);
    if tagged {
        return Some("mp3".to_string());
    }
    frame_format(bytes).map(str::to_string)
}

pub fn playable_as(bytes: &[u8]) -> Option<Playable> {
    let format = format_of(bytes)?;
    let said = &whole()["plays"][&format];
    Some(Playable {
        player: said["player"].as_str()?.to_string(),
        media_type: said["type"].as_str()?.to_string(),
        format,
    })
}
