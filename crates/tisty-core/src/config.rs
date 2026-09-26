use std::path::Path;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{
    Error, Result,
    event::DeviceId,
    paths::Paths,
    store,
    witness::{self, Fact, channel},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "how", content = "at")]
pub enum Sync {
    Local,
    Folder(std::path::PathBuf),
}

fn said_once(named: &str) -> bool {
    static SAID: std::sync::Mutex<Option<std::collections::BTreeSet<String>>> =
        std::sync::Mutex::new(None);
    let mut held = SAID.lock().unwrap_or_else(|e| e.into_inner());
    held.get_or_insert_with(Default::default)
        .insert(named.to_string())
}

impl Config {
    pub fn muted(&self) -> &[String] {
        self.quiet.as_deref().unwrap_or_default()
    }

    pub fn copies_up_to(&self) -> u64 {
        self.attach_up_to
            .unwrap_or(crate::attach::COPIED_UP_TO)
            .clamp(crate::attach::COPIED_LEAST, crate::attach::COPIED_MOST)
    }

    pub fn copies_in_a_doc(&self) -> u64 {
        crate::attach::COPIED_IN_DOC
    }

    /// Without a shared folder there is nowhere else, whatever the setting says.
    pub fn holds(&self) -> Holds {
        match self.sync {
            Some(Sync::Folder(_)) => self.holds.unwrap_or_default(),
            _ => Holds::Everywhere,
        }
    }

    /// The same ceiling a task allows at most: below it everything travels as it always did.
    pub fn only_shared_above(&self) -> u64 {
        crate::attach::COPIED_UP_TO
    }

    pub fn backs_up(&self) -> bool {
        !matches!(self.sync, Some(Sync::Folder(_)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Closing {
    Hide,
    Quit,
}

/// The look the person chose; absent, the window follows the computer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
}

impl std::str::FromStr for Theme {
    type Err = ();

    fn from_str(said: &str) -> std::result::Result<Self, ()> {
        match said {
            "light" => Ok(Self::Light),
            "dark" => Ok(Self::Dark),
            _ => Err(()),
        }
    }
}

/// Where a large attachment lives once there is a shared folder to keep it in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Holds {
    Everywhere,
    Mine,
    #[default]
    Shared,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub device_id: DeviceId,
    /// A second writer on this machine, for whatever files tasks on your behalf. Its own
    /// directory keeps undo apart: this machine never undoes what the agent wrote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<DeviceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opened_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_close: Option<Closing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub theme: Option<Theme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backed_up_at: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync: Option<Sync>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_at: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heard_at: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quiet: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at: Option<jiff::Timestamp>,
    /// What the last look found. Kept so that closing the window does not take the offer with it:
    /// the check only runs once a day, and without this the notice would vanish until tomorrow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub found_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub found_in_the_shop: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attach_up_to: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holds: Option<Holds>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guide: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sown: Option<bool>,
    /// Asked for, never arrived at: a copy is only ever walked onto the candidates' track by
    /// somebody saying so here, and a manifest cannot do it on its own.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidates: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub here_since: Option<jiff::Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asked_for_a_star: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asked_to_wire: Option<bool>,
}

impl Config {
    pub fn load_or_init(paths: &Paths) -> Result<Self> {
        if let Some(existing) = Self::load(&paths.config_file())? {
            if !store::is_device_name(&existing.device_id.0) && said_once(&existing.device_id.0) {
                witness::error(
                    channel::CONFIG,
                    "this machine is named in a way a device directory cannot be, so its history travels nowhere",
                    &[("at", Fact::Id(existing.device_id.0.clone()))],
                );
            }
            return Ok(existing);
        }

        let config = Self {
            device_id: DeviceId(new_device_id()),
            agent_id: None,
            locale: None,
            editor: None,
            quiet: None,
            checked_at: None,
            found_version: None,
            found_in_the_shop: None,
            attach_up_to: Some(crate::attach::COPIED_AT_FIRST),
            holds: None,
            opened_by: None,
            on_close: None,
            theme: None,
            backed_up_at: None,
            sync: None,
            synced_at: None,
            heard_at: None,
            guide: None,
            sown: Some(false),
            candidates: None,
            here_since: Some(jiff::Timestamp::now()),
            asked_for_a_star: None,
            asked_to_wire: None,
        };
        config.save(paths)?;
        Ok(config)
    }

    pub fn load(file: &Path) -> Result<Option<Self>> {
        match std::fs::read_to_string(file) {
            Ok(text) => {
                let mut config: Self = toml::from_str(&text)?;
                config.sown.get_or_insert(true);
                Ok(Some(config))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::Io(e)),
        }
    }

    pub fn save(&self, paths: &Paths) -> Result<()> {
        std::fs::create_dir_all(paths.config())?;
        if let Err(e) = crate::paths::ours_alone(paths.config()) {
            witness::warn(
                channel::CONFIG,
                "config folder not made private",
                &[
                    ("at", Fact::Path(paths.config().to_path_buf())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
        store::write_atomic(
            &paths.config_file(),
            toml::to_string_pretty(self)?.as_bytes(),
        )
    }
}

const TREES: [&str; 64] = [
    "abeto",
    "acacia",
    "alamo",
    "alerce",
    "algarrobo",
    "aliso",
    "almendro",
    "arce",
    "arrayan",
    "avellano",
    "azahar",
    "boj",
    "brezo",
    "cactus",
    "canelo",
    "carrasco",
    "castano",
    "cedro",
    "cerezo",
    "cipres",
    "ciruelo",
    "coihue",
    "drago",
    "encina",
    "enebro",
    "espino",
    "eucalipto",
    "fresno",
    "ginkgo",
    "granado",
    "haya",
    "helecho",
    "hiedra",
    "higuera",
    "jacaranda",
    "jazmin",
    "laurel",
    "lavanda",
    "lentisco",
    "lila",
    "madrono",
    "magnolio",
    "manzano",
    "membrillo",
    "menta",
    "mirto",
    "moral",
    "musgo",
    "nogal",
    "olivo",
    "olmo",
    "orquidea",
    "palmera",
    "peral",
    "pino",
    "quillay",
    "roble",
    "romero",
    "salvia",
    "sauce",
    "tejo",
    "tomillo",
    "trebol",
    "yuca",
];

pub fn nicknamed(device: &str) -> String {
    use sha2::{Digest, Sha256};
    let said = Sha256::digest(device.as_bytes());
    let word = TREES[(said[0] as usize) % TREES.len()];
    let number = (u16::from(said[1]) * 100 / 256) + 1;
    format!("{word} {number}")
}

pub fn new_device_id() -> String {
    let ulid = Ulid::generate().to_string().to_lowercase();
    format!("dev_{}", &ulid[ulid.len() - 8..])
}

#[cfg(test)]
#[path = "config_test.rs"]
mod tests;
