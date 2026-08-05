use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type ListId = Ulid;

/// A theme of work, not a project: "project" would promise dates, progress and
/// templates. Distinct from a tag — `bug` classifies, it is not a place.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct List {
    pub id: ListId,
    pub name: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub archived: bool,
}

impl List {
    pub fn new(id: ListId, name: impl Into<String>, order: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            order: order.into(),
            color: None,
            archived: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_list_serialises_to_the_minimum() {
        let json =
            serde_json::to_string(&List::new(Ulid::generate(), "checkout rewrite", "a0")).unwrap();
        assert!(!json.contains("color"));
        assert!(!json.contains("archived"));
    }

    #[test]
    fn round_trips() {
        let mut list = List::new(Ulid::generate(), "spring cleaning", "a1");
        list.color = Some("#e44".into());
        list.archived = true;
        let json = serde_json::to_string(&list).unwrap();
        assert_eq!(list, serde_json::from_str::<List>(&json).unwrap());
    }
}
