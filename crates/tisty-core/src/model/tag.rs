use std::fmt;

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("a tag needs two characters, one of them a letter")]
pub struct InvalidTag;

/// What an agent's filing is tagged with, so the person finds it however it is signed.
pub const AGENT_TAG: &str = "agent";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Tag(String);

impl Tag {
    pub fn new(raw: &str) -> Result<Self, InvalidTag> {
        // Accents come off: somebody writes #camion on Monday and #camión on Friday, and one tag
        // is what they meant both times. Read back through serde, so tags already written fold
        // themselves the next time the log is read.
        let normalised: String = crate::text::composed(raw)
            .trim()
            .to_lowercase()
            .nfd()
            .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
            .map(|c| {
                if c.is_whitespace() || c == '_' {
                    '-'
                } else {
                    c
                }
            })
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .collect();

        let trimmed = normalised.trim_matches('-');
        if !trimmed.chars().any(char::is_alphanumeric) {
            return Err(InvalidTag);
        }
        Ok(Self(collapse_dashes(trimmed)))
    }

    /// What a reader may take from writing nobody meant as a label. It stands apart from `new`,
    /// which stays as forgiving as the log it reads back: «#1234» is a ticket, not a tag, but a
    /// tag saved before this rule still has to deserialise.
    pub fn worth_reading(&self) -> bool {
        self.0.chars().nth(1).is_some() && self.0.chars().any(char::is_alphabetic)
    }

    pub fn written(raw: &str) -> Result<Self, InvalidTag> {
        Self::new(raw)
            .ok()
            .filter(Self::worth_reading)
            .ok_or(InvalidTag)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn collapse_dashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut previous_dash = false;
    for c in s.chars() {
        if c == '-' {
            if !previous_dash {
                out.push(c);
            }
            previous_dash = true;
        } else {
            out.push(c);
            previous_dash = false;
        }
    }
    out
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for Tag {
    type Error = InvalidTag;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl From<Tag> for String {
    fn from(tag: Tag) -> Self {
        tag.0
    }
}

impl std::str::FromStr for Tag {
    type Err = InvalidTag;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

#[cfg(test)]
#[path = "tag_test.rs"]
mod tests;
