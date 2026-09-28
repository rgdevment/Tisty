use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

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

fn machine() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .find_map(|k| std::env::var(k).ok())
        .or_else(system)
}

fn system() -> Option<String> {
    first_spoken(preferred_languages())
}

fn first_spoken(preferred: Vec<String>) -> Option<String> {
    preferred
        .into_iter()
        .find(|code| Lang::known(code).is_some())
}

#[cfg(any(windows, target_os = "macos"))]
fn preferred_languages() -> Vec<String> {
    sys_locale::get_locales().collect()
}

#[cfg(not(any(windows, target_os = "macos")))]
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

pub const FILTERS: &str = "today · tomorrow · week · overdue · inbox · archive · folded · stories · routines · trace · all · @list · #tag · !do";

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
        (
            "folded",
            &[
                "folded",
                "hidden",
                "ocultas",
                "ocultos",
                "plegadas",
                "descartadas",
                "dropped",
            ][..],
        ),
        ("story", &["stories", "story", "historias", "historia"][..]),
        ("routine", &["routines", "routine", "rutinas", "rutina"][..]),
        ("trace", &["traces", "trace", "rastro", "rastros"][..]),
    ] {
        if aliases.contains(&raw.as_str()) {
            return Some(canonical);
        }
    }
    None
}

#[cfg(test)]
#[path = "i18n_test.rs"]
mod tests;
