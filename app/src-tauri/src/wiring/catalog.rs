use std::path::PathBuf;

use super::places::{Os, Place};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    Plain,
    Stdio,
    Copilot,
    Opencode,
    Toml,
    Manual,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Client {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub also: Option<String>,
    pub shape: Shape,
    pub key: String,
    #[serde(default)]
    everywhere: Vec<String>,
    #[serde(default)]
    windows: Vec<String>,
    #[serde(default)]
    macos: Vec<String>,
    #[serde(default)]
    linux: Vec<String>,
    #[serde(default)]
    bins: Vec<String>,
    #[serde(default)]
    apps: Vec<String>,
    #[serde(default)]
    dirs: Vec<String>,
    #[serde(default)]
    pub cli: Option<String>,
}

#[derive(serde::Deserialize)]
struct Catalog {
    client: Vec<Client>,
}

pub fn all() -> Vec<Client> {
    toml::from_str::<Catalog>(include_str!("catalog.toml"))
        .map(|it| it.client)
        .unwrap_or_default()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Found {
    Sure,
    Maybe,
}

impl Client {
    pub fn files(&self, place: &Place) -> Vec<PathBuf> {
        let own = match place.os {
            Os::Windows => &self.windows,
            Os::Macos => &self.macos,
            Os::Linux => &self.linux,
        };
        self.everywhere
            .iter()
            .chain(own)
            .filter_map(|one| place.resolve(one))
            .collect()
    }

    /// The file to write: the first one there, else the first one this place has a way to.
    pub fn file(&self, place: &Place) -> Option<PathBuf> {
        let all = self.files(place);
        all.iter()
            .find(|at| at.is_file())
            .cloned()
            .or_else(|| all.into_iter().next())
    }

    // A folder outlives an uninstall, so on its own it only says the assistant may still be here.
    pub fn found(&self, place: &Place) -> Option<Found> {
        let sure = self.files(place).iter().any(|at| at.is_file())
            || self.bins.iter().any(|one| place.runs(one))
            || self
                .apps
                .iter()
                .filter_map(|one| place.resolve(one))
                .any(|at| at.exists());
        if sure {
            return Some(Found::Sure);
        }
        self.dirs
            .iter()
            .filter_map(|one| place.resolve(one))
            .any(|at| at.is_dir())
            .then_some(Found::Maybe)
    }
}

#[cfg(test)]
#[path = "catalog_test.rs"]
mod tests;
