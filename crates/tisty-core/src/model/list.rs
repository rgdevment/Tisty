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
    fn a_fresh_install_speaks_the_language_of_the_machine() {
        assert_eq!(
            first_lists("es").map(|(name, _)| name),
            [
                "Trabajo",
                "Personal",
                "Familia",
                "Finanzas",
                "Salud",
                "Educación"
            ]
        );
        assert_eq!(
            first_lists("en").map(|(name, _)| name),
            ["Work", "Personal", "Family", "Money", "Health", "Learning"]
        );
    }

    #[test]
    fn a_fresh_install_covers_what_a_life_is_made_of() {
        let names = first_lists("es").map(|(name, _)| name);
        assert_eq!(
            names.len(),
            6,
            "six is enough to start and few enough to read"
        );
        assert!(names.contains(&"Finanzas"));
        assert!(names.contains(&"Salud"));
        assert!(names.contains(&"Educación"));
    }

    #[test]
    fn a_regional_spanish_is_still_spanish() {
        assert_eq!(first_lists("es-CL").map(|(name, _)| name)[0], "Trabajo");
        assert_eq!(
            first_lists("es_419.UTF-8").map(|(name, _)| name)[0],
            "Trabajo"
        );
    }

    #[test]
    fn a_language_we_do_not_speak_is_served_in_english() {
        assert_eq!(first_lists("fr").map(|(name, _)| name)[0], "Work");
        assert_eq!(first_lists("").map(|(name, _)| name)[0], "Work");
    }

    #[test]
    fn every_first_list_wears_an_icon_the_catalogue_knows() {
        for (_, icon) in first_lists("en") {
            assert!(crate::model::icon::known(icon), "unknown icon: {icon}");
        }
    }

    #[test]
    fn the_three_are_sown_in_the_order_they_are_written() {
        let ops = sown("es");
        assert_eq!(ops.len(), FIRST.len() * 2);

        let mut names = Vec::new();
        let mut orders = Vec::new();
        for op in &ops {
            if let crate::event::Op::ListAdd { d, .. } = op {
                names.push(d.name.clone());
                orders.push(d.order.clone());
            }
        }
        assert_eq!(&names[..3], ["Trabajo", "Personal", "Familia"]);
        assert!(orders[0] < orders[1] && orders[1] < orders[2], "{orders:?}");
    }

    #[test]
    fn each_sown_list_is_dressed_right_after_it_is_made() {
        let ops = sown("en");
        let mut made = Vec::new();
        let mut dressed = Vec::new();
        for op in &ops {
            match op {
                crate::event::Op::ListAdd { id, .. } => made.push(*id),
                crate::event::Op::ListLook { id, .. } => dressed.push(*id),
                _ => panic!("a fresh install should only make and dress lists"),
            }
        }
        assert_eq!(made, dressed);
    }

    #[test]
    fn what_the_settings_say_wins_over_the_machine() {
        assert_eq!(spoken(Some("es")), "es");
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
