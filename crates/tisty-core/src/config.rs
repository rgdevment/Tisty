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

/// `None` means nobody has been asked yet, which is what shows the assistant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "how", content = "at")]
pub enum Sync {
    Local,
    /// A folder both machines reach; whoever keeps it in step is not our problem.
    Folder(std::path::PathBuf),
}

impl Config {
    /// Named channels this machine has been told to keep quiet.
    pub fn muted(&self) -> &[String] {
        self.quiet.as_deref().unwrap_or_default()
    }

    /// Clamped, not trusted: a hand-edited zero would copy nothing at all and a
    /// huge one would put a film inside the store.
    pub fn copies_up_to(&self) -> u64 {
        self.attach_up_to
            .unwrap_or(crate::attach::COPIED_UP_TO)
            .clamp(crate::attach::COPIED_LEAST, crate::attach::COPIED_MOST)
    }

    /// The shared folder already holds every machine's history, so a second
    /// snapshot beside it would be a rival truth nobody asked for.
    pub fn backs_up(&self) -> bool {
        !matches!(self.sync, Some(Sync::Folder(_)))
    }
}

/// `None` means nobody has been asked yet, which is what asks on first close.
/// Not a system convention: whoever knows how they use it is the person, not
/// the operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Closing {
    Hide,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Never synced — a shared id would put two machines in one segment file.
    pub device_id: DeviceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,
    /// The version that last opened this store. A different one means the
    /// program was just installed or updated, and the store may have been
    /// written by another machine since — or by an older Tisty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opened_by: Option<String>,
    /// What closing the window does. Ignored where there is no tray to hide in.
    /// Kept above `sync`, which becomes a TOML table: the serialiser floats
    /// tables to the end on its own, but a hand-edited file will not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_close: Option<Closing>,
    /// So the screen can say when the last copy was made. Never synced: a copy
    /// is a thing one machine did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backed_up_at: Option<jiff::Timestamp>,
    /// Never synced either: where this machine sends its own directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sync: Option<Sync>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synced_at: Option<jiff::Timestamp>,
    /// Which channels may speak. Absent means every one of them, so a new
    /// channel starts on without anyone having to opt in to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quiet: Option<Vec<String>>,
    /// Bytes above which an attachment is pointed at instead of copied in.
    /// §9 G3 promised this configurable and it had stayed a constant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attach_up_to: Option<u64>,
}

impl Config {
    pub fn load_or_init(paths: &Paths) -> Result<Self> {
        if let Some(existing) = Self::load(&paths.config_file())? {
            return Ok(existing);
        }

        let config = Self {
            device_id: DeviceId(new_device_id()),
            locale: None,
            editor: None,
            quiet: None,
            attach_up_to: None,
            opened_by: None,
            on_close: None,
            backed_up_at: None,
            sync: None,
            synced_at: None,
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
        // Where the device id and `private/` live, so a mode nobody narrowed
        // leaves both within reach of every other account on the machine.
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

pub fn new_device_id() -> String {
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
    #[test]
    fn a_table_valued_field_does_not_swallow_what_follows_it() {
        let config = Config {
            device_id: DeviceId("dev_a".into()),
            locale: Some("es".into()),
            editor: None,
            opened_by: Some("0.1.0".into()),
            on_close: Some(Closing::Hide),
            backed_up_at: None,
            sync: Some(Sync::Folder("G:/Mi unidad/Tisty".into())),
            synced_at: None,
            quiet: None,
            attach_up_to: None,
        };

        let written = toml::to_string_pretty(&config).unwrap();
        let read: Config = toml::from_str(&written).unwrap();
        assert_eq!(
            read, config,
            "round trip lost something:
{written}"
        );
    }
}
