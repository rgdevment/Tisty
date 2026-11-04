//! The contracts of Tisty, generated from their YAML. Change the YAML, never this.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::is_combining_mark;

pub const VERSION: &str = "1.0.0-draft.1";
pub const CONTRACT: &str = include_str!("contract.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Paper {
    A4,
    Letter,
    Folio,
    Tabloid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Orientation {
    Portrait,
    Landscape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SheetsWord {
    On,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SaidPriority {
    Do,
    Decide,
    Delegate,
    Minor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Holds,
    Revoked,
    Refused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Open,
    Done,
    Dropped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    Do,
    Decide,
    Delegate,
    Unset,
    Minor,
}

/// A file a page names. Pictures travel with the page. Everything else stays in Tisty, and whoever cannot show it shows a card that says so.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// A small isolated page inside a document, which can run its own script. Tisty runs it. Whoever does not implement this contract never runs it, and shows a card that says it is seen in Tisty.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Widget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub body: String,
}

/// A document is what a person writes and keeps. It is made of pages, in one language, by someone who signed it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paper: Option<Paper>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sheets: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<Signature>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub made: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrote: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub born_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_by: Option<String>,
    pub pages: Vec<Page>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<Property>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<Translation>>,
}

pub trait DocumentReader {
    fn show(&mut self, document: Document);
    fn set_paper(&mut self, paper: Paper);
    fn set_sheets(&mut self, on: bool);
    fn set_orientation(&mut self, orientation: Orientation);
    fn index(&mut self) -> Vec<String>;
    fn details(&mut self) -> Document;
    fn verdict(&mut self) -> Verdict;
    fn go_to_page(&mut self, at: u64);
    fn set_language(&mut self, lang: String);
    fn open_in_tisty(&mut self);
    fn copy_link(&mut self) -> String;
}

pub trait DocumentWriter {
    fn draw(&mut self, body: String) -> String;
    fn sign(&mut self, document: Document) -> Signature;
}

/// The properties at the start of a document, between two lines of three dashes. The first line is ---, the words start right under it, and --- or ... closes them. Past their limits they stay as written and none is read. A value that says nothing Tisty knows stays written too and does nothing, so no document fails for its properties: the writer reads them and gives the document what they mean. This holds what they mean, once read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontMatter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paper: Option<Paper>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sheets: Option<SheetsWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orientation: Option<Orientation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(rename = "translationKey", default, skip_serializing_if = "Option::is_none")]
    pub translation_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scheduled: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remind: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<SaidPriority>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<String>,
}

/// One property as it was written, including those Tisty does not know, which it keeps and shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Property {
    pub key: String,
    pub values: Vec<String>,
}

/// A page is a part of a document, with words of its own. It is what holds tables, diagrams and everything else.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub at: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub html: Option<String>,
}

/// Where the same document is in another language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Translation {
    pub lang: String,
    pub document: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// A word a document or a task is kept under, in the one form Tisty keeps it. Capitals and accents come off and a space or an underscore turns into a dash, so #Camión and #camion are one tag. In a sentence, #word needs two characters and a letter, and it does not count glued to the text before it, inside code or in [[#2]].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
}

/// Who answers for a document. The root key is who a person is; the machine key signs, under a certificate from the root.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub root: String,
    pub machine: String,
    pub since: String,
    pub certificate: String,
    pub signed: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A task is something to do. It ends, and is kept whole. Only whoever needs tasks implements this.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_of: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steps: Option<Vec<Step>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<Vec<Note>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reminders: Option<Vec<Reminder>>,
}

/// One line of a task's checklist.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub text: String,
    pub done: bool,
}

/// Something recorded on a task, with when.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub at: String,
    pub body: String,
}

/// When a task rings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reminder {
    pub at: String,
}

fn whole() -> Value {
    serde_json::from_str(CONTRACT).expect("the contract is generated as JSON")
}

