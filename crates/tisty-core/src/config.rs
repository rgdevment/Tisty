use std::path::Path;

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::{Error, Result, event::DeviceId, paths::Paths, store};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Machine-specific and never synced: two machines sharing it would write
    /// the same segment file, which is what makes conflicts impossible.
    pub device_id: DeviceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
}

impl Config {
    /// Generates and persists a device id on first run.
    pub fn load_or_init(paths: &Paths) -> Result<Self> {
        if let Some(existing) = Self::load(&paths.config_file())? {
            return Ok(existing);
        }

        let config = Self {
            device_id: DeviceId(new_device_id()),
            locale: None,
            editor: None,
        };
        config.save(paths)?;
        Ok(config)
    }

    pub fn load(file: &Path) -> Result<Option<Self>> {
        match std::fs::read_to_string(file) {
            Ok(text) => Ok(Some(toml::from_str(&text)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::Io(e)),
        }
    }

    pub fn save(&self, paths: &Paths) -> Result<()> {
        std::fs::create_dir_all(paths.config())?;
        store::write_atomic(
            &paths.config_file(),
            toml::to_string_pretty(self)?.as_bytes(),
        )
    }
}

fn new_device_id() -> String {
    let ulid = Ulid::generate().to_string().to_lowercase();
    format!("dev_{}", &ulid[ulid.len() - 8..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(tmp: &tempfile::TempDir) -> Paths {
        Paths::new(tmp.path().join("data"), tmp.path().join("config"))
    }

    #[test]
    fn the_device_id_is_generated_once_and_reused() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths(&tmp);

        let first = Config::load_or_init(&p).unwrap();
        let second = Config::load_or_init(&p).unwrap();

        assert_eq!(first.device_id, second.device_id);
    }

    #[test]
    fn two_installs_never_share_a_device_id() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();

        assert_ne!(
            Config::load_or_init(&paths(&a)).unwrap().device_id,
            Config::load_or_init(&paths(&b)).unwrap().device_id
        );
    }

    #[test]
    fn the_config_file_is_written_outside_the_synced_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths(&tmp);
        Config::load_or_init(&p).unwrap();

        assert!(p.config_file().exists());
        assert!(!p.config_file().starts_with(p.data()));
    }

    #[test]
    fn round_trips_through_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let p = paths(&tmp);

        let mut config = Config::load_or_init(&p).unwrap();
        config.locale = Some("es".into());
        config.editor = Some("hx".into());
        config.save(&p).unwrap();

        assert_eq!(Config::load(&p.config_file()).unwrap().unwrap(), config);
    }

    #[test]
    fn a_missing_config_is_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(
            Config::load(&tmp.path().join("absent.toml"))
                .unwrap()
                .is_none()
        );
    }
}
