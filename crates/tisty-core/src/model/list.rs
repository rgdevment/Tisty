use serde::{Deserialize, Serialize};
use ulid::Ulid;

pub type ListId = Ulid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct List {
    pub id: ListId,
    pub name: String,
    pub order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
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
            icon: None,
            archived: false,
        }
    }
}

pub const FIRST: [(&str, &str, &str); 6] = [
    ("Work", "Trabajo", "work"),
    ("Personal", "Personal", "star"),
    ("Family", "Familia", "family"),
    ("Money", "Finanzas", "coin"),
    ("Health", "Salud", "health"),
    ("Learning", "Educación", "study"),
];

pub fn first_lists(code: &str) -> [(&'static str, &'static str); 6] {
    let spanish = code.to_lowercase().starts_with("es");
    FIRST.map(|(english, castilian, icon)| (if spanish { castilian } else { english }, icon))
}

pub fn spoken(configured: Option<&str>) -> String {
    configured
        .map(str::to_string)
        .or_else(|| {
            ["LC_ALL", "LC_MESSAGES", "LANG"]
                .iter()
                .find_map(|key| std::env::var(key).ok())
        })
        .or_else(sys_locale::get_locale)
        .unwrap_or_default()
}

pub fn sown(code: &str) -> Vec<crate::event::Op> {
    let mut ops = Vec::new();
    let mut order = crate::order::first();
    for (name, icon) in first_lists(code) {
        let id = Ulid::generate();
        ops.push(crate::event::Op::ListAdd {
            id,
            d: crate::event::ListAdd {
                name: name.to_string(),
                order: order.clone(),
                color: None,
            },
        });
        ops.push(crate::event::Op::ListLook {
            id,
            d: crate::event::Look {
                icon: Some(Some(icon.to_string())),
                color: None,
            },
        });
        order = crate::order::after(&order);
    }
    ops
}

#[cfg(test)]
#[path = "list_test.rs"]
mod tests;
