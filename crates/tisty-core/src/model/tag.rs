use std::fmt;

use serde::{Deserialize, Serialize};

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
        tisty_markdown::tags::named(raw).map(Self).ok_or(InvalidTag)
    }

    /// What a reader may take from writing nobody meant as a label. It stands apart from `new`,
    /// which stays as forgiving as the log it reads back: «#1234» is a ticket, not a tag, but a
    /// tag saved before this rule still has to deserialise.
    pub fn worth_reading(&self) -> bool {
        tisty_markdown::tags::worth_reading(&self.0)
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