fn is_time(said: &str) -> bool {
    let b = said.as_bytes();
    let digits = |from: usize, to: usize| b[from..to].iter().all(u8::is_ascii_digit);
    let number = |from: usize, to: usize| said[from..to].parse::<u32>().unwrap_or(99);
    b.len() == 20
        && digits(0, 4)
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
        && (1..=31).contains(&number(8, 10))
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
    let Some(fields) = all["contracts"][name]["fields"].as_object() else {
        return;
    };
    for (field, said) in fields {
        let here = if at.is_empty() { field.clone() } else { format!("{at}.{field}") };
        let one = given.get(field).filter(|one| !one.is_null());
        let Some(one) = one else {
            if said["required"].as_bool() == Some(true) {
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
    check_fields(&all, contract, value, "", &mut out);
    out
}

type Opened = (String, Vec<(String, String)>);

// Each tag a page opens, with its attributes. It walks both quotes, since a quote can hide a > or a name.
fn opened(html: &str) -> Vec<Opened> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find('<') {
        rest = &rest[start + 1..];
        if !rest.starts_with(|one: char| one.is_ascii_alphabetic()) {
            continue;
        }
        let end = rest
            .find(|one: char| one.is_whitespace() || one == '/' || one == '>')
            .unwrap_or(rest.len());
        let tag = rest[..end].to_ascii_lowercase();
        rest = &rest[end..];
        let mut attributes = Vec::new();
        loop {
            rest = rest.trim_start_matches(|one: char| one.is_whitespace() || one == '/');
            if rest.is_empty() {
                break;
            }
            if let Some(after) = rest.strip_prefix('>') {
                rest = after;
                break;
            }
            let end = rest
                .find(|one: char| one.is_whitespace() || one == '=' || one == '/' || one == '>')
                .unwrap_or(rest.len());
            let name = rest[..end].to_ascii_lowercase();
            rest = rest[end..].trim_start();
            let Some(after) = rest.strip_prefix('=') else {
                attributes.push((name, String::new()));
                continue;
            };
            let after = after.trim_start();
            let (value, left) = match after.chars().next() {
                Some(quote @ ('"' | '\'')) => {
                    let inner = &after[1..];
                    let end = inner.find(quote).unwrap_or(inner.len());
                    (&inner[..end], inner.get(end + 1..).unwrap_or_default())
                }
                _ => after.split_at(
                    after
                        .find(|one: char| one.is_whitespace() || one == '>')
                        .unwrap_or(after.len()),
                ),
            };
            attributes.push((name, value.to_owned()));
            rest = left;
        }
        out.push((tag, attributes));
    }
    out
}

fn listed(many: &Value, one: &str) -> bool {
    many.as_array().is_some_and(|many| many.iter().any(|known| known == one))
}

fn broken_by(rule: &Value, value: &str) -> bool {
    value.split_whitespace().any(|one| listed(&rule["never"], one))
        || rule["starts_with"].as_str().is_some_and(|start| !value.starts_with(start))
}

/// What in the HTML of a page is outside the contracts. It is no cleaner: it says whether a page speaks this language.
pub fn check_html(html: &str) -> Vec<String> {
    let all = whole();
    let through = &all["passthrough"]["attributes"];
    let mut out: Vec<String> = Vec::new();
    for (tag, attributes) in opened(html) {
        let drawn = &all["allowed"][tag.as_str()];
        if drawn.is_null() && !listed(&all["passthrough"]["tags"], &tag) {
            out.push(format!("<{tag}> is in no contract"));
            continue;
        }
        for (name, value) in attributes {
            if !listed(&drawn["attributes"], &name)
                && !listed(&through["*"], &name)
                && !listed(&through[tag.as_str()], &name)
            {
                out.push(format!("{name} on <{tag}> is in no contract"));
            }
            if name == "class" {
                for one in value.split_whitespace() {
                    let starts = drawn["class_starts"].as_array().is_some_and(|many| {
                        many.iter().any(|start| start.as_str().is_some_and(|start| one.starts_with(start)))
                    });
                    if !starts && !listed(&drawn["classes"], one) {
                        out.push(format!("the class {one} on <{tag}> is in no contract"));
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
