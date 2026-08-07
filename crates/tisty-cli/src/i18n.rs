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
    #[serde(default)]
    weekday: BTreeMap<String, String>,
    #[serde(default)]
    month: BTreeMap<String, String>,
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
        Self::choose(configured, machine().as_deref())
    }

    fn choose(configured: Option<&str>, machine: Option<&str>) -> Self {
        configured
            .or(machine)
            .map_or_else(Self::default, Self::from_code)
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

    pub fn code(self) -> &'static str {
        self.0
    }

    /// Unlike `from_code`, an unknown code is refused rather than silently English.
    pub fn known(code: &str) -> Option<Self> {
        let code = code.to_lowercase();
        let tag = code.split(['_', '-', '.']).next().unwrap_or_default();
        LOCALES
            .iter()
            .find(|(name, _)| *name == tag)
            .map(|(name, _)| Self(name))
    }

    pub fn available() -> String {
        LOCALES
            .iter()
            .map(|(name, _)| *name)
            .collect::<Vec<_>>()
            .join(" · ")
    }

    /// A missing key renders visibly; a broken translation must not panic.
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

    /// strftime does not localise, so day and month names come from here.
    pub fn weekday(self, index: u8) -> &'static str {
        self.from(|c| c.weekday.get(&index.to_string()))
    }

    pub fn month(self, index: u8) -> &'static str {
        self.from(|c| c.month.get(&index.to_string()))
    }

    fn from(self, pick: impl Fn(&'static Catalog) -> Option<&'static String>) -> &'static str {
        catalog(self.0)
            .and_then(&pick)
            .or_else(|| catalog(FALLBACK).and_then(&pick))
            .map_or("⟨?⟩", |s| s.as_str())
    }

    pub fn fill(self, key: &str, args: &[(&str, &str)]) -> String {
        let mut out = self.get(key).to_string();
        for (name, value) in args {
            out = out.replace(&format!("{{{name}}}"), value);
        }
        out
    }
}

/// POSIX precedence, then the system: a Windows terminal sets none of these.
fn machine() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .or_else(system)
}

fn system() -> Option<String> {
    first_spoken(preferred_languages())
}

/// Windows returns a ranked list; the first one Tisty speaks beats the fallback.
fn first_spoken(preferred: Vec<String>) -> Option<String> {
    preferred
        .into_iter()
        .find(|code| Lang::known(code).is_some())
}

#[cfg(windows)]
fn preferred_languages() -> Vec<String> {
    sys_locale::get_locales().collect()
}

#[cfg(not(windows))]
fn preferred_languages() -> Vec<String> {
    Vec::new()
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

/// Canonical names only: the aliases would triple the line.
pub const FILTERS: &str =
    "today · tomorrow · week · overdue · inbox · archive · all · @list · #tag · !1";

/// Accepts every language, so scripts written in either keep working.
pub fn canonical_filter(raw: &str) -> Option<&'static str> {
    let raw = raw.to_lowercase();
    for (canonical, aliases) in [
        ("today", &["today", "hoy"][..]),
        ("tomorrow", &["tomorrow", "mañana", "manana"][..]),
        ("week", &["week", "semana"][..]),
        ("overdue", &["overdue", "vencidas", "atrasadas"][..]),
        ("all", &["all", "todas", "todo"][..]),
        ("inbox", &["inbox", "bandeja"][..]),
        ("archive", &["archive", "archivo", "hechas", "done"][..]),
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

    #[test]
    fn every_locale_defines_the_same_keys() {
        let reference = catalog(FALLBACK).unwrap();

        for (name, _) in LOCALES {
            let c = catalog(name).unwrap();
            for key in reference.messages.keys() {
                assert!(c.messages.contains_key(key), "{name} is missing «{key}»");
            }
            for key in reference.weekday.keys() {
                assert!(
                    c.weekday.contains_key(key),
                    "{name} is missing weekday «{key}»"
                );
            }
            for key in reference.month.keys() {
                assert!(c.month.contains_key(key), "{name} is missing month «{key}»");
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

    /// The environment is passed in, never read off the machine running this.
    #[test]
    fn english_is_the_fallback() {
        assert_eq!(Lang::choose(None, None).code(), "en");
        assert_eq!(Lang::choose(None, Some("fr_FR.UTF-8")).code(), "en");
        assert_eq!(Lang::from_code("fr_FR.UTF-8").code(), "en");
    }

    #[test]
    fn a_locale_is_detected_from_any_variant() {
        for code in ["es", "es_CL.UTF-8", "ES_ES", "es-419"] {
            assert_eq!(Lang::from_code(code).code(), "es", "{code}");
        }
    }

    #[test]
    fn an_unsupported_first_choice_falls_to_the_next_one_tisty_speaks() {
        let codes = |list: &[&str]| list.iter().map(|s| s.to_string()).collect();

        assert_eq!(
            first_spoken(codes(&["fr-FR", "es-ES", "en-US"])),
            Some("es-ES".into())
        );
        assert_eq!(
            first_spoken(codes(&["en-GB", "es-CL"])),
            Some("en-GB".into())
        );
        assert_eq!(first_spoken(codes(&["fr-FR", "de-DE"])), None);
        assert_eq!(first_spoken(Vec::new()), None);
    }

    #[test]
    fn configured_locale_wins_over_the_environment() {
        assert_eq!(Lang::choose(Some("es"), Some("en_US.UTF-8")).code(), "es");
        assert_eq!(Lang::choose(Some("en"), Some("es_CL.UTF-8")).code(), "en");
        assert_eq!(Lang::choose(None, Some("es_CL.UTF-8")).code(), "es");
    }

    #[test]
    fn plurals_agree_in_every_locale() {
        assert_eq!(Lang::from_code("en").plural("tasks", 1), "1 task");
        assert_eq!(Lang::from_code("en").plural("tasks", 3), "3 tasks");
        assert_eq!(Lang::from_code("es").plural("tasks", 1), "1 tarea");
        assert_eq!(Lang::from_code("es").plural("tasks", 3), "3 tareas");
    }

    #[test]
    fn day_and_month_names_are_localised() {
        assert_eq!(Lang::from_code("en").weekday(1), "mon");
        assert_eq!(Lang::from_code("es").weekday(1), "lun");
        assert_eq!(Lang::from_code("en").month(8), "aug");
        assert_eq!(Lang::from_code("es").month(8), "ago");
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
