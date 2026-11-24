use std::path::PathBuf;

use super::places::{Os, Place};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    Plain,
    Stdio,
    Copilot,
    Opencode,
    /// Any harness whose settings are a YAML file of loader entries; `key` names its MCP client plugin.
    Patch,
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
    also_apps: Vec<String>,
    #[serde(default)]
    also_clients: Vec<String>,
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

    pub fn reads(&self, text: &str) -> Option<String> {
        match self.shape {
            Shape::Toml => super::toml::reads(text, &self.key, super::NAME),
            Shape::Manual => None,
            Shape::Patch => super::patch::reads(text, &self.key, super::NAME),
            _ => super::json::reads(text, &self.key, super::NAME),
        }
    }

    pub fn holds(&self, text: &str) -> bool {
        match self.shape {
            Shape::Toml => super::toml::holds(text, &self.key, super::NAME),
            Shape::Manual => false,
            Shape::Patch => super::patch::holds(text, &self.key, super::NAME),
            _ => super::json::holds(text, &self.key, super::NAME),
        }
    }

    pub fn sound(&self, was: &str, now: &str) -> bool {
        match self.shape {
            Shape::Patch => super::patch::same_but(was, now, &self.key, super::NAME),
            _ => super::json::same_but(was, now, |doc| {
                let under = doc.get_mut(&self.key).and_then(|it| it.as_object_mut());
                if under.is_some_and(|it| {
                    it.remove(super::NAME);
                    it.is_empty()
                }) {
                    doc.remove(&self.key);
                }
            }),
        }
    }

    /// The file to write: the first one there, else the first one this place has a way to.
    pub fn file(&self, place: &Place) -> Option<PathBuf> {
        let all = self.files(place);
        all.iter()
            .find(|at| at.is_file())
            .cloned()
            .or_else(|| all.into_iter().next())
    }

    /// Settings outlive an uninstall, so only a program that reads them says they still matter, where one is named.
    pub fn installed(&self, place: &Place) -> bool {
        self.stands(place)
            || all()
                .iter()
                .filter(|one| self.also_clients.contains(&one.id))
                .any(|one| one.stands(place))
    }

    fn stands(&self, place: &Place) -> bool {
        let named =
            !self.bins.is_empty() || self.apps.iter().any(|one| place.resolve(one).is_some());
        !named
            || self.started_from(place)
            || self
                .also_apps
                .iter()
                .chain(&self.dirs)
                .any(|one| stands(place, one))
    }

    fn started_from(&self, place: &Place) -> bool {
        self.bins.iter().any(|one| place.runs(one))
            || self
                .apps
                .iter()
                .filter_map(|one| place.resolve(one))
                .any(|at| at.exists())
    }

    // A folder outlives an uninstall, so on its own it only says the assistant may still be here.
    pub fn found(&self, place: &Place) -> Option<Found> {
        let sure = self.files(place).iter().any(|at| at.is_file()) || self.started_from(place);
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

// A Store package takes its own folder away with it when it is uninstalled; any other folder stays behind.
fn stands(place: &Place, said: &str) -> bool {
    let at = place.resolve(said);
    match said.starts_with("store:") {
        true => at.is_some(),
        false => said.ends_with(".app") && at.is_some_and(|at| at.exists()),
    }
}

#[cfg(test)]
#[path = "catalog_test.rs"]
mod tests;
