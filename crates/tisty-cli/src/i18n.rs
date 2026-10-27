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

    pub fn known(code: &str) -> Option<Self> {
        let code = code.to_lowercase();
        let tag = code.split(['_', '-', '.']).next().unwrap_or_default();
        LOCALES
            .iter()
            .find(|(name, _)| *name == tag)
            .map(|(name, _)| Self(name))
    }

    pub fn every() -> impl Iterator<Item = Self> {
        LOCALES.iter().map(|(name, _)| Self(name))
    }

    pub fn get(self, key: &str) -> &'static str {
        catalog(self.0)
            .and_then(|c| c.messages.get(key))
            .or_else(|| catalog(FALLBACK).and_then(|c| c.messages.get(key)))
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

#[cfg(test)]
#[path = "i18n_test.rs"]
mod tests;
