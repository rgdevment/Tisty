use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type FolderId = Ulid;
pub type DocId = Ulid;

pub const DEEPEST: usize = 2;

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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kept {
    pub id: DocId,
    pub file: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<FolderId>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archived: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_folder_serialises_to_the_minimum() {
        let json = serde_json::to_string(&Folder::new(Ulid::generate(), "trabajo", "a0")).unwrap();

        assert!(!json.contains("parent"), "{json}");
        assert!(!json.contains("icon"), "{json}");
    }

    #[test]
    fn a_document_with_no_folder_is_unfiled_rather_than_absent() {
        let kept = Kept {
            id: Ulid::generate(),
            file: "a3f1-0001".into(),
            order: "a0".into(),
            folder: None,
            archived: false,
        };
        let json = serde_json::to_string(&kept).unwrap();

        assert!(!json.contains("folder"), "{json}");
        assert!(!json.contains("archived"), "{json}");
        assert_eq!(serde_json::from_str::<Kept>(&json).unwrap(), kept);
    }
}
