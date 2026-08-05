use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

/// Adding a language: drop a file in `locales/` and add it here. Nothing else.
const LOCALES: &[(&str, &str)] = &[
    ("en", include_str!("../locales/en.toml")),
    ("es", include_str!("../locales/es.toml")),
];

const FALLBACK: &str = "en";

#[derive(Debug, Deserialize)]
struct Catalog {
    #[serde(flatten)]
    messages: BTreeMap<String, String>,
    #[serde(default)]
    plural: BTreeMap<String, PluralForms>,
}

#[derive(Debug, Deserialize)]
struct PluralForms {
    one: String,
    other: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lang(&'static str);

impl Default for Lang {
    fn default() -> Self {
        Self(FALLBACK)
    }
}

impl Lang {
    pub fn detect(configured: Option<&str>) -> Self {
        configured
            .map(Self::from_code)
            .or_else(|| {
                ["LC_ALL", "LC_MESSAGES", "LANG"]
                    .iter()
                    .find_map(|k| std::env::var(k).ok())
                    .map(|v| Self::from_code(&v))
            })
            .unwrap_or_default()
    }

    pub fn from_code(code: &str) -> Self {
        let code = code.to_lowercase();
        let tag = code.split(['_', '-', '.']).next().unwrap_or_default();
        Self(
            LOCALES
                .iter()
                .find(|(name, _)| *name == tag)
                .map_or(FALLBACK, |(name, _)| name),
        )
    }

    #[cfg(test)]
    fn code(self) -> &'static str {
        self.0
    }

    /// A missing key renders visibly rather than panicking: a broken
    /// translation must not take the command down with it.
    pub fn get(self, key: &str) -> &'static str {
        catalog(self.0)
            .and_then(|c| c.messages.get(key))
            .or_else(|| catalog(FALLBACK).and_then(|c| c.messages.get(key)))
            .map_or("⟨?⟩", |s| s.as_str())
    }

    pub fn plural(self, key: &str, n: usize) -> String {
        let forms = catalog(self.0)
            .and_then(|c| c.plural.get(key))
            .or_else(|| catalog(FALLBACK).and_then(|c| c.plural.get(key)));

        match forms {
            Some(f) if n == 1 => f.one.clone(),
            Some(f) => f.other.replace("{n}", &n.to_string()),
            None => format!("⟨{key}⟩"),
        }
    }

    pub fn fill(self, key: &str, args: &[(&str, &str)]) -> String {
        let mut out = self.get(key).to_string();
        for (name, value) in args {
            out = out.replace(&format!("{{{name}}}"), value);
        }
        out
    }
}

fn catalog(code: &str) -> Option<&'static Catalog> {
    static LOADED: OnceLock<BTreeMap<&'static str, Catalog>> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            LOCALES
                .iter()
                .filter_map(|(name, raw)| toml::from_str(raw).ok().map(|c| (*name, c)))
                .collect()
        })
        .get(code)
}

/// Filters are accepted in any language so a Spanish user is not forced into
/// English mid-command, and scripts written either way keep working.
pub fn canonical_filter(raw: &str) -> Option<&'static str> {
    let raw = raw.to_lowercase();
    for (canonical, aliases) in [
        ("today", &["today", "hoy", "hoje"][..]),
        ("all", &["all", "todas", "todo", "todas-as"][..]),
        ("inbox", &["inbox", "bandeja", "caixa"][..]),
        (
            "archive",
            &["archive", "archivo", "hechas", "done", "feitas"][..],
        ),
    ] {
        if aliases.contains(&raw.as_str()) {
            return Some(canonical);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_locale_parses() {
        for (name, _) in LOCALES {
            assert!(catalog(name).is_some(), "{name} failed to parse");
        }
    }

    /// A key present in English and missing elsewhere would silently fall back
    /// and leave the interface half translated.
    #[test]
    fn every_locale_defines_the_same_keys() {
        let reference = catalog(FALLBACK).unwrap();

        for (name, _) in LOCALES {
            let c = catalog(name).unwrap();
            for key in reference.messages.keys() {
                assert!(c.messages.contains_key(key), "{name} is missing «{key}»");
            }
            for key in reference.plural.keys() {
                assert!(
                    c.plural.contains_key(key),
                    "{name} is missing plural «{key}»"
                );
            }
            for key in c.messages.keys() {
                assert!(
                    reference.messages.contains_key(key),
                    "{name} defines «{key}», which {FALLBACK} does not"
                );
            }
        }
    }

    /// Placeholders must survive translation, or the value never reaches the
    /// message and the user sees a sentence with a hole in it.
    #[test]
    fn placeholders_match_the_reference() {
        let reference = catalog(FALLBACK).unwrap();

        for (name, _) in LOCALES {
            let c = catalog(name).unwrap();
            for (key, text) in &reference.messages {
                let expected = placeholders(text);
                let actual = placeholders(&c.messages[key]);
                assert_eq!(expected, actual, "{name}: «{key}» has wrong placeholders");
            }
        }
    }

    fn placeholders(text: &str) -> Vec<String> {
        let mut found: Vec<String> = text
            .split('{')
            .skip(1)
            .filter_map(|s| s.split('}').next())
            .map(str::to_string)
            .collect();
        found.sort();
        found
    }

    #[test]
    fn english_is_the_fallback() {
        assert_eq!(Lang::detect(None).code(), "en");
        assert_eq!(Lang::from_code("fr_FR.UTF-8").code(), "en");
    }

    #[test]
    fn a_locale_is_detected_from_any_variant() {
        for code in ["es", "es_CL.UTF-8", "ES_ES", "es-419"] {
            assert_eq!(Lang::from_code(code).code(), "es", "{code}");
        }
    }

    #[test]
    fn configured_locale_wins_over_the_environment() {
        assert_eq!(Lang::detect(Some("es")).code(), "es");
        assert_eq!(Lang::detect(Some("en")).code(), "en");
    }

    #[test]
    fn plurals_agree_in_every_locale() {
        assert_eq!(Lang::from_code("en").plural("tasks", 1), "1 task");
        assert_eq!(Lang::from_code("en").plural("tasks", 3), "3 tasks");
        assert_eq!(Lang::from_code("es").plural("tasks", 1), "1 tarea");
        assert_eq!(Lang::from_code("es").plural("tasks", 3), "3 tareas");
    }

    #[test]
    fn a_missing_key_is_visible_not_fatal() {
        assert_eq!(Lang::from_code("en").get("no-such-key"), "⟨?⟩");
    }

    #[test]
    fn arguments_are_substituted() {
        let out = Lang::from_code("en").fill("not-found", &[("selector", "abc")]);
        assert!(out.contains("abc"), "{out}");
        assert!(!out.contains('{'), "{out}");
    }

    #[test]
    fn filters_are_accepted_in_any_language() {
        assert_eq!(canonical_filter("hoy"), canonical_filter("today"));
        assert_eq!(canonical_filter("hechas"), canonical_filter("done"));
        assert_eq!(canonical_filter("TODAS"), Some("all"));
        assert_eq!(canonical_filter("nonsense"), None);
    }
}
