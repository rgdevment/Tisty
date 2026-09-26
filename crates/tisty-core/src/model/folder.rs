use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type FolderId = Ulid;
pub type DocId = Ulid;

/// A move past this depth is refused while projecting, not only while writing, so two machines that
/// disagree would build different trees from one log. Raising it raises SCHEMA_VERSION with it.
pub const DEEPEST: usize = 4;

/// Longer than this and the name stops fitting the rail it is read from.
pub const FOLDER_NAME_AT_MOST: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    pub id: FolderId,
    pub name: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<FolderId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archived: bool,
}

impl Folder {
    pub fn new(id: FolderId, name: impl Into<String>, order: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            order: order.into(),
            parent: None,
            icon: None,
            color: None,
            archived: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flagged {
    pub at: jiff::Timestamp,
    pub by: crate::event::DeviceId,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kept {
    pub id: DocId,
    pub file: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrote: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub made: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub made_by: Option<crate::event::DeviceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrote_by: Option<crate::event::DeviceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub born_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edited_by: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub guest: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<FolderId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_of: Option<DocId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archived: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub locked: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<crate::model::Tag>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flagged: Option<Flagged>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder_was: Option<Vec<String>>,
}

#[cfg(test)]
#[path = "folder_test.rs"]
mod tests;
